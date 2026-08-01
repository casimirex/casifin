//! Rate conversion calculations.

use casifin_core::{CasifinError, Compounding, Rate};
use rust_decimal::{Decimal, MathematicalOps};

/// Effective Annual Rate.
///
/// # Formula
/// ```text
/// EAR = (1 + r/n)^n - 1
/// ```
/// For continuous compounding: `EAR = e^r - 1`.
///
/// # Arguments
/// * `stated_rate` - Stated annual rate as a decimal.
/// * `compounding` - Compounding frequency.
///
/// # Returns
/// `Ok(Decimal)` containing the EAR, or `Err(CasifinError::InvalidRate)` when the
/// stated rate is negative.
///
/// # Example
/// ```
/// use casifin_core::Compounding;
/// use casifin_ratios::effective_annual_rate;
/// use rust_decimal::Decimal;
///
/// let ear = effective_annual_rate(Decimal::new(12, 2), Compounding::Discrete(12)).unwrap();
/// assert!(ear > Decimal::new(12, 2) && ear < Decimal::new(13, 2));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Quantitative Methods, Reading 6.
///
/// # Panics
/// This function does not panic.
pub fn effective_annual_rate(
    stated_rate: Decimal,
    compounding: Compounding,
) -> Result<Decimal, CasifinError> {
    debug_assert!(
        stated_rate >= Decimal::ZERO,
        "stated_rate must be non-negative"
    );
    debug_assert!(
        matches!(compounding, Compounding::Discrete(n) if n > 0)
            || matches!(compounding, Compounding::Continuous),
        "compounding must be valid"
    );

    if stated_rate < Decimal::ZERO {
        return Err(CasifinError::InvalidRate(stated_rate));
    }

    let one = Decimal::ONE;
    match compounding {
        Compounding::Discrete(n) => {
            if n == 0 {
                return Err(CasifinError::InvalidCompounding);
            }
            let periodic =
                stated_rate
                    .checked_div(Decimal::from(n))
                    .ok_or(CasifinError::DivisionByZero {
                        operation: "effective_annual_rate periodic",
                    })?;
            let base = one
                .checked_add(periodic)
                .ok_or(CasifinError::ScheduleOverflow {
                    detail: "effective_annual_rate: base overflow".to_string(),
                })?;
            let power =
                base.checked_powd(Decimal::from(n))
                    .ok_or(CasifinError::ScheduleOverflow {
                        detail: "effective_annual_rate: power overflow".to_string(),
                    })?;
            power
                .checked_sub(one)
                .ok_or(CasifinError::ScheduleOverflow {
                    detail: "effective_annual_rate: result overflow".to_string(),
                })
        }
        Compounding::Continuous => {
            let power =
                Decimal::E
                    .checked_powd(stated_rate)
                    .ok_or(CasifinError::ScheduleOverflow {
                        detail: "effective_annual_rate: continuous power overflow".to_string(),
                    })?;
            power
                .checked_sub(one)
                .ok_or(CasifinError::ScheduleOverflow {
                    detail: "effective_annual_rate: result overflow".to_string(),
                })
        }
    }
}

