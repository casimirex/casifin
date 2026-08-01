//! Core types and primitives for the casifin financial computation engine.
//!
//! This crate provides the foundational types used throughout casifin:
//! - [`Money`] - A monetary value wrapper around `rust_decimal::Decimal`
//! - [`Rate`] - An interest rate with compounding metadata
//! - [`CasifinError`] - The error type for all casifin operations
//! - [`Config`] - Global configuration for calculations

#![deny(warnings)]

use std::{
    fmt,
    ops::{Add, Div, Mul, Neg, Sub},
    str::FromStr,
};

use rust_decimal::Decimal;
use thiserror::Error;

// ============================================================================
// Money
// ============================================================================

/// A monetary value with guaranteed precision.
///
/// Invariant: stores exact decimal representation.
///
/// # Panics
/// This type does not panic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Money(Decimal);

impl Money {
    pub const ZERO: Money = Money(Decimal::ZERO);

    /// Creates a new `Money` value from dollars and cents.
    ///
    /// # Formula
    /// ```text
    /// value = (dollars * 100 + cents) / 100
    /// ```
    ///
    /// # Arguments
    /// * `dollars` - The dollar amount.
    /// * `cents` - The cents amount (0-99).
    ///
    /// # Returns
    /// `Ok(Money)` on success, or `Err(CasifinError::InvalidInput)` if cents >= 100.
    ///
    /// # Example
    /// ```
    /// use casifin_core::Money;
    /// let m = Money::new(100, 50).unwrap();
    /// assert_eq!(m.to_string(), "100.50");
    /// ```
    ///
    /// # Panics
    /// This function does not panic.
    pub fn new(dollars: i64, cents: u32) -> Result<Self, CasifinError> {
        debug_assert!(cents < 100, "cents must be less than 100");
        debug_assert!(
            dollars.checked_mul(100).is_some(),
            "dollars multiplication must not overflow"
        );
        if cents >= 100 {
            return Err(CasifinError::InvalidInput {
                reason: format!("cents must be 0-99, got {cents}"),
            });
        }
        let dec = Decimal::new(dollars * 100 + i64::from(cents), 2);
        Ok(Money(dec))
    }

    /// Creates a `Money` value from a `Decimal`.
    ///
    /// # Arguments
    /// * `dec` - The exact decimal value to store.
    ///
    /// # Returns
    /// A new `Money` instance.
    ///
    /// # Example
    /// ```
    /// use casifin_core::Money;
    /// use rust_decimal::Decimal;
    /// let m = Money::from_decimal(Decimal::new(123450, 3));
    /// assert_eq!(m.to_string(), "123.45");
    /// ```
    ///
    /// # Panics
    /// This function does not panic.
    pub fn from_decimal(dec: Decimal) -> Self {
        Money(dec)
    }

    /// Parses a `Money` value from a string.
    ///
    /// # Arguments
    /// * `s` - A string representation of a decimal number (e.g., "1234.56").
    ///
    /// # Returns
    /// `Ok(Money)` on success, or `Err(CasifinError::ParseError)` on failure.
    ///
    /// # Example
    /// ```
    /// use casifin_core::Money;
    /// let m = Money::from_str("1234.56").unwrap();
    /// assert_eq!(m.to_string(), "1234.56");
    /// ```
    ///
    /// # Panics
    /// This function does not panic.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, CasifinError> {
        debug_assert!(!s.is_empty(), "input string must not be empty");
        let dec = Decimal::from_str(s).map_err(|e| CasifinError::ParseError(format!("{e}")))?;
        Ok(Money(dec))
    }

    /// Returns the inner `Decimal` value.
    ///
    /// # Returns
    /// The wrapped `Decimal` amount.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn inner(&self) -> Decimal {
        self.0
    }

    /// Returns the absolute value.
    ///
    /// # Returns
    /// A new `Money` containing `|self|`.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn abs(&self) -> Self {
        Money(self.0.abs())
    }

    /// Returns `true` if this value is zero.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn is_zero(&self) -> bool {
        self.0 == Decimal::ZERO
    }

    /// Returns `true` if this value is positive.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn is_positive(&self) -> bool {
        self.0 > Decimal::ZERO
    }

    /// Returns `true` if this value is negative.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn is_negative(&self) -> bool {
        self.0 < Decimal::ZERO
    }

    /// Rounds to 2 decimal places using standard rounding.
    ///
    /// # Returns
    /// A new `Money` rounded to cents.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn round_to_cents(&self) -> Self {
        Money(self.0.round_dp(2))
    }

    /// Checked division by a `Decimal`.
    ///
    /// # Arguments
    /// * `other` - The divisor.
    ///
    /// # Returns
    /// `Some(Money)` on success, or `None` if `other` is zero.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn checked_div_decimal(&self, other: Decimal) -> Option<Self> {
        debug_assert!(!other.is_zero(), "divisor must not be zero");
        self.0.checked_div(other).map(Money)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl Add for Money {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Money(self.0 + rhs.0)
    }
}

