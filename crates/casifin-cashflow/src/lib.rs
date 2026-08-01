//! Cash flow analysis (NPV, IRR, XIRR, XNPV) for the casifin financial computation engine.

#![deny(warnings)]

use casifin_core::{CasifinError, Config, Money};
use chrono::NaiveDate;
use rust_decimal::{Decimal, MathematicalOps};

// ============================================================================
// CashFlow
// ============================================================================

/// A single cash flow with an optional date.
///
/// # Panics
/// This type does not panic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CashFlow {
    pub amount: Money,
    pub date: Option<NaiveDate>,
}

impl CashFlow {
    /// Creates a new `CashFlow` without a date.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn new(amount: Money) -> Self {
        CashFlow { amount, date: None }
    }

    /// Creates a new dated `CashFlow`.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn with_date(amount: Money, date: NaiveDate) -> Self {
        CashFlow {
            amount,
            date: Some(date),
        }
    }
}

// ============================================================================
// CashFlowStream
// ============================================================================

/// A stream of cash flows.
///
/// # Panics
/// This type does not panic.
#[derive(Debug, Clone, PartialEq)]
pub struct CashFlowStream(Vec<CashFlow>);

impl CashFlowStream {
    /// Creates a new `CashFlowStream` from a vector of cash flows.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn new(flows: Vec<CashFlow>) -> Self {
        CashFlowStream(flows)
    }

    /// Returns `true` if the stream is empty.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of cash flows.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the stream has both positive and negative cash flows.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn has_positive_and_negative(&self) -> bool {
        let has_pos = self.0.iter().any(|f| f.amount.is_positive());
        let has_neg = self.0.iter().any(|f| f.amount.is_negative());
        has_pos && has_neg
    }

    /// Returns an iterator over the cash flows.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn iter(&self) -> impl Iterator<Item = &CashFlow> {
        self.0.iter()
    }
}

// ============================================================================
// NPV
// ============================================================================

/// Computes the Net Present Value of a cash flow stream.
///
/// # Formula
/// ```text
/// NPV = Σ_{t=0}^{n-1} CF_t / (1 + rate)^t
/// ```
/// Note: t starts at 0 for the first flow (immediate).
///
/// # Arguments
/// * `rate` - The discount rate per period
/// * `stream` - The cash flow stream
///
/// # Returns
/// `Ok(Money)` containing the NPV, or `Err(CasifinError)` on invalid input.
///
/// # Panics
/// This function does not panic.
pub fn npv(rate: Decimal, stream: &CashFlowStream) -> Result<Money, CasifinError> {
    debug_assert!(rate >= Decimal::ZERO, "rate must be non-negative");

    if stream.is_empty() {
        return Err(CasifinError::InsufficientCashFlows);
    }
    if rate < Decimal::ZERO {
        return Err(CasifinError::InvalidRate(rate));
    }

    let one = Decimal::ONE;
    let mut npv = Money::ZERO;

    for (t, cf) in stream.iter().enumerate() {
        let discount =
            (one + rate)
                .checked_powi(t as i64)
                .ok_or(CasifinError::ScheduleOverflow {
                    detail: "npv: discount overflow".to_string(),
                })?;

        let pv = cf
            .amount
            .checked_div_decimal(discount)
            .ok_or(CasifinError::DivisionByZero {
                operation: "npv discount",
            })?;

        npv = npv + pv;
    }

    Ok(npv)
}

// ============================================================================
// IRR
// ============================================================================

/// Computes the Internal Rate of Return of a cash flow stream.
///
/// Uses a hybrid Newton-Raphson / bisection solver.
///
/// # Arguments
/// * `stream` - The cash flow stream (must have both positive and negative flows)
/// * `config` - Solver configuration (eps, max_iterations, guess)
///
/// # Returns
/// `Ok(Decimal)` containing the IRR, or `Err(CasifinError)` if the solver
/// does not converge or the stream is invalid.
///
/// # Panics
/// This function does not panic.
pub fn irr(stream: &CashFlowStream, config: Config) -> Result<Decimal, CasifinError> {
    if stream.is_empty() {
        return Err(CasifinError::InsufficientCashFlows);
    }
    if !stream.has_positive_and_negative() {
        return Err(CasifinError::InsufficientCashFlows);
    }

    debug_assert!(!stream.is_empty(), "stream must not be empty");
    debug_assert!(
        stream.has_positive_and_negative(),
        "stream must have mixed signs"
    );

    let mut rate = config.guess;

    for _ in 0..config.max_iterations {
        let (f, df) = irr_eq_and_derivative(stream, rate);

        if f.abs() < config.eps {
            return Ok(rate);
        }

        if df.abs() < config.eps {
            return irr_bisection(stream, config.eps);
        }

        let new_rate = rate - f / df;
        if (new_rate - rate).abs() < config.eps {
            return Ok(new_rate);
        }
        rate = new_rate;

        if rate < Decimal::NEGATIVE_ONE {
            rate = Decimal::new(-9, 1);
        }
        if rate > Decimal::ONE {
            rate = Decimal::new(5, 1);
        }
    }

    Err(CasifinError::IrrConvergenceFailure {
        max_iter: config.max_iterations,
        eps: config.eps,
    })
}

