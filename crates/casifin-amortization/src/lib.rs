//! Amortization engines (fixed-rate and ARM) for the casifin financial computation engine.

#![deny(warnings)]

use casifin_core::{CasifinError, Money, Rate};
use casifin_tvm::pmt;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A single entry in an amortization schedule.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
    /// The date of this payment (optional).
    pub date: Option<NaiveDate>,
}

/// An amortization schedule for a loan.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    pub fn new() -> Self {
        Self::default()
    }
}

/// Builder for creating amortization schedules.
pub struct AmortizationBuilder {
    /// The loan principal.
    principal: Money,
    /// The interest rate.
    rate: Rate,
    /// The term in months.
    term_months: u32,
    /// Optional payment modifier function.
    payment_modifier: Option<Box<dyn Fn(u32, Money) -> Money>>,
    /// Day count convention (30/360 or Actual/365).
    day_count_360: bool,
    /// Start date (optional).
    start_date: Option<NaiveDate>,
    /// Configuration epsilon for final balance check.
    eps: Decimal,
}

impl std::fmt::Debug for AmortizationBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AmortizationBuilder")
            .field("principal", &self.principal)
            .field("rate", &self.rate)
            .field("term_months", &self.term_months)
            .field("day_count_360", &self.day_count_360)
            .field("eps", &self.eps)
            .finish_non_exhaustive()
    }
}

impl AmortizationBuilder {
    /// Creates a new `AmortizationBuilder`.
    ///
    /// # Arguments
    /// * `principal` - The loan amount
    /// * `rate` - The annual interest rate
    /// * `term_months` - The loan term in months
    pub fn new(principal: Money, rate: Rate, term_months: u32) -> Self {
        AmortizationBuilder {
            principal,
            rate,
            term_months,
            payment_modifier: None,
            day_count_360: false,
            start_date: None,
            eps: Decimal::new(1, 2), // 0.01 tolerance
        }
    }

    /// Sets a payment modifier function.
    ///
    /// The modifier receives the period number and base payment,
    /// and returns the modified payment amount.
    pub fn with_payment_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(u32, Money) -> Money + 'static,
    {
        self.payment_modifier = Some(Box::new(modifier));
        self
    }

    /// Clones the builder (without the payment modifier).
    pub fn clone_without_modifier(&self) -> Self {
        AmortizationBuilder {
            principal: self.principal,
            rate: self.rate,
            term_months: self.term_months,
            payment_modifier: None,
            day_count_360: self.day_count_360,
            start_date: self.start_date,
            eps: self.eps,
        }
    }
}

impl Clone for AmortizationBuilder {
    fn clone(&self) -> Self {
        self.clone_without_modifier()
    }
}

impl AmortizationBuilder {
    /// Sets the day count convention to 30/360.
    pub fn with_30_360(mut self) -> Self {
        self.day_count_360 = true;
        self
    }

    /// Sets the start date for the schedule.
    pub fn with_start_date(mut self, date: NaiveDate) -> Self {
        self.start_date = Some(date);
        self
    }

    /// Sets the convergence epsilon.
    pub fn with_eps(mut self, eps: Decimal) -> Self {
        self.eps = eps;
        self
    }

    /// Builds the amortization schedule.
    ///
    /// # Returns
    /// `Ok(AmortizationSchedule)` if successful, `Err(CasifinError)` if:
    /// - Principal is zero or negative
    /// - Term is zero
    /// - Schedule fails to pay off within epsilon
    pub fn build(self) -> Result<AmortizationSchedule, CasifinError> {
        debug_assert!(self.principal > Money::ZERO, "principal must be positive");
        debug_assert!(self.term_months > 0, "term_months must be positive");

        if self.principal <= Money::ZERO {
            return Err(CasifinError::InvalidAmount(self.principal));
        }
        if self.term_months == 0 {
            return Err(CasifinError::InvalidPeriod(0));
        }

        self.generate_schedule()
    }

    /// Computes the monthly payment and rate for the schedule.
    fn compute_base_payment(&self) -> Result<(Decimal, Money), CasifinError> {
        let monthly_rate = self.rate.annual_rate / Decimal::from(12);
        let base_payment = pmt(
            monthly_rate,
            self.term_months,
            self.principal,
            Money::ZERO,
            casifin_tvm::PaymentDue::End,
        )?;
        Ok((monthly_rate, base_payment))
    }

    /// Computes a single amortization entry for the given period.
    fn compute_period_entry(
        &self,
        period: u32,
        balance: Money,
        monthly_rate: Decimal,
        base_payment: Money,
    ) -> (AmortizationEntry, Money) {
        let interest = balance * monthly_rate;

        let payment = match &self.payment_modifier {
            Some(modifier) => modifier(period, base_payment),
            None => base_payment,
        };

        let principal_payment = payment - interest;
        let new_balance = balance - principal_payment;

        let (principal_payment, payment, new_balance) = if period == self.term_months {
            (balance, payment + new_balance, Money::ZERO)
        } else {
            (principal_payment, payment, new_balance)
        };

        let date = self.start_date.map(|start| {
            start
                .checked_add_months(chrono::Months::new(period))
                .unwrap_or(start)
        });

        let entry = AmortizationEntry {
            period,
            payment,
            principal: principal_payment,
            interest,
            balance: new_balance,
            date,
        };

        (entry, new_balance)
    }

