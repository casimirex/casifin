//! Mortgage Calculator Example
//!
//! Demonstrates fixed-rate vs ARM comparison.

use casifin_sdk::*;
use rust_decimal::Decimal;

fn main() -> Result<(), CasifinError> {
    println!("=== Mortgage Calculator ===\n");

    let casifin = Casifin::with_default_config();

    // Loan parameters
    let principal = Money::from(300_000);
    let fixed_rate = Rate::new(
        Decimal::new(6, 2),
        Compounding::MONTHLY,
        DayCount::Actual365,
    )?;
    let term_months = 360;

    // Fixed-rate mortgage
    println!("Fixed-Rate Mortgage (6% for 30 years):");
    let fixed_schedule = casifin
        .mortgage(principal, fixed_rate, term_months)
        .build()?;

    println!("  Monthly Payment: ${}", fixed_schedule.entries[0].payment);
    println!("  Total Interest:  ${}", fixed_schedule.total_interest);
    println!("  Total Payments:  ${}\n", fixed_schedule.total_payments);

    // ARM: 5% for first 5 years, then adjusts to 7%
    println!("Adjustable-Rate Mortgage (5% -> 7% after year 5):");
    let arm_initial = Rate::new(
        Decimal::new(5, 2),
        Compounding::MONTHLY,
        DayCount::Actual365,
    )?;
    let arm_adjusted = Rate::new(
        Decimal::new(7, 2),
        Compounding::MONTHLY,
        DayCount::Actual365,
    )?;

    let arm = casifin
        .arm(principal, arm_initial, term_months)
        .with_adjustment(61, arm_adjusted)
        .build()?;

    println!("  Initial Payment: ${}", arm.schedule.entries[0].payment);
    println!(
        "  Payment at month 61: ${}",
        arm.schedule.entries[60].payment
    );
    println!("  Total Interest:  ${}", arm.schedule.total_interest);
    println!("  Total Payments:  ${}\n", arm.schedule.total_payments);

    // Comparison
    let interest_diff = arm.schedule.total_interest - fixed_schedule.total_interest;
    if interest_diff > Money::ZERO {
        println!(
            "Fixed-rate saves ${} in interest vs this ARM scenario",
            interest_diff.abs()
        );
    } else {
        println!(
            "ARM saves ${} in interest vs fixed-rate",
            interest_diff.abs()
        );
    }

    Ok(())
}