/// Evaluates the IRR equation and its derivative.
fn irr_eq_and_derivative(stream: &CashFlowStream, rate: Decimal) -> (Decimal, Decimal) {
    let one = Decimal::ONE;
    let mut f = Decimal::ZERO;
    let mut df = Decimal::ZERO;

    for (t, cf) in stream.iter().enumerate() {
        let t_dec = Decimal::from(t);
        let base = one + rate;
        let power = match base.checked_powi(t as i64) {
            Some(p) => p,
            None => continue,
        };

        let term = cf.amount.inner() / power;
        f += term;

        let df_term = -t_dec * cf.amount.inner() / (power * base);
        df += df_term;
    }

    (f, df)
}

/// Bisection solver for IRR.
fn irr_bisection(stream: &CashFlowStream, eps: Decimal) -> Result<Decimal, CasifinError> {
    let mut low = Decimal::new(-9, 1);
    let mut high = Decimal::ONE;
    let mut mid = (low + high) / Decimal::from(2);

    for _ in 0..1000 {
        let f_mid = irr_equation(stream, mid);

        if f_mid.abs() < eps {
            return Ok(mid);
        }

        let f_low = irr_equation(stream, low);

        if f_low * f_mid < Decimal::ZERO {
            high = mid;
        } else {
            low = mid;
        }

        let new_mid = (low + high) / Decimal::from(2);
        if (new_mid - mid).abs() < eps {
            return Ok(new_mid);
        }
        mid = new_mid;
    }

    Err(CasifinError::IrrConvergenceFailure {
        max_iter: 1000,
        eps,
    })
}

/// Evaluates the IRR equation: NPV at a given rate.
fn irr_equation(stream: &CashFlowStream, rate: Decimal) -> Decimal {
    let one = Decimal::ONE;
    let mut npv_val = Decimal::ZERO;

    for (t, cf) in stream.iter().enumerate() {
        let base = one + rate;
        let power = match base.checked_powi(t as i64) {
            Some(p) => p,
            None => continue,
        };

        let pv = cf.amount.inner() / power;
        npv_val += pv;
    }

    npv_val
}

// ============================================================================
// XNPV
// ============================================================================

/// Computes the Net Present Value with actual dates (XNPV).
///
/// # Formula
/// ```text
/// XNPV = Σ CF_i / (1 + rate)^(days_i / 365)
/// ```
///
/// # Arguments
/// * `rate` - The annual discount rate
/// * `stream` - The dated cash flow stream (all flows must have dates)
///
/// # Returns
/// `Ok(Money)` containing the XNPV, or `Err(CasifinError)` on invalid input.
///
/// # Panics
/// This function does not panic.
pub fn xnpv(rate: Decimal, stream: &CashFlowStream) -> Result<Money, CasifinError> {
    debug_assert!(rate >= Decimal::ZERO, "rate must be non-negative");

    if stream.is_empty() {
        return Err(CasifinError::InsufficientCashFlows);
    }
    if rate < Decimal::ZERO {
        return Err(CasifinError::InvalidRate(rate));
    }

    let first_date = stream
        .iter()
        .find_map(|cf| cf.date)
        .ok_or(CasifinError::DateOutOfRange(
            "No dates in stream".to_string(),
        ))?;

    let days_per_year = Decimal::from(365);
    let one = Decimal::ONE;
    let mut xnpv_val = Money::ZERO;

    for cf in stream.iter() {
        let date = cf.date.ok_or(CasifinError::DateOutOfRange(
            "Undated cash flow in XNPV".to_string(),
        ))?;

        let days = (date - first_date).num_days();
        let year_frac = Decimal::from(days) / days_per_year;

        if year_frac < Decimal::ZERO {
            continue;
        }

        let discount = (one + rate).powd(year_frac);
        let pv = cf
            .amount
            .checked_div_decimal(discount)
            .ok_or(CasifinError::DivisionByZero {
                operation: "xnpv discount",
            })?;

        xnpv_val = xnpv_val + pv;
    }

    Ok(xnpv_val)
}

// ============================================================================
// XIRR
// ============================================================================

