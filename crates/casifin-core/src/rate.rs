//! Interest rate types with compounding and day count conventions.

use std::fmt;

use rust_decimal::{Decimal, MathematicalOps};
use serde::{Deserialize, Serialize};

use crate::CasifinError;

/// An interest rate with compounding metadata.
///
/// This struct encapsulates an annual interest rate along with its
/// compounding frequency and day count convention.
///
/// # Invariants
/// - `annual_rate` must be non-negative
/// - `frequency` (for `Discrete`) must be at least 1
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rate {
    /// The annual interest rate as a decimal (e.g., 0.0425 for 4.25%)
    pub annual_rate: Decimal,
    /// The compounding frequency
    pub compounding: Compounding,
    /// The day count convention
    pub convention: DayCount,
}

impl Rate {
    /// Creates a new `Rate` with the specified annual rate and compounding.
    ///
    /// # Arguments
    /// * `annual_rate` - The annual interest rate (e.g., 0.05 for 5%)
    /// * `compounding` - The compounding frequency
    /// * `convention` - The day count convention
    ///
    /// # Returns
    /// `Ok(Rate)` if the rate is non-negative, `Err(CasifinError::InvalidRate)` otherwise.
    ///
    /// # Example
    /// ```
    /// use casifin_core::{Rate, Compounding, DayCount};
    /// use rust_decimal::Decimal;
    ///
    /// let rate = Rate::new(
    ///     Decimal::new(5, 2), // 5%
    ///     Compounding::MONTHLY,
    ///     DayCount::Actual365,
    /// ).unwrap();
    /// ```
    pub fn new(
        annual_rate: Decimal,
        compounding: Compounding,
        convention: DayCount,
    ) -> Result<Self, CasifinError> {
        if annual_rate < Decimal::ZERO {
            return Err(CasifinError::InvalidRate(annual_rate));
        }

        Ok(Rate {
            annual_rate,
            compounding,
            convention,
        })
    }

    /// Returns the periodic rate for a given number of periods per year.
    ///
    /// # Arguments
    /// * `periods_per_year` - The number of compounding periods per year
    ///
    /// # Returns
    /// The periodic interest rate as a `Decimal`.
    pub fn periodic_rate(&self, periods_per_year: u32) -> Result<Decimal, CasifinError> {
        self.annual_rate
            .checked_div(Decimal::from(periods_per_year))
            .ok_or(CasifinError::DivisionByZero {
                operation: "periodic_rate calculation",
            })
    }

    /// Returns the effective annual rate.
    ///
    /// # Returns
    /// The effective annual rate as a `Decimal`.
    pub fn effective_annual_rate(&self) -> Result<Decimal, CasifinError> {
        match self.compounding {
            Compounding::Discrete(n) => {
                let periodic = self.periodic_rate(n)?;
                let one = Decimal::ONE;
                let base = one + periodic;
                let power = base
                    .checked_powi(n as i64)
                    .ok_or(CasifinError::ScheduleOverflow {
                        detail: "effective_annual_rate power overflow".to_string(),
                    })?;
                Ok(power - Decimal::ONE)
            }
            Compounding::Continuous => {
                // e^r - 1, using Taylor series approximation
                let r = self.annual_rate;
                let exp_r = Decimal::E.powd(r);
                Ok(exp_r - Decimal::ONE)
            }
        }
    }

    /// Returns the rate per period for TVM calculations.
    ///
    /// # Arguments
    /// * `payments_per_year` - The number of payments per year
    ///
    /// # Returns
    /// The rate per payment period as a `Decimal`.
    pub fn rate_per_period(&self, payments_per_year: u32) -> Result<Decimal, CasifinError> {
        self.periodic_rate(payments_per_year)
    }
}

/// The compounding frequency for an interest rate.
///
/// Specifies how often interest is compounded per year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Compounding {
    /// Compounded `n` times per year (e.g., 12 for monthly)
    Discrete(u32),
    /// Compounded continuously (using exponential function)
    Continuous,
}

impl Compounding {
    /// Monthly compounding (12 times per year).
    pub const MONTHLY: Self = Compounding::Discrete(12);

    /// Quarterly compounding (4 times per year).
    pub const QUARTERLY: Self = Compounding::Discrete(4);

    /// Semi-annual compounding (2 times per year).
    pub const SEMI_ANNUAL: Self = Compounding::Discrete(2);

    /// Annual compounding (1 time per year).
    pub const ANNUAL: Self = Compounding::Discrete(1);

    /// Daily compounding (365 times per year).
    pub const DAILY: Self = Compounding::Discrete(365);
}

impl Default for Compounding {
    fn default() -> Self {
        Compounding::MONTHLY
    }
}

impl fmt::Display for Compounding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Compounding::Discrete(n) => write!(f, "{} times/year", n),
            Compounding::Continuous => write!(f, "Continuous"),
        }
    }
}

/// The day count convention for interest calculations.
///
/// Specifies how to count days between dates for interest accrual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DayCount {
    /// Actual days / 365
    #[default]
    Actual365,
    /// Actual days / 360
    Actual360,
    /// 30-day months / 360-day year (US method)
    Thirty360,
    /// 30-day months / 360-day year (European method)
    ThirtyE360,
}

impl DayCount {
    /// Returns the denominator for this convention.
    pub const fn denominator(&self) -> u32 {
        match self {
            DayCount::Actual365 | DayCount::Thirty360 | DayCount::ThirtyE360 => 365,
            DayCount::Actual360 => 360,
        }
    }

    /// Calculates the year fraction between two dates.
    ///
    /// # Arguments
    /// * `start` - The start date (days since epoch)
    /// * `end` - The end date (days since epoch)
    ///
    /// # Returns
    /// The year fraction as a `Decimal`.
    pub fn year_fraction(&self, start: i64, end: i64) -> Decimal {
        let days = end - start;
        Decimal::from(days) / Decimal::from(self.denominator())
    }
}

impl fmt::Display for DayCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DayCount::Actual365 => write!(f, "Actual/365"),
            DayCount::Actual360 => write!(f, "Actual/360"),
            DayCount::Thirty360 => write!(f, "30/360"),
            DayCount::ThirtyE360 => write!(f, "30E/360"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_creation() {
        let rate = Rate::new(
            Decimal::new(5, 2),
            Compounding::MONTHLY,
            DayCount::Actual365,
        );
        assert!(rate.is_ok());
    }

    #[test]
    fn test_rate_negative_rejected() {
        let rate = Rate::new(
            Decimal::new(-5, 2),
            Compounding::MONTHLY,
            DayCount::Actual365,
        );
        assert_eq!(rate, Err(CasifinError::InvalidRate(Decimal::new(-5, 2))));
    }

    #[test]
    fn test_periodic_rate() {
        let rate = Rate::new(
            Decimal::new(12, 2),
            Compounding::Discrete(12),
            DayCount::Actual365,
        )
        .unwrap();
        let periodic = rate.periodic_rate(12).unwrap();
        assert_eq!(periodic, Decimal::new(1, 2)); // 1% per month
    }

    #[test]
    fn test_effective_annual_rate() {
        let rate = Rate::new(
            Decimal::new(12, 2),
            Compounding::Discrete(12),
            DayCount::Actual365,
        )
        .unwrap();
        let effective = rate.effective_annual_rate().unwrap();
        // (1 + 0.01)^12 - 1 ≈ 0.1268
        assert!(effective > Decimal::new(12, 2));
    }

    #[test]
    fn test_day_count_denominator() {
        assert_eq!(DayCount::Actual365.denominator(), 365);
        assert_eq!(DayCount::Actual360.denominator(), 360);
    }
}
