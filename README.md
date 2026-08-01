# casifin

A Rust-based financial computation engine with NASA-grade engineering standards.

## Overview

`casifin` is a comprehensive financial calculation library providing:

- **Time Value of Money (TVM)**: PV, FV, PMT, NPER, RATE
- **Cash Flow Analysis**: NPV, IRR, XIRR, XNPV
- **Amortization**: Fixed-rate and ARM loan schedules
- **Financial Ratios**: Liquidity, solvency, profitability, returns
- **Depreciation**: Straight-line, double-declining balance
- **Inventory Costing**: FIFO, LIFO, weighted average

## Design Principles

This library follows NASA JPL's Power of Ten rules for safety-critical code:

1. Simple control flow (no `goto`, max cyclomatic complexity 10)
2. Fixed loop bounds
3. No dynamic allocation after init
4. No long functions (max 60 lines)
5. Minimum two assertions per function
6. Data at smallest scope
7. Check all return values
8. Limited preprocessor use
9. No raw pointers
10. Compile with warnings-as-errors

## Installation

```toml
[dependencies]
casifin-sdk = "0.1.0"
```

## Quick Start

```rust
use casifin_sdk::*;
use rust_decimal::Decimal;

fn main() -> Result<(), CasifinError> {
    let casifin = Casifin::with_default_config();

    // Calculate a mortgage payment
    let principal = Money::from(200000);
    let rate = Rate::new(
        Decimal::new(6, 2), // 6% annual
        Compounding::MONTHLY,
        DayCount::Actual365,
    )?;

    let schedule = casifin
        .mortgage(principal, rate, 360)
        .build()?;

    println!("Monthly payment: ${}", schedule.entries[0].payment);
    println!("Total interest: ${}", schedule.total_interest);

    // Calculate NPV/IRR
    let flows = CashFlowStream::from_vec(vec![
        Money::from(-1000), // Initial investment
        Money::from(500),
        Money::from(500),
        Money::from(500),
    ]);

    let npv = casifin.npv(Decimal::new(10, 2), &flows)?;
    let irr = casifin.irr(&flows)?;

    println!("NPV: ${}", npv);
    println!("IRR: {:.2}%", irr * Decimal::from(100));

    Ok(())
}
```

## Workspace Structure

```
casifin/
├── crates/
│   ├── casifin-core/          # Foundation types
│   ├── casifin-tvm/           # Time Value of Money
│   ├── casifin-cashflow/      # NPV, IRR, XIRR
│   ├── casifin-amortization/  # Loan amortization
│   ├── casifin-ratios/        # Financial metrics
│   ├── casifin-depreciation/  # Depreciation methods
│   ├── casifin-inventory/     # Inventory costing
│   └── casifin-sdk/           # Unified API
├── examples/
├── benches/
└── tests/
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
