//! Return metric calculations.

use casifin_cashflow::{CashFlow, CashFlowStream};
use casifin_core::{CasifinError, Config, Money};
use chrono::NaiveDate;
use rust_decimal::{Decimal, MathematicalOps};

/// Holding Period Return.
///
/// # Formula
/// ```text
/// HPR = (Sale Price - Purchase Price + Income) / Purchase Price
/// ```
///
/// # Arguments
/// * `purchase_price` - Initial investment or purchase price.
/// * `sale_price` - Proceeds from sale.
/// * `income` - Dividends or other income received.
///
/// # Returns
/// `Ok(Decimal)` containing the holding period return, or `Err(CasifinError::DivisionByZero)`
/// when `purchase_price` is zero.
///
/// # Example
/// ```
/// use casifin_core::Money;
/// use casifin_ratios::holding_period_return;
///
/// let hpr = holding_period_return(
///     Money::from(100),
///     Money::from(115),
///     Money::from(5),
/// ).unwrap();
/// assert_eq!(hpr, rust_decimal::Decimal::new(2, 1));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Quantitative Methods, Reading 6.
///
/// # Panics
/// This function does not panic.
pub fn holding_period_return(
    purchase_price: Money,
    sale_price: Money,
    income: Money,
) -> Result<Decimal, CasifinError> {
    debug_assert!(!purchase_price.is_zero(), "purchase_price must not be zero");
    debug_assert!(
        purchase_price >= Money::ZERO && sale_price >= Money::ZERO && income >= Money::ZERO,
        "prices and income must be non-negative"
    );

    if purchase_price.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "holding_period_return",
        });
    }

    let gain = sale_price
        .inner()
        .checked_sub(purchase_price.inner())
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "holding_period_return: gain overflow".to_string(),
        })?
        .checked_add(income.inner())
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "holding_period_return: gain overflow".to_string(),
        })?;

    gain.checked_div(purchase_price.inner())
        .ok_or(CasifinError::DivisionByZero {
            operation: "holding_period_return",
        })
}

/// Arithmetic Mean Return.
///
/// # Formula
/// ```text
/// AM = (r1 + r2 + ... + rn) / n
/// ```
///
/// # Arguments
/// * `returns` - Slice of periodic returns expressed as decimals.
///
/// # Returns
/// `Ok(Decimal)` containing the arithmetic mean, or `Err(CasifinError::InsufficientCashFlows)`
/// when `returns` is empty.
///
/// # Example
/// ```
/// use casifin_ratios::arithmetic_mean_return;
/// use rust_decimal::Decimal;
///
/// let returns = vec![Decimal::new(10, 2), Decimal::new(20, 2), Decimal::new(30, 2)];
/// let mean = arithmetic_mean_return(&returns).unwrap();
/// assert_eq!(mean, Decimal::new(2, 1));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Quantitative Methods, Reading 6.
///
/// # Panics
/// This function does not panic.
pub fn arithmetic_mean_return(returns: &[Decimal]) -> Result<Decimal, CasifinError> {
    debug_assert!(!returns.is_empty(), "returns must not be empty");
    debug_assert!(
        returns.len() <= u32::MAX as usize,
        "returns length must fit in u32"
    );

    if returns.is_empty() {
        return Err(CasifinError::InsufficientCashFlows);
    }

    let mut sum = Decimal::ZERO;
    for r in returns {
        sum = sum.checked_add(*r).ok_or(CasifinError::ScheduleOverflow {
            detail: "arithmetic_mean_return: sum overflow".to_string(),
        })?;
    }

    let n = Decimal::from(returns.len());
    sum.checked_div(n).ok_or(CasifinError::DivisionByZero {
        operation: "arithmetic_mean_return",
    })
}