    /// Generates the amortization schedule.
    fn generate_schedule(&self) -> Result<AmortizationSchedule, CasifinError> {
        let (monthly_rate, base_payment) = self.compute_base_payment()?;

        let mut schedule = AmortizationSchedule::new();
        let mut balance = self.principal;
        let mut total_payments = Money::ZERO;
        let mut total_interest = Money::ZERO;
        let mut total_principal = Money::ZERO;

        for period in 1..=self.term_months {
            let (entry, new_balance) =
                self.compute_period_entry(period, balance, monthly_rate, base_payment);

            total_payments = total_payments + entry.payment;
            total_interest = total_interest + entry.interest;
            total_principal = total_principal + entry.principal;
            balance = new_balance;

            schedule.entries.push(entry);
        }

        schedule.total_payments = total_payments;
        schedule.total_interest = total_interest;
        schedule.total_principal = total_principal;

        self.verify_final_balance(&schedule)?;
        Ok(schedule)
    }

    /// Verifies the final balance is within epsilon of zero.
    fn verify_final_balance(&self, schedule: &AmortizationSchedule) -> Result<(), CasifinError> {
        let final_balance = schedule
            .entries
            .last()
            .map(|e| e.balance)
            .unwrap_or(Money::ZERO);

        if final_balance.abs().inner() > self.eps {
            return Err(CasifinError::ScheduleOverflow {
                detail: format!(
                    "amortization did not fully pay off: balance={}",
                    final_balance
                ),
            });
        }
        Ok(())
    }
}

/// Rate caps for an ARM.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RateCaps {
    /// Maximum rate increase per adjustment period.
    pub periodic_cap: Decimal,
    /// Maximum rate increase over the life of the loan.
    pub lifetime_cap: Decimal,
    /// Initial rate cap (for first adjustment).
    pub initial_cap: Option<Decimal>,
}

impl RateCaps {
    /// Creates new rate caps.
    pub fn new(periodic_cap: Decimal, lifetime_cap: Decimal) -> Self {
        RateCaps {
            periodic_cap,
            lifetime_cap,
            initial_cap: None,
        }
    }

    /// Sets the initial cap.
    pub fn with_initial_cap(mut self, cap: Decimal) -> Self {
        self.initial_cap = Some(cap);
        self
    }
}

/// An adjustable-rate mortgage (ARM) schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustableRateSchedule {
    /// The initial rate.
    pub initial_rate: Rate,
    /// Rate adjustments: (period, new_rate).
    pub adjustments: Vec<(u32, Rate)>,
    /// Rate caps (optional).
    pub caps: Option<RateCaps>,
    /// The generated schedule.
    pub schedule: AmortizationSchedule,
}

/// Builder for adjustable-rate schedules.
#[derive(Debug, Clone)]
pub struct AdjustableRateBuilder {
    principal: Money,
    initial_rate: Rate,
    term_months: u32,
    adjustments: Vec<(u32, Rate)>,
    caps: Option<RateCaps>,
    #[allow(dead_code)]
    eps: Decimal,
}

impl AdjustableRateBuilder {
    /// Creates a new ARM builder.
    pub fn new(principal: Money, initial_rate: Rate, term_months: u32) -> Self {
        AdjustableRateBuilder {
            principal,
            initial_rate,
            term_months,
            adjustments: Vec::new(),
            caps: None,
            eps: Decimal::new(1, 2),
        }
    }

    /// Adds a rate adjustment at the specified period.
    pub fn with_adjustment(mut self, period: u32, rate: Rate) -> Self {
        self.adjustments.push((period, rate));
        self
    }

    /// Sets rate caps.
    pub fn with_caps(mut self, caps: RateCaps) -> Self {
        self.caps = Some(caps);
        self
    }