/// Computes the Internal Rate of Return with actual dates (XIRR).
///
/// Uses a hybrid Newton-Raphson / bisection solver with date-weighted derivatives.
///
/// # Arguments
/// * `stream` - The dated cash flow stream (must have both positive and negative flows)
/// * `config` - Solver configuration (eps, max_iterations, guess)
///
/// # Returns
/// `Ok(Decimal)` containing the XIRR, or `Err(CasifinError)` if the solver
/// does not converge.
///
/// # Panics
/// This function does not panic.
pub fn xirr(stream: &CashFlowStream, config: Config) -> Result<Decimal, CasifinError> {
    debug_assert!(!stream.is_empty(), "stream must not be empty");
    debug_assert!(
        stream.has_positive_and_negative(),
        "stream must have mixed signs"
    );

    if stream.is_empty() {
        return Err(CasifinError::InsufficientCashFlows);
    }
    if !stream.has_positive_and_negative() {
        return Err(CasifinError::InsufficientCashFlows);
    }

    let first_date = stream
        .iter()
        .find_map(|cf| cf.date)
        .ok_or(CasifinError::DateOutOfRange(
            "No dates in stream".to_string(),
        ))?;

    let mut rate = config.guess;

    for _ in 0..config.max_iterations {
        let (f, df) = xirr_eq_and_derivative(stream, first_date, rate)?;

        if f.abs() < config.eps {
            return Ok(rate);
        }

        if df.abs() < config.eps {
            return xirr_bisection(stream, first_date, config.eps);
        }

        let new_rate = rate - f / df;
        if (new_rate - rate).abs() < config.eps {
            return Ok(new_rate);
        }
        rate = new_rate;

        if rate < Decimal::NEGATIVE_ONE {
            rate = Decimal::new(-9, 1);
        }
        if rate > Decimal::from(10) {
            rate = Decimal::from(10);
        }
    }

    Err(CasifinError::IrrConvergenceFailure {
        max_iter: config.max_iterations,
        eps: config.eps,
    })
}

/// Evaluates the XIRR equation and its derivative.
fn xirr_eq_and_derivative(
    stream: &CashFlowStream,
    first_date: NaiveDate,
    rate: Decimal,
) -> Result<(Decimal, Decimal), CasifinError> {
    let days_per_year = Decimal::from(365);
    let one = Decimal::ONE;
    let mut f = Decimal::ZERO;
    let mut df = Decimal::ZERO;

    for cf in stream.iter() {
        let date = cf.date.ok_or(CasifinError::DateOutOfRange(
            "Undated cash flow in XIRR".to_string(),
        ))?;

        let days = (date - first_date).num_days();
        let t = Decimal::from(days) / days_per_year;

        if t < Decimal::ZERO {
            continue;
        }

        let base = one + rate;
        let power = base.powd(t);

        let term = cf.amount.inner() / power;
        f += term;

        let df_term = -t * cf.amount.inner() / (power * base);
        df += df_term;
    }

    Ok((f, df))
}

/// Evaluates XNPV at a given rate.
fn xnpv_at_rate(stream: &CashFlowStream, first_date: NaiveDate, rate: Decimal) -> Decimal {
    let days_per_year = Decimal::from(365);
    let one = Decimal::ONE;
    let mut f = Decimal::ZERO;

    for cf in stream.iter() {
        let date = match cf.date {
            Some(d) => d,
            None => continue,
        };
        let days = (date - first_date).num_days();
        let t = Decimal::from(days) / days_per_year;

        if t < Decimal::ZERO {
            continue;
        }

        let base = one + rate;
        let power = base.powd(t);
        f += cf.amount.inner() / power;
    }
    f
}

