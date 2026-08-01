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

use casifin_core::{CasifinError, Config, Money, PaymentDue, Rate};
use rust_decimal::{Decimal, MathematicalOps};

// ============================================================================
// Present Value
// ============================================================================

/// Computes the present value of an annuity.
///
/// # Formula
/// ```text
/// If rate == 0: PV = -(FV + PMT * nper)
/// If due == End:  PV = [PMT * (1 - (1+r)^-n) / r] + FV * (1+r)^-n
/// If due == Beginning: multiply PMT term by (1+r)
/// ```
///
/// # Arguments
/// * `rate` - The interest rate per period
/// * `nper` - The total number of payment periods
/// * `pmt` - The payment made each period
/// * `fv` - The future value (cash balance after last payment)
/// * `due` - Whether payments are due at beginning or end of period
///
/// # Returns
/// `Ok(Money)` containing the present value, or `Err(CasifinError)` on invalid input.
///
/// # Example
/// ```
/// use casifin_core::{Money, Rate, Compounding, PaymentDue};
/// use casifin_tvm::pv;
/// use rust_decimal::Decimal;
///
/// let rate = Rate::new(Decimal::new(5, 2), Compounding::Discrete(1)).unwrap();
/// let result = pv(rate, 5, Money::from_decimal(Decimal::new(1000, 0)), Money::ZERO, PaymentDue::End).unwrap();
/// ```
///
/// # Panics
/// This function does not panic.
pub fn pv(
    rate: Rate,
    nper: u32,
    pmt: Money,
    fv: Money,
    due: PaymentDue,
) -> Result<Money, CasifinError> {
    debug_assert!(nper > 0, "nper must be positive");

    if nper == 0 {
        return Err(CasifinError::InvalidPeriod(0));
    }

    let r = rate.periodic_rate()?;
    let one = Decimal::ONE;

    if r.is_zero() {
        let total_pmt = pmt * Decimal::from(nper);
        return Ok(fv + total_pmt);
    }

    let base = one + r;
    let neg_power = base
        .checked_powi(-(nper as i64))
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "pv: power calculation overflow".to_string(),
        })?;

    let annuity_factor = (one - neg_power)
        .checked_div(r)
        .ok_or(CasifinError::DivisionByZero {
            operation: "pv annuity factor",
        })?;

    let pv_pmt = match due {
        PaymentDue::End => pmt * annuity_factor,
        PaymentDue::Beginning => pmt * annuity_factor * base,
    };

    let fv_discounted = fv * neg_power;

    Ok(pv_pmt + fv_discounted)
}

// ============================================================================
// Future Value
// ============================================================================

/// Computes the future value of an annuity.
///
/// # Formula
/// ```text
/// If rate == 0: FV = -(PV + PMT * nper)
/// If due == End:  FV = [PMT * ((1+r)^n - 1) / r] + PV * (1+r)^n
/// If due == Beginning: multiply PMT term by (1+r)
/// ```
///
/// # Arguments
/// * `rate` - The interest rate per period
/// * `nper` - The total number of payment periods
/// * `pmt` - The payment made each period
/// * `pv` - The present value (initial investment)
/// * `due` - Whether payments are due at beginning or end of period
///
/// # Returns
/// `Ok(Money)` containing the future value, or `Err(CasifinError)` on invalid input.
///
/// # Panics
/// This function does not panic.
pub fn fv(
    rate: Rate,
    nper: u32,
    pmt: Money,
    pv: Money,
    due: PaymentDue,
) -> Result<Money, CasifinError> {
    debug_assert!(nper > 0, "nper must be positive");

    if nper == 0 {
        return Err(CasifinError::InvalidPeriod(0));
    }

    let r = rate.periodic_rate()?;
    let one = Decimal::ONE;

    if r.is_zero() {
        let total_pmt = pmt * Decimal::from(nper);
        return Ok(pv + total_pmt);
    }

    let base = one + r;
    let power = base
        .checked_powi(nper as i64)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "fv: power calculation overflow".to_string(),
        })?;

    let fv_factor = (power - one)
        .checked_div(r)
        .ok_or(CasifinError::DivisionByZero {
            operation: "fv factor",
        })?;

    let fv_pmt = match due {
        PaymentDue::End => pmt * fv_factor,
        PaymentDue::Beginning => pmt * fv_factor * base,
    };

    let fv_pv = pv * power;

    Ok(fv_pmt + fv_pv)
}

