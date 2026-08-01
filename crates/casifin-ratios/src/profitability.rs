//! Profitability ratio calculations.

use casifin_core::{CasifinError, Money};
use rust_decimal::Decimal;

/// Gross Profit Margin.
///
/// # Formula
/// ```text
/// Gross Profit Margin = Gross Profit / Revenue
/// ```
///
/// # Arguments
/// * `gross_profit` - Gross profit.
/// * `revenue` - Total revenue.
///
/// # Returns
/// `Ok(Decimal)` containing the gross profit margin, or `Err(CasifinError::DivisionByZero)`
/// when `revenue` is zero.
///
/// # Example
/// ```
/// use casifin_core::Money;
/// use casifin_ratios::gross_profit_margin;
///
/// let margin = gross_profit_margin(Money::from(300_000), Money::from(1_000_000)).unwrap();
/// assert_eq!(margin, rust_decimal::Decimal::new(3, 1));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Financial Statement Analysis, Reading 25.
///
/// # Panics
/// This function does not panic.
pub fn gross_profit_margin(gross_profit: Money, revenue: Money) -> Result<Decimal, CasifinError> {
    debug_assert!(!revenue.is_zero(), "revenue must not be zero");
    debug_assert!(
        gross_profit >= Money::ZERO && revenue >= Money::ZERO,
        "gross_profit and revenue must be non-negative"
    );

    if revenue.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "gross_profit_margin",
        });
    }

    gross_profit
        .inner()
        .checked_div(revenue.inner())
        .ok_or(CasifinError::DivisionByZero {
            operation: "gross_profit_margin",
        })
}

/// Operating Profit Margin.
///
/// # Formula
/// ```text
/// Operating Profit Margin = Operating Income / Revenue
/// ```
///
/// # Arguments
/// * `operating_income` - Operating income.
/// * `revenue` - Total revenue.
///
/// # Returns
/// `Ok(Decimal)` containing the operating profit margin, or `Err(CasifinError::DivisionByZero)`
/// when `revenue` is zero.
///
/// # Example
/// ```
/// use casifin_core::Money;
/// use casifin_ratios::operating_profit_margin;
///
/// let margin = operating_profit_margin(Money::from(200_000), Money::from(1_000_000)).unwrap();
/// assert_eq!(margin, rust_decimal::Decimal::new(2, 1));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Financial Statement Analysis, Reading 25.
///
/// # Panics
/// This function does not panic.
pub fn operating_profit_margin(
    operating_income: Money,
    revenue: Money,
) -> Result<Decimal, CasifinError> {
    debug_assert!(!revenue.is_zero(), "revenue must not be zero");
    debug_assert!(
        operating_income >= Money::ZERO && revenue >= Money::ZERO,
        "operating_income and revenue must be non-negative"
    );

    if revenue.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "operating_profit_margin",
        });
    }

    operating_income
        .inner()
        .checked_div(revenue.inner())
        .ok_or(CasifinError::DivisionByZero {
            operation: "operating_profit_margin",
        })
}

/// Net Profit Margin.
///
/// # Formula
/// ```text
/// Net Profit Margin = Net Income / Revenue
/// ```
///
/// # Arguments
/// * `net_income` - Net income.
/// * `revenue` - Total revenue.
///
/// # Returns
/// `Ok(Decimal)` containing the net profit margin, or `Err(CasifinError::DivisionByZero)`
/// when `revenue` is zero.
///
/// # Example
/// ```
/// use casifin_core::Money;
/// use casifin_ratios::net_profit_margin;
///
/// let margin = net_profit_margin(Money::from(100_000), Money::from(1_000_000)).unwrap();
/// assert_eq!(margin, rust_decimal::Decimal::new(1, 1));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Financial Statement Analysis, Reading 25.
///
/// # Panics
/// This function does not panic.
pub fn net_profit_margin(net_income: Money, revenue: Money) -> Result<Decimal, CasifinError> {
    debug_assert!(!revenue.is_zero(), "revenue must not be zero");
    debug_assert!(
        net_income >= Money::ZERO && revenue >= Money::ZERO,
        "net_income and revenue must be non-negative"
    );

    if revenue.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "net_profit_margin",
        });
    }

    net_income
        .inner()
        .checked_div(revenue.inner())
        .ok_or(CasifinError::DivisionByZero {
            operation: "net_profit_margin",
        })
}

