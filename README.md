# casifin

A Rust-based financial computation engine built with NASA-grade engineering standards.

[![CI](https://github.com/yourorg/casifin/actions/workflows/ci.yml/badge.svg)](https://github.com/yourorg/casifin/actions/workflows/ci.yml)

## Overview

`casifin` is a comprehensive financial calculation workspace providing:

- **Time Value of Money (TVM)**: PV, FV, PMT, NPER, RATE, perpetuities, uneven cash flows
- **Cash Flow Analysis**: NPV, IRR, XIRR, XNPV
- **Amortization**: Fixed-rate schedules with payment modifiers
- **Financial Ratios**: Liquidity, solvency, profitability, returns, yields, rates, statistics
- **Depreciation**: Straight-line and double-declining balance (with auto SL switch)
- **Inventory Costing**: FIFO, LIFO, weighted average
- **Unified SDK**: Single facade over all crates

See the interactive documentation site in [`docs/index.html`](docs/index.html) for a feature tour,
tutorials, and SDK usage examples.

## Safety Standards

Every crate enforces NASA/JPL Power-of-Ten inspired rules:

1. `#![deny(warnings)]` in every `lib.rs`
2. No `.unwrap()` or `.expect()` in library code
3. All `Result` values propagated with `?`
4. Checked `rust_decimal` arithmetic only (`checked_add`, `checked_div`, ...)
5. Max 60 lines per function
6. Max cyclomatic complexity 10 (clippy enforced)
7. Two `debug_assert!` preconditions per public function
8. Complete doc comments with formulas, examples, and `# Panics`
9. All money uses `Decimal` (never `f64`)
10. All types derive `Debug, Clone` (`Money` derives `Copy`)

## Installation

```toml
[dependencies]
casifin_sdk = "0.1.0"
rust_decimal = { version = "1.35", features = ["maths"] }
```

## Quick Start

```rust
use casifin_sdk::*;
use rust_decimal::Decimal;

fn main() -> Result<(), CasifinError> {
    let casifin = Casifin::with_defaults();

    // Calculate a mortgage payment
    let principal = Money::from(200_000);
    let rate = Rate::new(
        Decimal::new(6, 2), // 6% annual
        Compounding::Discrete(12),
    )?
    .with_convention(DayCount::Actual365);

    let schedule = casifin
        .mortgage(principal, rate, 360)
        .build()?;

    println!("Monthly payment: ${}", schedule.entries[0].payment);
    println!("Total interest: ${}", schedule.total_interest);

    // Calculate NPV/IRR
    let flows = CashFlowStream::new(vec![
        CashFlow::new(Money::from(-1000)), // Initial investment
        CashFlow::new(Money::from(500)),
        CashFlow::new(Money::from(500)),
        CashFlow::new(Money::from(500)),
    ]);

    let npv = casifin.npv(Decimal::new(10, 2), &flows)?;
    let irr = casifin.irr(&flows)?;

    println!("NPV: ${}", npv);
    println!("IRR: {:.2}%", irr * Decimal::from(100));

    Ok(())
}
```

## Examples

Run any of the bundled examples:

```bash
cargo run --example tutorial_01_getting_started
cargo run --example mortgage_calculator
cargo run --example investment_analysis
cargo run --example bond_pricing
```

## Workspace Structure

```
casifin/
├── crates/
│   ├── casifin-core/          # Foundation: Money, Rate, Config, errors
│   ├── casifin-tvm/           # Time Value of Money
│   ├── casifin-cashflow/      # NPV, IRR, XIRR, XNPV
│   ├── casifin-amortization/  # Loan amortization schedules
│   ├── casifin-ratios/        # Financial metrics and ratios
│   ├── casifin-depreciation/  # Depreciation methods
│   ├── casifin-inventory/     # Inventory costing (FIFO, LIFO, WA)
│   └── casifin-sdk/           # Unified API facade
├── examples/                  # Runnable tutorials
├── benches/                   # Criterion benchmarks
├── tests/                     # Integration and differential tests
└── docs/index.html            # Interactive feature documentation
```

## Quality Gates

All changes must pass:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
cargo run --example tutorial_01_getting_started
cargo bench
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
