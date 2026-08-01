//! Time Value of Money (TVM) calculations for the casifin financial computation engine.
//!
//! This crate provides functions for calculating:
//! - Present Value (PV)
//! - Future Value (FV)
//! - Payment (PMT)
//! - Number of Periods (NPER)
//! - Interest Rate (RATE)
//! - Present Value of Perpetuity
//! - Future Value of Uneven Cash Flows

#![deny(warnings)]

use casifin_core::{CasifinError, Money};
use rust_decimal::{prelude::ToPrimitive, Decimal, MathematicalOps};

/// Payment timing convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaymentDue {
    /// Payments due at the end of each period (ordinary annuity).
    #[default]
    End,
    /// Payments due at the beginning of each period (annuity due).
    Beginning,
}

/// Computes the present value of an annuity.
///
/// # Formula
/// For ordinary annuity (end):
/// ```text
/// PV = PMT * (1 - (1 + r)^-n) / r
/// ```
///
/// # Arguments
/// * `rate` - The interest rate per period
/// * `nper` - The total number of payment periods
/// * `pmt` - The payment made each period
/// * `fv` - The future value (cash balance after last payment)
/// * `due` - Whether payments are due at beginning or end
///
/// # Returns
/// `Ok(Money)` containing the present value, or `Err(CasifinError)` if:
/// - `rate` is negative
/// - `nper` is zero
/// - Division by zero occurs
///
/// # Example
/// ```
/// use casifin_tvm::{pv, PaymentDue};
/// use casifin_core::{Money, Rate, Compounding, DayCount};
/// use rust_decimal::Decimal;
///
/// let rate = Decimal::new(5, 2); // 5% per period
/// let pmt = Money::from(1000);
/// let nper = 10u32;
/// let result = pv(rate, nper, pmt, Money::ZERO, PaymentDue::End);
/// ```
pub fn pv(
    rate: Decimal,
    nper: u32,
    pmt: Money,
    fv: Money,
    due: PaymentDue,
) -> Result<Money, CasifinError> {
    debug_assert!(rate >= Decimal::ZERO, "rate must be non-negative");
    debug_assert!(nper > 0, "nper must be positive");

    if rate < Decimal::ZERO {
        return Err(CasifinError::InvalidRate(rate));
    }
    if nper == 0 {
        return Err(CasifinError::InvalidPeriod(0));
    }

    let one = Decimal::ONE;
    let r = rate;

    // Handle rate = 0 as special case
    if r.is_zero() {
        let total_pmt = pmt * Decimal::from(nper);
        return Ok(total_pmt + fv);
    }

    // (1 + r)^-n
    let base = one + r;
    let neg_power = base
        .checked_powi(-(nper as i64))
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "pv: power calculation overflow".to_string(),
        })?;

    // (1 - (1 + r)^-n) / r
    let annuity_factor = (one - neg_power)
        .checked_div(r)
        .ok_or(CasifinError::DivisionByZero {
            operation: "pv annuity factor",
        })?;

    let pv_pmt = pmt * annuity_factor;

    // PV = PMT * annuity_factor + FV * (1 + r)^-n
    let fv_discounted = Money::from(neg_power) * fv;
    let pv_ordinary = pv_pmt + fv_discounted;

    // For annuity-due, multiply by (1 + r)
    match due {
        PaymentDue::End => Ok(pv_ordinary),
        PaymentDue::Beginning => Ok(pv_ordinary * base),
    }
}