/// Return on Assets.
///
/// # Formula
/// ```text
/// Return on Assets = Net Income / Total Assets
/// ```
///
/// # Arguments
/// * `net_income` - Net income.
/// * `total_assets` - Total assets.
///
/// # Returns
/// `Ok(Decimal)` containing ROA, or `Err(CasifinError::DivisionByZero)` when `total_assets`
/// is zero.
///
/// # Example
/// ```
/// use casifin_core::Money;
/// use casifin_ratios::return_on_assets;
///
/// let roa = return_on_assets(Money::from(100_000), Money::from(1_000_000)).unwrap();
/// assert_eq!(roa, rust_decimal::Decimal::new(1, 1));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Financial Statement Analysis, Reading 25.
///
/// # Panics
/// This function does not panic.
pub fn return_on_assets(net_income: Money, total_assets: Money) -> Result<Decimal, CasifinError> {
    debug_assert!(!total_assets.is_zero(), "total_assets must not be zero");
    debug_assert!(
        net_income >= Money::ZERO && total_assets >= Money::ZERO,
        "net_income and total_assets must be non-negative"
    );

    if total_assets.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "return_on_assets",
        });
    }

    net_income
        .inner()
        .checked_div(total_assets.inner())
        .ok_or(CasifinError::DivisionByZero {
            operation: "return_on_assets",
        })
}

/// Return on Equity.
///
/// # Formula
/// ```text
/// Return on Equity = Net Income / Total Equity
/// ```
///
/// # Arguments
/// * `net_income` - Net income.
/// * `total_equity` - Total shareholders' equity.
///
/// # Returns
/// `Ok(Decimal)` containing ROE, or `Err(CasifinError::DivisionByZero)` when `total_equity`
/// is zero.
///
/// # Example
/// ```
/// use casifin_core::Money;
/// use casifin_ratios::return_on_equity;
///
/// let roe = return_on_equity(Money::from(100_000), Money::from(500_000)).unwrap();
/// assert_eq!(roe, rust_decimal::Decimal::new(2, 1));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Financial Statement Analysis, Reading 25.
///
/// # Panics
/// This function does not panic.
pub fn return_on_equity(net_income: Money, total_equity: Money) -> Result<Decimal, CasifinError> {
    debug_assert!(!total_equity.is_zero(), "total_equity must not be zero");
    debug_assert!(
        net_income >= Money::ZERO && total_equity >= Money::ZERO,
        "net_income and total_equity must be non-negative"
    );

    if total_equity.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "return_on_equity",
        });
    }

    net_income
        .inner()
        .checked_div(total_equity.inner())
        .ok_or(CasifinError::DivisionByZero {
            operation: "return_on_equity",
        })
}

/// Basic Earnings Per Share.
///
/// # Formula
/// ```text
/// Basic EPS = (Net Income - Preferred Dividends) / Shares Outstanding
/// ```
///
/// # Arguments
/// * `net_income` - Net income.
/// * `preferred_dividends` - Preferred dividends.
/// * `shares_outstanding` - Weighted average common shares outstanding.
///
/// # Returns
/// `Ok(Decimal)` containing basic EPS, or `Err(CasifinError::DivisionByZero)` when
/// `shares_outstanding` is zero.
///
/// # Example
/// ```
/// use casifin_core::Money;
/// use casifin_ratios::basic_eps;
/// use rust_decimal::Decimal;
///
/// let eps = basic_eps(Money::from(1_000_000), Money::from(100_000), Decimal::from(100_000)).unwrap();
/// assert_eq!(eps, Decimal::new(9, 0));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Financial Statement Analysis, Reading 25.
///
/// # Panics
/// This function does not panic.
pub fn basic_eps(
    net_income: Money,
    preferred_dividends: Money,
    shares_outstanding: Decimal,
) -> Result<Decimal, CasifinError> {
    debug_assert!(
        !shares_outstanding.is_zero(),
        "shares_outstanding must not be zero"
    );
    debug_assert!(
        shares_outstanding > Decimal::ZERO,
        "shares must be positive"
    );

    if shares_outstanding.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "basic_eps",
        });
    }

    let numerator = net_income
        .inner()
        .checked_sub(preferred_dividends.inner())
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "basic_eps: numerator overflow".to_string(),
        })?;

    numerator
        .checked_div(shares_outstanding)
        .ok_or(CasifinError::DivisionByZero {
            operation: "basic_eps",
        })
}

