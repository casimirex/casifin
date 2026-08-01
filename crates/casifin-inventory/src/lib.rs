//! Inventory costing methods (FIFO, LIFO, Weighted Average) for casifin.

#![deny(warnings)]

use casifin_core::{CasifinError, Money};
use chrono::NaiveDate;
use rust_decimal::Decimal;

/// An inventory lot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InventoryLot {
    /// Number of units in the lot.
    pub units: u32,
    /// Cost per unit.
    pub unit_cost: Money,
    /// Date the lot was acquired.
    pub date: NaiveDate,
}

impl InventoryLot {
    /// Creates a new inventory lot.
    pub fn new(units: u32, unit_cost: Money, date: NaiveDate) -> Self {
        InventoryLot {
            units,
            unit_cost,
            date,
        }
    }

    /// Returns the total value of the lot.
    pub fn total_value(&self) -> Money {
        self.unit_cost * Decimal::from(self.units)
    }
}

/// An inventory costing method trait.
pub trait InventoryMethod {
    /// Computes Cost of Goods Sold (COGS).
    ///
    /// # Arguments
    /// * `lots` - The inventory lots available
    /// * `units_sold` - Number of units sold
    fn cogs(&self, lots: &[InventoryLot], units_sold: u32) -> Result<Money, CasifinError>;

    /// Computes ending inventory value.
    ///
    /// # Arguments
    /// * `lots` - The inventory lots available
    /// * `units_sold` - Number of units sold
    fn ending_inventory(
        &self,
        lots: &[InventoryLot],
        units_sold: u32,
    ) -> Result<Money, CasifinError> {
        let total_units: u32 = lots.iter().map(|l| l.units).sum();
        let total_value: Money = lots.iter().map(|l| l.total_value()).sum();

        if units_sold > total_units {
            return Err(CasifinError::InventoryError(format!(
                "units_sold ({}) exceeds available inventory ({})",
                units_sold, total_units
            )));
        }

        let cogs = self.cogs(lots, units_sold)?;
        Ok(total_value - cogs)
    }
}

/// First-In, First-Out (FIFO) inventory method.
///
/// Assumes the oldest inventory items are sold first.
///
/// # Formula
/// ```text
/// COGS = Σ (units_from_lot_i * cost_i) for i = oldest..newest
/// ```
/// Lots are sorted by acquisition date (oldest first), and units are drawn
/// from the oldest lots until the sold quantity is satisfied.
#[derive(Debug, Clone, Copy, Default)]
pub struct Fifo;

impl InventoryMethod for Fifo {
    fn cogs(&self, lots: &[InventoryLot], units_sold: u32) -> Result<Money, CasifinError> {
        if lots.is_empty() {
            return Err(CasifinError::InventoryError(
                "no inventory lots available".to_string(),
            ));
        }

        let total_units: u32 = lots.iter().map(|l| l.units).sum();
        if units_sold > total_units {
            return Err(CasifinError::InventoryError(format!(
                "units_sold ({}) exceeds available inventory ({})",
                units_sold, total_units
            )));
        }

        debug_assert!(!lots.is_empty(), "lots must not be empty");
        debug_assert!(units_sold > 0, "units_sold must be positive");

        // Sort lots by date (oldest first)
        let mut sorted_lots: Vec<InventoryLot> = lots.to_vec();
        sorted_lots.sort_by_key(|l| l.date);

        let mut remaining = units_sold;
        let mut cogs = Money::ZERO;

        for lot in sorted_lots {
            if remaining == 0 {
                break;
            }

            let units_from_lot = remaining.min(lot.units);
            cogs = cogs + lot.unit_cost * Decimal::from(units_from_lot);
            remaining -= units_from_lot;
        }

        Ok(cogs)
    }
}

/// Last-In, First-Out (LIFO) inventory method.
///
/// Assumes the newest inventory items are sold first.
///
/// # Formula
/// ```text
/// COGS = Σ (units_from_lot_i * cost_i) for i = newest..oldest
/// ```
/// Lots are sorted by acquisition date (newest first), and units are drawn
/// from the newest lots until the sold quantity is satisfied.
#[derive(Debug, Clone, Copy, Default)]
pub struct Lifo;

