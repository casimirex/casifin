//! Statistical utility calculations.

use casifin_core::CasifinError;
use rust_decimal::{Decimal, MathematicalOps};

/// Coefficient of Variation.
///
/// # Formula
/// ```text
/// CV = Standard Deviation / Mean
/// ```
///
/// # Arguments
/// * `mean` - Mean of the distribution.
/// * `std_dev` - Standard deviation of the distribution.
///
/// # Returns
/// `Ok(Decimal)` containing the coefficient of variation, or `Err(CasifinError::DivisionByZero)`
/// when `mean` is zero.
///
/// # Example
/// ```
/// use casifin_ratios::coefficient_of_variation;
/// use rust_decimal::Decimal;
///
/// let cv = coefficient_of_variation(Decimal::new(10, 0), Decimal::new(2, 0)).unwrap();
/// assert_eq!(cv, Decimal::new(2, 1));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Quantitative Methods, Reading 6.
///
/// # Panics
/// This function does not panic.
pub fn coefficient_of_variation(mean: Decimal, std_dev: Decimal) -> Result<Decimal, CasifinError> {
    debug_assert!(!mean.is_zero(), "mean must not be zero");
    debug_assert!(std_dev >= Decimal::ZERO, "std_dev must be non-negative");

    if mean.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "coefficient_of_variation",
        });
    }

    std_dev
        .checked_div(mean)
        .ok_or(CasifinError::DivisionByZero {
            operation: "coefficient_of_variation",
        })
}

/// Weighted Mean.
///
/// # Formula
/// ```text
/// WM = Σ(wi * xi) / Σwi
/// ```
///
/// # Arguments
/// * `values` - Observations.
/// * `weights` - Corresponding weights.
///
/// # Returns
/// `Ok(Decimal)` containing the weighted mean, or `Err(CasifinError::InvalidInput)` when the
/// input lengths differ, or `Err(CasifinError::DivisionByZero)` when the sum of weights is zero.
///
/// # Example
/// ```
/// use casifin_ratios::weighted_mean;
/// use rust_decimal::Decimal;
///
/// let values = vec![Decimal::new(10, 0), Decimal::new(20, 0), Decimal::new(30, 0)];
/// let weights = vec![Decimal::new(1, 0), Decimal::new(2, 0), Decimal::new(3, 0)];
/// let wm = weighted_mean(&values, &weights).unwrap();
/// let expected = Decimal::new(140, 0).checked_div(Decimal::new(6, 0)).unwrap();
/// assert_eq!(wm, expected);
///
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Quantitative Methods, Reading 6.
///
/// # Panics
/// This function does not panic.
pub fn weighted_mean(values: &[Decimal], weights: &[Decimal]) -> Result<Decimal, CasifinError> {
    debug_assert!(!values.is_empty(), "values must not be empty");
    debug_assert_eq!(
        values.len(),
        weights.len(),
        "values and weights must have the same length"
    );

    if values.len() != weights.len() {
        return Err(CasifinError::InvalidInput {
            reason: "values and weights must have the same length".to_string(),
        });
    }

    let mut weighted_sum = Decimal::ZERO;
    let mut weight_sum = Decimal::ZERO;

    for (v, w) in values.iter().zip(weights.iter()) {
        let term = v.checked_mul(*w).ok_or(CasifinError::ScheduleOverflow {
            detail: "weighted_mean: weighted term overflow".to_string(),
        })?;
        weighted_sum = weighted_sum
            .checked_add(term)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "weighted_mean: weighted sum overflow".to_string(),
            })?;
        weight_sum = weight_sum
            .checked_add(*w)
            .ok_or(CasifinError::ScheduleOverflow {
                detail: "weighted_mean: weight sum overflow".to_string(),
            })?;
    }

    if weight_sum.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "weighted_mean",
        });
    }

    weighted_sum
        .checked_div(weight_sum)
        .ok_or(CasifinError::DivisionByZero {
            operation: "weighted_mean",
        })
}

/// Harmonic Mean.
///
/// # Formula
/// ```text
/// HM = n / (Σ(1 / xi))
/// ```
///
/// # Arguments
/// * `values` - Positive observations.
///
/// # Returns
/// `Ok(Decimal)` containing the harmonic mean, or `Err(CasifinError::InsufficientCashFlows)`
/// when the slice is empty, or `Err(CasifinError::DivisionByZero)` when any value is zero.
///
/// # Example
/// ```
/// use casifin_ratios::harmonic_mean;
/// use rust_decimal::Decimal;
///
/// let values = vec![Decimal::from(2), Decimal::from(4), Decimal::from(8)];
/// let hm = harmonic_mean(&values).unwrap();
/// assert!(hm > Decimal::from(3) && hm < Decimal::from(4));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Quantitative Methods, Reading 6.
///
/// # Panics
/// This function does not panic.
pub fn harmonic_mean(values: &[Decimal]) -> Result<Decimal, CasifinError> {
    debug_assert!(!values.is_empty(), "values must not be empty");
    debug_assert!(
        values.iter().all(|v| *v > Decimal::ZERO),
        "values must be positive"
    );

    if values.is_empty() {
        return Err(CasifinError::InsufficientCashFlows);
    }

    let n = Decimal::from(values.len());
    let mut reciprocal_sum = Decimal::ZERO;

    for v in values {
        if v.is_zero() {
            return Err(CasifinError::DivisionByZero {
                operation: "harmonic_mean",
            });
        }
        let reciprocal = Decimal::ONE
            .checked_div(*v)
            .ok_or(CasifinError::DivisionByZero {
                operation: "harmonic_mean reciprocal",
            })?;
        reciprocal_sum =
            reciprocal_sum
                .checked_add(reciprocal)
                .ok_or(CasifinError::ScheduleOverflow {
                    detail: "harmonic_mean: reciprocal sum overflow".to_string(),
                })?;
    }

    n.checked_div(reciprocal_sum)
        .ok_or(CasifinError::DivisionByZero {
            operation: "harmonic_mean",
        })
}