/// Diluted Earnings Per Share.
///
/// # Formula
/// ```text
/// Diluted EPS = (Net Income - Preferred Dividends)
///               / (Weighted Shares + Potential Shares)
/// ```
///
/// # Arguments
/// * `net_income` - Net income.
/// * `preferred_dividends` - Preferred dividends.
/// * `weighted_shares` - Weighted average common shares outstanding.
/// * `potential_shares` - Dilutive potential common shares.
///
/// # Returns
/// `Ok(Decimal)` containing diluted EPS, or `Err(CasifinError::DivisionByZero)` when the
/// total share count is zero.
///
/// # Example
/// ```
/// use casifin_core::Money;
/// use casifin_ratios::diluted_eps;
/// use rust_decimal::Decimal;
///
/// let eps = diluted_eps(
///     Money::from(1_000_000),
///     Money::from(100_000),
///     Decimal::from(100_000),
///     Decimal::from(25_000),
/// ).unwrap();
/// assert_eq!(eps, Decimal::new(72, 1));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Financial Statement Analysis, Reading 25.
///
/// # Panics
/// This function does not panic.
pub fn diluted_eps(
    net_income: Money,
    preferred_dividends: Money,
    weighted_shares: Decimal,
    potential_shares: Decimal,
) -> Result<Decimal, CasifinError> {
    let total_shares =
        weighted_shares
            .checked_add(potential_shares)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "diluted_eps: share count overflow".to_string(),
            })?;

    debug_assert!(
        !total_shares.is_zero(),
        "total diluted shares must not be zero"
    );
    debug_assert!(
        total_shares > Decimal::ZERO,
        "total diluted shares must be positive"
    );

    if total_shares.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "diluted_eps",
        });
    }

    let numerator = net_income
        .inner()
        .checked_sub(preferred_dividends.inner())
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "diluted_eps: numerator overflow".to_string(),
        })?;

    numerator
        .checked_div(total_shares)
        .ok_or(CasifinError::DivisionByZero {
            operation: "diluted_eps",
        })
}

#[cfg(test)]
mod tests {
    use casifin_core::Money;
    use rust_decimal::Decimal;

    use super::*;

    #[test]
    fn gross_profit_margin_known_value() {
        let margin = gross_profit_margin(Money::from(300_000), Money::from(1_000_000)).unwrap();
        assert_eq!(margin, Decimal::new(3, 1));
    }

    #[test]
    fn operating_profit_margin_known_value() {
        let margin = operating_profit_margin(Money::from(200_000), Money::from(1_000_000)).unwrap();
        assert_eq!(margin, Decimal::new(2, 1));
    }

    #[test]
    fn net_profit_margin_known_value() {
        let margin = net_profit_margin(Money::from(100_000), Money::from(1_000_000)).unwrap();
        assert_eq!(margin, Decimal::new(1, 1));
    }

    #[test]
    fn return_on_assets_known_value() {
        let roa = return_on_assets(Money::from(100_000), Money::from(1_000_000)).unwrap();
        assert_eq!(roa, Decimal::new(1, 1));
    }

    #[test]
    fn return_on_equity_known_value() {
        let roe = return_on_equity(Money::from(100_000), Money::from(500_000)).unwrap();
        assert_eq!(roe, Decimal::new(2, 1));
    }

    #[test]
    fn basic_eps_known_value() {
        let eps = basic_eps(
            Money::from(1_000_000),
            Money::from(100_000),
            Decimal::from(100_000),
        )
        .unwrap();
        assert_eq!(eps, Decimal::new(9, 0));
    }

    #[test]
    fn diluted_eps_known_value() {
        let eps = diluted_eps(
            Money::from(1_000_000),
            Money::from(100_000),
            Decimal::from(100_000),
            Decimal::from(25_000),
        )
        .unwrap();
        assert_eq!(eps, Decimal::new(72, 1));
    }
}