/// Stated Rate from Effective Annual Rate.
///
/// # Formula
/// ```text
/// r = n * ((1 + EAR)^(1/n) - 1)
/// ```
/// For continuous compounding: `r = ln(1 + EAR)`.
///
/// # Arguments
/// * `effective_rate` - Effective annual rate as a decimal.
/// * `compounding` - Compounding frequency.
///
/// # Returns
/// `Ok(Decimal)` containing the stated rate, or `Err(CasifinError::InvalidCompounding)`
/// when `compounding` is discrete with zero periods, or `Err(CasifinError::InvalidRate)` when
/// the logarithm argument is non-positive.
///
/// # Example
/// ```
/// use casifin_core::Compounding;
/// use casifin_ratios::{effective_annual_rate, stated_from_effective};
/// use rust_decimal::Decimal;
///
/// let ear = effective_annual_rate(Decimal::new(12, 2), Compounding::Discrete(12)).unwrap();
/// let stated = stated_from_effective(ear, Compounding::Discrete(12)).unwrap();
/// assert!((stated - Decimal::new(12, 2)).abs() < Decimal::new(1, 10));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Quantitative Methods, Reading 6.
///
/// # Panics
/// This function does not panic.
pub fn stated_from_effective(
    effective_rate: Decimal,
    compounding: Compounding,
) -> Result<Decimal, CasifinError> {
    debug_assert!(
        effective_rate >= Decimal::NEGATIVE_ONE,
        "effective_rate must be greater than -100%"
    );
    debug_assert!(
        matches!(compounding, Compounding::Discrete(n) if n > 0)
            || matches!(compounding, Compounding::Continuous),
        "compounding must be valid"
    );

    let one = Decimal::ONE;
    match compounding {
        Compounding::Discrete(n) => {
            if n == 0 {
                return Err(CasifinError::InvalidCompounding);
            }
            let exponent =
                one.checked_div(Decimal::from(n))
                    .ok_or(CasifinError::DivisionByZero {
                        operation: "stated_from_effective exponent",
                    })?;
            let base = one
                .checked_add(effective_rate)
                .ok_or(CasifinError::ScheduleOverflow {
                    detail: "stated_from_effective: base overflow".to_string(),
                })?;
            let root = base
                .checked_powd(exponent)
                .ok_or(CasifinError::ScheduleOverflow {
                    detail: "stated_from_effective: root overflow".to_string(),
                })?;
            let spread = root
                .checked_sub(one)
                .ok_or(CasifinError::ScheduleOverflow {
                    detail: "stated_from_effective: spread overflow".to_string(),
                })?;
            spread
                .checked_mul(Decimal::from(n))
                .ok_or(CasifinError::ScheduleOverflow {
                    detail: "stated_from_effective: result overflow".to_string(),
                })
        }
        Compounding::Continuous => one
            .checked_add(effective_rate)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "stated_from_effective: base overflow".to_string(),
            })?
            .checked_ln()
            .ok_or(CasifinError::InvalidRate(effective_rate)),
    }
}

/// Continuous Rate to Nominal Rate.
///
/// # Formula
/// ```text
/// r = ln(1 + continuous_rate)
/// ```
///
/// # Arguments
/// * `continuous_rate` - Continuously compounded rate as a decimal.
///
/// # Returns
/// `Ok(Decimal)` containing the equivalent nominal rate, or `Err(CasifinError::InvalidRate)`
/// when `1 + continuous_rate` is non-positive.
///
/// # Example
/// ```
/// use casifin_ratios::continuous_to_nominal;
/// use rust_decimal::Decimal;
///
/// let nominal = continuous_to_nominal(Decimal::new(5, 2)).unwrap();
/// assert!(nominal > Decimal::new(4, 2) && nominal < Decimal::new(6, 2));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Quantitative Methods, Reading 6.
///
/// # Panics
/// This function does not panic.
pub fn continuous_to_nominal(continuous_rate: Decimal) -> Result<Decimal, CasifinError> {
    debug_assert!(
        continuous_rate >= Decimal::NEGATIVE_ONE,
        "continuous_rate must be greater than -100%"
    );
    debug_assert!(
        continuous_rate <= Decimal::from(10),
        "continuous_rate must be reasonable"
    );

    if continuous_rate < Decimal::NEGATIVE_ONE {
        return Err(CasifinError::InvalidRate(continuous_rate));
    }

    let one = Decimal::ONE;
    one.checked_add(continuous_rate)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "continuous_to_nominal: base overflow".to_string(),
        })?
        .checked_ln()
        .ok_or(CasifinError::InvalidRate(continuous_rate))
}

