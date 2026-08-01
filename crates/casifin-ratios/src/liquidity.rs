//! Liquidity ratio calculations.

use casifin_core::{CasifinError, Money};
use rust_decimal::Decimal;

/// Current Ratio.
///
/// # Formula
/// ```text
/// Current Ratio = Current Assets / Current Liabilities
/// ```
///
/// # Arguments
/// * `current_assets` - Total current assets.
/// * `current_liabilities` - Total current liabilities.
///
/// # Returns
/// `Ok(Decimal)` containing the current ratio, or `Err(CasifinError::DivisionByZero)`
/// when `current_liabilities` is zero.
///
/// # Example
/// ```
/// use casifin_core::Money;
/// use casifin_ratios::current_ratio;
///
/// let ratio = current_ratio(Money::from(500_000), Money::from(250_000)).unwrap();
/// assert_eq!(ratio, rust_decimal::Decimal::from(2));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Financial Statement Analysis, Reading 24.
///
/// # Panics
/// This function does not panic.
pub fn current_ratio(
    current_assets: Money,
    current_liabilities: Money,
) -> Result<Decimal, CasifinError> {
    debug_assert!(
        current_assets >= Money::ZERO,
        "current_assets must be non-negative"
    );
    debug_assert!(
        current_liabilities >= Money::ZERO,
        "current_liabilities must be non-negative"
    );

    if current_liabilities.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "current_ratio",
        });
    }

    current_assets
        .inner()
        .checked_div(current_liabilities.inner())
        .ok_or(CasifinError::DivisionByZero {
            operation: "current_ratio",
        })
}

/// Quick Ratio (Acid-Test Ratio).
///
/// # Formula
/// ```text
/// Quick Ratio = (Cash + Marketable Securities + Receivables) / Current Liabilities
/// ```
///
/// # Arguments
/// * `cash` - Cash and cash equivalents.
/// * `marketable_securities` - Short-term marketable securities.
/// * `receivables` - Accounts receivable.
/// * `current_liabilities` - Total current liabilities.
///
/// # Returns
/// `Ok(Decimal)` containing the quick ratio, or `Err(CasifinError::DivisionByZero)`
/// when `current_liabilities` is zero.
///
/// # Example
/// ```
/// use casifin_core::Money;
/// use casifin_ratios::quick_ratio;
///
/// let ratio = quick_ratio(
///     Money::from(50_000),
///     Money::from(30_000),
///     Money::from(20_000),
///     Money::from(50_000),
/// ).unwrap();
/// assert_eq!(ratio, rust_decimal::Decimal::new(2, 0));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Financial Statement Analysis, Reading 24.
///
/// # Panics
/// This function does not panic.
pub fn quick_ratio(
    cash: Money,
    marketable_securities: Money,
    receivables: Money,
    current_liabilities: Money,
) -> Result<Decimal, CasifinError> {
    debug_assert!(
        cash >= Money::ZERO && marketable_securities >= Money::ZERO && receivables >= Money::ZERO,
        "quick assets must be non-negative"
    );
    debug_assert!(
        current_liabilities >= Money::ZERO,
        "current_liabilities must be non-negative"
    );

    if current_liabilities.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "quick_ratio",
        });
    }

    let quick_assets = cash
        .inner()
        .checked_add(marketable_securities.inner())
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "quick_ratio: cash + marketable_securities overflow".to_string(),
        })?
        .checked_add(receivables.inner())
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "quick_ratio: quick assets sum overflow".to_string(),
        })?;

    quick_assets
        .checked_div(current_liabilities.inner())
        .ok_or(CasifinError::DivisionByZero {
            operation: "quick_ratio",
        })
}

