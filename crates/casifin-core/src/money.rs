//! Monetary value type with guaranteed precision.

use std::{
    fmt,
    iter::Sum,
    ops::{Add, Div, Mul, Neg, Sub},
};

use rust_decimal::{prelude::FromPrimitive, Decimal};
use serde::{Deserialize, Serialize};

/// A monetary value with guaranteed precision.
///
/// This newtype wraps `rust_decimal::Decimal` to provide type-safe
/// monetary arithmetic. All arithmetic operations are checked to
/// prevent overflow and underflow.
///
/// # Invariants
/// - Always stores values with full precision (up to 28 decimal places)
/// - Rounding is only applied at display/conversion boundaries
/// - Negative values are allowed (representing debits, losses, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Money(Decimal);

impl Money {
    /// Zero monetary value.
    pub const ZERO: Self = Money(Decimal::ZERO);

    /// One monetary unit.
    pub const ONE: Self = Money(Decimal::ONE);

    /// Creates a new `Money` value from a `Decimal`.
    ///
    /// # Arguments
    /// * `value` - The decimal value to wrap
    ///
    /// # Example
    /// ```
    /// use casifin_core::Money;
    /// use rust_decimal::Decimal;
    ///
    /// let money = Money::new(Decimal::new(10000, 2)); // $100.00
    /// ```
    pub const fn new(value: Decimal) -> Self {
        Money(value)
    }

    /// Creates a `Money` value from cents.
    ///
    /// # Arguments
    /// * `cents` - The value in cents (or smallest currency unit)
    ///
    /// # Example
    /// ```
    /// use casifin_core::Money;
    ///
    /// let money = Money::from_cents(10000); // $100.00
    /// ```
    pub fn from_cents(cents: i64) -> Self {
        Money(Decimal::new(cents, 2))
    }

    /// Creates a `Money` value from a whole number.
    ///
    /// # Arguments
    /// * `whole` - The whole number value
    ///
    /// # Example
    /// ```
    /// use casifin_core::Money;
    ///
    /// let money = Money::from(100); // $100.00
    /// ```
    pub fn from<T: Into<Decimal>>(value: T) -> Self {
        Money(value.into())
    }

    /// Returns the inner `Decimal` value.
    pub const fn inner(&self) -> Decimal {
        self.0
    }

    /// Returns the absolute value.
    pub fn abs(&self) -> Self {
        Money(self.0.abs())
    }

    /// Checked addition. Returns `None` on overflow.
    pub fn checked_add(&self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Money)
    }

    /// Checked subtraction. Returns `None` on overflow.
    pub fn checked_sub(&self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Money)
    }

    /// Checked multiplication. Returns `None` on overflow.
    pub fn checked_mul(&self, other: Self) -> Option<Self> {
        self.0.checked_mul(other.0).map(Money)
    }

    /// Checked division. Returns `None` on division by zero.
    pub fn checked_div(&self, other: Self) -> Option<Self> {
        self.0.checked_div(other.0).map(Money)
    }

    /// Checked division by a scalar `Decimal`. Returns `None` on division by zero.
    pub fn checked_div_decimal(&self, other: Decimal) -> Option<Self> {
        self.0.checked_div(other).map(Money)
    }

    /// Returns `true` if this value is zero.
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Returns `true` if this value is positive.
    pub fn is_positive(&self) -> bool {
        self.0.is_sign_positive()
    }

    /// Returns `true` if this value is negative.
    pub fn is_negative(&self) -> bool {
        self.0.is_sign_negative()
    }

    /// Rounds to 2 decimal places using standard rounding.
    pub fn round_to_cents(&self) -> Self {
        Money(self.0.round_dp(2))
    }

    /// Creates a `Money` value from an `f64`, returning `None` if the value
    /// is NaN, infinite, or cannot be represented exactly.
    ///
    /// Prefer this over `Money::from(f64)` for robust error handling.
    pub fn from_f64_fallible(value: f64) -> Option<Self> {
        Decimal::from_f64(value).map(Money)
    }
}

impl From<Decimal> for Money {
    fn from(value: Decimal) -> Self {
        Money(value)
    }
}

impl From<Money> for Decimal {
    fn from(value: Money) -> Self {
        value.0
    }
}

impl From<i64> for Money {
    fn from(value: i64) -> Self {
        Money(Decimal::from(value))
    }
}

impl From<f64> for Money {
    /// Converts an `f64` to `Money`.
    ///
    /// # Warning
    /// NaN and infinite values are silently converted to zero.
    /// Prefer [`Money::from_f64_fallible`] for explicit error handling.
    fn from(value: f64) -> Self {
        Money(Decimal::from_f64(value).unwrap_or(Decimal::ZERO))
    }
}

impl Add for Money {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Money(self.0 + other.0)
    }
}

impl Sub for Money {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Money(self.0 - other.0)
    }
}

impl Mul for Money {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Money(self.0 * other.0)
    }
}

impl Mul<Decimal> for Money {
    type Output = Self;

    fn mul(self, other: Decimal) -> Self {
        Money(self.0 * other)
    }
}

impl Div for Money {
    type Output = Decimal;

    fn div(self, other: Self) -> Decimal {
        self.0 / other.0
    }
}

impl Div<Decimal> for Money {
    type Output = Self;

    fn div(self, other: Decimal) -> Self {
        Money(self.0 / other)
    }
}

impl Neg for Money {
    type Output = Self;

    fn neg(self) -> Self {
        Money(-self.0)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:.2}", self.0)
    }
}

impl Default for Money {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<'a> Sum<&'a Money> for Money {
    fn sum<I: Iterator<Item = &'a Money>>(iter: I) -> Self {
        iter.fold(Money::ZERO, |acc, &m| acc + m)
    }
}

impl Sum<Money> for Money {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Self {
        iter.fold(Money::ZERO, |acc, m| acc + m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_creation() {
        let m = Money::from_cents(10000);
        assert_eq!(m.inner(), Decimal::new(10000, 2));
    }

    #[test]
    fn test_money_arithmetic() {
        let a = Money::from(100);
        let b = Money::from(50);
        assert_eq!(a + b, Money::from(150));
        assert_eq!(a - b, Money::from(50));
        assert_eq!(a * b, Money::from(5000));
    }

    #[test]
    fn test_money_display() {
        let m = Money::from(100);
        assert_eq!(format!("{}", m), "$100.00");
    }

    #[test]
    fn test_checked_operations() {
        let a = Money::from(100);
        let b = Money::from(50);
        assert_eq!(a.checked_add(b), Some(Money::from(150)));
        assert_eq!(a.checked_sub(b), Some(Money::from(50)));
        assert_eq!(a.checked_div(b), Some(Money::from(2)));
    }
}