impl InventoryMethod for Lifo {
    fn cogs(&self, lots: &[InventoryLot], units_sold: u32) -> Result<Money, CasifinError> {
        if lots.is_empty() {
            return Err(CasifinError::InventoryError(
                "no inventory lots available".to_string(),
            ));
        }

        let total_units: u32 = lots.iter().map(|l| l.units).sum();
        if units_sold > total_units {
            return Err(CasifinError::InventoryError(format!(
                "units_sold ({}) exceeds available inventory ({})",
                units_sold, total_units
            )));
        }

        debug_assert!(!lots.is_empty(), "lots must not be empty");
        debug_assert!(units_sold > 0, "units_sold must be positive");

        // Sort lots by date descending (newest first)
        let mut sorted_lots: Vec<InventoryLot> = lots.to_vec();
        sorted_lots.sort_by_key(|l| l.date);
        sorted_lots.reverse();

        let mut remaining = units_sold;
        let mut cogs = Money::ZERO;

        for lot in sorted_lots {
            if remaining == 0 {
                break;
            }

            let units_from_lot = remaining.min(lot.units);
            cogs = cogs + lot.unit_cost * Decimal::from(units_from_lot);
            remaining -= units_from_lot;
        }

        Ok(cogs)
    }
}

/// Weighted Average inventory method.
///
/// Uses the average cost of all available units.
///
/// # Formula
/// ```text
/// Avg Cost = Total Value / Total Units
/// COGS = Avg Cost * Units Sold
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct WeightedAverage;

impl InventoryMethod for WeightedAverage {
    fn cogs(&self, lots: &[InventoryLot], units_sold: u32) -> Result<Money, CasifinError> {
        if lots.is_empty() {
            return Err(CasifinError::InventoryError(
                "no inventory lots available".to_string(),
            ));
        }

        let total_units: u32 = lots.iter().map(|l| l.units).sum();
        if units_sold > total_units {
            return Err(CasifinError::InventoryError(format!(
                "units_sold ({}) exceeds available inventory ({})",
                units_sold, total_units
            )));
        }

        debug_assert!(!lots.is_empty(), "lots must not be empty");
        debug_assert!(units_sold > 0, "units_sold must be positive");

        let total_value: Money = lots.iter().map(|l| l.total_value()).sum();
        let total_units_dec = Decimal::from(total_units);

        let avg_cost = total_value.checked_div_decimal(total_units_dec).ok_or(
            CasifinError::DivisionByZero {
                operation: "weighted_average_cost",
            },
        )?;

        Ok(avg_cost * Decimal::from(units_sold))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_lots() -> [InventoryLot; 3] {
        [
            InventoryLot::new(
                100,
                Money::from(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            ),
            InventoryLot::new(
                100,
                Money::from(12),
                NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            ),
            InventoryLot::new(
                100,
                Money::from(14),
                NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            ),
        ]
    }

    #[test]
    fn test_fifo_cogs() {
        let lots = test_lots();
        let fifo = Fifo;

        // Sell 150 units: 100 @ $10 + 50 @ $12 = $1600
        let cogs = fifo.cogs(&lots, 150).unwrap();
        assert_eq!(cogs, Money::from(1600));

        let ending = fifo.ending_inventory(&lots, 150).unwrap();
        // Remaining: 50 @ $12 + 100 @ $14 = $2000
        assert_eq!(ending, Money::from(2000));
    }

    #[test]
    fn test_lifo_cogs() {
        let lots = test_lots();
        let lifo = Lifo;

        // Sell 150 units: 100 @ $14 + 50 @ $12 = $2000
        let cogs = lifo.cogs(&lots, 150).unwrap();
        assert_eq!(cogs, Money::from(2000));

        let ending = lifo.ending_inventory(&lots, 150).unwrap();
        // Remaining: 50 @ $12 + 100 @ $10 = $1600
        assert_eq!(ending, Money::from(1600));
    }

    #[test]
    fn test_weighted_average_cogs() {
        let lots = test_lots();
        let wa = WeightedAverage;

        // Total: 300 units, total value = 1000 + 1200 + 1400 = 3600
        // Avg cost = $12
        // COGS for 150 units = 150 * $12 = $1800
        let cogs = wa.cogs(&lots, 150).unwrap();
        assert_eq!(cogs, Money::from(1800));

        let ending = wa.ending_inventory(&lots, 150).unwrap();
        // Remaining: 150 * $12 = $1800
        assert_eq!(ending, Money::from(1800));
    }

    #[test]
    fn test_exceeds_inventory() {
        let lots = test_lots();
        let fifo = Fifo;

        assert!(fifo.cogs(&lots, 500).is_err());
    }

    #[test]
    fn test_empty_lots() {
        let lots: [InventoryLot; 0] = [];
        let fifo = Fifo;

        assert!(fifo.cogs(&lots, 10).is_err());
    }
}
