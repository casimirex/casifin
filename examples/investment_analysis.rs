//! Investment Analysis Example
//!
//! Demonstrates NPV and IRR calculations for a real estate investment.

use casifin_sdk::*;
use rust_decimal::Decimal;

fn main() -> Result<(), CasifinError> {
    println!("=== Investment Analysis ===\n");

    let casifin = Casifin::with_defaults();

    // Real estate investment scenario
    println!("Property Investment Analysis:");
    println!("  Purchase Price: $500,000");
    println!("  Down Payment: $100,000");
    println!("  Annual Cash Flow: $15,000 (years 1-5)");
    println!("  Sale Price (year 5): $550,000 (after costs)\n");

    // Cash flows
    let flows = CashFlowStream::new(vec![
        CashFlow::new(Money::from(-100_000)), // Initial down payment
        CashFlow::new(Money::from(15_000)),   // Year 1 cash flow
        CashFlow::new(Money::from(15_000)),   // Year 2
        CashFlow::new(Money::from(15_000)),   // Year 3
        CashFlow::new(Money::from(15_000)),   // Year 4
        CashFlow::new(Money::from(15_000 + 550_000)), // Year 5: cash flow + sale proceeds
    ]);

    // Calculate NPV at 10% required return
    let required_return = Decimal::new(10, 2);
    let npv = casifin.npv(required_return, &flows)?;
    println!("Results:");
    println!("  NPV at 10%: ${}", npv);

    // Calculate IRR
    let irr = casifin.irr(&flows)?;
    println!("  IRR: {:.2}%", irr * Decimal::from(100));

    // Investment decision
    println!("\nRecommendation:");
    if npv > Money::ZERO {
        println!("  ✓ ACCEPT: Positive NPV (creates value)");
    } else {
        println!("  ✗ REJECT: Negative NPV (destroys value)");
    }

    if irr > required_return {
        println!("  ✓ ACCEPT: IRR exceeds required return");
    } else {
        println!("  ✗ REJECT: IRR below required return");
    }

    // Sensitivity analysis
    println!("\nSensitivity Analysis:");
    for rate in [8, 10, 12, 15].iter() {
        let r = Decimal::new(*rate, 2);
        let npv_at_rate = casifin.npv(r, &flows)?;
        println!("  NPV at {}%: ${}", rate, npv_at_rate);
    }

    Ok(())
}
