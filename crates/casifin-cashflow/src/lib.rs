//! Cash flow analysis (NPV, IRR, XIRR, XNPV) for the casifin financial computation engine.

#![deny(warnings)]

use casifin_core::{CasifinError, Money};
use chrono::NaiveDate;
use rust_decimal::{Decimal, MathematicalOps};

/// A single cash flow with an optional date.
///
/// # Fields
/// * `amount` - The monetary amount (positive for inflow, negative for outflow)
/// * `date` - Optional date; if `None`, the cash flow is indexed by period
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CashFlow {
    /// The monetary amount.
    pub amount: Money,
    /// The date of the cash flow, if any.
    pub date: Option<NaiveDate>,
}

impl CashFlow {
    /// Creates a new `CashFlow` without a date.
    pub fn new(amount: Money) -> Self {
        CashFlow { amount, date: None }
    }

    /// Creates a new dated `CashFlow`.
    pub fn dated(amount: Money, date: NaiveDate) -> Self {
        CashFlow {
            amount,
            date: Some(date),
        }
    }
}

/// A stream of cash flows.
#[derive(Debug, Clone, Default)]
pub struct CashFlowStream(Vec<CashFlow>);

impl CashFlowStream {
    /// Creates a new `CashFlowStream` from a vector of cash flows.
    pub fn new(flows: Vec<CashFlow>) -> Self {
        CashFlowStream(flows)
    }

    /// Creates a `CashFlowStream` from a vector of `Money` values (undated).
    pub fn from_vec(flows: Vec<Money>) -> Self {
        CashFlowStream(flows.into_iter().map(CashFlow::new).collect())
    }

    /// Returns the number of cash flows.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the stream is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the cash flow at the specified index.
    pub fn get(&self, index: usize) -> Option<&CashFlow> {
        self.0.get(index)
    }

    /// Returns an iterator over the cash flows.
    pub fn iter(&self) -> impl Iterator<Item = &CashFlow> {
        self.0.iter()
    }

    /// Validates that the stream contains both positive and negative cash flows.
    pub fn has_mixed_signs(&self) -> bool {
        let mut has_positive = false;
        let mut has_negative = false;
        for cf in &self.0 {
            if cf.amount.inner() > Decimal::ZERO {
                has_positive = true;
            } else if cf.amount.inner() < Decimal::ZERO {
                has_negative = true;
            }
            if has_positive && has_negative {
                return true;
            }
        }
        false
    }
}