/// Geometric Mean Return.
///
/// # Formula
/// ```text
/// GM = ((1 + r1) * (1 + r2) * ... * (1 + rn))^(1/n) - 1
/// ```
///
/// # Arguments
/// * `returns` - Slice of periodic returns expressed as decimals.
///
/// # Returns
/// `Ok(Decimal)` containing the geometric mean return, or `Err(CasifinError::InsufficientCashFlows)`
/// when `returns` is empty.
///
/// # Example
/// ```
/// use casifin_ratios::geometric_mean_return;
/// use rust_decimal::Decimal;
///
/// let returns = vec![Decimal::new(10, 2), Decimal::new(20, 2), Decimal::new(30, 2)];
/// let gm = geometric_mean_return(&returns).unwrap();
/// assert!(gm > Decimal::new(19, 2) && gm < Decimal::new(20, 1));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Quantitative Methods, Reading 6.
///
/// # Panics
/// This function does not panic.
pub fn geometric_mean_return(returns: &[Decimal]) -> Result<Decimal, CasifinError> {
    debug_assert!(!returns.is_empty(), "returns must not be empty");
    debug_assert!(
        returns.len() <= u32::MAX as usize,
        "returns length must fit in u32"
    );

    if returns.is_empty() {
        return Err(CasifinError::InsufficientCashFlows);
    }

    let one = Decimal::ONE;
    let mut product = Decimal::ONE;
    for r in returns {
        let base = one.checked_add(*r).ok_or(CasifinError::ScheduleOverflow {
            detail: "geometric_mean_return: base overflow".to_string(),
        })?;
        product = product
            .checked_mul(base)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "geometric_mean_return: product overflow".to_string(),
            })?;
    }

    let n = Decimal::from(returns.len());
    let exponent = one.checked_div(n).ok_or(CasifinError::DivisionByZero {
        operation: "geometric_mean_return",
    })?;
    let gm = product
        .checked_powd(exponent)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "geometric_mean_return: power overflow".to_string(),
        })?;

    gm.checked_sub(one).ok_or(CasifinError::ScheduleOverflow {
        detail: "geometric_mean_return: result overflow".to_string(),
    })
}

/// Time-Weighted Rate of Return.
///
/// # Formula
/// ```text
/// TWRR = (1 + r1) * (1 + r2) * ... * (1 + rn) - 1
/// ```
///
/// # Arguments
/// * `period_returns` - Slice of sub-period returns expressed as decimals.
///
/// # Returns
/// `Ok(Decimal)` containing the TWRR, or `Err(CasifinError::InsufficientCashFlows)` when
/// the slice is empty.
///
/// # Example
/// ```
/// use casifin_ratios::time_weighted_rate_of_return;
/// use rust_decimal::Decimal;
///
/// let returns = vec![Decimal::new(10, 2), Decimal::new(-5, 2)];
/// let twrr = time_weighted_rate_of_return(&returns).unwrap();
/// assert_eq!(twrr, Decimal::new(45, 3)); // 1.10 * 0.95 - 1 = 0.045
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Quantitative Methods, Reading 6.
///
/// # Panics
/// This function does not panic.
pub fn time_weighted_rate_of_return(period_returns: &[Decimal]) -> Result<Decimal, CasifinError> {
    debug_assert!(
        !period_returns.is_empty(),
        "period_returns must not be empty"
    );
    debug_assert!(
        period_returns.len() <= u32::MAX as usize,
        "period_returns length must fit in u32"
    );

    if period_returns.is_empty() {
        return Err(CasifinError::InsufficientCashFlows);
    }

    let one = Decimal::ONE;
    let mut twrr = Decimal::ONE;
    for r in period_returns {
        let base = one.checked_add(*r).ok_or(CasifinError::ScheduleOverflow {
            detail: "time_weighted_rate_of_return: base overflow".to_string(),
        })?;
        twrr = twrr
            .checked_mul(base)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "time_weighted_rate_of_return: product overflow".to_string(),
            })?;
    }

    twrr.checked_sub(one).ok_or(CasifinError::ScheduleOverflow {
        detail: "time_weighted_rate_of_return: result overflow".to_string(),
    })
}

