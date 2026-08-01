//! Amortization engines (fixed-rate and ARM) for the casifin financial computation engine.

#![deny(warnings)]

use casifin_core::{CasifinError, Config, Money, PaymentDue, Rate};
use rust_decimal::Decimal;

// ============================================================================
// AmortizationEntry
// ============================================================================

/// A single entry in an amortization schedule.
///
/// # Panics
/// This type does not panic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AmortizationEntry {
    /// The period number (1-indexed).
    pub period: u32,
    /// The total payment for this period.
    pub payment: Money,
    /// The principal portion of the payment.
    pub principal: Money,
    /// The interest portion of the payment.
    pub interest: Money,
    /// The remaining balance after this payment.
    pub balance: Money,
}

// ============================================================================
// AmortizationSchedule
// ============================================================================

/// An amortization schedule for a loan.
///
/// # Panics
/// This type does not panic.
#[derive(Debug, Clone, PartialEq)]
pub struct AmortizationSchedule {
    /// The individual schedule entries.
    pub entries: Vec<AmortizationEntry>,
    /// Total payments over the life of the loan.
    pub total_payments: Money,
    /// Total interest paid over the life of the loan.
    pub total_interest: Money,
    /// Total principal paid (should equal original principal).
    pub total_principal: Money,
}

impl AmortizationSchedule {
    /// Creates a new empty schedule.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            total_payments: Money::ZERO,
            total_interest: Money::ZERO,
            total_principal: Money::ZERO,
        }
    }
}

impl Default for AmortizationSchedule {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// AmortizationBuilder
// ============================================================================

/// Builder for creating amortization schedules.
///
/// # Panics
/// This type does not panic.
pub struct AmortizationBuilder {
    principal: Money,
    rate: Rate,
    term_months: u32,
    config: Config,
    payment_modifier: Option<Box<dyn Fn(u32, Money) -> Money>>,
}

impl std::fmt::Debug for AmortizationBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AmortizationBuilder")
            .field("principal", &self.principal)
            .field("rate", &self.rate)
            .field("term_months", &self.term_months)
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl Clone for AmortizationBuilder {
    fn clone(&self) -> Self {
        AmortizationBuilder {
            principal: self.principal,
            rate: self.rate,
            term_months: self.term_months,
            config: self.config,
            payment_modifier: None,
        }
    }
}

impl AmortizationBuilder {
    /// Creates a new `AmortizationBuilder`.
    ///
    /// # Arguments
    /// * `principal` - The loan amount.
    /// * `rate` - The annual interest rate.
    /// * `term_months` - The loan term in months.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn new(principal: Money, rate: Rate, term_months: u32) -> Self {
        AmortizationBuilder {
            principal,
            rate,
            term_months,
            config: Config::default(),
            payment_modifier: None,
        }
    }

    /// Sets the solver configuration.
    ///
    /// # Arguments
    /// * `config` - The configuration to use.
    ///
    /// # Returns
    /// The builder.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    /// Sets a payment modifier function.
    ///
    /// The modifier receives the period number and base payment,
    /// and returns the modified payment amount.
    ///
    /// # Arguments
    /// * `modifier` - The modifier function.
    ///
    /// # Returns
    /// The builder.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn with_payment_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(u32, Money) -> Money + 'static,
    {
        self.payment_modifier = Some(Box::new(modifier));
        self
    }

    /// Builds the amortization schedule.
    ///
    /// # Returns
    /// `Ok(AmortizationSchedule)` if successful, `Err(CasifinError)` if:
    /// - Principal is zero or negative
    /// - Term is zero
    /// - Schedule fails to pay off within epsilon
    ///
    /// # Panics
    /// This function does not panic.
    pub fn build(self) -> Result<AmortizationSchedule, CasifinError> {
        if self.principal <= Money::ZERO {
            return Err(CasifinError::InvalidAmount(self.principal));
        }
        if self.term_months == 0 {
            return Err(CasifinError::InvalidPeriod(0));
        }

        debug_assert!(self.principal > Money::ZERO, "principal must be positive");
        debug_assert!(self.term_months > 0, "term_months must be positive");
        debug_assert!(
            self.rate.annual_rate >= Decimal::ZERO,
            "annual_rate must be non-negative"
        );

        let schedule = self.generate_schedule()?;
        self.verify_invariants(&schedule)?;
        Ok(schedule)
    }

    /// Computes the monthly rate from the annual rate.
    fn monthly_rate(&self) -> Result<Decimal, CasifinError> {
        debug_assert!(
            self.rate.annual_rate >= Decimal::ZERO,
            "annual_rate must be non-negative"
        );
        self.rate
            .annual_rate
            .checked_div(Decimal::from(12))
            .ok_or(CasifinError::DivisionByZero {
                operation: "monthly rate",
            })
    }