// ============================================================================
// Payment
// ============================================================================

/// Computes the payment amount for an annuity.
///
/// # Formula
/// ```text
/// If rate == 0: PMT = -(PV + FV) / nper
/// If due == End:  PMT = -(PV*r + FV*r/((1+r)^n - 1)) / (1 - (1+r)^-n)
/// ```
///
/// # Arguments
/// * `rate` - The interest rate per period
/// * `nper` - The total number of payment periods
/// * `pv` - The present value (loan amount or investment)
/// * `fv` - The future value (desired balance after last payment)
/// * `due` - Whether payments are due at beginning or end of period
///
/// # Returns
/// `Ok(Money)` containing the payment amount, or `Err(CasifinError)` on invalid input.
///
/// # Panics
/// This function does not panic.
pub fn pmt(
    rate: Rate,
    nper: u32,
    pv: Money,
    fv: Money,
    due: PaymentDue,
) -> Result<Money, CasifinError> {
    debug_assert!(nper > 0, "nper must be positive");

    if nper == 0 {
        return Err(CasifinError::InvalidPeriod(0));
    }

    let r = rate.periodic_rate()?;
    let one = Decimal::ONE;

    if r.is_zero() {
        let total = -(pv + fv);
        return total.checked_div_decimal(Decimal::from(nper)).ok_or(
            CasifinError::DivisionByZero {
                operation: "pmt zero rate",
            },
        );
    }

    let base = one + r;
    let power = base
        .checked_powi(nper as i64)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "pmt: power calculation overflow".to_string(),
        })?;

    let neg_power = one.checked_div(power).ok_or(CasifinError::DivisionByZero {
        operation: "pmt neg_power",
    })?;

    let denominator = one - neg_power;

    // PMT = -(PV*r + FV*r/((1+r)^n - 1)) / (1 - (1+r)^-n)
    let power_minus_one = power - one;
    let fv_term = fv.inner()
        * r.checked_div(power_minus_one)
            .ok_or(CasifinError::DivisionByZero {
                operation: "pmt fv_term",
            })?;
    let pmt_ordinary = -(pv.inner() * r + fv_term);

    let pmt_ordinary =
        pmt_ordinary
            .checked_div(denominator)
            .ok_or(CasifinError::DivisionByZero {
                operation: "pmt denominator",
            })?;

    let pmt_value = match due {
        PaymentDue::End => pmt_ordinary,
        PaymentDue::Beginning => {
            pmt_ordinary
                .checked_div(base)
                .ok_or(CasifinError::DivisionByZero {
                    operation: "pmt annuity-due",
                })?
        }
    };

    Ok(Money::from_decimal(pmt_value))
}

// ============================================================================
// Number of Periods
// ============================================================================

/// Computes the number of periods required to reach a future value.
///
/// # Formula
/// ```text
/// If rate == 0: nper = -(PV + FV) / PMT
/// Else: nper = ln((PMT - FV*r) / (PMT + PV*r)) / ln(1+r)
/// ```
///
/// # Arguments
/// * `rate` - The interest rate per period
/// * `pmt` - The payment made each period
/// * `pv` - The present value
/// * `fv` - The future value
/// * `due` - Whether payments are due at beginning or end of period
///
/// # Returns
/// `Ok(Decimal)` containing the number of periods, or `Err(CasifinError)` on invalid input.
///
/// # Panics
/// This function does not panic.
pub fn nper(
    rate: Rate,
    pmt: Money,
    pv: Money,
    fv: Money,
    _due: PaymentDue,
) -> Result<Decimal, CasifinError> {
    debug_assert!(!pmt.is_zero(), "pmt must be non-zero");

    if pmt.is_zero() {
        return Err(CasifinError::InvalidAmount(pmt));
    }

    let r = rate.periodic_rate()?;
    let one = Decimal::ONE;

    if r.is_zero() {
        let total = -(pv + fv);
        return total
            .inner()
            .checked_div(pmt.inner())
            .ok_or(CasifinError::DivisionByZero {
                operation: "nper zero rate",
            });
    }

    // nper = ln((PMT - FV*r) / (PMT + PV*r)) / ln(1+r)
    let fv_r = fv.inner() * r;
    let pv_r = pv.inner() * r;

    let numerator = pmt.inner() - fv_r;
    let denominator = pmt.inner() + pv_r;

    let ratio = numerator
        .checked_div(denominator)
        .ok_or(CasifinError::DivisionByZero {
            operation: "nper ratio",
        })?;

    if ratio <= Decimal::ZERO {
        return Err(CasifinError::ScheduleOverflow {
            detail: "nper: invalid ratio - loan cannot be paid off".to_string(),
        });
    }

    let ln_ratio = ratio.ln();
    let ln_base = (one + r).ln();

    if ln_base.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "nper ln_base",
        });
    }

    Ok(ln_ratio / ln_base)
}

