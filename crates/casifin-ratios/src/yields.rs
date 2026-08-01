//! Yield metric calculations.

use casifin_core::{CasifinError, Money};
use rust_decimal::Decimal;

/// Bank Discount Yield.
///
/// # Formula
/// ```text
/// BDY = ((Face Value - Purchase Price) / Face Value) * (360 / Days to Maturity)
/// ```
///
/// # Arguments
/// * `face_value` - Face value of the money-market instrument.
/// * `purchase_price` - Purchase price of the instrument.
/// * `days_to_maturity` - Number of days until maturity.
///
/// # Returns
/// `Ok(Decimal)` containing the bank discount yield, or `Err(CasifinError::DivisionByZero)`
/// when `face_value` or `days_to_maturity` is zero.
///
/// # Example
/// ```
/// use casifin_core::Money;
/// use casifin_ratios::bank_discount_yield;
/// use rust_decimal::Decimal;
///
/// let bdy = bank_discount_yield(Money::from(1_000), Money::from(990), 90).unwrap();
/// assert_eq!(bdy, Decimal::new(4, 2)); // 0.04
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Fixed Income, Reading 42.
///
/// # Panics
/// This function does not panic.
pub fn bank_discount_yield(
    face_value: Money,
    purchase_price: Money,
    days_to_maturity: u32,
) -> Result<Decimal, CasifinError> {
    debug_assert!(!face_value.is_zero(), "face_value must not be zero");
    debug_assert!(
        face_value >= Money::ZERO && purchase_price >= Money::ZERO && days_to_maturity > 0,
        "face_value, purchase_price must be non-negative and days must be positive"
    );

    if face_value.is_zero() || days_to_maturity == 0 {
        return Err(CasifinError::DivisionByZero {
            operation: "bank_discount_yield",
        });
    }

    let discount = face_value
        .inner()
        .checked_sub(purchase_price.inner())
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "bank_discount_yield: discount overflow".to_string(),
        })?;

    let days_factor = Decimal::from(360)
        .checked_div(Decimal::from(days_to_maturity))
        .ok_or(CasifinError::DivisionByZero {
            operation: "bank_discount_yield days factor",
        })?;

    discount
        .checked_div(face_value.inner())
        .ok_or(CasifinError::DivisionByZero {
            operation: "bank_discount_yield",
        })?
        .checked_mul(days_factor)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "bank_discount_yield: yield overflow".to_string(),
        })
}

/// Money Market Yield (CD Equivalent Yield).
///
/// # Formula
/// ```text
/// MMY = ((Face Value - Purchase Price) / Purchase Price) * (360 / Days to Maturity)
/// ```
///
/// # Arguments
/// * `face_value` - Face value of the money-market instrument.
/// * `purchase_price` - Purchase price of the instrument.
/// * `days_to_maturity` - Number of days until maturity.
///
/// # Returns
/// `Ok(Decimal)` containing the money market yield, or `Err(CasifinError::DivisionByZero)`
/// when `purchase_price` or `days_to_maturity` is zero.
///
/// # Example
/// ```
/// use casifin_core::Money;
/// use casifin_ratios::money_market_yield;
/// use rust_decimal::Decimal;
///
/// let mmy = money_market_yield(Money::from(1_000), Money::from(990), 90).unwrap();
/// assert!(mmy > Decimal::new(4, 2));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Fixed Income, Reading 42.
///
/// # Panics
/// This function does not panic.
pub fn money_market_yield(
    face_value: Money,
    purchase_price: Money,
    days_to_maturity: u32,
) -> Result<Decimal, CasifinError> {
    debug_assert!(!purchase_price.is_zero(), "purchase_price must not be zero");
    debug_assert!(
        face_value >= Money::ZERO && purchase_price >= Money::ZERO && days_to_maturity > 0,
        "face_value, purchase_price must be non-negative and days must be positive"
    );

    if purchase_price.is_zero() || days_to_maturity == 0 {
        return Err(CasifinError::DivisionByZero {
            operation: "money_market_yield",
        });
    }

    let discount = face_value
        .inner()
        .checked_sub(purchase_price.inner())
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "money_market_yield: discount overflow".to_string(),
        })?;

    let days_factor = Decimal::from(360)
        .checked_div(Decimal::from(days_to_maturity))
        .ok_or(CasifinError::DivisionByZero {
            operation: "money_market_yield days factor",
        })?;

    discount
        .checked_div(purchase_price.inner())
        .ok_or(CasifinError::DivisionByZero {
            operation: "money_market_yield",
        })?
        .checked_mul(days_factor)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "money_market_yield: yield overflow".to_string(),
        })
}