/// Computes the Net Present Value of a cash flow stream.
///
/// # Formula
/// ```text
/// NPV = Σ CF_t / (1 + r)^t
/// ```
///
/// # Arguments
/// * `rate` - The discount rate per period
/// * `stream` - The cash flow stream
///
/// # Returns
/// `Ok(Money)` containing the NPV, or `Err(CasifinError)` if:
/// - The stream is empty
/// - `rate` is negative
pub fn npv(rate: Decimal, stream: &CashFlowStream) -> Result<Money, CasifinError> {
    debug_assert!(rate >= Decimal::ZERO, "rate must be non-negative");

    if stream.is_empty() {
        return Err(CasifinError::EmptyCashFlowStream);
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

/// Computes the Internal Rate of Return of a cash flow stream.
///
/// Uses a hybrid Newton-Raphson / bisection solver.
///
/// # Arguments
/// * `stream` - The cash flow stream
/// * `guess` - Initial guess for the rate
/// * `max_iter` - Maximum iterations
/// * `eps` - Convergence threshold
///
/// # Returns
/// `Ok(Decimal)` containing the IRR, or `Err(CasifinError::IrrConvergenceFailure)`
/// if the solver does not converge.
pub fn irr(
    stream: &CashFlowStream,
    guess: Decimal,
    max_iter: u32,
    eps: Decimal,
) -> Result<Decimal, CasifinError> {
    debug_assert!(!stream.is_empty(), "stream must not be empty");
    debug_assert!(stream.has_mixed_signs(), "stream must have mixed signs");
    debug_assert!(max_iter > 0, "max_iter must be positive");
    debug_assert!(eps > Decimal::ZERO, "eps must be positive");

    if stream.is_empty() {
        return Err(CasifinError::EmptyCashFlowStream);
    }
    if !stream.has_mixed_signs() {
        return Err(CasifinError::XirrSignRequirement);
    }

    // Newton-Raphson solver
    let mut rate = guess;

    for _ in 0..max_iter {
        let (f, df) = irr_equation_and_derivative(stream, rate);

        if f.abs() < eps {
            return Ok(rate);
        }

        if df.abs() < eps {
            // Derivative too small, switch to bisection
            return irr_bisection(stream, eps);
        }

        let new_rate = rate - f / df;
        if (new_rate - rate).abs() < eps {
            return Ok(new_rate);
        }
        rate = new_rate;

        // Clamp rate to valid range
        if rate < Decimal::NEGATIVE_ONE {
            rate = Decimal::new(-9, 1); // -0.9
        }
        if rate > Decimal::ONE {
            rate = Decimal::new(5, 1); // 5.0
        }
    }

    Err(CasifinError::IrrConvergenceFailure { max_iter, eps })
}

/// Evaluates the IRR equation and its derivative.
fn irr_equation_and_derivative(stream: &CashFlowStream, rate: Decimal) -> (Decimal, Decimal) {
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

        // f = CF / (1 + r)^t
        let term = cf.amount.inner() / power;
        f += term;

        // df/drate = -t * CF / (1 + r)^(t+1)
        let df_term = -t_dec * cf.amount.inner() / (power * base);
        df += df_term;
    }

    (f, df)
}

/// Bisection solver for IRR.
fn irr_bisection(stream: &CashFlowStream, eps: Decimal) -> Result<Decimal, CasifinError> {
    let mut low = Decimal::new(-9, 1); // -0.9
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

/// Evaluates the IRR equation.
fn irr_equation(stream: &CashFlowStream, rate: Decimal) -> Decimal {
    let one = Decimal::ONE;
    let mut npv = Decimal::ZERO;

    for (t, cf) in stream.iter().enumerate() {
        let base = one + rate;
        let power = match base.checked_powi(t as i64) {
            Some(p) => p,
            None => continue,
        };

        let pv = cf.amount.inner() / power;
        npv += pv;
    }

    npv
}

/// Computes the NPV with actual dates (XNPV).
///
/// # Formula
/// ```text
/// XNPV = Σ CF_t / (1 + r)^(days_t / 365)
/// ```
///
/// # Arguments
/// * `rate` - The annual discount rate
/// * `stream` - The dated cash flow stream
///
/// # Returns
/// `Ok(Money)` containing the XNPV, or `Err(CasifinError)` if:
/// - The stream is empty or contains undated flows
/// - `rate` is negative
pub fn xnpv(rate: Decimal, stream: &CashFlowStream) -> Result<Money, CasifinError> {
    debug_assert!(rate >= Decimal::ZERO, "rate must be non-negative");

    if stream.is_empty() {
        return Err(CasifinError::EmptyCashFlowStream);
    }
    if rate < Decimal::ZERO {
        return Err(CasifinError::InvalidRate(rate));
    }

    // Find the first date
    let first_date = stream
        .iter()
        .find_map(|cf| cf.date)
        .ok_or(CasifinError::DateOutOfRange(
            "No dates in stream".to_string(),
        ))?;

    let one = Decimal::ONE;
    let days_per_year = Decimal::from(365);
    let mut xnpv = Money::ZERO;

    for cf in stream.iter() {
        let date = cf.date.ok_or(CasifinError::DateOutOfRange(
            "Undated cash flow in XNPV".to_string(),
        ))?;

        let days = (date - first_date).num_days();
        let year_frac = Decimal::from(days) / days_per_year;

        if year_frac < Decimal::ZERO {
            continue; // Skip flows before the first date
        }

        let discount = (one + rate).powd(year_frac);
        let pv = cf
            .amount
            .checked_div_decimal(discount)
            .ok_or(CasifinError::DivisionByZero {
                operation: "xnpv discount",
            })?;

        xnpv = xnpv + pv;
    }

    Ok(xnpv)
}

/// Computes the IRR with actual dates (XIRR).
///
/// Uses Newton-Raphson with date-weighted derivatives.
///
/// # Arguments
/// * `stream` - The dated cash flow stream
/// * `guess` - Initial guess for the rate
/// * `max_iter` - Maximum iterations
/// * `eps` - Convergence threshold
///
/// # Returns
/// `Ok(Decimal)` containing the XIRR, or `Err(CasifinError)` if:
/// - The stream is empty or lacks mixed signs
/// - The solver does not converge
pub fn xirr(
    stream: &CashFlowStream,
    guess: Decimal,
    max_iter: u32,
    eps: Decimal,
) -> Result<Decimal, CasifinError> {
    debug_assert!(!stream.is_empty(), "stream must not be empty");
    debug_assert!(stream.has_mixed_signs(), "stream must have mixed signs");
    debug_assert!(max_iter > 0, "max_iter must be positive");
    debug_assert!(eps > Decimal::ZERO, "eps must be positive");

    if stream.is_empty() {
        return Err(CasifinError::EmptyCashFlowStream);
    }
    if !stream.has_mixed_signs() {
        return Err(CasifinError::XirrSignRequirement);
    }

    // Find the first date
    let first_date = stream
        .iter()
        .find_map(|cf| cf.date)
        .ok_or(CasifinError::DateOutOfRange(
            "No dates in stream".to_string(),
        ))?;

    // Newton-Raphson solver
    let mut rate = guess;

    for _ in 0..max_iter {
        let (f, df) = xirr_equation_and_derivative_dated(stream, first_date, rate)?;

        if f.abs() < eps {
            return Ok(rate);
        }

        if df.abs() < eps {
            return xirr_bisection(stream, first_date, eps);
        }

        let new_rate = rate - f / df;
        if (new_rate - rate).abs() < eps {
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

    Err(CasifinError::IrrConvergenceFailure { max_iter, eps })
}

/// Evaluates the XIRR equation and its derivative for dated cash flows.
fn xirr_equation_and_derivative_dated(
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
        let power = match base.powd(t).into() {
            Some(p) => p,
            None => continue,
        };

        let term = cf.amount.inner() / power;
        f += term;

        let df_term = -t * cf.amount.inner() / (power * base);
        df += df_term;
    }

    Ok((f, df))
}

/// Evaluates XNPV at a given rate for bisection.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npv_simple() {
        let flows = CashFlowStream::from_vec(vec![
            Money::from(-1000),
            Money::from(500),
            Money::from(500),
            Money::from(500),
        ]);
        let rate = Decimal::new(10, 2); // 10%
        let result = npv(rate, &flows).unwrap();
        // NPV ≈ -1000 + 500/1.1 + 500/1.1^2 + 500/1.1^3 ≈ 243.43
        assert!(result > Money::from(200));
        assert!(result < Money::from(300));
    }

    #[test]
    fn test_npv_zero_rate() {
        let flows =
            CashFlowStream::from_vec(vec![Money::from(-1000), Money::from(500), Money::from(500)]);
        let rate = Decimal::ZERO;
        let result = npv(rate, &flows).unwrap();
        // NPV = -1000 + 500 + 500 = 0
        assert_eq!(result, Money::ZERO);
    }

    #[test]
    fn test_irr_simple() {
        let flows = CashFlowStream::from_vec(vec![
            Money::from(-1000),
            Money::from(500),
            Money::from(500),
            Money::from(500),
        ]);
        let result = irr(&flows, Decimal::new(1, 1), 1000, Decimal::new(1, 12));
        assert!(result.is_ok());
        let irr_val = result.unwrap();
        // IRR ≈ 23.4%
        assert!(irr_val > Decimal::new(2, 1));
        assert!(irr_val < Decimal::new(3, 1));
    }

    #[test]
    fn test_xnpv_dated() {
        let date1 = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let date3 = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();

        let flows = CashFlowStream::new(vec![
            CashFlow::dated(Money::from(-1000), date1),
            CashFlow::dated(Money::from(500), date2),
            CashFlow::dated(Money::from(500), date3),
        ]);

        let rate = Decimal::new(10, 2);
        let result = xnpv(rate, &flows).unwrap();
        assert!(result > Money::from(-1000));
    }

    #[test]
    fn test_cashflow_mixed_signs() {
        let flows =
            CashFlowStream::from_vec(vec![Money::from(-1000), Money::from(500), Money::from(500)]);
        assert!(flows.has_mixed_signs());

        let flows2 =
            CashFlowStream::from_vec(vec![Money::from(100), Money::from(200), Money::from(300)]);
        assert!(!flows2.has_mixed_signs());
    }
}