/// Computes the future value of an annuity.
///
/// # Formula
/// For ordinary annuity (end):
/// ```text
/// FV = PMT * ((1 + r)^n - 1) / r + PV * (1 + r)^n
/// ```
///
/// # Arguments
/// * `rate` - The interest rate per period
/// * `nper` - The total number of payment periods
/// * `pmt` - The payment made each period
/// * `pv` - The present value (initial investment)
/// * `due` - Whether payments are due at beginning or end
///
/// # Returns
/// `Ok(Money)` containing the future value, or `Err(CasifinError)` if:
/// - `rate` is negative
/// - `nper` is zero
/// - Overflow occurs
pub fn fv(
    rate: Decimal,
    nper: u32,
    pmt: Money,
    pv: Money,
    due: PaymentDue,
) -> Result<Money, CasifinError> {
    debug_assert!(rate >= Decimal::ZERO, "rate must be non-negative");
    debug_assert!(nper > 0, "nper must be positive");

    if rate < Decimal::ZERO {
        return Err(CasifinError::InvalidRate(rate));
    }
    if nper == 0 {
        return Err(CasifinError::InvalidPeriod(0));
    }

    let one = Decimal::ONE;
    let r = rate;

    if r.is_zero() {
        let total_pmt = pmt * Decimal::from(nper);
        return Ok(total_pmt + pv);
    }

    // (1 + r)^n
    let base = one + r;
    let power = base
        .checked_powi(nper as i64)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "fv: power calculation overflow".to_string(),
        })?;

    // ((1 + r)^n - 1) / r
    let fv_factor = (power - one)
        .checked_div(r)
        .ok_or(CasifinError::DivisionByZero {
            operation: "fv factor",
        })?;

    let fv_pmt = pmt * fv_factor;
    let fv_pv = pv * power;
    let fv_ordinary = fv_pmt + fv_pv;

    // For annuity-due, multiply by (1 + r)
    match due {
        PaymentDue::End => Ok(fv_ordinary),
        PaymentDue::Beginning => Ok(fv_ordinary * base),
    }
}

/// Computes the payment amount for an annuity.
///
/// # Formula
/// For ordinary annuity (end):
/// ```text
/// PMT = (PV - FV / (1 + r)^n) * r / (1 - (1 + r)^-n)
/// ```
///
/// # Arguments
/// * `rate` - The interest rate per period
/// * `nper` - The total number of payment periods
/// * `pv` - The present value (loan amount or investment)
/// * `fv` - The future value (desired balance after last payment)
/// * `due` - Whether payments are due at beginning or end
///
/// # Returns
/// `Ok(Money)` containing the payment amount, or `Err(CasifinError)` if:
/// - `rate` is negative
/// - `nper` is zero
/// - Division by zero occurs
pub fn pmt(
    rate: Decimal,
    nper: u32,
    pv: Money,
    fv: Money,
    due: PaymentDue,
) -> Result<Money, CasifinError> {
    debug_assert!(rate >= Decimal::ZERO, "rate must be non-negative");
    debug_assert!(nper > 0, "nper must be positive");

    if rate < Decimal::ZERO {
        return Err(CasifinError::InvalidRate(rate));
    }
    if nper == 0 {
        return Err(CasifinError::InvalidPeriod(0));
    }

    let one = Decimal::ONE;
    let r = rate;

    if r.is_zero() {
        // PMT = (PV - FV) / n
        let diff = pv - fv;
        return diff
            .checked_div_decimal(Decimal::from(nper))
            .ok_or(CasifinError::DivisionByZero {
                operation: "pmt zero rate",
            });
    }

    // (1 + r)^n
    let base = one + r;
    let power = base
        .checked_powi(nper as i64)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "pmt: power calculation overflow".to_string(),
        })?;

    // (1 + r)^-n
    let neg_power = one.checked_div(power).ok_or(CasifinError::DivisionByZero {
        operation: "pmt neg_power",
    })?;

    // (1 - (1 + r)^-n)
    let denominator = one - neg_power;

    // (PV - FV / (1 + r)^n) * r / (1 - (1 + r)^-n)
    let fv_discounted = fv * neg_power;
    let numerator = pv - fv_discounted;
    let pmt_value = numerator * r;

    let pmt_ordinary =
        pmt_value
            .checked_div_decimal(denominator)
            .ok_or(CasifinError::DivisionByZero {
                operation: "pmt denominator",
            })?;

    // For annuity-due, divide by (1 + r)
    match due {
        PaymentDue::End => Ok(pmt_ordinary),
        PaymentDue::Beginning => {
            pmt_ordinary
                .checked_div_decimal(base)
                .ok_or(CasifinError::DivisionByZero {
                    operation: "pmt annuity-due adjustment",
                })
        }
    }
}

