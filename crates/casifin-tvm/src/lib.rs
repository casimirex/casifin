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
// Helpers
// ============================================================================

/// Converts a `PaymentDue` value into the numeric flag used by TVM formulas.
///
/// `End` maps to `0`, `Beginning` maps to `1`.
///
/// # Panics
/// This function does not panic.
fn due_flag(due: PaymentDue) -> Decimal {
    match due {
        PaymentDue::End => Decimal::ZERO,
        PaymentDue::Beginning => Decimal::ONE,
    }
}

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
/// * `rate` - The interest rate per period.
/// * `nper` - The total number of payment periods.
/// * `pmt` - The payment made each period.
/// * `fv` - The future value (cash balance after last payment).
/// * `due` - Whether payments are due at beginning or end of period.
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
    debug_assert!(
        rate.annual_rate >= Decimal::ZERO,
        "annual_rate must be non-negative"
    );

    if nper == 0 {
        return Err(CasifinError::InvalidPeriod(0));
    }

    let r = rate.periodic_rate()?;
    let one = Decimal::ONE;

    if r.is_zero() {
        let total_pmt = (pmt * Decimal::from(nper)).inner();
        let pv_zero = fv
            .inner()
            .checked_add(total_pmt)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "pv: zero-rate overflow".to_string(),
            })?;
        return Ok(Money::from_decimal(
            pv_zero
                .checked_mul(Decimal::NEGATIVE_ONE)
                .ok_or(CasifinError::ScheduleOverflow {
                    detail: "negation overflow".to_string(),
                })?,
        ));
    }

    let base = one.checked_add(r).ok_or(CasifinError::ScheduleOverflow {
        detail: "pv: base overflow".to_string(),
    })?;
    let neg_power = base
        .checked_powi(-(nper as i64))
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "pv: power calculation overflow".to_string(),
        })?;

    let one_minus_neg_power = one
        .checked_sub(neg_power)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "pv: subtraction overflow".to_string(),
        })?;
    let annuity_factor =
        one_minus_neg_power
            .checked_div(r)
            .ok_or(CasifinError::DivisionByZero {
                operation: "pv annuity factor",
            })?;

    let due_mult = match due {
        PaymentDue::End => Decimal::ONE,
        PaymentDue::Beginning => base,
    };

    let pv_pmt = pmt
        .inner()
        .checked_mul(annuity_factor)
        .and_then(|v| v.checked_mul(due_mult))
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "pv: payment term overflow".to_string(),
        })?;

    let fv_discounted =
        fv.inner()
            .checked_mul(neg_power)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "pv: fv discount overflow".to_string(),
            })?;

    let result = pv_pmt
        .checked_add(fv_discounted)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "pv: result overflow".to_string(),
        })?;

    Ok(Money::from_decimal(result))
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
/// * `rate` - The interest rate per period.
/// * `nper` - The total number of payment periods.
/// * `pmt` - The payment made each period.
/// * `pv` - The present value (initial investment).
/// * `due` - Whether payments are due at beginning or end of period.
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
    debug_assert!(
        rate.annual_rate >= Decimal::ZERO,
        "annual_rate must be non-negative"
    );

    if nper == 0 {
        return Err(CasifinError::InvalidPeriod(0));
    }

    let r = rate.periodic_rate()?;
    let one = Decimal::ONE;

    if r.is_zero() {
        let total_pmt = (pmt * Decimal::from(nper)).inner();
        let fv_zero = pv
            .inner()
            .checked_add(total_pmt)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "fv: zero-rate overflow".to_string(),
            })?;
        return Ok(Money::from_decimal(
            fv_zero
                .checked_mul(Decimal::NEGATIVE_ONE)
                .ok_or(CasifinError::ScheduleOverflow {
                    detail: "negation overflow".to_string(),
                })?,
        ));
    }

    let base = one.checked_add(r).ok_or(CasifinError::ScheduleOverflow {
        detail: "fv: base overflow".to_string(),
    })?;
    let power = base
        .checked_powi(nper as i64)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "fv: power calculation overflow".to_string(),
        })?;

    let power_minus_one = power
        .checked_sub(one)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "fv: subtraction overflow".to_string(),
        })?;
    let fv_factor = power_minus_one
        .checked_div(r)
        .ok_or(CasifinError::DivisionByZero {
            operation: "fv factor",
        })?;

    let due_mult = match due {
        PaymentDue::End => Decimal::ONE,
        PaymentDue::Beginning => base,
    };

    let fv_pmt = pmt
        .inner()
        .checked_mul(fv_factor)
        .and_then(|v| v.checked_mul(due_mult))
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "fv: payment term overflow".to_string(),
        })?;

    let fv_pv = pv
        .inner()
        .checked_mul(power)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "fv: pv growth overflow".to_string(),
        })?;

    let result = fv_pmt
        .checked_add(fv_pv)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "fv: result overflow".to_string(),
        })?;

    Ok(Money::from_decimal(result))
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
/// If due == Beginning: PMT = PMT_end / (1 + r)
/// ```
///
/// # Arguments
/// * `rate` - The interest rate per period.
/// * `nper` - The total number of payment periods.
/// * `pv` - The present value (loan amount or investment).
/// * `fv` - The future value (desired balance after last payment).
/// * `due` - Whether payments are due at beginning or end of period.
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
    debug_assert!(
        rate.annual_rate >= Decimal::ZERO,
        "annual_rate must be non-negative"
    );

    if nper == 0 {
        return Err(CasifinError::InvalidPeriod(0));
    }

    let r = rate.periodic_rate()?;
    let one = Decimal::ONE;

    if r.is_zero() {
        let total = pv
            .inner()
            .checked_add(fv.inner())
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "pmt: zero-rate overflow".to_string(),
            })?;
        let pmt_zero =
            total
                .checked_div(Decimal::from(nper))
                .ok_or(CasifinError::DivisionByZero {
                    operation: "pmt zero rate",
                })?;
        return Ok(Money::from_decimal(
            pmt_zero
                .checked_mul(Decimal::NEGATIVE_ONE)
                .ok_or(CasifinError::ScheduleOverflow {
                    detail: "negation overflow".to_string(),
                })?,
        ));
    }

    let base = one.checked_add(r).ok_or(CasifinError::ScheduleOverflow {
        detail: "pmt: base overflow".to_string(),
    })?;
    let power = base
        .checked_powi(nper as i64)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "pmt: power calculation overflow".to_string(),
        })?;

    let neg_power = one.checked_div(power).ok_or(CasifinError::DivisionByZero {
        operation: "pmt neg_power",
    })?;

    let denominator = one
        .checked_sub(neg_power)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "pmt: denominator overflow".to_string(),
        })?;

    let power_minus_one = power
        .checked_sub(one)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "pmt: power_minus_one overflow".to_string(),
        })?;
    let fv_rate = fv
        .inner()
        .checked_mul(r)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "pmt: fv_rate overflow".to_string(),
        })?;
    let fv_term = fv_rate
        .checked_div(power_minus_one)
        .ok_or(CasifinError::DivisionByZero {
            operation: "pmt fv_term",
        })?;

    let pv_rate = pv
        .inner()
        .checked_mul(r)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "pmt: pv_rate overflow".to_string(),
        })?;
    let numerator = pv_rate
        .checked_add(fv_term)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "pmt: numerator overflow".to_string(),
        })?;
    let pmt_ordinary = numerator
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

    Ok(Money::from_decimal(
        pmt_value
            .checked_mul(Decimal::NEGATIVE_ONE)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "negation overflow".to_string(),
            })?,
    ))
}

