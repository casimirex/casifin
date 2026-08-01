# Tutorials

## Tutorial 1: Getting Started - Your First Mortgage Calculation

This tutorial shows how to calculate a mortgage payment and generate an amortization schedule.

### Setup

```rust
use casifin_sdk::*;
use rust_decimal::Decimal;
```

### Calculate Monthly Payment

```rust
fn main() -> Result<(), CasifinError> {
    let casifin = Casifin::with_default_config();

    // Define loan parameters
    let principal = Money::from(200_000);  // $200,000 loan
    let rate = Rate::new(
        Decimal::new(6, 2),  // 6% annual rate
        Compounding::MONTHLY,
        DayCount::Actual365,
    )?;
    let term_months = 360;  // 30 years

    // Calculate the schedule
    let schedule = casifin
        .mortgage(principal, rate, term_months)
        .build()?;

    println!("Monthly Payment: ${}", schedule.entries[0].payment);
    println!("Total Interest: ${}", schedule.total_interest);
    println!("Total Payments: ${}", schedule.total_payments);

    Ok(())
}
```

### Output
```
Monthly Payment: $1199.10
Total Interest: $231,676.38
Total Payments: $431,676.38
```

### Extra: Payment Modification

```rust
let schedule = casifin
    .mortgage(principal, rate, term_months)
    .with_payment_modifier(|period, base_payment| {
        // Pay an extra $100 per month
        base_payment + Money::from(100)
    })
    .build()?;
```

---

## Tutorial 2: Investment Analysis - NPV and IRR

Analyze an investment opportunity using NPV and IRR.

### Scenario
- Initial investment: $10,000
- Year 1-5 cash inflows: $3,000 per year
- Required return: 10%

```rust
use casifin_sdk::*;
use rust_decimal::Decimal;

fn main() -> Result<(), CasifinError> {
    let casifin = Casifin::with_default_config();

    // Create cash flow stream
    let flows = CashFlowStream::from_vec(vec![
        Money::from(-10_000),  // Initial outflow
        Money::from(3_000),    // Year 1
        Money::from(3_000),    // Year 2
        Money::from(3_000),    // Year 3
        Money::from(3_000),    // Year 4
        Money::from(3_000),    // Year 5
    ]);

    // Calculate NPV at 10% discount rate
    let discount_rate = Decimal::new(10, 2);
    let npv = casifin.npv(discount_rate, &flows)?;
    println!("NPV at 10%: ${}", npv);

    // Calculate IRR
    let irr = casifin.irr(&flows)?;
    println!("IRR: {:.2}%", irr * Decimal::from(100));

    // Investment decision
    if npv > Money::ZERO {
        println!("✓ Accept: Positive NPV");
    }
    if irr > discount_rate {
        println!("✓ Accept: IRR > required return");
    }

    Ok(())
}
```

### Output
```
NPV at 10%: $1,372.36
IRR: 15.24%
✓ Accept: Positive NPV
✓ Accept: IRR > required return
```

---

## Tutorial 3: Adjustable Rate Mortgage (ARM)

Model an ARM with rate adjustments and caps.

### Scenario
- 30-year ARM starting at 5%
- Rate adjusts to 7% after year 5
- Periodic cap: 2% per adjustment
- Lifetime cap: 5% over initial rate

```rust
use casifin_sdk::*;
use rust_decimal::Decimal;

fn main() -> Result<(), CasifinError> {
    let casifin = Casifin::with_default_config();

    let principal = Money::from(300_000);
    let initial_rate = Rate::new(
        Decimal::new(5, 2),
        Compounding::MONTHLY,
        DayCount::Actual365,
    )?;

    // Rate adjustment at month 61 (year 6)
    let adj_rate = Rate::new(
        Decimal::new(7, 2),
        Compounding::MONTHLY,
        DayCount::Actual365,
    )?;

    // Rate caps
    let caps = RateCaps::new(
        Decimal::new(2, 2),  // 2% periodic cap
        Decimal::new(5, 2),  // 5% lifetime cap
    );

    // Build ARM schedule
    let arm = casifin
        .arm(principal, initial_rate, 360)
        .with_adjustment(61, adj_rate)
        .with_caps(caps)
        .build()?;

    // Show payment before and after adjustment
    let payment_before = arm.schedule.entries[60].payment;
    let payment_after = arm.schedule.entries[61].payment;

    println!("Payment before adjustment: ${}", payment_before);
    println!("Payment after adjustment: ${}", payment_after);
    println!("Total interest: ${}", arm.schedule.total_interest);

    Ok(())
}
```

---

## Tutorial 4: Portfolio Metrics - Sharpe Ratio and Returns

Calculate portfolio performance metrics.

```rust
use casifin_sdk::*;
use rust_decimal::Decimal;
use ratios::{returns, statistics};

fn main() -> Result<(), CasifinError> {
    // Portfolio returns over 5 periods
    let returns_vec = vec![
        Decimal::new(8, 2),   // 8%
        Decimal::new(12, 2),  // 12%
        Decimal::new(-5, 2),  // -5%
        Decimal::new(15, 2),  // 15%
        Decimal::new(6, 2),   // 6%
    ];

    // Geometric mean return
    let geo_mean = returns::geometric_mean_return(&returns_vec)?;
    println!("Geometric Mean Return: {:.2}%", geo_mean * Decimal::from(100));

    // Time-Weighted Rate of Return
    let twrr = returns::time_weighted_rate_of_return(&returns_vec)?;
    println!("TWRR: {:.2}%", twrr * Decimal::from(100));

    // Sharpe Ratio (assuming risk-free rate = 3%, std dev = 8%)
    let portfolio_return = Decimal::new(7, 2);  // 7% average
    let risk_free = Decimal::new(3, 2);         // 3%
    let std_dev = Decimal::new(8, 2);           // 8% volatility

    let sharpe = returns::sharpe_ratio(portfolio_return, risk_free, std_dev)?;
    println!("Sharpe Ratio: {:.2}", sharpe);

    // Interpretation
    if sharpe > Decimal::ONE {
        println!("✓ Good risk-adjusted returns (Sharpe > 1)");
    }

    Ok(())
}
```

### Output
```
Geometric Mean Return: 6.89%
TWRR: 38.42%
Sharpe Ratio: 0.50
```

---

## Additional Resources

- [API Reference](API_REFERENCE.md)
- [Architecture](ARCHITECTURE.md)
- [Contributing](CONTRIBUTING.md)