/// Bond Equivalent Yield.
///
/// # Formula
/// ```text
/// BEY = Semi-Annual Yield * 2
/// ```
///
/// # Arguments
/// * `semi_annual_yield` - Semi-annual yield as a decimal.
///
/// # Returns
/// `Ok(Decimal)` containing the bond equivalent yield.
///
/// # Example
/// ```
/// use casifin_ratios::bond_equivalent_yield;
/// use rust_decimal::Decimal;
///
/// let bey = bond_equivalent_yield(Decimal::new(3, 2)).unwrap();
/// assert_eq!(bey, Decimal::new(6, 2));
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Fixed Income, Reading 42.
///
/// # Panics
/// This function does not panic.
pub fn bond_equivalent_yield(semi_annual_yield: Decimal) -> Result<Decimal, CasifinError> {
    debug_assert!(
        semi_annual_yield >= Decimal::NEGATIVE_ONE,
        "semi_annual_yield must be greater than -100%"
    );
    debug_assert!(
        semi_annual_yield <= Decimal::from(10),
        "semi_annual_yield must be reasonable"
    );

    let two = Decimal::from(2);
    semi_annual_yield
        .checked_mul(two)
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "bond_equivalent_yield: overflow".to_string(),
        })
}

/// Holding Period Yield.
///
/// # Formula
/// ```text
/// HPY = (Sale Price - Purchase Price + Coupon) / Purchase Price
/// ```
///
/// # Arguments
/// * `purchase_price` - Bond purchase price.
/// * `sale_price` - Bond sale price.
/// * `coupon` - Coupon income received over the holding period.
///
/// # Returns
/// `Ok(Decimal)` containing the holding period yield, or `Err(CasifinError::DivisionByZero)`
/// when `purchase_price` is zero.
///
/// # Example
/// ```
/// use casifin_core::Money;
/// use casifin_ratios::holding_period_yield;
/// use rust_decimal::Decimal;
///
/// let hpy = holding_period_yield(Money::from(1_000), Money::from(1_050), Money::from(40)).unwrap();
/// assert_eq!(hpy, Decimal::new(9, 2)); // 0.09
/// ```
///
/// # Curriculum Reference
/// CFA Level I, Fixed Income, Reading 42.
///
/// # Panics
/// This function does not panic.
pub fn holding_period_yield(
    purchase_price: Money,
    sale_price: Money,
    coupon: Money,
) -> Result<Decimal, CasifinError> {
    debug_assert!(!purchase_price.is_zero(), "purchase_price must not be zero");
    debug_assert!(
        purchase_price >= Money::ZERO && sale_price >= Money::ZERO && coupon >= Money::ZERO,
        "prices and coupon must be non-negative"
    );

    if purchase_price.is_zero() {
        return Err(CasifinError::DivisionByZero {
            operation: "holding_period_yield",
        });
    }

    let gain = sale_price
        .inner()
        .checked_sub(purchase_price.inner())
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "holding_period_yield: gain overflow".to_string(),
        })?
        .checked_add(coupon.inner())
        .ok_or(CasifinError::ScheduleOverflow {
            detail: "holding_period_yield: gain overflow".to_string(),
        })?;

    gain.checked_div(purchase_price.inner())
        .ok_or(CasifinError::DivisionByZero {
            operation: "holding_period_yield",
        })
}

#[cfg(test)]
mod tests {
    use casifin_core::Money;
    use rust_decimal::Decimal;

    use super::*;

    #[test]
    fn bank_discount_yield_known_value() {
        let bdy = bank_discount_yield(Money::from(1_000), Money::from(990), 90).unwrap();
        assert_eq!(bdy, Decimal::new(4, 2));
    }

    #[test]
    fn money_market_yield_known_value() {
        let mmy = money_market_yield(Money::from(1_000), Money::from(990), 90).unwrap();
        let expected = Decimal::from(10)
            .checked_div(Decimal::from(990))
            .unwrap()
            .checked_mul(Decimal::from(4))
            .unwrap();
        assert_eq!(mmy, expected);
    }

    #[test]
    fn bond_equivalent_yield_known_value() {
        let bey = bond_equivalent_yield(Decimal::new(3, 2)).unwrap();
        assert_eq!(bey, Decimal::new(6, 2));
    }

    #[test]
    fn holding_period_yield_known_value() {
        let hpy =
            holding_period_yield(Money::from(1_000), Money::from(1_050), Money::from(40)).unwrap();
        assert_eq!(hpy, Decimal::new(9, 2));
    }
}