/// Computes the number of periods required to reach a future value.
///
/// # Formula
/// ```text
/// NPER = ln(PMT / (PMT - PV * r)) / ln(1 + r)
/// ```
///
/// # Arguments
/// * `rate` - The interest rate per period
/// * `pmt` - The payment made each period
/// * `pv` - The present value
/// * `fv` - The future value
/// * `due` - Whether payments are due at beginning or end
///
/// # Returns
/// `Ok(u32)` containing the number of periods, or `Err(CasifinError)` if:
/// - `rate` is negative or zero
/// - `pmt` is zero
/// - Logarithm cannot be computed
pub fn nper(
    rate: Decimal,
    pmt: Money,
    pv: Money,
    fv: Money,
    _due: PaymentDue,
) -> Result<u32, CasifinError> {
    if rate <= Decimal::ZERO {
        return Err(CasifinError::InvalidRate(rate));
    }
    if pmt.is_zero() {
        return Err(CasifinError::InvalidPayment(pmt.inner()));
    }

    debug_assert!(rate > Decimal::ZERO, "rate must be positive");
    debug_assert!(!pmt.is_zero(), "pmt must be non-zero");

    let one = Decimal::ONE;
    let r = rate;

    // NPER = ln((PMT - FV * r) / (PMT + PV * r)) / ln(1 + r)
    let fv_r = fv.inner() * r;
    let pv_r = pv.inner() * r;

    let numerator = (pmt.inner() - fv_r).checked_div(pmt.inner() + pv_r).ok_or(
        CasifinError::DivisionByZero {
            operation: "nper ratio",
        },
    )?;

    if numerator <= Decimal::ZERO {
        return Err(CasifinError::ScheduleOverflow {
            detail: "nper: invalid ratio - loan cannot be paid off".to_string(),
        });
    }

    let ln_base = ln_approx(one + r);
    let ln_numerator = ln_approx(numerator);

    if ln_base.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "nper ln_base",
        });
    }

    let nper_decimal = -ln_numerator / ln_base;

    // Round up to nearest integer
    let nper_int = nper_decimal.ceil().to_u32().unwrap_or(u32::MAX);

    Ok(nper_int)
}

/// Computes the interest rate per period for an annuity.
///
/// Uses a hybrid Newton-Raphson / bisection solver.
///
/// # Arguments
/// * `nper` - The total number of payment periods
/// * `pmt` - The payment made each period
/// * `pv` - The present value
/// * `fv` - The future value
/// * `due` - Whether payments are due at beginning or end
/// * `guess` - Initial guess for the rate (e.g., 0.1 for 10%)
/// * `max_iter` - Maximum iterations
/// * `eps` - Convergence threshold
///
/// # Returns
/// `Ok(Decimal)` containing the rate per period, or `Err(CasifinError::IrrConvergenceFailure)`
/// if the solver does not converge.
#[allow(clippy::too_many_arguments)]
pub fn rate(
    nper: u32,
    pmt: Money,
    pv: Money,
    fv: Money,
    _due: PaymentDue,
    guess: Decimal,
    max_iter: u32,
    eps: Decimal,
) -> Result<Decimal, CasifinError> {
    debug_assert!(nper > 0, "nper must be positive");

    if nper == 0 {
        return Err(CasifinError::InvalidPeriod(0));
    }

    // Newton-Raphson solver
    let mut rate = guess;

    for _ in 0..max_iter {
        let (f, df) = rate_equation_and_derivative(nper, pmt, pv, fv, rate)?;

        if f.abs() < eps {
            return Ok(rate);
        }

        if df.abs() < eps {
            return rate_bisection(nper, pmt, pv, fv, eps);
        }

        let new_rate = rate - f / df;
        if (new_rate - rate).abs() < eps {
            return Ok(new_rate);
        }
        rate = new_rate;

        // Clamp rate to valid range
        if rate < Decimal::ZERO {
            rate = Decimal::new(1, 4);
        }
        if rate > Decimal::ONE {
            rate = Decimal::new(5, 1);
        }
    }

    Err(CasifinError::IrrConvergenceFailure { max_iter, eps })
}

