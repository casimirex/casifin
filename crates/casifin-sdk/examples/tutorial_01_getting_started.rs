//! Getting Started Tutorial
//!
//! Your first steps with casifin - basic TVM calculations.

use casifin_sdk::*;
use rust_decimal::Decimal;

fn main() -> Result<(), CasifinError> {
    println!("=== Getting Started with casifin ===\n");

    // Create the main casifin instance
    let casifin = Casifin::with_default_config();

    // Example 1: Future Value of a lump sum
    println!("1. Future Value of $10,000 at 5% for 10 years:");
    let rate = Rate::new(Decimal::new(5, 2), Compounding::Discrete(1))?;
    let fv = casifin.fv(
        rate,
        10,                  // 10 years
        Money::ZERO,         // No periodic payment
        Money::from(10_000), // Present value
        PaymentDue::End,
    )?;
    println!("   FV = ${}\n", fv);

    // Example 2: Present Value needed for retirement
    println!("2. Present Value needed to have $1,000,000 in 30 years at 7%:");
    let rate = Rate::new(Decimal::new(7, 2), Compounding::Discrete(1))?;
    let pv = casifin.pv(
        rate,
        30,                     // 30 years
        Money::ZERO,            // No periodic contribution
        Money::from(1_000_000), // Future value goal
        PaymentDue::End,
    )?;
    println!("   PV = ${}\n", pv);

    // Example 3: Monthly savings needed for retirement
    println!("3. Monthly payment to reach $1,000,000 in 30 years at 7%:");
    let monthly_rate = Rate::new(Decimal::new(7, 2), Compounding::Discrete(12))?;
    let months = 30 * 12;
    let pmt = casifin.pmt(
        monthly_rate,
        months,
        Money::ZERO, // Starting from zero
        Money::from(1_000_000),
        PaymentDue::End,
    )?;
    println!("   Monthly Payment = ${}\n", pmt.abs());

    // Example 4: How long to double your money with monthly contributions?
    println!("4. Years to reach $20,000 starting with $10,000, adding $100/month at 8%:");
    let rate = Rate::new(Decimal::new(8, 2), Compounding::Discrete(12))?;
    let nper = casifin.nper(
        rate,
        Money::from(-100),    // Monthly contribution (negative = outflow)
        Money::from(-10_000), // Initial investment
        Money::from(20_000),  // Target
        PaymentDue::End,
    )?;
    println!(
        "   NPER = {} months ({} years)\n",
        nper,
        nper / Decimal::from(12)
    );

    // Example 5: What rate of return?
    println!("5. Rate needed to grow $10,000 to $50,000 in 15 years:");
    let rate = casifin.rate(
        15,
        Money::ZERO,
        Money::from(-10_000),
        Money::from(50_000),
        PaymentDue::End,
    )?;
    println!("   Rate = {:.2}% per year\n", rate * Decimal::from(100));

    println!("=== Tutorial Complete ===");
    println!("\nNext steps:");
    println!("  - Try the mortgage_calculator example");
    println!("  - Explore investment_analysis for NPV/IRR");
    println!("  - See bond_pricing for fixed income calculations");

    Ok(())
}
