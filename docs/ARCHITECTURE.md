# Architecture

This document describes the architecture of the `casifin` financial computation engine.

## Overview

`casifin` is a Rust workspace consisting of multiple crates organized in a layered architecture:

```
┌─────────────────────────────────────────────────────────────┐
│                      casifin-sdk                            │
│                   (Unified API Layer)                       │
├──────────┬──────────┬──────────────┬───────────────┬────────┤
│ casifin  │ casifin  │ casifin      │ casifin       │ ...    │
│ -tvm     │ -cashflow│ -amortization│ -ratios       │        │
├──────────┴──────────┴──────────────┴───────────────┴────────┤
│                    casifin-core                              │
│                 (Foundation Layer)                           │
└─────────────────────────────────────────────────────────────┘
```

## Crate Dependencies

```
casifin-sdk
├── casifin-core
├── casifin-tvm
├── casifin-cashflow
├── casifin-amortization
├── casifin-ratios
├── casifin-depreciation
└── casifin-inventory

casifin-amortization
├── casifin-core
└── casifin-tvm

casifin-ratios
├── casifin-core
└── casifin-cashflow

casifin-tvm
└── casifin-core

casifin-cashflow
└── casifin-core
```

## Core Types

### Money

A newtype wrapper around `rust_decimal::Decimal` providing:
- Type-safe monetary arithmetic
- Full precision (28 decimal places)
- Checked operations to prevent overflow

```rust
pub struct Money(Decimal);
```

### Rate

An interest rate with compounding metadata:

```rust
pub struct Rate {
    pub annual_rate: Decimal,
    pub compounding: Compounding,
    pub convention: DayCount,
}
```

### Compounding

```rust
pub enum Compounding {
    Discrete(u32),   // n times per year
    Continuous,
}
```

### DayCount

```rust
pub enum DayCount {
    Actual365,
    Actual360,
    Thirty360,
    ThirtyE360,
}
```

### CasifinError

The error type covering all possible error conditions:

```rust
pub enum CasifinError {
    InvalidRate(Decimal),
    InvalidPeriod(u32),
    IrrConvergenceFailure { max_iter: u32, eps: Decimal },
    DivisionByZero { operation: &'static str },
    ScheduleOverflow { detail: String },
    DateOutOfRange(String),
    // ... and more
}
```

## Module Responsibilities

### casifin-core

Foundation layer providing:
- `Money` type with arithmetic operations
- `Rate` type with compounding and day count
- `Config` for global settings
- Shared traits: `FinancialCalculation`, `Schedulable`, `CashFlowStream`

### casifin-tvm

Time Value of Money calculations:
- `pv` - Present Value
- `fv` - Future Value
- `pmt` - Payment
- `nper` - Number of Periods
- `rate` - Interest Rate (Newton-Raphson solver)
- `pv_perpetuity` - Present Value of Perpetuity
- `fv_uneven_cashflows` - FV of irregular payments

### casifin-cashflow

Cash flow analysis:
- `npv` - Net Present Value
- `irr` - Internal Rate of Return (hybrid Newton/bisection)
- `xnpv` - NPV with actual dates
- `xirr` - IRR with actual dates

### casifin-amortization

Loan amortization engines:
- `AmortizationBuilder` - Fluent API for fixed-rate loans
- `AmortizationSchedule` - Complete payment schedule
- `AdjustableRateBuilder` - ARM support with rate caps
- Payment modifier hooks for custom scenarios

### casifin-ratios

Financial metrics (FinCal port):
- Liquidity ratios: current, quick, cash
- Solvency ratios: debt, debt-to-equity, leverage
- Profitability ratios: margins, EPS
- Return metrics: HPR, geometric mean, Sharpe ratio
- Yield metrics: BDY, MMY, BEY
- Rate conversions: EAR, stated, continuous

### casifin-depreciation

Asset depreciation methods:
- `StraightLine` - Linear depreciation
- `DoubleDecliningBalance` - Accelerated with auto-switch to SL

### casifin-inventory

Inventory costing methods:
- `Fifo` - First-In, First-Out
- `Lifo` - Last-In, First-Out
- `WeightedAverage` - Average cost method

## Error Handling Strategy

All library functions return `Result<T, CasifinError>`:
- No `.unwrap()` or `.expect()` in library code
- Checked arithmetic using `rust_decimal` methods
- Convergence failures return diagnostic information
- Precondition validation with descriptive errors

## Numerical Algorithms

### Newton-Raphson Solver

Used in `rate()` and `irr()` functions:
- Hybrid approach with bisection fallback
- Maximum iterations: 1000 (configurable)
- Convergence threshold: 1e-12 (configurable)
- Rate clamping to prevent divergence

### Day Count Calculations

Date-based calculations use:
- `chrono::NaiveDate` for date arithmetic
- Actual/365 or 30/360 conventions
- Year fraction: `days / denominator`

## Testing Strategy

### Unit Tests
- Happy path with reference values
- Boundary conditions (zero, negative inputs)
- Error path validation

### Integration Tests
- End-to-end mortgage calculations
- Full NPV/IRR pipelines
- Cross-crate functionality

### Property Tests (proptest)
- NPV at zero rate equals sum of cash flows
- Depreciation totals equal depreciable base
- Amortization schedules sum to principal

## Build Configuration

### rustfmt.toml
```toml
max_width = 100
format_strings = true
wrap_comments = true
```

### .clippy.toml
```toml
cognitive-complexity-threshold = 10
too-many-arguments-threshold = 7
```

### CI Pipeline
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo audit`