// ============================================================================
// Number of Periods
// ============================================================================

/// Computes the number of periods required to reach a future value.
///
/// # Formula
/// ```text
/// If rate == 0: nper = -(PV + FV) / PMT
/// Else:
///   nper = ln((PMT*(1+r*due) - FV*r) / (PMT*(1+r*due) + PV*r)) / ln(1+r)
/// ```
///
/// # Arguments
/// * `rate` - The interest rate per period.
/// * `pmt` - The payment made each period.
/// * `pv` - The present value.
/// * `fv` - The future value.
/// * `due` - Whether payments are due at beginning or end of period.
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
    due: PaymentDue,
) -> Result<Decimal, CasifinError> {
    debug_assert!(!pmt.is_zero(), "pmt must be non-zero");
    debug_assert!(
        rate.annual_rate >= Decimal::ZERO,
        "annual_rate must be non-negative"
    );

    if pmt.is_zero() {
        return Err(CasifinError::InvalidAmount(pmt));
    }

    let r = rate.periodic_rate()?;
    let one = Decimal::ONE;

    if r.is_zero() {
        let total = pv
            .inner()
            .checked_add(fv.inner())
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "nper: zero-rate overflow".to_string(),
            })?;
        let periods = total
            .checked_div(pmt.inner())
            .ok_or(CasifinError::DivisionByZero {
                operation: "nper zero rate",
            })?;
        return periods
            .checked_mul(Decimal::NEGATIVE_ONE)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "nper: negation overflow".to_string(),
            });
    }

    let due_mult = one
        .checked_add(
            r.checked_mul(due_flag(due))
                .ok_or(CasifinError::ScheduleOverflow {
                    detail: "nper: due flag overflow".to_string(),
                })?,
        )
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "nper: due_mult overflow".to_string(),
        })?;

    let pmt_due = pmt
        .inner()
        .checked_mul(due_mult)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "nper: pmt_due overflow".to_string(),
        })?;
    let fv_r = fv
        .inner()
        .checked_mul(r)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "nper: fv_r overflow".to_string(),
        })?;
    let pv_r = pv
        .inner()
        .checked_mul(r)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "nper: pv_r overflow".to_string(),
        })?;

    let numerator = pmt_due
        .checked_sub(fv_r)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "nper: numerator overflow".to_string(),
        })?;
    let denominator = pmt_due
        .checked_add(pv_r)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "nper: denominator overflow".to_string(),
        })?;

    let ratio = numerator
        .checked_div(denominator)
        .ok_or(CasifinError::DivisionByZero {
            operation: "nper ratio",
        })?;

    if ratio <= Decimal::ZERO {
        return Err(CasifinError::InvalidInput {
            reason: "nper: ratio must be positive; loan cannot be paid off".to_string(),
        });
    }

    let ln_ratio = ratio.ln();
    let base = one.checked_add(r).ok_or(CasifinError::ScheduleOverflow {
        detail: "nper: base overflow".to_string(),
    })?;
    let ln_base = base.ln();

    if ln_base.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "nper ln_base",
        });
    }

    ln_ratio
        .checked_div(ln_base)
        .ok_or(CasifinError::DivisionByZero {
            operation: "nper ln division",
        })
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
/// * `nper` - The total number of payment periods.
/// * `pmt` - The payment made each period.
/// * `pv` - The present value.
/// * `fv` - The future value.
/// * `due` - Whether payments are due at beginning or end of period.
/// * `guess` - Initial guess for the rate (defaults to Config::guess if None).
/// * `config` - Solver configuration (eps, max_iterations).
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
    debug_assert!(config.eps > Decimal::ZERO, "eps must be positive");
    debug_assert!(config.max_iterations > 0, "max_iterations must be positive");

    if nper == 0 {
        return Err(CasifinError::InvalidPeriod(0));
    }

    let df = due_flag(due);
    let mut rate_val = guess.unwrap_or(config.guess);

    for _ in 0..config.max_iterations {
        let (f, f_prime) = rate_equation_and_derivative(nper, pmt, pv, fv, rate_val, df)?;

        if f.abs() < config.eps {
            return Ok(rate_val);
        }

        if f_prime.abs() < config.eps {
            return rate_bisection(nper, pmt, pv, fv, df, config);
        }

        let step = f.checked_div(f_prime).ok_or(CasifinError::DivisionByZero {
            operation: "rate newton step",
        })?;
        let new_rate = rate_val
            .checked_sub(step)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "rate: newton overflow".to_string(),
            })?;

        if (new_rate - rate_val).abs() < config.eps {
            return Ok(new_rate);
        }
        rate_val = new_rate;
    }

    Err(CasifinError::IrrConvergenceFailure {
        max_iter: config.max_iterations,
        eps: config.eps,
    })
}

