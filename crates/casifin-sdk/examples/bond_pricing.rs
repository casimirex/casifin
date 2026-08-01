//! Bond Pricing Example
//!
//! Demonstrates bond valuation and yield calculations.

use casifin_sdk::*;
use rust_decimal::Decimal;

fn main() -> Result<(), CasifinError> {
    println!("=== Bond Pricing ===\n");

    let casifin = Casifin::with_default_config();

    // Bond parameters
    let face_value = Money::from(1000);
    let coupon_rate = Decimal::new(5, 2); // 5% annual coupon
    let years_to_maturity = 10u32;
    let market_yield = Rate::new(Decimal::new(6, 2), Compounding::Discrete(1))?; // 6% market yield

    println!("Bond Details:");
    println!("  Face Value: ${}", face_value);
    println!("  Coupon Rate: {:.2}%", coupon_rate * Decimal::from(100));
    println!("  Years to Maturity: {}", years_to_maturity);
    println!(
        "  Market Yield: {:.2}%\n",
        market_yield.annual_rate * Decimal::from(100)
    );

    // Calculate annual coupon payment
    let annual_coupon = face_value * coupon_rate;
    println!("  Annual Coupon: ${}\n", annual_coupon);

    // Price the bond: PV of coupons + PV of face value
    // Coupons are an annuity
    let pv_coupons = casifin.pv(
        market_yield,
        years_to_maturity,
        annual_coupon,
        Money::ZERO,
        PaymentDue::End,
    )?;

    // Face value is a single future payment
    let pv_face = casifin.pv(
        market_yield,
        years_to_maturity,
        Money::ZERO,
        face_value,
        PaymentDue::End,
    )?;

    let bond_price = pv_coupons + pv_face;

    println!("Bond Valuation:");
    println!("  PV of Coupons: ${}", pv_coupons);
    println!("  PV of Face Value: ${}", pv_face);
    println!("  Bond Price: ${}\n", bond_price);

    // Premium/Discount analysis
    let premium_discount = bond_price - face_value;
    if premium_discount > Money::ZERO {
        println!("  Bond trades at PREMIUM of ${}", premium_discount);
    } else {
        println!("  Bond trades at DISCOUNT of ${}", premium_discount.abs());
    }

    // Current yield
    let current_yield = annual_coupon.inner() / bond_price.inner();
    println!(
        "\n  Current Yield: {:.2}%",
        current_yield * Decimal::from(100)
    );

    // Price sensitivity (duration approximation)
    println!("\nPrice Sensitivity:");
    for yield_change in [-1, 0, 1, 2].iter() {
        let new_annual_rate = market_yield.annual_rate + Decimal::new(*yield_change, 2);
        if new_annual_rate > Decimal::ZERO {
            let new_yield = Rate::new(new_annual_rate, Compounding::Discrete(1))?;
            let pv_c = casifin.pv(
                new_yield,
                years_to_maturity,
                annual_coupon,
                Money::ZERO,
                PaymentDue::End,
            )?;
            let pv_f = casifin.pv(
                new_yield,
                years_to_maturity,
                Money::ZERO,
                face_value,
                PaymentDue::End,
            )?;
            let price = pv_c + pv_f;
            println!(
                "  Yield {:.2}%: Price ${}",
                new_annual_rate * Decimal::from(100),
                price
            );
        }
    }

    Ok(())
}