    /// Generates the amortization schedule.
    fn generate_schedule(&self) -> Result<AmortizationSchedule, CasifinError> {
        let monthly_rate = self.monthly_rate()?;

        // Compute base payment using TVM: pv = -principal, fv = 0, nper = term_months.
        // PMT returns a negative outflow; take the absolute value for the schedule.
        let base_payment = casifin_tvm::pmt(
            self.rate,
            self.term_months,
            -self.principal,
            Money::ZERO,
            PaymentDue::End,
        )?
        .abs();

        let mut schedule = AmortizationSchedule::new();
        let mut balance = self.principal;

        let mut total_payments = Money::ZERO;
        let mut total_interest = Money::ZERO;
        let mut total_principal = Money::ZERO;

        for period in 1..=self.term_months {
            if balance <= Money::ZERO {
                break;
            }

            let interest = balance * monthly_rate;

            let payment = match &self.payment_modifier {
                Some(modifier) => modifier(period, base_payment),
                None => base_payment,
            };

            let principal_payment = payment - interest;
            let mut new_balance = balance - principal_payment;
            let mut actual_payment = payment;
            let mut actual_principal = principal_payment;

            // If paying off early or final period, adjust to exact remaining balance.
            if new_balance <= Money::ZERO || period == self.term_months {
                actual_payment = payment + new_balance;
                actual_principal = balance;
                new_balance = Money::ZERO;
            }

            let entry = AmortizationEntry {
                period,
                payment: actual_payment,
                principal: actual_principal,
                interest,
                balance: new_balance,
            };

            total_payments = total_payments + actual_payment;
            total_interest = total_interest + interest;
            total_principal = total_principal + actual_principal;
            balance = new_balance;

            schedule.entries.push(entry);
        }

        schedule.total_payments = total_payments;
        schedule.total_interest = total_interest;
        schedule.total_principal = total_principal;

        Ok(schedule)
    }

    /// Verifies the post-build invariants of the schedule.
    fn verify_invariants(&self, schedule: &AmortizationSchedule) -> Result<(), CasifinError> {
        let eps = self.config.eps;

        // Final balance must be near zero.
        let final_balance = schedule
            .entries
            .last()
            .map(|e| e.balance)
            .unwrap_or(Money::ZERO);

        if final_balance.abs().inner() > eps {
            return Err(CasifinError::ScheduleOverflow {
                detail: format!(
                    "amortization did not fully pay off: balance={}",
                    final_balance
                ),
            });
        }

        // Total principal should equal original principal.
        let principal_diff = (schedule.total_principal - self.principal).abs();
        if principal_diff.inner() > eps {
            return Err(CasifinError::ScheduleOverflow {
                detail: format!(
                    "total principal {} does not match original principal {}",
                    schedule.total_principal, self.principal
                ),
            });
        }

        // total_payments == total_principal + total_interest
        let payments_diff =
            (schedule.total_payments - (schedule.total_principal + schedule.total_interest)).abs();
        if payments_diff.inner() > eps {
            return Err(CasifinError::ScheduleOverflow {
                detail: "payment/principal/interest identity violated".to_string(),
            });
        }

        // Each entry: payment == principal + interest
        for entry in &schedule.entries {
            let entry_diff = (entry.payment - (entry.principal + entry.interest)).abs();
            if entry_diff.inner() > eps {
                return Err(CasifinError::ScheduleOverflow {
                    detail: format!(
                        "entry {} payment identity violated: payment={}, principal+interest={}",
                        entry.period,
                        entry.payment,
                        entry.principal + entry.interest
                    ),
                });
            }
        }

        Ok(())
    }
}

// ============================================================================
// ArmSchedule
// ============================================================================

/// Configuration for an adjustable-rate mortgage (ARM) schedule.
///
/// This type holds ARM parameters; full ARM schedule generation is left to
/// future extensions.
///
/// # Panics
/// This type does not panic.
#[derive(Debug, Clone, PartialEq)]
pub struct ArmSchedule {
    pub adjustments: Vec<(u32, Rate)>,
    pub periodic_cap: Option<Decimal>,
    pub lifetime_cap: Option<Decimal>,
    pub lifetime_floor: Option<Decimal>,
}