impl Sub for Money {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Money(self.0 - rhs.0)
    }
}

impl Mul<Decimal> for Money {
    type Output = Self;
    fn mul(self, rhs: Decimal) -> Self::Output {
        Money(self.0 * rhs)
    }
}

impl Div<Decimal> for Money {
    type Output = Self;
    fn div(self, rhs: Decimal) -> Self::Output {
        Money(self.0 / rhs)
    }
}

impl Neg for Money {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Money(-self.0)
    }
}

impl std::iter::Sum for Money {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Money(iter.map(|m| m.0).sum())
    }
}

impl From<Decimal> for Money {
    fn from(value: Decimal) -> Self {
        Money(value)
    }
}

impl From<i64> for Money {
    fn from(value: i64) -> Self {
        Money(Decimal::from(value))
    }
}

// ============================================================================
// Compounding
// ============================================================================

/// The compounding frequency for an interest rate.
///
/// # Panics
/// This type does not panic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Compounding {
    /// Discrete compounding n times per year.
    /// Invariant: n >= 1.
    Discrete(u32),
    /// Continuous compounding (e^x).
    Continuous,
}

impl Compounding {
    /// Returns the number of compounding periods per year.
    ///
    /// Returns `None` for continuous compounding.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn periods_per_year(&self) -> Option<u32> {
        match self {
            Compounding::Discrete(n) => Some(*n),
            Compounding::Continuous => None,
        }
    }
}

// ============================================================================
// DayCount
// ============================================================================

/// Day count convention for interest accrual.
///
/// # Panics
/// This type does not panic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DayCount {
    Actual365,
    Actual360,
    Thirty360,
    ThirtyE360,
    ActualActualIsda,
}

// ============================================================================
// PaymentDue
// ============================================================================

/// When payments are due within a period.
///
/// # Panics
/// This type does not panic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaymentDue {
    Beginning,
    End,
}

// ============================================================================
// Rate
// ============================================================================

/// An interest rate with compounding metadata.
///
/// Invariant: annual_rate >= 0.
///
/// # Panics
/// This type does not panic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rate {
    pub annual_rate: Decimal,
    pub compounding: Compounding,
    pub convention: DayCount,
}

impl Rate {
    /// Creates a new `Rate` with the given annual rate and compounding frequency.
    ///
    /// # Arguments
    /// * `annual_rate` - The annual interest rate (must be non-negative).
    /// * `compounding` - The compounding frequency.
    ///
    /// # Returns
    /// `Ok(Rate)` on success, or `Err(CasifinError)` if the rate is negative
    /// or the discrete compounding frequency is zero.
    ///
    /// # Example
    /// ```
    /// use casifin_core::{Rate, Compounding};
    /// use rust_decimal::Decimal;
    ///
    /// let rate = Rate::new(Decimal::new(5, 2), Compounding::Discrete(12)).unwrap();
    /// ```
    ///
    /// # Panics
    /// This function does not panic.
    pub fn new(annual_rate: Decimal, compounding: Compounding) -> Result<Self, CasifinError> {
        if annual_rate < Decimal::ZERO {
            return Err(CasifinError::InvalidRate(annual_rate));
        }
        if let Compounding::Discrete(n) = compounding {
            if n == 0 {
                return Err(CasifinError::InvalidCompounding);
            }
        }
        debug_assert!(
            annual_rate >= Decimal::ZERO,
            "annual_rate must be non-negative"
        );
        if let Compounding::Discrete(n) = compounding {
            debug_assert!(n > 0, "discrete compounding must have n > 0");
        }
        Ok(Rate {
            annual_rate,
            compounding,
            convention: DayCount::Actual365,
        })
    }