/// Evaluates the TVM equation and its analytical derivative at a given rate.
///
/// # Formula
/// ```text
/// f(r)  = PV*(1+r)^n + PMT*((1+r)^n - 1)/r * (1+r*due) + FV
/// f'(r) = PV*n*(1+r)^(n-1)
///         + PMT * [ (n*(1+r)^(n-1)/r - ((1+r)^n - 1)/r^2) * (1+r*due)
///                   + ((1+r)^n - 1)/r * due ]
/// ```
#[allow(clippy::too_many_arguments)]
fn rate_equation_and_derivative(
    nper: u32,
    pmt: Money,
    pv: Money,
    fv: Money,
    r: Decimal,
    due_flag: Decimal,
) -> Result<(Decimal, Decimal), CasifinError> {
    let one = Decimal::ONE;
    let n = Decimal::from(nper);

    let base = one.checked_add(r).ok_or(CasifinError::ScheduleOverflow {
        detail: "rate: base overflow".to_string(),
    })?;

    let power = base
        .checked_powi(nper as i64)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "rate: power overflow".to_string(),
        })?;
    let power_prev = if nper == 1 {
        one
    } else {
        base.checked_powi((nper - 1) as i64)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "rate: power_prev overflow".to_string(),
            })?
    };

    // Annuity factor A = ((1+r)^n - 1) / r, with limit n when r -> 0.
    let (annuity_factor, annuity_derivative) = if r.is_zero() {
        let a = n;
        let da =
            n.checked_mul(n.checked_sub(one).ok_or(CasifinError::ScheduleOverflow {
                detail: "rate: n-1 overflow".to_string(),
            })?)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "rate: annuity derivative overflow".to_string(),
            })? / Decimal::from(2);
        (a, da)
    } else {
        let power_minus_one = power
            .checked_sub(one)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "rate: power_minus_one overflow".to_string(),
            })?;
        let a = power_minus_one
            .checked_div(r)
            .ok_or(CasifinError::DivisionByZero {
                operation: "rate annuity factor",
            })?;
        let da = n
            .checked_mul(power_prev)
            .and_then(|v| v.checked_sub(a))
            .and_then(|v| v.checked_div(r))
            .ok_or(CasifinError::DivisionByZero {
                operation: "rate annuity derivative",
            })?;
        (a, da)
    };

    let due_factor = one
        .checked_add(
            r.checked_mul(due_flag)
                .ok_or(CasifinError::ScheduleOverflow {
                    detail: "rate: due flag overflow".to_string(),
                })?,
        )
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "rate: due_factor overflow".to_string(),
        })?;

    // f(r) = PV*power + PMT*annuity_factor*due_factor + FV
    let pv_term = pv
        .inner()
        .checked_mul(power)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "rate: pv_term overflow".to_string(),
        })?;
    let pmt_term = pmt
        .inner()
        .checked_mul(annuity_factor)
        .and_then(|v| v.checked_mul(due_factor))
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "rate: pmt_term overflow".to_string(),
        })?;
    let f = pv_term
        .checked_add(pmt_term)
        .and_then(|v| v.checked_add(fv.inner()))
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "rate: f overflow".to_string(),
        })?;

    // f'(r) = PV*n*power_prev
    //         + PMT * (da * due_factor + a * due_flag)
    let pv_derivative = pv
        .inner()
        .checked_mul(n)
        .and_then(|v| v.checked_mul(power_prev))
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "rate: pv_derivative overflow".to_string(),
        })?;
    let pmt_derivative_a =
        annuity_derivative
            .checked_mul(due_factor)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "rate: pmt_derivative_a overflow".to_string(),
            })?;
    let pmt_derivative_b =
        annuity_factor
            .checked_mul(due_flag)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "rate: pmt_derivative_b overflow".to_string(),
            })?;
    let pmt_derivative =
        pmt_derivative_a
            .checked_add(pmt_derivative_b)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "rate: pmt_derivative overflow".to_string(),
            })?;
    let pmt_derivative_total =
        pmt.inner()
            .checked_mul(pmt_derivative)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "rate: pmt_derivative_total overflow".to_string(),
            })?;
    let f_prime =
        pv_derivative
            .checked_add(pmt_derivative_total)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "rate: f_prime overflow".to_string(),
            })?;

    Ok((f, f_prime))
}