impl ArmSchedule {
    /// Creates a new `ArmSchedule`.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn new() -> Self {
        Self {
            adjustments: Vec::new(),
            periodic_cap: None,
            lifetime_cap: None,
            lifetime_floor: None,
        }
    }

    /// Adds a rate adjustment.
    ///
    /// # Arguments
    /// * `period` - The period at which the rate changes.
    /// * `rate` - The new rate.
    ///
    /// # Returns
    /// The builder.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn with_adjustment(mut self, period: u32, rate: Rate) -> Self {
        self.adjustments.push((period, rate));
        self
    }

    /// Sets the periodic cap.
    ///
    /// # Arguments
    /// * `cap` - The maximum change per adjustment.
    ///
    /// # Returns
    /// The builder.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn with_periodic_cap(mut self, cap: Decimal) -> Self {
        self.periodic_cap = Some(cap);
        self
    }

    /// Sets the lifetime cap.
    ///
    /// # Arguments
    /// * `cap` - The maximum rate over the life of the loan.
    ///
    /// # Returns
    /// The builder.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn with_lifetime_cap(mut self, cap: Decimal) -> Self {
        self.lifetime_cap = Some(cap);
        self
    }

    /// Sets the lifetime floor.
    ///
    /// # Arguments
    /// * `floor` - The minimum rate over the life of the loan.
    ///
    /// # Returns
    /// The builder.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn with_lifetime_floor(mut self, floor: Decimal) -> Self {
        self.lifetime_floor = Some(floor);
        self
    }
}

impl Default for ArmSchedule {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use casifin_core::Compounding;

    use super::*;

    #[test]
    fn fixed_30_year_mortgage() {
        let principal = Money::from_decimal(Decimal::new(300000, 0));
        let rate = Rate::new(Decimal::new(425, 4), Compounding::Discrete(12)).unwrap();

        let schedule = AmortizationBuilder::new(principal, rate, 360)
            .build()
            .unwrap();

        assert_eq!(schedule.entries.len(), 360);

        let first = &schedule.entries[0];
        // First payment should be approximately $1,475.82
        assert!(first.payment > Money::from_decimal(Decimal::new(1475, 0)));
        assert!(first.payment < Money::from_decimal(Decimal::new(1476, 0)));

        // First interest should be approximately $1,062.50
        assert!(first.interest > Money::from_decimal(Decimal::new(1062, 0)));
        assert!(first.interest < Money::from_decimal(Decimal::new(1063, 0)));

        // First principal should be approximately $413.32
        assert!(first.principal > Money::from_decimal(Decimal::new(413, 0)));
        assert!(first.principal < Money::from_decimal(Decimal::new(414, 0)));

        // Final balance should be zero
        let final_balance = schedule.entries.last().unwrap().balance;
        assert!(final_balance.abs() <= Money::from_decimal(Decimal::new(1, 2)));
    }

    #[test]
    fn payment_modifier() {
        let principal = Money::from_decimal(Decimal::new(300000, 0));
        let rate = Rate::new(Decimal::new(425, 4), Compounding::Discrete(12)).unwrap();

        // Extra $100 principal each month
        let schedule = AmortizationBuilder::new(principal, rate, 360)
            .with_payment_modifier(|_period, base_payment| {
                base_payment + Money::from_decimal(Decimal::new(100, 0))
            })
            .build()
            .unwrap();

        // Should pay off earlier than 360 months
        assert!(schedule.entries.len() < 360);
    }

    #[test]
    fn zero_rate() {
        let principal = Money::from_decimal(Decimal::new(120000, 0));
        let rate = Rate::new(Decimal::ZERO, Compounding::Discrete(12)).unwrap();

        let schedule = AmortizationBuilder::new(principal, rate, 360)
            .build()
            .unwrap();

        // Each period principal = 120000 / 360 = 333.33
        let first = &schedule.entries[0];
        assert_eq!(
            first.principal,
            Money::from_decimal(Decimal::new(120000, 0)) / Decimal::from(360)
        );
    }

    #[test]
    fn invalid_principal() {
        let rate = Rate::new(Decimal::new(5, 2), Compounding::Discrete(12)).unwrap();
        let result = AmortizationBuilder::new(Money::ZERO, rate, 360).build();
        assert!(result.is_err());
    }

    #[test]
    fn invariant_final_balance() {
        let principal = Money::from_decimal(Decimal::new(200000, 0));
        let rate = Rate::new(Decimal::new(6, 2), Compounding::Discrete(12)).unwrap();

        let schedule = AmortizationBuilder::new(principal, rate, 360)
            .build()
            .unwrap();

        let final_balance = schedule.entries.last().unwrap().balance;
        assert!(final_balance.abs() <= Money::from_decimal(Decimal::new(1, 2)));

        let principal_diff = (schedule.total_principal - principal).abs();
        assert!(principal_diff <= Money::from_decimal(Decimal::new(1, 2)));
    }
}