    /// Sets the day count convention.
    ///
    /// # Arguments
    /// * `convention` - The day count convention to use.
    ///
    /// # Returns
    /// The same `Rate` with the updated convention.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn with_convention(mut self, convention: DayCount) -> Self {
        self.convention = convention;
        self
    }

    /// Returns the periodic rate for one compounding period.
    ///
    /// # Returns
    /// `Ok(Decimal)` containing the period rate, or `Err(CasifinError::DivisionByZero)`
    /// if the discrete compounding frequency is zero.
    ///
    /// # Example
    /// ```
    /// use casifin_core::{Rate, Compounding};
    /// use rust_decimal::Decimal;
    ///
    /// let rate = Rate::new(Decimal::new(5, 2), Compounding::Discrete(12)).unwrap();
    /// let periodic = rate.periodic_rate().unwrap();
    /// // 0.05 / 12 ≈ 0.004166...
    /// assert!(periodic > Decimal::ZERO);
    /// ```
    ///
    /// # Panics
    /// This function does not panic.
    pub fn periodic_rate(&self) -> Result<Decimal, CasifinError> {
        match self.compounding {
            Compounding::Discrete(n) => {
                debug_assert!(n > 0, "discrete compounding must have n > 0");
                self.annual_rate
                    .checked_div(Decimal::from(n))
                    .ok_or(CasifinError::DivisionByZero {
                        operation: "periodic_rate",
                    })
            }
            Compounding::Continuous => Ok(self.annual_rate),
        }
    }
}

// ============================================================================
// Config
// ============================================================================

/// Global configuration for numerical methods.
///
/// # Panics
/// This type does not panic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Config {
    pub eps: Decimal,
    pub max_iterations: u32,
    pub guess: Decimal,
    pub business_days_only: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            eps: Decimal::new(1, 12), // 1e-12
            max_iterations: 1000,
            guess: Decimal::new(1, 1), // 0.1
            business_days_only: false,
        }
    }
}

impl Config {
    /// Creates a new `ConfigBuilder`.
    ///
    /// # Returns
    /// A default [`ConfigBuilder`].
    ///
    /// # Panics
    /// This function does not panic.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

/// Builder for [`Config`].
///
/// # Panics
/// This type does not panic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfigBuilder {
    eps: Decimal,
    max_iterations: u32,
    guess: Decimal,
    business_days_only: bool,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        ConfigBuilder {
            eps: Decimal::new(1, 12),
            max_iterations: 1000,
            guess: Decimal::new(1, 1),
            business_days_only: false,
        }
    }
}

impl ConfigBuilder {
    /// Sets the convergence epsilon.
    ///
    /// # Arguments
    /// * `eps` - The desired precision.
    ///
    /// # Returns
    /// The builder.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn eps(mut self, eps: Decimal) -> Self {
        debug_assert!(eps > Decimal::ZERO, "eps must be positive");
        self.eps = eps;
        self
    }

    /// Sets the maximum number of iterations for iterative solvers.
    ///
    /// # Arguments
    /// * `n` - The iteration cap.
    ///
    /// # Returns
    /// The builder.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn max_iterations(mut self, n: u32) -> Self {
        debug_assert!(n > 0, "max_iterations must be positive");
        self.max_iterations = n;
        self
    }

    /// Sets the initial guess for root-finding solvers.
    ///
    /// # Arguments
    /// * `guess` - The initial guess.
    ///
    /// # Returns
    /// The builder.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn guess(mut self, guess: Decimal) -> Self {
        self.guess = guess;
        self
    }

    /// Sets whether to use business days only.
    ///
    /// # Arguments
    /// * `v` - Whether to restrict to business days.
    ///
    /// # Returns
    /// The builder.
    ///
    /// # Panics
    /// This function does not panic.
    pub fn business_days_only(mut self, v: bool) -> Self {
        self.business_days_only = v;
        self
    }

    /// Builds the [`Config`].
    ///
    /// # Returns
    /// A fully populated [`Config`].
    ///
    /// # Panics
    /// This function does not panic.
    pub fn build(self) -> Config {
        debug_assert!(self.eps > Decimal::ZERO, "eps must be positive");
        debug_assert!(self.max_iterations > 0, "max_iterations must be positive");
        Config {
            eps: self.eps,
            max_iterations: self.max_iterations,
            guess: self.guess,
            business_days_only: self.business_days_only,
        }
    }
}

