//! Solvency and leverage ratio calculations.

use casifin_core::{CasifinError, Money};
use rust_decimal::Decimal;

/// Debt Ratio.
///
/// # Formula
/// ```text
/// Debt Ratio = Total Debt / Total Assets
/// ```
///
/// # Arguments
/// * `total_debt` - Total debt.
/// * `total_assets` - Total assets.
///
/// # Returns
/// `Ok(Decimal)` containing the debt ratio, or `Err(CasifinError::DivisionByZero)`
/// when `total_assets` is zero.
///
/// # Example
/// ```
/// use casifin_core::Money;
/// use casifin_ratios::debt_ratio;
///
/// let ratio = debt_ratio(Money::from(200_000), Money::from(500_000)).unwrap();
/// assert_eq!(ratio, rust_decimal::Decimal::new(4, 1));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Financial Statement Analysis, Reading 24.
///
/// # Panics
/// This function does not panic.
pub fn debt_ratio(total_debt: Money, total_assets: Money) -> Result<Decimal, CasifinError> {
    debug_assert!(!total_assets.is_zero(), "total_assets must not be zero");
    debug_assert!(
        total_debt >= Money::ZERO && total_assets >= Money::ZERO,
        "debt and assets must be non-negative"
    );

    if total_assets.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "debt_ratio",
        });
    }

    total_debt
        .inner()
        .checked_div(total_assets.inner())
        .ok_or(CasifinError::DivisionByZero {
            operation: "debt_ratio",
        })
}

/// Debt-to-Equity Ratio.
///
/// # Formula
/// ```text
/// Debt-to-Equity = Total Debt / Total Equity
/// ```
///
/// # Arguments
/// * `total_debt` - Total debt.
/// * `total_equity` - Total shareholders' equity.
///
/// # Returns
/// `Ok(Decimal)` containing the debt-to-equity ratio, or `Err(CasifinError::DivisionByZero)`
/// when `total_equity` is zero.
///
/// # Example
/// ```
/// use casifin_core::Money;
/// use casifin_ratios::debt_to_equity;
///
/// let ratio = debt_to_equity(Money::from(300_000), Money::from(200_000)).unwrap();
/// assert_eq!(ratio, rust_decimal::Decimal::new(15, 1));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Financial Statement Analysis, Reading 24.
///
/// # Panics
/// This function does not panic.
pub fn debt_to_equity(total_debt: Money, total_equity: Money) -> Result<Decimal, CasifinError> {
    debug_assert!(!total_equity.is_zero(), "total_equity must not be zero");
    debug_assert!(
        total_debt >= Money::ZERO && total_equity >= Money::ZERO,
        "debt and equity must be non-negative"
    );

    if total_equity.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "debt_to_equity",
        });
    }

    total_debt
        .inner()
        .checked_div(total_equity.inner())
        .ok_or(CasifinError::DivisionByZero {
            operation: "debt_to_equity",
        })
}

/// Financial Leverage.
///
/// # Formula
/// ```text
/// Financial Leverage = Total Assets / Total Equity
/// ```
///
/// # Arguments
/// * `total_assets` - Total assets.
/// * `total_equity` - Total shareholders' equity.
///
/// # Returns
/// `Ok(Decimal)` containing the leverage ratio, or `Err(CasifinError::DivisionByZero)`
/// when `total_equity` is zero.
///
/// # Example
/// ```
/// use casifin_core::Money;
/// use casifin_ratios::financial_leverage;
///
/// let leverage = financial_leverage(Money::from(500_000), Money::from(200_000)).unwrap();
/// assert_eq!(leverage, rust_decimal::Decimal::new(25, 1));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Financial Statement Analysis, Reading 24.
///
/// # Panics
/// This function does not panic.
pub fn financial_leverage(
    total_assets: Money,
    total_equity: Money,
) -> Result<Decimal, CasifinError> {
    debug_assert!(!total_equity.is_zero(), "total_equity must not be zero");
    debug_assert!(
        total_assets >= Money::ZERO && total_equity >= Money::ZERO,
        "assets and equity must be non-negative"
    );

    if total_equity.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "financial_leverage",
        });
    }

    total_assets
        .inner()
        .checked_div(total_equity.inner())
        .ok_or(CasifinError::DivisionByZero {
            operation: "financial_leverage",
        })
}

/// Interest Coverage Ratio.
///
/// # Formula
/// ```text
/// Interest Coverage = EBIT / Interest Expense
/// ```
///
/// # Arguments
/// * `ebit` - Earnings before interest and taxes.
/// * `interest_expense` - Interest expense.
///
/// # Returns
/// `Ok(Decimal)` containing the interest coverage ratio, or `Err(CasifinError::DivisionByZero)`
/// when `interest_expense` is zero.
///
/// # Example
/// ```
/// use casifin_core::Money;
/// use casifin_ratios::interest_coverage;
///
/// let coverage = interest_coverage(Money::from(200_000), Money::from(50_000)).unwrap();
/// assert_eq!(coverage, rust_decimal::Decimal::from(4));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Financial Statement Analysis, Reading 24.
///
/// # Panics
/// This function does not panic.
pub fn interest_coverage(ebit: Money, interest_expense: Money) -> Result<Decimal, CasifinError> {
    debug_assert!(
        !interest_expense.is_zero(),
        "interest_expense must not be zero"
    );
    debug_assert!(
        ebit >= Money::ZERO && interest_expense >= Money::ZERO,
        "ebit and interest_expense must be non-negative"
    );

    if interest_expense.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "interest_coverage",
        });
    }

    ebit.inner()
        .checked_div(interest_expense.inner())
        .ok_or(CasifinError::DivisionByZero {
            operation: "interest_coverage",
        })
}

#[cfg(test)]
mod tests {
    use casifin_core::Money;
    use rust_decimal::Decimal;

    use super::*;

    #[test]
    fn debt_ratio_known_value() {
        let ratio = debt_ratio(Money::from(200_000), Money::from(500_000)).unwrap();
        assert_eq!(ratio, Decimal::new(4, 1));
    }

    #[test]
    fn debt_to_equity_known_value() {
        let ratio = debt_to_equity(Money::from(300_000), Money::from(200_000)).unwrap();
        assert_eq!(ratio, Decimal::new(15, 1));
    }

    #[test]
    fn financial_leverage_known_value() {
        let leverage = financial_leverage(Money::from(500_000), Money::from(200_000)).unwrap();
        assert_eq!(leverage, Decimal::new(25, 1));
    }

    #[test]
    fn interest_coverage_known_value() {
        let coverage = interest_coverage(Money::from(200_000), Money::from(50_000)).unwrap();
        assert_eq!(coverage, Decimal::from(4));
    }
}