/// Cash Ratio.
///
/// # Formula
/// ```text
/// Cash Ratio = Cash / Current Liabilities
/// ```
///
/// # Arguments
/// * `cash` - Cash and cash equivalents.
/// * `current_liabilities` - Total current liabilities.
///
/// # Returns
/// `Ok(Decimal)` containing the cash ratio, or `Err(CasifinError::DivisionByZero)`
/// when `current_liabilities` is zero.
///
/// # Example
/// ```
/// use casifin_core::Money;
/// use casifin_ratios::cash_ratio;
///
/// let ratio = cash_ratio(Money::from(100_000), Money::from(200_000)).unwrap();
/// assert_eq!(ratio, rust_decimal::Decimal::new(5, 1));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Financial Statement Analysis, Reading 24.
///
/// # Panics
/// This function does not panic.
pub fn cash_ratio(cash: Money, current_liabilities: Money) -> Result<Decimal, CasifinError> {
    debug_assert!(cash >= Money::ZERO, "cash must be non-negative");
    debug_assert!(
        current_liabilities >= Money::ZERO,
        "current_liabilities must be non-negative"
    );

    if current_liabilities.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "cash_ratio",
        });
    }

    cash.inner()
        .checked_div(current_liabilities.inner())
        .ok_or(CasifinError::DivisionByZero {
            operation: "cash_ratio",
        })
}

/// Defensive Interval.
///
/// # Formula
/// ```text
/// Defensive Interval = (Cash + Marketable Securities + Receivables)
///                      / Daily Cash Expenditures
/// ```
///
/// # Arguments
/// * `cash` - Cash and cash equivalents.
/// * `marketable_securities` - Short-term marketable securities.
/// * `receivables` - Accounts receivable.
/// * `daily_cash_expenditures` - Average daily cash outflows.
///
/// # Returns
/// `Ok(Decimal)` containing the number of days the company can operate without
/// cash inflows, or `Err(CasifinError::DivisionByZero)` when daily expenditures
/// are zero.
///
/// # Example
/// ```
/// use casifin_core::Money;
/// use casifin_ratios::defensive_interval;
///
/// let interval = defensive_interval(
///     Money::from(50_000),
///     Money::from(30_000),
///     Money::from(20_000),
///     Money::from(10_000),
/// ).unwrap();
/// assert_eq!(interval, rust_decimal::Decimal::from(10));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Financial Statement Analysis, Reading 24.
///
/// # Panics
/// This function does not panic.
pub fn defensive_interval(
    cash: Money,
    marketable_securities: Money,
    receivables: Money,
    daily_cash_expenditures: Money,
) -> Result<Decimal, CasifinError> {
    debug_assert!(
        cash >= Money::ZERO && marketable_securities >= Money::ZERO && receivables >= Money::ZERO,
        "liquid assets must be non-negative"
    );
    debug_assert!(
        daily_cash_expenditures >= Money::ZERO,
        "daily_cash_expenditures must be non-negative"
    );

    if daily_cash_expenditures.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "defensive_interval",
        });
    }

    let liquid_assets = cash
        .inner()
        .checked_add(marketable_securities.inner())
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "defensive_interval: liquid assets sum overflow".to_string(),
        })?
        .checked_add(receivables.inner())
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "defensive_interval: liquid assets sum overflow".to_string(),
        })?;

    liquid_assets
        .checked_div(daily_cash_expenditures.inner())
        .ok_or(CasifinError::DivisionByZero {
            operation: "defensive_interval",
        })
}

#[cfg(test)]
mod tests {
    use casifin_core::Money;
    use rust_decimal::Decimal;

    use super::*;

    #[test]
    fn current_ratio_known_value() {
        let ratio = current_ratio(Money::from(500_000), Money::from(250_000)).unwrap();
        assert_eq!(ratio, Decimal::from(2));
    }

    #[test]
    fn quick_ratio_known_value() {
        let ratio = quick_ratio(
            Money::from(50_000),
            Money::from(30_000),
            Money::from(20_000),
            Money::from(50_000),
        )
        .unwrap();
        assert_eq!(ratio, Decimal::new(2, 0));
    }

    #[test]
    fn cash_ratio_known_value() {
        let ratio = cash_ratio(Money::from(100_000), Money::from(200_000)).unwrap();
        assert_eq!(ratio, Decimal::new(5, 1));
    }

    #[test]
    fn defensive_interval_known_value() {
        let interval = defensive_interval(
            Money::from(50_000),
            Money::from(30_000),
            Money::from(20_000),
            Money::from(10_000),
        )
        .unwrap();
        assert_eq!(interval, Decimal::from(10));
    }

    #[test]
    fn current_ratio_zero_liabilities_errors() {
        let result = current_ratio(Money::from(100), Money::ZERO);
        assert!(matches!(result, Err(CasifinError::DivisionByZero { .. })));
    }
}