// ============================================================================
// CasifinError
// ============================================================================

/// Error type for all casifin operations.
///
/// # Panics
/// This type does not panic.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CasifinError {
    #[error("invalid rate: {0}. Rate must be non-negative.")]
    InvalidRate(Decimal),

    #[error("invalid compounding frequency: must be >= 1 for discrete compounding")]
    InvalidCompounding,

    #[error("invalid period: {0}. Period must be positive.")]
    InvalidPeriod(u32),

    #[error("invalid amount: {0}. Amount must be non-zero for this operation.")]
    InvalidAmount(Money),

    #[error("IRR did not converge within {max_iter} iterations (eps={eps})")]
    IrrConvergenceFailure { max_iter: u32, eps: Decimal },

    #[error("division by zero in operation: {operation}")]
    DivisionByZero { operation: &'static str },

    #[error("amortization schedule overflow: {detail}")]
    ScheduleOverflow { detail: String },

    #[error("date out of range: {0}")]
    DateOutOfRange(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("insufficient cash flows: at least one positive and one negative required")]
    InsufficientCashFlows,

    #[error("invalid input: {reason}")]
    InvalidInput { reason: String },
}

// ============================================================================
// Shared Traits
// ============================================================================

/// Trait for types that can perform a financial calculation.
pub trait FinancialCalculation {
    type Output;
    fn calculate(&self) -> Result<Self::Output, CasifinError>;
}

/// Trait for types that generate a schedule of entries.
pub trait Schedulable {
    type Entry;
    fn schedule(&self) -> Result<Vec<Self::Entry>, CasifinError>;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn money_new_valid() {
        let m = Money::new(100, 50).unwrap();
        assert_eq!(m.to_string(), "100.50");
    }

    #[test]
    fn money_from_str_valid() {
        let m = Money::from_str("1234.56").unwrap();
        assert_eq!(m.to_string(), "1234.56");
    }

    #[test]
    fn money_from_str_invalid() {
        let result = Money::from_str("abc");
        assert!(matches!(result, Err(CasifinError::ParseError(_))));
    }

    #[test]
    fn money_arithmetic() {
        let a = Money::from_decimal(Decimal::new(100, 0));
        let b = Money::from_decimal(Decimal::new(50, 0));
        assert_eq!(a + b, Money::from_decimal(Decimal::new(150, 0)));
        assert_eq!(a - b, Money::from_decimal(Decimal::new(50, 0)));
        assert_eq!(
            a * Decimal::new(2, 0),
            Money::from_decimal(Decimal::new(200, 0))
        );
        assert_eq!(
            a / Decimal::new(2, 0),
            Money::from_decimal(Decimal::new(50, 0))
        );
    }

    #[test]
    fn rate_new_negative() {
        let result = Rate::new(Decimal::new(-1, 0), Compounding::Discrete(12));
        assert!(matches!(result, Err(CasifinError::InvalidRate(_))));
    }

    #[test]
    fn rate_new_zero_compounding() {
        let result = Rate::new(Decimal::new(5, 2), Compounding::Discrete(0));
        assert!(matches!(result, Err(CasifinError::InvalidCompounding)));
    }

    #[test]
    fn rate_periodic() {
        let rate = Rate::new(Decimal::new(5, 2), Compounding::Discrete(12)).unwrap();
        let periodic = rate.periodic_rate().unwrap();
        // 0.05 / 12 ≈ 0.004166...
        assert!(periodic > Decimal::ZERO);
        assert!(periodic < Decimal::new(5, 3)); // < 0.005
    }

    #[test]
    fn config_builder() {
        let config = Config::builder()
            .eps(Decimal::new(1, 10))
            .max_iterations(500)
            .guess(Decimal::new(5, 2))
            .build();
        assert_eq!(config.eps, Decimal::new(1, 10));
        assert_eq!(config.max_iterations, 500);
        assert_eq!(config.guess, Decimal::new(5, 2));
    }

    #[test]
    fn config_default() {
        let config = Config::default();
        assert_eq!(config.eps, Decimal::new(1, 12));
        assert_eq!(config.max_iterations, 1000);
        assert_eq!(config.guess, Decimal::new(1, 1));
        assert!(!config.business_days_only);
    }
}