/// Nominal Rate to Continuous Rate.
///
/// # Formula
/// ```text
/// continuous_rate = e^nominal_rate - 1
/// ```
///
/// # Arguments
/// * `nominal_rate` - Nominal annual rate as a decimal.
///
/// # Returns
/// `Ok(Decimal)` containing the equivalent continuous rate, or `Err(CasifinError::InvalidRate)`
/// when the nominal rate is negative.
///
/// # Example
/// ```
/// use casifin_ratios::nominal_to_continuous;
/// use rust_decimal::Decimal;
///
/// let continuous = nominal_to_continuous(Decimal::new(5, 2)).unwrap();
/// assert!(continuous > Decimal::new(5, 2) && continuous < Decimal::new(6, 2));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Quantitative Methods, Reading 6.
///
/// # Panics
/// This function does not panic.
pub fn nominal_to_continuous(nominal_rate: Decimal) -> Result<Decimal, CasifinError> {
    debug_assert!(
        nominal_rate >= Decimal::ZERO,
        "nominal_rate must be non-negative"
    );
    debug_assert!(
        nominal_rate <= Decimal::from(10),
        "nominal_rate must be reasonable"
    );

    if nominal_rate < Decimal::ZERO {
        return Err(CasifinError::InvalidRate(nominal_rate));
    }

    let one = Decimal::ONE;
    let power = Decimal::E
        .checked_powd(nominal_rate)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "nominal_to_continuous: power overflow".to_string(),
        })?;
    power
        .checked_sub(one)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "nominal_to_continuous: result overflow".to_string(),
        })
}

/// Equivalent Rate between Compounding Conventions.
///
/// # Formula
/// ```text
/// r_target = (1 + r_source / n_source)^(n_source / n_target) - 1
/// ```
///
/// # Arguments
/// * `rate` - Source rate with its compounding convention.
/// * `target_compounding` - Desired compounding convention.
///
/// # Returns
/// `Ok(Rate)` containing the equivalent rate with the target convention, or
/// `Err(CasifinError)` on invalid input.
///
/// # Example
/// ```
/// use casifin_core::{Rate, Compounding};
/// use casifin_ratios::equivalent_rate;
/// use rust_decimal::Decimal;
///
/// let source = Rate::new(Decimal::new(12, 2), Compounding::Discrete(12)).unwrap();
/// let target = equivalent_rate(source, Compounding::Discrete(2)).unwrap();
/// assert_eq!(target.compounding, Compounding::Discrete(2));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Quantitative Methods, Reading 6.
///
/// # Panics
/// This function does not panic.
#[allow(clippy::needless_pass_by_value)]
pub fn equivalent_rate(rate: Rate, target_compounding: Compounding) -> Result<Rate, CasifinError> {
    debug_assert!(
        rate.annual_rate >= Decimal::ZERO,
        "rate must be non-negative"
    );
    debug_assert!(
        matches!(target_compounding, Compounding::Discrete(n) if n > 0)
            || matches!(target_compounding, Compounding::Continuous),
        "target_compounding must be valid"
    );

    let effective = effective_annual_rate(rate.annual_rate, rate.compounding)?;
    let stated = stated_from_effective(effective, target_compounding)?;
    Rate::new(stated, target_compounding).map(|r| r.with_convention(rate.convention))
}

#[cfg(test)]
mod tests {
    use casifin_core::{Compounding, Rate};
    use rust_decimal::Decimal;

    use super::*;

    #[test]
    fn effective_annual_rate_known_value() {
        let ear = effective_annual_rate(Decimal::new(12, 2), Compounding::Discrete(12)).unwrap();
        assert!(ear > Decimal::new(12, 2));
        assert!(ear < Decimal::new(13, 2));
    }

    #[test]
    fn stated_from_effective_round_trip() {
        let stated = Decimal::new(12, 2);
        let ear = effective_annual_rate(stated, Compounding::Discrete(12)).unwrap();
        let recovered = stated_from_effective(ear, Compounding::Discrete(12)).unwrap();
        assert!((recovered - stated).abs() < Decimal::new(1, 10));
    }

    #[test]
    fn continuous_to_nominal_known_value() {
        let nominal = continuous_to_nominal(Decimal::new(5, 2)).unwrap();
        assert!(nominal > Decimal::new(4, 2));
        assert!(nominal < Decimal::new(6, 2));
    }

    #[test]
    fn nominal_to_continuous_known_value() {
        let continuous = nominal_to_continuous(Decimal::new(5, 2)).unwrap();
        assert!(continuous > Decimal::new(5, 2));
        assert!(continuous < Decimal::new(6, 2));
    }

    #[test]
    fn equivalent_rate_known_value() {
        let source = Rate::new(Decimal::new(12, 2), Compounding::Discrete(12)).unwrap();
        let target = equivalent_rate(source, Compounding::Discrete(2)).unwrap();
        assert_eq!(target.compounding, Compounding::Discrete(2));
        assert!(target.annual_rate > Decimal::new(12, 2));
    }
}