    /// Builds the ARM schedule.
    pub fn build(self) -> Result<AdjustableRateSchedule, CasifinError> {
        debug_assert!(self.principal > Money::ZERO, "principal must be positive");
        debug_assert!(self.term_months > 0, "term_months must be positive");

        if self.principal <= Money::ZERO {
            return Err(CasifinError::InvalidAmount(self.principal));
        }
        if self.term_months == 0 {
            return Err(CasifinError::InvalidPeriod(0));
        }

        let mut schedule = AmortizationSchedule::new();
        let mut balance = self.principal;
        let mut current_rate = self.initial_rate;
        let mut adjustment_idx = 0;

        let mut total_payments = Money::ZERO;
        let mut total_interest = Money::ZERO;
        let mut total_principal = Money::ZERO;

        let mut period = 1u32;
        while period <= self.term_months && balance.inner() > Decimal::ZERO {
            // Check for rate adjustment
            if let Some(new_rate) =
                self.check_rate_adjustment(period, &mut adjustment_idx, &mut current_rate)
            {
                current_rate = new_rate;
            }

            let (entry, new_balance) = self.compute_arm_period_entry(period, balance, current_rate);

            total_payments = total_payments + entry.payment;
            total_interest = total_interest + entry.interest;
            total_principal = total_principal + entry.principal;
            balance = new_balance;

            schedule.entries.push(entry);
            period += 1;
        }

        schedule.total_payments = total_payments;
        schedule.total_interest = total_interest;
        schedule.total_principal = total_principal;

        Ok(AdjustableRateSchedule {
            initial_rate: self.initial_rate,
            adjustments: self.adjustments,
            caps: self.caps,
            schedule,
        })
    }

    /// Checks and applies any rate adjustment for the current period.
    /// Returns the new rate if an adjustment was applied.
    fn check_rate_adjustment(
        &self,
        period: u32,
        adjustment_idx: &mut usize,
        current_rate: &mut Rate,
    ) -> Option<Rate> {
        if *adjustment_idx >= self.adjustments.len() {
            return None;
        }
        let (adj_period, new_rate) = self.adjustments[*adjustment_idx];
        if period < adj_period {
            return None;
        }
        *adjustment_idx += 1;

        let adjusted = if let Some(caps) = &self.caps {
            Self::apply_rate_caps(*current_rate, new_rate, caps)
        } else {
            new_rate
        };
        Some(adjusted)
    }

    /// Applies rate caps to limit the adjustment magnitude.
    fn apply_rate_caps(current: Rate, proposed: Rate, caps: &RateCaps) -> Rate {
        let rate_diff = proposed.annual_rate - current.annual_rate;
        let max_increase = caps.periodic_cap;
        if rate_diff > max_increase {
            Rate {
                annual_rate: current.annual_rate + max_increase,
                ..proposed
            }
        } else if rate_diff < -max_increase {
            Rate {
                annual_rate: current.annual_rate - max_increase,
                ..proposed
            }
        } else {
            proposed
        }
    }

    /// Computes a single ARM period entry.
    fn compute_arm_period_entry(
        &self,
        period: u32,
        balance: Money,
        current_rate: Rate,
    ) -> (AmortizationEntry, Money) {
        let monthly_rate = current_rate.annual_rate / Decimal::from(12);
        let remaining_periods = self.term_months - period + 1;

        let payment = pmt(
            monthly_rate,
            remaining_periods,
            balance,
            Money::ZERO,
            casifin_tvm::PaymentDue::End,
        )
        .unwrap_or(Money::ZERO);

        let interest = balance * monthly_rate;
        let principal_payment = payment - interest;
        let new_balance = balance - principal_payment;

        let (principal_payment, payment, new_balance) =
            if new_balance <= Money::ZERO || period == self.term_months {
                (balance, payment + new_balance, Money::ZERO)
            } else {
                (principal_payment, payment, new_balance)
            };

        let entry = AmortizationEntry {
            period,
            payment,
            principal: principal_payment,
            interest,
            balance: new_balance,
            date: None,
        };

        (entry, new_balance)
    }
}

#[cfg(test)]
mod tests {
    use casifin_core::Compounding;

    use super::*;

    #[test]
    fn test_fixed_rate_amortization() {
        let principal = Money::from(200000);
        let rate = Rate::new(
            Decimal::new(6, 2),
            Compounding::MONTHLY,
            casifin_core::DayCount::Actual365,
        )
        .unwrap();

        let schedule = AmortizationBuilder::new(principal, rate, 360)
            .build()
            .unwrap();

        assert_eq!(schedule.entries.len(), 360);
        assert!(schedule.total_interest > Money::ZERO);
        // Allow for small rounding differences
        let principal_diff = (schedule.total_principal - principal).abs();
        assert!(principal_diff <= Money::from(1));
        assert!(schedule.entries.last().unwrap().balance <= Money::from(1));
    }

    #[test]
    fn test_arm_schedule() {
        let principal = Money::from(200000);
        let initial_rate = Rate::new(
            Decimal::new(5, 2),
            Compounding::MONTHLY,
            casifin_core::DayCount::Actual365,
        )
        .unwrap();

        let adj_rate = Rate::new(
            Decimal::new(7, 2),
            Compounding::MONTHLY,
            casifin_core::DayCount::Actual365,
        )
        .unwrap();

        let arm = AdjustableRateBuilder::new(principal, initial_rate, 360)
            .with_adjustment(61, adj_rate)
            .build()
            .unwrap();

        assert!(!arm.schedule.entries.is_empty());
    }
}