/// Money-Weighted Rate of Return.
///
/// # Formula
/// ```text
/// 0 = Σ CF_t / (1 + MWRR)^t        (undated IRR)
/// 0 = Σ CF_i / (1 + MWRR)^(days_i / 365)   (dated XIRR)
/// ```
///
/// # Arguments
/// * `cash_flows` - Cash flows (negative = outflow, positive = inflow).
/// * `dates` - Optional dates for each cash flow; when provided, `xirr` is used.
///
/// # Returns
/// `Ok(Decimal)` containing the money-weighted return, or `Err(CasifinError)` if the stream
/// is invalid or the solver does not converge.
///
/// # Example
/// ```
/// use casifin_core::Money;
/// use casifin_ratios::money_weighted_return;
///
/// let flows = vec![
///     Money::from_decimal(rust_decimal::Decimal::new(-1000, 0)),
///     Money::from_decimal(rust_decimal::Decimal::new(300, 0)),
///     Money::from_decimal(rust_decimal::Decimal::new(400, 0)),
///     Money::from_decimal(rust_decimal::Decimal::new(400, 0)),
///     Money::from_decimal(rust_decimal::Decimal::new(300, 0)),
/// ];
/// let mwrr = money_weighted_return(&flows, None).unwrap();
/// assert!(mwrr > rust_decimal::Decimal::new(14, 2));
/// assert!(mwrr < rust_decimal::Decimal::new(15, 2));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Quantitative Methods, Reading 6.
///
/// # Panics
/// This function does not panic.
pub fn money_weighted_return(
    cash_flows: &[Money],
    dates: Option<&[NaiveDate]>,
) -> Result<Decimal, CasifinError> {
    debug_assert!(!cash_flows.is_empty(), "cash_flows must not be empty");
    debug_assert!(
        dates.map(|d| d.len() == cash_flows.len()).unwrap_or(true),
        "dates length must match cash_flows length"
    );

    if cash_flows.is_empty() {
        return Err(CasifinError::InsufficientCashFlows);
    }

    if let Some(dates) = dates {
        if dates.len() != cash_flows.len() {
            return Err(CasifinError::InvalidInput {
                reason: "dates length must match cash_flows length".to_string(),
            });
        }

        let flows: Vec<CashFlow> = cash_flows
            .iter()
            .zip(dates.iter())
            .map(|(amount, date)| CashFlow::with_date(*amount, *date))
            .collect();

        return casifin_cashflow::xirr(&CashFlowStream::new(flows), Config::default());
    }

    let flows: Vec<CashFlow> = cash_flows
        .iter()
        .map(|amount| CashFlow::new(*amount))
        .collect();
    casifin_cashflow::irr(&CashFlowStream::new(flows), Config::default())
}

/// Sharpe Ratio.
///
/// # Formula
/// ```text
/// Sharpe Ratio = (Portfolio Return - Risk-Free Rate) / Standard Deviation
/// ```
///
/// # Arguments
/// * `portfolio_return` - Portfolio return as a decimal.
/// * `risk_free_rate` - Risk-free rate as a decimal.
/// * `std_dev` - Standard deviation of portfolio excess returns.
///
/// # Returns
/// `Ok(Decimal)` containing the Sharpe ratio, or `Err(CasifinError::DivisionByZero)` when
/// `std_dev` is zero.
///
/// # Example
/// ```
/// use casifin_ratios::sharpe_ratio;
/// use rust_decimal::Decimal;
///
/// let sharpe = sharpe_ratio(Decimal::new(12, 2), Decimal::new(3, 2), Decimal::new(15, 2)).unwrap();
/// assert_eq!(sharpe, Decimal::new(6, 1));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Quantitative Methods, Reading 6.
///
/// # Panics
/// This function does not panic.
pub fn sharpe_ratio(
    portfolio_return: Decimal,
    risk_free_rate: Decimal,
    std_dev: Decimal,
) -> Result<Decimal, CasifinError> {
    debug_assert!(!std_dev.is_zero(), "std_dev must not be zero");
    debug_assert!(std_dev > Decimal::ZERO, "std_dev must be positive");

    if std_dev.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "sharpe_ratio",
        });
    }

    let excess_return =
        portfolio_return
            .checked_sub(risk_free_rate)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "sharpe_ratio: excess return overflow".to_string(),
            })?;

    excess_return
        .checked_div(std_dev)
        .ok_or(CasifinError::DivisionByZero {
            operation: "sharpe_ratio",
        })
}