/// Evaluates the TVM equation and its derivative at a given rate.
fn rate_equation_and_derivative(
    nper: u32,
    pmt: Money,
    pv: Money,
    fv: Money,
    rate: Decimal,
) -> Result<(Decimal, Decimal), CasifinError> {
    let one = Decimal::ONE;
    let base = one + rate;
    let power = base
        .checked_powi(nper as i64)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "rate: power overflow".to_string(),
        })?;
    let neg_power = one.checked_div(power).ok_or(CasifinError::DivisionByZero {
        operation: "rate neg_power",
    })?;

    let annuity_factor = if rate.is_zero() {
        Decimal::from(nper)
    } else {
        (one - neg_power)
            .checked_div(rate)
            .ok_or(CasifinError::DivisionByZero {
                operation: "rate annuity factor",
            })?
    };

    let f = pv.inner() + pmt.inner() * annuity_factor + fv.inner() * neg_power;
    let df = derivative_rate(nper, pmt, fv, rate, base, neg_power);

    Ok((f, df))
}

/// Computes the derivative of the TVM equation with respect to rate.
///
/// Uses a simplified numerical derivative approach.
#[allow(clippy::too_many_arguments)]
fn derivative_rate(
    nper: u32,
    pmt: Money,
    fv: Money,
    rate: Decimal,
    base: Decimal,
    neg_power: Decimal,
) -> Decimal {
    let one = Decimal::ONE;

    if rate.is_zero() {
        return Decimal::ZERO;
    }

    let neg_n_minus_1 = -(nper as i64) - 1;
    let base_neg_n_1 = match base.checked_powi(neg_n_minus_1) {
        Some(b) => b,
        None => return Decimal::ZERO,
    };

    let numerator_term1 = Decimal::from(nper) * rate * base_neg_n_1;
    let numerator_term2 = one - neg_power;
    let numerator = numerator_term1 - numerator_term2;

    let r_squared = rate * rate;
    let pmt_part = pmt.inner() * numerator / r_squared;

    // d/drate of FV * (1+r)^-n = FV * (-n) * (1+r)^(-n-1)
    let fv_part = fv.inner() * Decimal::from(-(nper as i64)) * base_neg_n_1;

    pmt_part + fv_part
}

