//! Mortgage Calculator Example
//!
//! Demonstrates fixed-rate mortgage with optional extra principal payments.

use casifin_sdk::*;
use rust_decimal::Decimal;

fn main() -> Result<(), CasifinError> {
    println!("=== Mortgage Calculator ===\n");

    let casifin = Casifin::with_default_config();

    // Loan parameters
    let principal = Money::from(300_000);
    let fixed_rate = Rate::new(Decimal::new(6, 2), Compounding::Discrete(12))?
        .with_convention(DayCount::Actual365);
    let term_months = 360;

    // Fixed-rate mortgage
    println!("Fixed-Rate Mortgage (6% for 30 years):");
    let fixed_schedule = casifin
        .mortgage(principal, fixed_rate, term_months)
        .build()?;

    println!("  Monthly Payment: ${}", fixed_schedule.entries[0].payment);
    println!("  Total Interest:  ${}", fixed_schedule.total_interest);
    println!("  Total Payments:  ${}\n", fixed_schedule.total_payments);

    // With extra principal payments
    println!("With Extra $200 Principal Payment Each Month:");
    let extra_schedule = casifin
        .mortgage(principal, fixed_rate, term_months)
        .with_payment_modifier(|_period, base_payment| base_payment + Money::from(200))
        .build()?;

    println!("  Monthly Payment: ${}", extra_schedule.entries[0].payment);
    println!("  Total Interest:  ${}", extra_schedule.total_interest);
    println!(
        "  Payoff Periods:  {} (saves {} months)",
        extra_schedule.entries.len(),
        term_months - extra_schedule.entries.len() as u32
    );

    let interest_savings = fixed_schedule.total_interest - extra_schedule.total_interest;
    println!("  Interest Saved:  ${}\n", interest_savings);

    Ok(())
}
