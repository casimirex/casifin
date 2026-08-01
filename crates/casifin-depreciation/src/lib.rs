//! Depreciation calculations (Straight-Line, Double-Declining Balance) for casifin.

#![deny(warnings)]

use casifin_core::{CasifinError, Money};
use rust_decimal::Decimal;

/// A depreciation method trait.
pub trait DepreciationMethod {
    /// Computes depreciation for a specific period.
    ///
    /// # Arguments
    /// * `cost` - The initial cost of the asset
    /// * `salvage` - The salvage/residual value
    /// * `life_years` - The useful life in years
    /// * `period` - The period number (1-indexed)
    fn depreciate(
        &self,
        cost: Money,
        salvage: Money,
        life_years: u32,
        period: u32,
    ) -> Result<Money, CasifinError>;

    /// Generates a full depreciation schedule.
    fn schedule(
        &self,
        cost: Money,
        salvage: Money,
        life_years: u32,
    ) -> Result<Vec<Money>, CasifinError> {
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
        if period > life_years {
            return Err(CasifinError::InvalidPeriod(period));
        }

        debug_assert!(cost >= salvage, "cost must be >= salvage");
        debug_assert!(life_years > 0, "life_years must be positive");
        debug_assert!(period > 0, "period must be positive");
        debug_assert!(period <= life_years, "period must not exceed life_years");

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
#[derive(Debug, Clone, Copy, Default)]
pub struct DoubleDecliningBalance;

impl DoubleDecliningBalance {
    /// Computes the DDB rate.
    fn rate(life_years: u32) -> Decimal {
        Decimal::from(2) / Decimal::from(life_years)
    }
}

impl DoubleDecliningBalance {
    /// Computes the book value at the start of the given period.
    fn book_value_at_period(cost: Money, salvage: Money, life_years: u32, period: u32) -> Money {
        let mut book_value = cost;
        for _p in 1..period {
            let ddb_rate = Self::rate(life_years);
            let ddb_dep = book_value * ddb_rate;
            let remaining = book_value - salvage;
            let actual_dep = if ddb_dep > remaining {
                remaining
            } else {
                ddb_dep
            };
            book_value = book_value - actual_dep;
        }
        book_value
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
        if period > life_years {
            return Err(CasifinError::InvalidPeriod(period));
        }

        debug_assert!(cost >= salvage, "cost must be >= salvage");
        debug_assert!(life_years > 0, "life_years must be positive");
        debug_assert!(period > 0, "period must be positive");
        debug_assert!(period <= life_years, "period must not exceed life_years");

        let book_value = Self::book_value_at_period(cost, salvage, life_years, period);

        let ddb_rate = Self::rate(life_years);
        let ddb_depreciation = book_value * ddb_rate;
        let remaining = book_value - salvage;
        let ddb_limited = if ddb_depreciation > remaining {
            remaining
        } else {
            ddb_depreciation
        };

        // Switch to SL if SL produces higher depreciation
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
    fn test_straight_line() {
        let cost = Money::from(10000);
        let salvage = Money::from(1000);
        let life = 5u32;

        let sl = StraightLine;
        let annual = sl.depreciate(cost, salvage, life, 1).unwrap();

        // (10000 - 1000) / 5 = 1800
        assert_eq!(annual, Money::from(1800));

        let schedule = sl.schedule(cost, salvage, life).unwrap();
        assert_eq!(schedule.len(), 5);

        // Total depreciation should equal depreciable base
        let total: Money = schedule.iter().copied().sum();
        assert_eq!(total, cost - salvage);
    }

    #[test]
    fn test_double_declining_balance() {
        let cost = Money::from(10000);
        let salvage = Money::from(1000);
        let life = 5u32;

        let ddb = DoubleDecliningBalance;

        // Year 1: 10000 * (2/5) = 4000
        let year1 = ddb.depreciate(cost, salvage, life, 1).unwrap();
        assert_eq!(year1, Money::from(4000));

        let schedule = ddb.schedule(cost, salvage, life).unwrap();
        assert_eq!(schedule.len(), 5);

        // Total should approach depreciable base
        let total: Money = schedule.iter().copied().sum();
        assert!(total >= cost - salvage - Money::from(1));
        assert!(total <= cost - salvage + Money::from(1));
    }

    #[test]
    fn test_ddb_switches_to_sl() {
        let cost = Money::from(10000);
        let salvage = Money::from(1000);
        let life = 5u32;

        let ddb = DoubleDecliningBalance;
        let schedule = ddb.schedule(cost, salvage, life).unwrap();

        // Should have 5 periods
        assert_eq!(schedule.len(), 5);

        // Total should equal depreciable base (cost - salvage)
        let total: Money = schedule.iter().copied().sum();
        let expected = cost - salvage;
        let diff = (total - expected).abs();
        assert!(diff <= Money::from(10));
    }

    #[test]
    fn test_invalid_inputs() {
        // Test zero life (invalid)
        let cost = Money::from(1000);
        let zero_life = StraightLine;
        assert!(zero_life.depreciate(cost, Money::ZERO, 0, 1).is_err());

        // Test period > life (invalid)
        assert!(zero_life.depreciate(cost, Money::ZERO, 5, 10).is_err());
    }
}