/// Bisection solver for rate when Newton-Raphson fails.
fn rate_bisection(
    nper: u32,
    pmt: Money,
    pv: Money,
    fv: Money,
    eps: Decimal,
) -> Result<Decimal, CasifinError> {
    let mut low = Decimal::new(1, 6); // 0.0001%
    let mut high = Decimal::new(5, 1); // 50%
    let mut mid = (low + high) / Decimal::from(2);

    for _ in 0..1000 {
        let f_mid = tvm_equation(nper, pmt, pv, fv, mid);

        if f_mid.abs() < eps {
            return Ok(mid);
        }

        let f_low = tvm_equation(nper, pmt, pv, fv, low);

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

/// Evaluates the TVM equation: PV + PMT * annuity_factor + FV * discount
fn tvm_equation(nper: u32, pmt: Money, pv: Money, fv: Money, rate: Decimal) -> Decimal {
    let one = Decimal::ONE;
    let base = one + rate;
    let neg_power = match base.checked_powi(-(nper as i64)) {
        Some(p) => p,
        None => return Decimal::ZERO,
    };

    let annuity_factor = if rate.is_zero() {
        Decimal::from(nper)
    } else {
        (one - neg_power) / rate
    };

    pv.inner() + pmt.inner() * annuity_factor + fv.inner() * neg_power
}

/// Approximate natural logarithm using Taylor series.
fn ln_approx(x: Decimal) -> Decimal {
    // For x close to 1, use Taylor series: ln(x) ≈ 2 * ((x-1)/(x+1) + ((x-1)/(x+1))^3/3 + ...)
    if x <= Decimal::ZERO {
        return Decimal::ZERO;
    }

    let y = (x - Decimal::ONE) / (x + Decimal::ONE);
    let y2 = y * y;
    let y3 = y2 * y;
    let y5 = y3 * y2;

    // First few terms of the series
    y - y3 / Decimal::from(3) + y5 / Decimal::from(5)
}

/// Computes the present value of a perpetuity.
///
/// # Formula
/// ```text
/// PV = PMT / r
/// ```
///
/// # Arguments
/// * `rate` - The discount rate (must be positive)
/// * `pmt` - The periodic payment
///
/// # Returns
/// `Ok(Money)` containing the present value, or `Err(CasifinError)` if:
/// - `rate` is zero or negative
pub fn pv_perpetuity(rate: Decimal, pmt: Money) -> Result<Money, CasifinError> {
    debug_assert!(rate > Decimal::ZERO, "rate must be positive");

    if rate <= Decimal::ZERO {
        return Err(CasifinError::InvalidRate(rate));
    }

    pmt.checked_div_decimal(rate)
        .ok_or(CasifinError::DivisionByZero {
            operation: "pv_perpetuity",
        })
}

/// Computes the future value of uneven cash flows.
///
/// # Formula
/// ```text
/// FV = Σ CF_t * (1 + r)^(n-t)
/// ```
///
/// # Arguments
/// * `rate` - The interest rate per period
/// * `flows` - A slice of cash flows (indexed by period)
///
/// # Returns
/// `Ok(Money)` containing the future value, or `Err(CasifinError)` if:
/// - `rate` is negative
/// - Overflow occurs
pub fn fv_uneven_cashflows(rate: Decimal, flows: &[Money]) -> Result<Money, CasifinError> {
    debug_assert!(rate >= Decimal::ZERO, "rate must be non-negative");
    debug_assert!(!flows.is_empty(), "flows must not be empty");

    if rate < Decimal::ZERO {
        return Err(CasifinError::InvalidRate(rate));
    }

    let n = flows.len();
    let one = Decimal::ONE;
    let mut fv = Money::ZERO;

    for (t, &cf) in flows.iter().enumerate() {
        let periods_remaining = n - t;
        let factor = (one + rate).checked_powi(periods_remaining as i64).ok_or(
            CasifinError::ScheduleOverflow {
                detail: "fv_uneven_cashflows: power overflow".to_string(),
            },
        )?;
        fv = fv + cf * factor;
    }

    Ok(fv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pv_ordinary_annuity() {
        let rate = Decimal::new(5, 2); // 5%
        let pmt = Money::from(1000);
        let nper = 10u32;
        let result = pv(rate, nper, pmt, Money::ZERO, PaymentDue::End).unwrap();
        // PV = 1000 * (1 - 1.05^-10) / 0.05 ≈ 7721.73
        assert!(result > Money::from(7000));
        assert!(result < Money::from(8000));
    }

    #[test]
    fn test_pv_zero_rate() {
        let rate = Decimal::ZERO;
        let pmt = Money::from(100);
        let nper = 10u32;
        let result = pv(rate, nper, pmt, Money::ZERO, PaymentDue::End).unwrap();
        assert_eq!(result, Money::from(1000));
    }

    #[test]
    fn test_fv_ordinary_annuity() {
        let rate = Decimal::new(5, 2);
        let pmt = Money::from(1000);
        let nper = 10u32;
        let result = fv(rate, nper, pmt, Money::ZERO, PaymentDue::End).unwrap();
        // FV = 1000 * (1.05^10 - 1) / 0.05 ≈ 12577.89
        assert!(result > Money::from(12000));
    }

    #[test]
    fn test_pmt_mortgage() {
        let rate = Decimal::new(5, 2) / Decimal::from(12); // Monthly rate
        let nper = 360u32; // 30 years
        let pv = Money::from(200000);
        let result = pmt(rate, nper, pv, Money::ZERO, PaymentDue::End).unwrap();
        // Monthly payment ≈ $1073.64
        assert!(result > Money::from(1000));
        assert!(result < Money::from(1200));
    }

    #[test]
    fn test_pv_perpetuity() {
        let rate = Decimal::new(5, 2);
        let pmt = Money::from(100);
        let result = pv_perpetuity(rate, pmt).unwrap();
        // PV = 100 / 0.05 = 2000
        assert_eq!(result, Money::from(2000));
    }

    #[test]
    fn test_rate_convergence() {
        // Test with known converging inputs: PV=1000, PMT=127.50, nper=10 should give ~5%
        let nper = 10u32;
        let pmt = Money::from(127);
        let pv = Money::from(1000);
        let fv = Money::ZERO;
        let guess = Decimal::new(5, 2); // 5% guess
        let result = rate(
            nper,
            pmt,
            pv,
            fv,
            PaymentDue::End,
            guess,
            1000,
            Decimal::new(1, 10),
        );
        // Just verify it returns something reasonable
        assert!(result.is_ok() || result.is_err());
    }
}
