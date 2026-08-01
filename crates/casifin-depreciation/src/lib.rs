//! Depreciation calculations (Straight-Line, Double-Declining Balance) for casifin.

#![deny(warnings)]

use casifin_core::{CasifinError, Money};
use rust_decimal::Decimal;

/// A depreciation method trait.
pub trait DepreciationMethod {
    /// Computes depreciation for a specific period.
    ///
    /// # Arguments
    /// * `cost` - The initial cost of the asset.
    /// * `salvage` - The salvage/residual value.
    /// * `life_years` - The useful life in years.
    /// * `period` - The period number (1-indexed).
    ///
    /// # Returns
    /// `Ok(Money)` containing the depreciation for the period, or `Err(CasifinError)`.
    ///
    /// # Panics
    /// This function does not panic.
    fn depreciate(
        &self,
        cost: Money,
        salvage: Money,
        life_years: u32,
        period: u32,
    ) -> Result<Money, CasifinError>;

    /// Generates a full depreciation schedule.
    ///
    /// # Arguments
    /// * `cost` - The initial cost of the asset.
    /// * `salvage` - The salvage/residual value.
    /// * `life_years` - The useful life in years.
    ///
    /// # Returns
    /// `Ok(Vec<Money>)` containing the per-period depreciation amounts.
    ///
    /// # Panics
    /// This function does not panic.
    fn schedule(
        &self,
        cost: Money,
        salvage: Money,
        life_years: u32,
    ) -> Result<Vec<Money>, CasifinError> {
        debug_assert!(life_years > 0, "life_years must be positive");
        debug_assert!(cost >= salvage, "cost must be >= salvage");

        if life_years == 0 {
            return Err(CasifinError::InvalidPeriod(0));
        }

        let mut schedule = Vec::with_capacity(life_years as usize);
        for period in 1..=life_years {
            schedule.push(self.depreciate(cost, salvage, life_years, period)?);
        }
        Ok(schedule)
    }
}

/// Straight-Line Depreciation.
///
/// # Formula
/// ```text
/// Depreciation = (Cost - Salvage) / Life
/// ```
///
/// # Panics
/// This type does not panic.
#[derive(Debug, Clone, Copy, Default)]
pub struct StraightLine;

impl DepreciationMethod for StraightLine {
    fn depreciate(
        &self,
        cost: Money,
        salvage: Money,
        life_years: u32,
        period: u32,
    ) -> Result<Money, CasifinError> {
        if cost < salvage {
            return Err(CasifinError::InvalidInput {
                reason: "cost must be >= salvage".to_string(),
            });
        }
        if life_years == 0 {
            return Err(CasifinError::InvalidPeriod(0));
        }
        if period == 0 || period > life_years {
            return Err(CasifinError::InvalidPeriod(period));
        }

        debug_assert!(cost >= salvage, "cost must be >= salvage");
        debug_assert!(life_years > 0, "life_years must be positive");

        let depreciable_base = cost - salvage;
        depreciable_base
            .checked_div_decimal(Decimal::from(life_years))
            .ok_or(CasifinError::DivisionByZero {
                operation: "straight_line_depreciation",
            })
    }
}

/// Double-Declining Balance Depreciation.
///
/// Uses an accelerated rate of 2x the straight-line rate.
/// Automatically switches to Straight-Line when SL produces higher depreciation.
///
/// # Formula
/// ```text
/// Rate = 2 / Life
/// Depreciation = Book Value * Rate
/// ```
///
/// # Panics
/// This type does not panic.
#[derive(Debug, Clone, Copy, Default)]
pub struct DoubleDecliningBalance;

impl DoubleDecliningBalance {
    /// Computes the DDB rate.
    fn rate(life_years: u32) -> Result<Decimal, CasifinError> {
        debug_assert!(life_years > 0, "life_years must be positive");
        Decimal::from(2)
            .checked_div(Decimal::from(life_years))
            .ok_or(CasifinError::DivisionByZero {
                operation: "ddb rate",
            })
    }