/// Roy's Safety-First Ratio.
///
/// # Formula
/// ```text
/// SF Ratio = (Expected Return - Threshold Return) / Standard Deviation
/// ```
///
/// # Arguments
/// * `expected_return` - Expected portfolio return as a decimal.
/// * `threshold` - Minimum acceptable return as a decimal.
/// * `std_dev` - Standard deviation of portfolio returns.
///
/// # Returns
/// `Ok(Decimal)` containing the safety-first ratio, or `Err(CasifinError::DivisionByZero)`
/// when `std_dev` is zero.
///
/// # Example
/// ```
/// use casifin_ratios::roys_safety_first_ratio;
/// use rust_decimal::Decimal;
///
/// let sf = roys_safety_first_ratio(Decimal::new(12, 2), Decimal::new(5, 2), Decimal::new(15, 2)).unwrap();
/// let expected = Decimal::new(7, 2).checked_div(Decimal::new(15, 2)).unwrap();
/// assert_eq!(sf, expected);
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Quantitative Methods, Reading 6.
///
/// # Panics
/// This function does not panic.
pub fn roys_safety_first_ratio(
    expected_return: Decimal,
    threshold: Decimal,
    std_dev: Decimal,
) -> Result<Decimal, CasifinError> {
    debug_assert!(!std_dev.is_zero(), "std_dev must not be zero");
    debug_assert!(std_dev > Decimal::ZERO, "std_dev must be positive");

    if std_dev.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "roys_safety_first_ratio",
        });
    }

    let excess = expected_return
        .checked_sub(threshold)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "roys_safety_first_ratio: excess return overflow".to_string(),
        })?;

    excess
        .checked_div(std_dev)
        .ok_or(CasifinError::DivisionByZero {
            operation: "roys_safety_first_ratio",
        })
}

/// Sortino Ratio.
///
/// # Formula
/// ```text
/// Sortino Ratio = (Portfolio Return - Target Return) / Downside Deviation
/// ```
///
/// # Arguments
/// * `portfolio_return` - Portfolio return as a decimal.
/// * `target_return` - Target or minimum acceptable return as a decimal.
/// * `downside_deviation` - Downside deviation of portfolio returns.
///
/// # Returns
/// `Ok(Decimal)` containing the Sortino ratio, or `Err(CasifinError::DivisionByZero)` when
/// `downside_deviation` is zero.
///
/// # Example
/// ```
/// use casifin_ratios::sortino_ratio;
/// use rust_decimal::Decimal;
///
/// let sortino = sortino_ratio(Decimal::new(12, 2), Decimal::new(5, 2), Decimal::new(10, 2)).unwrap();
/// assert_eq!(sortino, Decimal::new(7, 1)); // 0.70
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Quantitative Methods, Reading 6.
///
/// # Panics
/// This function does not panic.
pub fn sortino_ratio(
    portfolio_return: Decimal,
    target_return: Decimal,
    downside_deviation: Decimal,
) -> Result<Decimal, CasifinError> {
    debug_assert!(
        !downside_deviation.is_zero(),
        "downside_deviation must not be zero"
    );
    debug_assert!(
        downside_deviation > Decimal::ZERO,
        "downside_deviation must be positive"
    );

    if downside_deviation.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "sortino_ratio",
        });
    }

    let excess_return =
        portfolio_return
            .checked_sub(target_return)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "sortino_ratio: excess return overflow".to_string(),
            })?;

    excess_return
        .checked_div(downside_deviation)
        .ok_or(CasifinError::DivisionByZero {
            operation: "sortino_ratio",
        })
}