/// Bisection solver for rate when Newton-Raphson fails.
#[allow(clippy::too_many_arguments)]
fn rate_bisection(
    nper: u32,
    pmt: Money,
    pv: Money,
    fv: Money,
    due_flag: Decimal,
    config: Config,
) -> Result<Decimal, CasifinError> {
    let mut low = Decimal::new(-9999, 4); // -0.9999
    let mut high = Decimal::ONE;
    let mut mid = (low + high) / Decimal::from(2);

    for _ in 0..config.max_iterations {
        let f_mid = tvm_equation(nper, pmt, pv, fv, mid, due_flag);

        if f_mid.abs() < config.eps {
            return Ok(mid);
        }

        let f_low = tvm_equation(nper, pmt, pv, fv, low, due_flag);

        if f_low * f_mid < Decimal::ZERO {
            high = mid;
        } else {
            low = mid;
        }

        let new_mid = (low + high) / Decimal::from(2);
        if (new_mid - mid).abs() < config.eps {
            return Ok(new_mid);
        }
        mid = new_mid;
    }

    Err(CasifinError::IrrConvergenceFailure {
        max_iter: config.max_iterations,
        eps: config.eps,
    })
}

/// Evaluates the TVM equation: f(r) = PV*(1+r)^n + PMT*((1+r)^n - 1)/r * (1+r*due) + FV
#[allow(clippy::too_many_arguments)]
fn tvm_equation(
    nper: u32,
    pmt: Money,
    pv: Money,
    fv: Money,
    r: Decimal,
    due_flag: Decimal,
) -> Decimal {
    let one = Decimal::ONE;
    let base = match one.checked_add(r) {
        Some(b) => b,
        None => return Decimal::ZERO,
    };

    let power = match base.checked_powi(nper as i64) {
        Some(p) => p,
        None => return Decimal::ZERO,
    };

    let annuity_factor = if r.is_zero() {
        Decimal::from(nper)
    } else {
        match power.checked_sub(one).and_then(|v| v.checked_div(r)) {
            Some(af) => af,
            None => return Decimal::ZERO,
        }
    };

    let due_factor = match one.checked_add(r.checked_mul(due_flag).unwrap_or(Decimal::ZERO)) {
        Some(df) => df,
        None => return Decimal::ZERO,
    };

    match pv
        .inner()
        .checked_mul(power)
        .and_then(|v| {
            pmt.inner()
                .checked_mul(annuity_factor)
                .and_then(|w| w.checked_mul(due_factor))
                .map(|w| v + w)
        })
        .and_then(|v| v.checked_add(fv.inner()))
    {
        Some(result) => result,
        None => Decimal::ZERO,
    }
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
/// * `rate` - The discount rate (must be positive).
/// * `pmt` - The periodic payment.
///
/// # Returns
/// `Ok(Money)` containing the present value, or `Err(CasifinError)` if rate is zero.
///
/// # Panics
/// This function does not panic.
pub fn pv_perpetuity(rate: Rate, pmt: Money) -> Result<Money, CasifinError> {
    debug_assert!(
        rate.annual_rate >= Decimal::ZERO,
        "annual_rate must be non-negative"
    );

    let r = rate.periodic_rate()?;

    debug_assert!(r >= Decimal::ZERO, "periodic rate must be non-negative");

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
/// * `rate` - The interest rate per period.
/// * `flows` - A slice of cash flows (indexed by period).
///
/// # Returns
/// `Ok(Money)` containing the future value, or `Err(CasifinError)` on invalid input.
///
/// # Panics
/// This function does not panic.
pub fn fv_uneven_cashflows(rate: Rate, flows: &[Money]) -> Result<Money, CasifinError> {
    debug_assert!(!flows.is_empty(), "flows must not be empty");
    debug_assert!(
        rate.annual_rate >= Decimal::ZERO,
        "annual_rate must be non-negative"
    );

    if flows.is_empty() {
        return Err(CasifinError::InvalidInput {
            reason: "fv_uneven_cashflows: flows must not be empty".to_string(),
        });
    }

    let r = rate.periodic_rate()?;
    let n = flows.len();
    let one = Decimal::ONE;
    let mut total = Decimal::ZERO;

    for (t, &cf) in flows.iter().enumerate() {
        let periods_remaining = n - t;
        let base = one.checked_add(r).ok_or(CasifinError::ScheduleOverflow {
            detail: "fv_uneven_cashflows: base overflow".to_string(),
        })?;
        let factor =
            base.checked_powi(periods_remaining as i64)
                .ok_or(CasifinError::ScheduleOverflow {
                    detail: "fv_uneven_cashflows: power overflow".to_string(),
                })?;
        let contribution =
            cf.inner()
                .checked_mul(factor)
                .ok_or(CasifinError::ScheduleOverflow {
                    detail: "fv_uneven_cashflows: contribution overflow".to_string(),
                })?;
        total = total
            .checked_add(contribution)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "fv_uneven_cashflows: total overflow".to_string(),
            })?;
    }

    Ok(Money::from_decimal(total))
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
        // PV of $1000/year for 5 years at 5% = $4,329.476671...
        let rate = make_rate(5, 1);
        let result = pv(
            rate,
            5,
            Money::from_decimal(Decimal::new(1000, 0)),
            Money::ZERO,
            PaymentDue::End,
        )
        .unwrap();
        let expected = Decimal::new(4329476671_i64, 6); // 4329.476671
        assert_near(result.inner(), expected, Decimal::new(1, 6));
    }

    #[test]
    fn pv_annuity_begin() {
        // PV of $1000/year for 5 years at 5%, beginning = $4,545.950504...
        let rate = make_rate(5, 1);
        let result = pv(
            rate,
            5,
            Money::from_decimal(Decimal::new(1000, 0)),
            Money::ZERO,
            PaymentDue::Beginning,
        )
        .unwrap();
        let expected = Decimal::new(4545950504_i64, 6); // 4545.950504
        assert_near(result.inner(), expected, Decimal::new(1, 6));
    }

    #[test]
    fn fv_annuity_end() {
        // FV of $1000/year for 5 years at 5% = $5,525.631250
        let rate = make_rate(5, 1);
        let result = fv(
            rate,
            5,
            Money::from_decimal(Decimal::new(1000, 0)),
            Money::ZERO,
            PaymentDue::End,
        )
        .unwrap();
        let expected = Decimal::new(5525631250_i64, 6); // 5525.631250
        assert_near(result.inner(), expected, Decimal::new(1, 6));
    }

    #[test]
    fn pmt_mortgage() {
        // PMT on $300,000 at 4.25% for 30 years (360 months) ≈ -$1,475.82
        let rate = Rate::new(Decimal::new(425, 4), Compounding::Discrete(12)).unwrap();
        let result = pmt(
            rate,
            360,
            Money::from_decimal(Decimal::new(300000, 0)),
            Money::ZERO,
            PaymentDue::End,
        )
        .unwrap();
        let expected = Decimal::new(1475817865_i64, 6); // 1475.817865 (Excel reference)
        assert_near(result.inner().abs(), expected, Decimal::new(1, 2));
    }

    #[test]
    fn nper_loan() {
        // NPER to pay off $10,000 at $200/month, 0% = 50 periods
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
        // RATE to grow $1000 to $2000 with $0 PMT over 10 years = 7.177346%
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
        let expected = Decimal::new(7177346, 8); // 0.07177346
        assert_near(result, expected, Decimal::new(1, 6));
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
        // Tight epsilon and one iteration forces convergence failure.
        let config = Config::builder()
            .max_iterations(1)
            .eps(Decimal::new(1, 15))
            .build();
        let result = rate(
            10,
            Money::from_decimal(Decimal::new(-100, 0)),
            Money::from_decimal(Decimal::new(1000, 0)),
            Money::from_decimal(Decimal::new(2000, 0)),
            PaymentDue::End,
            Some(Decimal::new(1, 1)),
            config,
        );
        assert!(matches!(
            result,
            Err(CasifinError::IrrConvergenceFailure { .. })
        ));
    }
}