// ============================================================================
// Interest Rate (Newton-Raphson + Bisection)
// ============================================================================

/// Computes the interest rate per period for an annuity.
///
/// Uses a hybrid Newton-Raphson / bisection solver.
///
/// # Formula
/// ```text
/// f(r) = PV*(1+r)^n + PMT*((1+r)^n - 1)/r * (1+r*due) + FV = 0
/// ```
/// where due = 0 for End, due = 1 for Beginning.
///
/// # Arguments
/// * `nper` - The total number of payment periods
/// * `pmt` - The payment made each period
/// * `pv` - The present value
/// * `fv` - The future value
/// * `due` - Whether payments are due at beginning or end of period
/// * `guess` - Initial guess for the rate (defaults to Config::guess if None)
/// * `config` - Solver configuration (eps, max_iterations)
///
/// # Returns
/// `Ok(Decimal)` containing the rate per period, or `Err(CasifinError::IrrConvergenceFailure)`
/// if the solver does not converge.
///
/// # Panics
/// This function does not panic.
#[allow(clippy::too_many_arguments)]
pub fn rate(
    nper: u32,
    pmt: Money,
    pv: Money,
    fv: Money,
    due: PaymentDue,
    guess: Option<Decimal>,
    config: Config,
) -> Result<Decimal, CasifinError> {
    debug_assert!(nper > 0, "nper must be positive");

    if nper == 0 {
        return Err(CasifinError::InvalidPeriod(0));
    }

    let mut rate_val = guess.unwrap_or(config.guess);
    let due_flag = match due {
        PaymentDue::End => Decimal::ZERO,
        PaymentDue::Beginning => Decimal::ONE,
    };

    for _ in 0..config.max_iterations {
        let (f, df) = rate_eq_and_derivative(nper, pmt, pv, fv, rate_val, due_flag)?;

        if f.abs() < config.eps {
            return Ok(rate_val);
        }

        if df.abs() < config.eps {
            return rate_bisection(nper, pmt, pv, fv, due_flag, config.eps);
        }

        let new_rate = rate_val - f / df;
        if (new_rate - rate_val).abs() < config.eps {
            return Ok(new_rate);
        }
        rate_val = new_rate;

        if rate_val < Decimal::NEGATIVE_ONE + Decimal::new(1, 4) {
            rate_val = Decimal::new(-9999, 4); // -0.9999
        }
        if rate_val > Decimal::ONE {
            rate_val = Decimal::ONE;
        }
    }

    Err(CasifinError::IrrConvergenceFailure {
        max_iter: config.max_iterations,
        eps: config.eps,
    })
}

/// Evaluates the TVM equation and its derivative at a given rate.
#[allow(clippy::too_many_arguments)]
fn rate_eq_and_derivative(
    nper: u32,
    pmt: Money,
    pv: Money,
    fv: Money,
    r: Decimal,
    due_flag: Decimal,
) -> Result<(Decimal, Decimal), CasifinError> {
    let one = Decimal::ONE;
    let base = one + r;

    let power = base
        .checked_powi(nper as i64)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "rate: power overflow".to_string(),
        })?;

    // f(r) = PV*(1+r)^n + PMT*((1+r)^n - 1)/r * (1+r*due) + FV
    let annuity_factor = if r.is_zero() {
        Decimal::from(nper)
    } else {
        (power - one)
            .checked_div(r)
            .ok_or(CasifinError::DivisionByZero {
                operation: "rate annuity factor",
            })?
    };

    let due_factor = one + r * due_flag;
    let f = pv.inner() * power + pmt.inner() * annuity_factor * due_factor + fv.inner();

    // Derivative approximation
    let h = Decimal::new(1, 8); // 1e-8
    let r_plus = r + h;
    let base_plus = one + r_plus;
    let power_plus = base_plus
        .checked_powi(nper as i64)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "rate: derivative power overflow".to_string(),
        })?;
    let annuity_plus = if r_plus.is_zero() {
        Decimal::from(nper)
    } else {
        (power_plus - one)
            .checked_div(r_plus)
            .ok_or(CasifinError::DivisionByZero {
                operation: "rate derivative annuity",
            })?
    };
    let due_plus = one + r_plus * due_flag;
    let f_plus = pv.inner() * power_plus + pmt.inner() * annuity_plus * due_plus + fv.inner();

    let df = (f_plus - f)
        .checked_div(h)
        .ok_or(CasifinError::DivisionByZero {
            operation: "rate derivative",
        })?;

    Ok((f, df))
}