#[cfg(test)]
mod tests {
    use casifin_core::Money;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    use super::*;

    #[test]
    fn holding_period_return_known_value() {
        let hpr =
            holding_period_return(Money::from(100), Money::from(115), Money::from(5)).unwrap();
        assert_eq!(hpr, Decimal::new(2, 1));
    }

    #[test]
    fn arithmetic_mean_return_known_value() {
        let returns = vec![
            Decimal::new(10, 2),
            Decimal::new(20, 2),
            Decimal::new(30, 2),
        ];
        let mean = arithmetic_mean_return(&returns).unwrap();
        assert_eq!(mean, Decimal::new(2, 1));
    }

    #[test]
    fn geometric_mean_return_known_value() {
        let returns = vec![
            Decimal::new(10, 2),
            Decimal::new(20, 2),
            Decimal::new(30, 2),
        ];
        let gm = geometric_mean_return(&returns).unwrap();
        assert!(gm > Decimal::new(19, 2));
        assert!(gm < Decimal::new(20, 1));
    }

    #[test]
    fn time_weighted_rate_of_return_known_value() {
        let returns = vec![Decimal::new(10, 2), Decimal::new(-5, 2)];
        let twrr = time_weighted_rate_of_return(&returns).unwrap();
        assert_eq!(twrr, Decimal::new(45, 3));
    }

    #[test]
    fn money_weighted_return_known_value() {
        let flows = vec![
            Money::from_decimal(Decimal::new(-1000, 0)),
            Money::from_decimal(Decimal::new(300, 0)),
            Money::from_decimal(Decimal::new(400, 0)),
            Money::from_decimal(Decimal::new(400, 0)),
            Money::from_decimal(Decimal::new(300, 0)),
        ];
        let mwrr = money_weighted_return(&flows, None).unwrap();
        assert!(mwrr > Decimal::new(14, 2));
        assert!(mwrr < Decimal::new(15, 2));
    }

    #[test]
    fn money_weighted_return_dated_known_value() {
        let base = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let dates: Vec<NaiveDate> = (0..5)
            .map(|i| base.checked_add_days(chrono::Days::new(i * 365)).unwrap())
            .collect();
        let flows = vec![
            Money::from_decimal(Decimal::new(-1000, 0)),
            Money::from_decimal(Decimal::new(300, 0)),
            Money::from_decimal(Decimal::new(400, 0)),
            Money::from_decimal(Decimal::new(400, 0)),
            Money::from_decimal(Decimal::new(300, 0)),
        ];
        let mwrr = money_weighted_return(&flows, Some(&dates)).unwrap();
        assert!(mwrr > Decimal::new(14, 2));
        assert!(mwrr < Decimal::new(15, 2));
    }

    #[test]
    fn sharpe_ratio_known_value() {
        let sharpe =
            sharpe_ratio(Decimal::new(12, 2), Decimal::new(3, 2), Decimal::new(15, 2)).unwrap();
        assert_eq!(sharpe, Decimal::new(6, 1));
    }

    #[test]
    fn roys_safety_first_ratio_known_value() {
        let sf =
            roys_safety_first_ratio(Decimal::new(12, 2), Decimal::new(5, 2), Decimal::new(15, 2))
                .unwrap();
        let expected = Decimal::new(7, 2).checked_div(Decimal::new(15, 2)).unwrap();
        assert_eq!(sf, expected);
    }

    #[test]
    fn sortino_ratio_known_value() {
        let sortino =
            sortino_ratio(Decimal::new(12, 2), Decimal::new(5, 2), Decimal::new(10, 2)).unwrap();
        assert_eq!(sortino, Decimal::new(7, 1));
    }
}