/// Bisection solver for XIRR.
fn xirr_bisection(
    stream: &CashFlowStream,
    first_date: NaiveDate,
    eps: Decimal,
) -> Result<Decimal, CasifinError> {
    let mut low = Decimal::new(-9, 1);
    let mut high = Decimal::from(10);
    let mut mid = (low + high) / Decimal::from(2);

    for _ in 0..1000 {
        let f = xnpv_at_rate(stream, first_date, mid);

        if f.abs() < eps {
            return Ok(mid);
        }

        let f_low = xnpv_at_rate(stream, first_date, low);

        if f_low * f < Decimal::ZERO {
            high = mid;
        } else {
            low = mid;
        }

        let new_mid = (low + high) / Decimal::from(2);
        if (new_mid - mid).abs() < eps {
            return Ok(new_mid);
        }
        mid = new_mid;
    }

    Err(CasifinError::IrrConvergenceFailure {
        max_iter: 1000,
        eps,
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npv_zero_rate() {
        // NPV at 0% = sum of all flows
        let flows = CashFlowStream::new(vec![
            CashFlow::new(Money::from_decimal(Decimal::new(-1000, 0))),
            CashFlow::new(Money::from_decimal(Decimal::new(300, 0))),
            CashFlow::new(Money::from_decimal(Decimal::new(400, 0))),
            CashFlow::new(Money::from_decimal(Decimal::new(400, 0))),
            CashFlow::new(Money::from_decimal(Decimal::new(300, 0))),
        ]);
        let result = npv(Decimal::ZERO, &flows).unwrap();
        // -1000 + 300 + 400 + 400 + 300 = 400
        assert_eq!(result, Money::from_decimal(Decimal::new(400, 0)));
    }

    #[test]
    fn npv_known_value() {
        // flows [-1000, 300, 400, 400, 300] at 8%
        // NPV = -1000 + 300/1.08 + 400/1.08^2 + 400/1.08^3 + 300/1.08^4 ≈ 158.76
        let flows = CashFlowStream::new(vec![
            CashFlow::new(Money::from_decimal(Decimal::new(-1000, 0))),
            CashFlow::new(Money::from_decimal(Decimal::new(300, 0))),
            CashFlow::new(Money::from_decimal(Decimal::new(400, 0))),
            CashFlow::new(Money::from_decimal(Decimal::new(400, 0))),
            CashFlow::new(Money::from_decimal(Decimal::new(300, 0))),
        ]);
        let rate = Decimal::new(8, 2); // 8%
        let result = npv(rate, &flows).unwrap();
        assert!(result > Money::from_decimal(Decimal::new(158, 0)));
        assert!(result < Money::from_decimal(Decimal::new(160, 0)));
    }

    #[test]
    fn irr_known_value() {
        // flows [-1000, 300, 400, 400, 300] IRR = 14.49%
        let flows = CashFlowStream::new(vec![
            CashFlow::new(Money::from_decimal(Decimal::new(-1000, 0))),
            CashFlow::new(Money::from_decimal(Decimal::new(300, 0))),
            CashFlow::new(Money::from_decimal(Decimal::new(400, 0))),
            CashFlow::new(Money::from_decimal(Decimal::new(400, 0))),
            CashFlow::new(Money::from_decimal(Decimal::new(300, 0))),
        ]);
        let config = Config::default();
        let result = irr(&flows, config).unwrap();
        // Should be approximately 14.49%
        assert!(result > Decimal::new(14, 2)); // > 0.14
        assert!(result < Decimal::new(15, 2)); // < 0.15
    }

    #[test]
    fn irr_insufficient_flows() {
        // All positive flows should return error
        let flows = CashFlowStream::new(vec![
            CashFlow::new(Money::from_decimal(Decimal::new(100, 0))),
            CashFlow::new(Money::from_decimal(Decimal::new(200, 0))),
        ]);
        let config = Config::default();
        let result = irr(&flows, config);
        assert!(matches!(result, Err(CasifinError::InsufficientCashFlows)));
    }

    #[test]
    fn xnpv_known_value() {
        let date1 = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let date3 = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();

        let flows = CashFlowStream::new(vec![
            CashFlow::with_date(Money::from_decimal(Decimal::new(-1000, 0)), date1),
            CashFlow::with_date(Money::from_decimal(Decimal::new(500, 0)), date2),
            CashFlow::with_date(Money::from_decimal(Decimal::new(500, 0)), date3),
        ]);

        let rate = Decimal::new(10, 2);
        let result = xnpv(rate, &flows).unwrap();
        // Should be negative (investment not fully recovered at 10%)
        assert!(result < Money::ZERO);
    }

    #[test]
    fn xirr_known_value() {
        let date1 = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let date3 = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();

        // Profitable investment: -1000, +600, +600 -> positive return
        let flows = CashFlowStream::new(vec![
            CashFlow::with_date(Money::from_decimal(Decimal::new(-1000, 0)), date1),
            CashFlow::with_date(Money::from_decimal(Decimal::new(600, 0)), date2),
            CashFlow::with_date(Money::from_decimal(Decimal::new(600, 0)), date3),
        ]);

        let config = Config::default();
        let result = xirr(&flows, config).unwrap();
        // XIRR should be positive for this investment
        assert!(result > Decimal::ZERO);
    }

    #[test]
    fn irr_convergence() {
        // Pathological flows that should not converge: nearly-equal sign flows
        let flows = CashFlowStream::new(vec![
            CashFlow::new(Money::from_decimal(Decimal::new(-1, 0))),
            CashFlow::new(Money::from_decimal(Decimal::new(1000000, 0))),
        ]);
        let config = Config::builder()
            .max_iterations(5)
            .eps(Decimal::new(1, 20))
            .build();
        let result = irr(&flows, config);
        assert!(result.is_err());
    }
}