/// Bisection solver for rate when Newton-Raphson fails.
#[allow(clippy::too_many_arguments)]
fn rate_bisection(
    nper: u32,
    pmt: Money,
    pv: Money,
    fv: Money,
    due_flag: Decimal,
    eps: Decimal,
) -> Result<Decimal, CasifinError> {
    let mut low = Decimal::new(-9999, 4); // -0.9999
    let mut high = Decimal::ONE;
    let mut mid = (low + high) / Decimal::from(2);

    for _ in 0..1000 {
        let f_mid = tvm_eq(nper, pmt, pv, fv, mid, due_flag);

        if f_mid.abs() < eps {
            return Ok(mid);
        }

        let f_low = tvm_eq(nper, pmt, pv, fv, low, due_flag);

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

/// Evaluates the TVM equation: f(r) = PV*(1+r)^n + PMT*((1+r)^n - 1)/r * (1+r*due) + FV
#[allow(clippy::too_many_arguments)]
fn tvm_eq(nper: u32, pmt: Money, pv: Money, fv: Money, r: Decimal, due_flag: Decimal) -> Decimal {
    let one = Decimal::ONE;
    let base = one + r;

    let power = match base.checked_powi(nper as i64) {
        Some(p) => p,
        None => return Decimal::ZERO,
    };

    let annuity_factor = if r.is_zero() {
        Decimal::from(nper)
    } else {
        match (power - one).checked_div(r) {
            Some(af) => af,
            None => return Decimal::ZERO,
        }
    };

    let due_factor = one + r * due_flag;
    pv.inner() * power + pmt.inner() * annuity_factor * due_factor + fv.inner()
}

// ============================================================================
// Perpetuity
// ============================================================================

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
/// `Ok(Money)` containing the present value, or `Err(CasifinError)` if rate is zero.
///
/// # Panics
/// This function does not panic.
pub fn pv_perpetuity(rate: Rate, pmt: Money) -> Result<Money, CasifinError> {
    let r = rate.periodic_rate()?;

    debug_assert!(r > Decimal::ZERO, "rate must be positive for perpetuity");

    if r.is_zero() {
        return Err(CasifinError::InvalidRate(r));
    }

    pmt.checked_div_decimal(r)
        .ok_or(CasifinError::DivisionByZero {
            operation: "pv_perpetuity",
        })
}

// ============================================================================
// Future Value of Uneven Cash Flows
// ============================================================================

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
/// `Ok(Money)` containing the future value, or `Err(CasifinError)` on invalid input.
///
/// # Panics
/// This function does not panic.
pub fn fv_uneven_cashflows(rate: Rate, flows: &[Money]) -> Result<Money, CasifinError> {
    debug_assert!(!flows.is_empty(), "flows must not be empty");

    let r = rate.periodic_rate()?;
    let n = flows.len();
    let one = Decimal::ONE;
    let mut fv = Money::ZERO;

    for (t, &cf) in flows.iter().enumerate() {
        let periods_remaining = n - t;
        let factor = (one + r).checked_powi(periods_remaining as i64).ok_or(
            CasifinError::ScheduleOverflow {
                detail: "fv_uneven_cashflows: power overflow".to_string(),
            },
        )?;
        fv = fv + cf * factor;
    }

    Ok(fv)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use casifin_core::Compounding;

    use super::*;

    fn make_rate(annual_pct: i64, periods: u32) -> Rate {
        let dec = Decimal::new(annual_pct, 2);
        Rate::new(dec, Compounding::Discrete(periods)).unwrap()
    }

    fn assert_near(actual: Decimal, expected: Decimal, tolerance: Decimal) {
        let diff = (actual - expected).abs();
        assert!(
            diff <= tolerance,
            "assertion failed: {} not within {} of {}",
            actual,
            tolerance,
            expected
        );
    }

    #[test]
    fn pv_annuity_end() {
        // PV of $1000/year for 5 years at 5% = $4,329.48
        let rate = make_rate(5, 1);
        let result = pv(
            rate,
            5,
            Money::from_decimal(Decimal::new(1000, 0)),
            Money::ZERO,
            PaymentDue::End,
        )
        .unwrap();
        let expected = Decimal::new(432948, 2); // 4329.48
        assert_near(result.inner(), expected, Decimal::new(1, 1));
    }

    #[test]
    fn pv_annuity_begin() {
        // PV of $1000/year for 5 years at 5%, beginning = $4,545.95
        let rate = make_rate(5, 1);
        let result = pv(
            rate,
            5,
            Money::from_decimal(Decimal::new(1000, 0)),
            Money::ZERO,
            PaymentDue::Beginning,
        )
        .unwrap();
        let expected = Decimal::new(454595, 2); // 4545.95
        assert_near(result.inner(), expected, Decimal::new(1, 1));
    }

    #[test]
    fn fv_annuity_end() {
        // FV of $1000/year for 5 years at 5% = $5,525.63
        let rate = make_rate(5, 1);
        let result = fv(
            rate,
            5,
            Money::from_decimal(Decimal::new(1000, 0)),
            Money::ZERO,
            PaymentDue::End,
        )
        .unwrap();
        let expected = Decimal::new(552563, 2); // 5525.63
        assert_near(result.inner(), expected, Decimal::new(1, 1));
    }

    #[test]
    fn pmt_mortgage() {
        // PMT on $300,000 at 4.25% for 30 years (360 months) = -$1,475.82
        // (negative because it is an outflow from borrower's perspective)
        let rate = Rate::new(Decimal::new(425, 4), Compounding::Discrete(12)).unwrap();
        let result = pmt(
            rate,
            360,
            Money::from_decimal(Decimal::new(300000, 0)),
            Money::ZERO,
            PaymentDue::End,
        )
        .unwrap();
        let expected = Decimal::new(147582, 2); // 1475.82
        assert_near(result.inner().abs(), expected, Decimal::new(1, 1));
    }

    #[test]
    fn nper_loan() {
        // NPER to pay off $10,000 at $200/month, 0% = 50 periods
        // PMT is negative (outflow), PV is positive (loan received).
        let rate = Rate::new(Decimal::ZERO, Compounding::Discrete(12)).unwrap();
        let result = nper(
            rate,
            Money::from_decimal(Decimal::new(-200, 0)),
            Money::from_decimal(Decimal::new(10000, 0)),
            Money::ZERO,
            PaymentDue::End,
        )
        .unwrap();
        assert_eq!(result, Decimal::new(50, 0));
    }

    #[test]
    fn rate_savings() {
        // RATE to grow $1000 to $2000 with $0 PMT over 10 years = 7.18%
        let config = Config::default();
        let result = rate(
            10,
            Money::ZERO,
            Money::from_decimal(Decimal::new(-1000, 0)),
            Money::from_decimal(Decimal::new(2000, 0)),
            PaymentDue::End,
            Some(Decimal::new(1, 1)),
            config,
        )
        .unwrap();
        // Should be approximately 7.18%
        assert!(result > Decimal::new(7, 2)); // > 0.07
        assert!(result < Decimal::new(8, 2)); // < 0.08
    }

    #[test]
    fn pv_perpetuity_5pct() {
        // $100 / 0.05 = $2,000
        let rate = make_rate(5, 1);
        let result = pv_perpetuity(rate, Money::from_decimal(Decimal::new(100, 0))).unwrap();
        assert_eq!(result, Money::from_decimal(Decimal::new(2000, 0)));
    }

    #[test]
    fn rate_convergence_failure() {
        // Pathological inputs that should not converge
        let config = Config::builder()
            .max_iterations(10)
            .eps(Decimal::new(1, 15))
            .build();
        let result = rate(
            1000,
            Money::from_decimal(Decimal::new(1, 0)),
            Money::from_decimal(Decimal::new(1, 0)),
            Money::from_decimal(Decimal::new(1000000, 0)),
            PaymentDue::End,
            Some(Decimal::new(5, 1)),
            config,
        );
        assert!(result.is_err());
    }
}