/// Sampling Error of the Mean (Standard Error).
///
/// # Formula
/// ```text
/// SE = σ / sqrt(n)
/// ```
///
/// # Arguments
/// * `population_std_dev` - Population standard deviation.
/// * `sample_size` - Number of observations.
///
/// # Returns
/// `Ok(Decimal)` containing the standard error, or `Err(CasifinError::DivisionByZero)` when
/// `sample_size` is zero.
///
/// # Example
/// ```
/// use casifin_ratios::sampling_error;
/// use rust_decimal::Decimal;
///
/// let se = sampling_error(Decimal::from(10), 100).unwrap();
/// assert_eq!(se, Decimal::from(1));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Quantitative Methods, Reading 6.
///
/// # Panics
/// This function does not panic.
pub fn sampling_error(
    population_std_dev: Decimal,
    sample_size: u32,
) -> Result<Decimal, CasifinError> {
    debug_assert!(sample_size > 0, "sample_size must be positive");
    debug_assert!(
        population_std_dev >= Decimal::ZERO,
        "population_std_dev must be non-negative"
    );

    if sample_size == 0 {
        return Err(CasifinError::DivisionByZero {
            operation: "sampling_error",
        });
    }

    let n = Decimal::from(sample_size);
    let sqrt_n = n.sqrt().ok_or(CasifinError::InvalidInput {
        reason: "sampling_error: sqrt failed".to_string(),
    })?;

    population_std_dev
        .checked_div(sqrt_n)
        .ok_or(CasifinError::DivisionByZero {
            operation: "sampling_error",
        })
}

/// Standard Error of the Mean.
///
/// This is an alias for [`sampling_error`].
///
/// # Formula
/// ```text
/// SE = σ / sqrt(n)
/// ```
///
/// # Arguments
/// * `std_dev` - Standard deviation.
/// * `sample_size` - Number of observations.
///
/// # Returns
/// `Ok(Decimal)` containing the standard error, or `Err(CasifinError::DivisionByZero)` when
/// `sample_size` is zero.
///
/// # Example
/// ```
/// use casifin_ratios::standard_error;
/// use rust_decimal::Decimal;
///
/// let se = standard_error(Decimal::from(10), 100).unwrap();
/// assert_eq!(se, Decimal::from(1));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Quantitative Methods, Reading 6.
///
/// # Panics
/// This function does not panic.
pub fn standard_error(std_dev: Decimal, sample_size: u32) -> Result<Decimal, CasifinError> {
    debug_assert!(sample_size > 0, "sample_size must be positive");
    debug_assert!(std_dev >= Decimal::ZERO, "std_dev must be non-negative");

    sampling_error(std_dev, sample_size)
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::*;

    #[test]
    fn coefficient_of_variation_known_value() {
        let cv = coefficient_of_variation(Decimal::new(10, 0), Decimal::new(2, 0)).unwrap();
        assert_eq!(cv, Decimal::new(2, 1));
    }

    #[test]
    fn weighted_mean_known_value() {
        let values = vec![
            Decimal::new(10, 0),
            Decimal::new(20, 0),
            Decimal::new(30, 0),
        ];
        let weights = vec![Decimal::new(1, 0), Decimal::new(2, 0), Decimal::new(3, 0)];
        let wm = weighted_mean(&values, &weights).unwrap();
        let expected = Decimal::new(140, 0)
            .checked_div(Decimal::new(6, 0))
            .unwrap();
        assert_eq!(wm, expected);
    }

    #[test]
    fn harmonic_mean_known_value() {
        let values = vec![Decimal::from(2), Decimal::from(4), Decimal::from(8)];
        let hm = harmonic_mean(&values).unwrap();
        let expected = Decimal::from(3)
            .checked_div(Decimal::new(875, 3) /* 0.875 */)
            .unwrap();
        assert_eq!(hm, expected);
    }

    #[test]
    fn sampling_error_known_value() {
        let se = sampling_error(Decimal::from(10), 100).unwrap();
        assert_eq!(se, Decimal::from(1));
    }

    #[test]
    fn standard_error_known_value() {
        let se = standard_error(Decimal::from(10), 100).unwrap();
        assert_eq!(se, Decimal::from(1));
    }
}