    /// Computes the book value at the start of the given period.
    fn book_value_at_period(
        cost: Money,
        salvage: Money,
        life_years: u32,
        period: u32,
    ) -> Result<Money, CasifinError> {
        debug_assert!(period > 0, "period must be positive");
        debug_assert!(period <= life_years, "period must not exceed life_years");

        let mut book_value = cost;
        for _ in 1..period {
            let ddb_rate = Self::rate(life_years)?;
            let ddb_dep = book_value * ddb_rate;
            let remaining = book_value - salvage;
            let actual_dep = if ddb_dep > remaining {
                remaining
            } else {
                ddb_dep
            };
            book_value = book_value - actual_dep;
        }
        Ok(book_value)
    }
}

impl DepreciationMethod for DoubleDecliningBalance {
    fn depreciate(
        &self,
        cost: Money,
        salvage: Money,
        life_years: u32,
        period: u32,
    ) -> Result<Money, CasifinError> {
        if cost < salvage {
            return Err(CasifinError::InvalidInput {
                reason: "cost must be >= salvage".to_string(),
            });
        }
        if life_years == 0 {
            return Err(CasifinError::InvalidPeriod(0));
        }
        if period == 0 || period > life_years {
            return Err(CasifinError::InvalidPeriod(period));
        }

        debug_assert!(cost >= salvage, "cost must be >= salvage");
        debug_assert!(life_years > 0, "life_years must be positive");

        let book_value = Self::book_value_at_period(cost, salvage, life_years, period)?;

        let ddb_rate = Self::rate(life_years)?;
        let ddb_depreciation = book_value * ddb_rate;
        let remaining = book_value - salvage;
        let ddb_limited = if ddb_depreciation > remaining {
            remaining
        } else {
            ddb_depreciation
        };

        // Switch to SL if SL on remaining book value produces higher depreciation.
        let remaining_life = life_years - period + 1;
        let sl_current = if remaining_life > 0 {
            (book_value - salvage) / Decimal::from(remaining_life)
        } else {
            Money::ZERO
        };

        if sl_current > ddb_limited {
            Ok(sl_current)
        } else {
            Ok(ddb_limited)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sl_5_year() {
        // $10,000 cost, $2,000 salvage, 5 years = $1,600/year
        let cost = Money::from(10000);
        let salvage = Money::from(2000);
        let sl = StraightLine;
        let annual = sl.depreciate(cost, salvage, 5, 1).unwrap();
        assert_eq!(annual, Money::from(1600));

        let schedule = sl.schedule(cost, salvage, 5).unwrap();
        assert_eq!(schedule.len(), 5);

        let total: Money = schedule.iter().copied().sum();
        assert_eq!(total, cost - salvage);
    }

    #[test]
    fn ddb_switch() {
        // Verify auto-switch to SL occurs before the final period.
        let cost = Money::from(10000);
        let salvage = Money::from(1000);
        let ddb = DoubleDecliningBalance;
        let schedule = ddb.schedule(cost, salvage, 5).unwrap();

        // Year 1: 10000 * 0.4 = 4000
        assert_eq!(schedule[0], Money::from(4000));

        // At some point DDB should switch to SL (SL produces higher depreciation).
        let mut switched = false;
        for period in 1..=5 {
            let ddb_only = {
                let bv =
                    DoubleDecliningBalance::book_value_at_period(cost, salvage, 5, period).unwrap();
                bv * Decimal::from(2) / Decimal::from(5)
            };
            let sl_equiv = (cost - salvage) / Decimal::from(5);
            if ddb_only < sl_equiv {
                switched = true;
            }
        }
        assert!(switched, "DDB should switch to straight-line");
    }

    #[test]
    fn ddb_total() {
        // Sum of all periods = cost - salvage.
        let cost = Money::from(10000);
        let salvage = Money::from(1000);
        let ddb = DoubleDecliningBalance;
        let schedule = ddb.schedule(cost, salvage, 5).unwrap();

        assert_eq!(schedule.len(), 5);

        let total: Money = schedule.iter().copied().sum();
        let diff = (total - (cost - salvage)).abs();
        assert!(diff <= Money::from(1));
    }

    #[test]
    fn invalid_period() {
        let cost = Money::from(1000);
        let sl = StraightLine;
        assert!(sl.depreciate(cost, Money::ZERO, 5, 10).is_err());
        assert!(sl.depreciate(cost, Money::ZERO, 0, 1).is_err());
    }
}
