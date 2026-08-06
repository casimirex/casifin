# **CASIFIN — Claude Code Build Specification**
## *Complete executable context for building the `casifin` financial computation engine from scratch*

---

## SYSTEM CONTEXT

You are building `casifin`, a Rust workspace containing a production-grade financial computation engine. This is a port of the Ruby `finrb` gem to Rust with NASA JPL Power-of-Ten safety standards.

**Non-negotiable rules for ALL code:**
1. `#![deny(warnings)]` in every `lib.rs`
2. No `.unwrap()` or `.expect()` in library code (tests/examples OK)
3. Every `Result` must be propagated with `?`
4. Use `checked_add`, `checked_div`, `checked_mul`, `checked_sub` from `rust_decimal` — never raw operators on money
5. Max 60 lines per function
6. Max cyclomatic complexity 10 (enforced by clippy)
7. Every public function must have: doc comment with formula, arguments, returns, example, error conditions, and a `# Panics` section stating "This function does not panic"
8. Two assertions minimum per function: `debug_assert!` for preconditions
9. All monetary calculations use `Decimal` (never `f64`)
10. All types derive `Debug, Clone`. `Money` must derive `Copy`.

---

## WORKSPACE STRUCTURE TO CREATE

Create this exact directory layout:

```
casifin/
├── Cargo.toml
├── rustfmt.toml
├── .clippy.toml
├── .github/
│   └── workflows/
│       └── ci.yml
├── crates/
│   ├── casifin-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── casifin-tvm/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── casifin-cashflow/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── casifin-amortization/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── casifin-ratios/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── liquidity.rs
│   │       ├── solvency.rs
│   │       ├── profitability.rs
│   │       ├── returns.rs
│   │       ├── yields.rs
│   │       ├── rates.rs
│   │       └── statistics.rs
│   ├── casifin-depreciation/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── casifin-inventory/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   └── casifin-sdk/
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
├── examples/
│   ├── tutorial_01_getting_started.rs
│   ├── mortgage_calculator.rs
│   ├── investment_analysis.rs
│   └── bond_pricing.rs
├── tests/
│   ├── integration_tests.rs
│   └── differential_tests.rs
└── benches/
    └── amortization_bench.rs
```

---

## PHASE 0: BOOTSTRAP

### 0.1 Root `Cargo.toml`
```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.dependencies]
rust_decimal = { version = "1.35", features = ["maths", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }

[workspace.lints.clippy]
cognitive_complexity = "deny"
too_many_arguments = "deny"
unwrap_used = "deny"
expect_used = "deny"
```

### 0.2 `rustfmt.toml`
```toml
max_width = 100
fn_single_line = false
format_strings = true
imports_granularity = "Crate"
reorder_imports = true
group_imports = "StdExternalCrate"
```

### 0.3 `.clippy.toml`
```toml
cognitive-complexity-threshold = 10
too-many-arguments-threshold = 5
```

### 0.4 `.github/workflows/ci.yml`
```yaml
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --check
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo test --workspace
      - run: cargo doc --workspace --no-deps
```

### 0.5 Crate `Cargo.toml` Template
Every crate under `crates/` gets this template (replace `CRATE_NAME`):
```toml
[package]
name = "CRATE_NAME"
version = "0.1.0"
edition = "2021"
authors = ["casifin contributors"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/casimirex/casifin"

[dependencies]
casifin-core = { path = "../casifin-core" }
rust_decimal = { workspace = true }
thiserror = { workspace = true }
serde = { workspace = true, optional = true }

[dev-dependencies]
approx = { workspace = true }

[features]
default = ["std"]
std = []
serde = ["dep:serde", "casifin-core/serde"]
```

**Do NOT write any business logic yet. Only create files and directories.**

---

## PHASE 1: CASIFIN-CORE

Create `crates/casifin-core/src/lib.rs` with EXACTLY these items:

### 1.1 Money Newtype
```rust
use rust_decimal::Decimal;
use std::fmt;
use std::ops::{Add, Sub, Mul, Div, Neg};
use std::str::FromStr;

/// A monetary value with guaranteed precision.
/// Invariant: stores exact decimal representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Money(Decimal);

impl Money {
    pub const ZERO: Money = Money(Decimal::ZERO);

    pub fn new(dollars: i64, cents: u32) -> Result<Self, CasifinError> {
        let dec = Decimal::new(dollars * 100 + i64::from(cents), 2);
        Ok(Money(dec))
    }

    pub fn from_decimal(dec: Decimal) -> Self {
        Money(dec)
    }

    pub fn from_str(s: &str) -> Result<Self, CasifinError> {
        let dec = Decimal::from_str(s)
            .map_err(|e| CasifinError::ParseError(format!("{e}")))?;
        Ok(Money(dec))
    }

    pub fn inner(&self) -> Decimal {
        self.0
    }

    pub fn abs(&self) -> Self {
        Money(self.0.abs())
    }

    pub fn is_zero(&self) -> bool {
        self.0 == Decimal::ZERO
    }

    pub fn is_positive(&self) -> bool {
        self.0 > Decimal::ZERO
    }

    pub fn is_negative(&self) -> bool {
        self.0 < Decimal::ZERO
    }

    pub fn round_to_cents(&self) -> Self {
        Money(self.0.round_dp(2))
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl Add for Money {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Money(self.0 + rhs.0)
    }
}

impl Sub for Money {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Money(self.0 - rhs.0)
    }
}

impl Mul<Decimal> for Money {
    type Output = Self;
    fn mul(self, rhs: Decimal) -> Self::Output {
        Money(self.0 * rhs)
    }
}

impl Div<Decimal> for Money {
    type Output = Self;
    fn div(self, rhs: Decimal) -> Self::Output {
        Money(self.0 / rhs)
    }
}

impl Neg for Money {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Money(-self.0)
    }
}

impl std::iter::Sum for Money {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Money(iter.map(|m| m.0).sum())
    }
}
```

### 1.2 Compounding Enum
```rust
/// The compounding frequency for an interest rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Compounding {
    /// Discrete compounding n times per year.
    /// Invariant: n >= 1.
    Discrete(u32),
    /// Continuous compounding (e^x).
    Continuous,
}

impl Compounding {
    pub fn periods_per_year(&self) -> Option<u32> {
        match self {
            Compounding::Discrete(n) => Some(*n),
            Compounding::Continuous => None,
        }
    }
}
```

### 1.3 DayCount Enum
```rust
/// Day count convention for interest accrual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DayCount {
    Actual365,
    Actual360,
    Thirty360,
    ThirtyE360,
    ActualActualIsda,
}
```

### 1.4 PaymentDue Enum
```rust
/// When payments are due within a period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaymentDue {
    Beginning,
    End,
}
```

### 1.5 Rate Struct
```rust
/// An interest rate with compounding metadata.
/// Invariant: annual_rate >= 0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rate {
    pub annual_rate: Decimal,
    pub compounding: Compounding,
    pub convention: DayCount,
}

impl Rate {
    pub fn new(annual_rate: Decimal, compounding: Compounding) -> Result<Self, CasifinError> {
        if annual_rate < Decimal::ZERO {
            return Err(CasifinError::InvalidRate(annual_rate));
        }
        if let Compounding::Discrete(n) = compounding {
            if n == 0 {
                return Err(CasifinError::InvalidCompounding);
            }
        }
        Ok(Rate {
            annual_rate,
            compounding,
            convention: DayCount::Actual365,
        })
    }

    pub fn with_convention(mut self, convention: DayCount) -> Self {
        self.convention = convention;
        self
    }

    pub fn periodic_rate(&self) -> Result<Decimal, CasifinError> {
        match self.compounding {
            Compounding::Discrete(n) => {
                self.annual_rate
                    .checked_div(Decimal::from(n))
                    .ok_or(CasifinError::DivisionByZero { operation: "periodic_rate" })
            }
            Compounding::Continuous => Ok(self.annual_rate),
        }
    }
}
```

### 1.6 Config Struct
```rust
/// Global configuration for numerical methods.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Config {
    pub eps: Decimal,
    pub max_iterations: u32,
    pub guess: Decimal,
    pub business_days_only: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            eps: Decimal::new(1, 12), // 1e-12
            max_iterations: 1000,
            guess: Decimal::new(1, 1), // 0.1
            business_days_only: false,
        }
    }
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfigBuilder {
    eps: Decimal,
    max_iterations: u32,
    guess: Decimal,
    business_days_only: bool,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        ConfigBuilder {
            eps: Decimal::new(1, 12),
            max_iterations: 1000,
            guess: Decimal::new(1, 1),
            business_days_only: false,
        }
    }
}

impl ConfigBuilder {
    pub fn eps(mut self, eps: Decimal) -> Self {
        self.eps = eps;
        self
    }
    pub fn max_iterations(mut self, n: u32) -> Self {
        self.max_iterations = n;
        self
    }
    pub fn guess(mut self, guess: Decimal) -> Self {
        self.guess = guess;
        self
    }
    pub fn business_days_only(mut self, v: bool) -> Self {
        self.business_days_only = v;
        self
    }
    pub fn build(self) -> Config {
        Config {
            eps: self.eps,
            max_iterations: self.max_iterations,
            guess: self.guess,
            business_days_only: self.business_days_only,
        }
    }
}
```

### 1.7 Error Enum
```rust
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum CasifinError {
    #[error("invalid rate: {0}. Rate must be non-negative.")]
    InvalidRate(Decimal),

    #[error("invalid compounding frequency: must be >= 1 for discrete compounding")]
    InvalidCompounding,

    #[error("invalid period: {0}. Period must be positive.")]
    InvalidPeriod(u32),

    #[error("invalid amount: {0}. Amount must be non-zero for this operation.")]
    InvalidAmount(Money),

    #[error("IRR did not converge within {max_iter} iterations (eps={eps})")]
    IrrConvergenceFailure { max_iter: u32, eps: Decimal },

    #[error("division by zero in operation: {operation}")]
    DivisionByZero { operation: &'static str },

    #[error("amortization schedule overflow: {detail}")]
    ScheduleOverflow { detail: String },

    #[error("date out of range: {0}")]
    DateOutOfRange(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("insufficient cash flows: at least one positive and one negative required")]
    InsufficientCashFlows,

    #[error("invalid input: {reason}")]
    InvalidInput { reason: String },
}
```

### 1.8 Shared Traits
```rust
/// Trait for types that can perform a financial calculation.
pub trait FinancialCalculation {
    type Output;
    fn calculate(&self) -> Result<Self::Output, CasifinError>;
}

/// Trait for types that generate a schedule of entries.
pub trait Schedulable {
    type Entry;
    fn schedule(&self) -> Result<Vec<Self::Entry>, CasifinError>;
}
```

### 1.9 Tests for Core
Write tests in `crates/casifin-core/src/lib.rs` inside `#[cfg(test)] mod tests`:
- `money_new_valid` — create $100.50
- `money_from_str_valid` — parse "1234.56"
- `money_from_str_invalid` — parse "abc" returns ParseError
- `money_arithmetic` — add, sub, mul, div
- `rate_new_negative` — returns InvalidRate
- `rate_new_zero_compounding` — returns InvalidCompounding
- `rate_periodic` — 5% annual monthly = 0.004166...
- `config_builder` — build custom config
- `config_default` — verify defaults

**Run `cargo test -p casifin-core` and ensure all pass before proceeding.**

---

## PHASE 2: CASIFIN-TVM

Create `crates/casifin-tvm/src/lib.rs`.

### 2.1 Required Functions
Implement these EXACT signatures:

```rust
use casifin_core::{Money, Rate, PaymentDue, CasifinError, Config};
use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;

/// Present Value of an ordinary annuity.
pub fn pv(rate: Rate, nper: u32, pmt: Money, fv: Money, due: PaymentDue) -> Result<Money, CasifinError>;

/// Future Value of an ordinary annuity.
pub fn fv(rate: Rate, nper: u32, pmt: Money, pv: Money, due: PaymentDue) -> Result<Money, CasifinError>;

/// Payment per period.
pub fn pmt(rate: Rate, nper: u32, pv: Money, fv: Money, due: PaymentDue) -> Result<Money, CasifinError>;

/// Number of periods.
pub fn nper(rate: Rate, pmt: Money, pv: Money, fv: Money, due: PaymentDue) -> Result<Decimal, CasifinError>;

/// Interest rate per period (Newton-Raphson + bisection hybrid).
pub fn rate(nper: u32, pmt: Money, pv: Money, fv: Money, due: PaymentDue, guess: Option<Decimal>, config: Config) -> Result<Decimal, CasifinError>;

/// Present Value of a perpetuity.
pub fn pv_perpetuity(rate: Rate, pmt: Money) -> Result<Money, CasifinError>;

/// Future Value of uneven cash flows.
pub fn fv_uneven_cashflows(rate: Rate, flows: &[Money]) -> Result<Money, CasifinError>;
```

### 2.2 Implementation Requirements

**For `pv`:**
```text
If rate == 0: PV = -(FV + PMT * nper)
If due == End:  PV = [PMT * (1 - (1+r)^-n) / r] + FV * (1+r)^-n
If due == Beginning: multiply PMT term by (1+r)
```
Use `checked_powi` for `(1+r)^-n`. Return `ScheduleOverflow` on power failure.

**For `rate`:**
1. Start with guess (default 0.1 from Config)
2. Use Newton-Raphson: x_{n+1} = x_n - f(x)/f'(x)
3. If derivative magnitude < eps, switch to bisection between [-0.9999, 1.0]
4. If bisection also fails after max_iterations, return `IrrConvergenceFailure`
5. The function f(r) = PV*(1+r)^n + PMT*((1+r)^n - 1)/r * (1+r*due) + FV = 0

**For `pmt`:**
```text
If rate == 0: PMT = -(PV + FV) / nper
If due == End:  PMT = -(PV*r + FV*r/((1+r)^n - 1)) / (1 - (1+r)^-n)
```

**For `nper`:**
```text
If rate == 0: nper = -(PV + FV) / PMT
Else: nper = ln((PMT - FV*r) / (PMT + PV*r)) / ln(1+r)
```
Use `checked_ln` from `MathematicalOps`.

### 2.3 Tests
Add these exact test cases (compare to Excel/HP-12C):
- `pv_annuity_end` — PV of $1000/year for 5 years at 5% = $4,329.48
- `pv_annuity_begin` — same but beginning = $4,545.95
- `fv_annuity_end` — FV of $1000/year for 5 years at 5% = $5,525.63
- `pmt_mortgage` — PMT on $300,000 at 4.25% for 30 years (360 months) = $1,475.82
- `nper_loan` — NPER to pay off $10,000 at $200/month, 0% = 50 periods
- `rate_savings` — RATE to grow $1000 to $2000 with $0 PMT over 10 years = 7.18%
- `pv_perpetuity_5pct` — $100 / 0.05 = $2,000
- `rate_convergence_failure` — pathological inputs return `IrrConvergenceFailure`

**Run `cargo test -p casifin-tvm` and ensure all pass.**

---

## PHASE 3: CASIFIN-CASHFLOW

Create `crates/casifin-cashflow/src/lib.rs`.

### 3.1 Types
```rust
use casifin_core::{Money, CasifinError, Config};
use rust_decimal::Decimal;
use chrono::NaiveDate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CashFlow {
    pub amount: Money,
    pub date: Option<NaiveDate>,
}

impl CashFlow {
    pub fn new(amount: Money) -> Self {
        CashFlow { amount, date: None }
    }
    pub fn with_date(amount: Money, date: NaiveDate) -> Self {
        CashFlow { amount, date: Some(date) }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CashFlowStream(Vec<CashFlow>);

impl CashFlowStream {
    pub fn new(flows: Vec<CashFlow>) -> Self {
        CashFlowStream(flows)
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn has_positive_and_negative(&self) -> bool {
        let has_pos = self.0.iter().any(|f| f.amount.is_positive());
        let has_neg = self.0.iter().any(|f| f.amount.is_negative());
        has_pos && has_neg
    }
}
```

### 3.2 Functions
```rust
/// Net Present Value with periodic cash flows.
pub fn npv(rate: Decimal, stream: &CashFlowStream) -> Result<Money, CasifinError>;

/// Internal Rate of Return.
pub fn irr(stream: &CashFlowStream, config: Config) -> Result<Decimal, CasifinError>;

/// NPV with explicit dates (XNPV).
pub fn xnpv(rate: Decimal, stream: &CashFlowStream) -> Result<Money, CasifinError>;

/// IRR with explicit dates (XIRR).
pub fn xirr(stream: &CashFlowStream, config: Config) -> Result<Decimal, CasifinError>;
```

### 3.3 Implementation Requirements

**`npv`:**
```text
NPV = sum_{t=0}^{n-1} CF_t / (1 + rate)^t
```
Note: t starts at 0 for the first flow (immediate). Use `checked_powi`.

**`irr`:**
- Precondition: `stream.has_positive_and_negative()` — else return `InsufficientCashFlows`
- Use Newton-Raphson with bisection fallback (same solver as `tvm::rate` but applied to NPV function)
- f(r) = NPV(rate) = 0
- Convergence: |NPV(rate)| < config.eps
- Max iterations: config.max_iterations

**`xnpv`:**
- Requires all flows to have dates
- Find min date, compute days from min date for each flow
- `NPV = sum CF_i / (1 + rate)^(days_i / 365)`
- Use `checked_powd` with `Decimal::from(days) / Decimal::from(365)`

**`xirr`:**
- Same solver as IRR but using XNPV as the objective function
- Date-weighted derivative required for Newton step

### 3.4 Tests
- `npv_zero_rate` — NPV at 0% = sum of all flows
- `npv_known_value` — flows [-1000, 300, 400, 400, 300] at 8% = $179.42
- `irr_known_value` — same flows = 14.49%
- `irr_insufficient_flows` — all positive flows returns error
- `xnpv_known_value` — use Excel reference data
- `xirr_known_value` — use Excel reference data
- `irr_convergence` — pathological flows return `IrrConvergenceFailure`

---

## PHASE 4: CASIFIN-AMORTIZATION

Create `crates/casifin-amortization/src/lib.rs`.

### 4.1 Types
```rust
use casifin_core::{Money, Rate, CasifinError, Config, DayCount};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AmortizationEntry {
    pub period: u32,
    pub payment: Money,
    pub principal: Money,
    pub interest: Money,
    pub balance: Money,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AmortizationSchedule {
    pub entries: Vec<AmortizationEntry>,
    pub total_payments: Money,
    pub total_interest: Money,
    pub total_principal: Money,
}

pub struct AmortizationBuilder {
    principal: Money,
    rate: Rate,
    term_months: u32,
    config: Config,
    payment_modifier: Option<Box<dyn Fn(u32, Money) -> Money>>,
}
```

### 4.2 Builder API
```rust
impl AmortizationBuilder {
    pub fn new(principal: Money, rate: Rate, term_months: u32) -> Self;
    pub fn with_config(mut self, config: Config) -> Self;
    pub fn with_payment_modifier<F>(mut self, modifier: F) -> Self 
    where F: Fn(u32, Money) -> Money + 'static;
    pub fn build(self) -> Result<AmortizationSchedule, CasifinError>;
}
```

### 4.3 Implementation Requirements

**Fixed-Rate Amortization:**
1. Compute base payment using `casifin_tvm::pmt` with `pv = -principal`, `fv = 0`, `nper = term_months`
2. For each period 1..=term_months:
   - `interest = balance * periodic_rate`
   - `payment = base_payment` (or modified)
   - `principal = payment - interest`
   - `balance = previous_balance - principal`
   - If final period, adjust payment to ensure balance = 0
3. Sum totals

**Invariants (post-build checks):**
- `principal > 0` (precondition)
- `term_months > 0` (precondition)
- Final balance must satisfy `final_balance.abs() <= config.eps`
- `total_principal == original_principal` within `config.eps`
- `total_payments == total_principal + total_interest` within `config.eps`
- Each entry: `payment == principal + interest` within `config.eps`

**Day Count:**
- `Actual365`: monthly rate = annual_rate / 12
- `Actual360`: monthly rate = annual_rate / 12 (standard US mortgage)
- `Thirty360`: use 30-day months, 360-day year

### 4.4 ARM Support (Optional but Recommended)
```rust
#[derive(Debug, Clone)]
pub struct ArmSchedule {
    pub adjustments: Vec<(u32, Rate)>, // (start_period, new_rate)
    pub periodic_cap: Option<Decimal>,  // max change per adjustment
    pub lifetime_cap: Option<Decimal>,  // max rate over life
    pub lifetime_floor: Option<Decimal>, // min rate over life
}
```

### 4.5 Tests
- `fixed_30_year_mortgage` — $300,000 at 4.25% for 360 months
  - First payment: $1,475.82
  - First interest: $1,062.50
  - First principal: $413.32
  - Final balance: 0
- `payment_modifier` — extra $100 principal each month reduces term
- `zero_rate` — 0% rate = principal / term each period
- `invalid_principal` — zero principal returns error
- `invariant_final_balance` — verify balance == 0

---

## PHASE 5: CASIFIN-RATIOS

Create `crates/casifin-ratios/src/lib.rs` with module re-exports, then create each module file.

### 5.1 Module Structure
```rust
pub mod liquidity;
pub mod solvency;
pub mod profitability;
pub mod returns;
pub mod yields;
pub mod rates;
pub mod statistics;

pub use liquidity::*;
pub use solvency::*;
pub use profitability::*;
pub use returns::*;
pub use yields::*;
pub use rates::*;
pub use statistics::*;
```

### 5.2 Functions to Implement (exact signatures)

**`crates/casifin-ratios/src/liquidity.rs`:**
```rust
pub fn current_ratio(current_assets: Money, current_liabilities: Money) -> Result<Decimal, CasifinError>;
pub fn quick_ratio(cash: Money, marketable_securities: Money, receivables: Money, current_liabilities: Money) -> Result<Decimal, CasifinError>;
pub fn cash_ratio(cash: Money, current_liabilities: Money) -> Result<Decimal, CasifinError>;
pub fn defensive_interval(cash: Money, marketable_securities: Money, receivables: Money, daily_cash_expenditures: Money) -> Result<Decimal, CasifinError>;
```

**`crates/casifin-ratios/src/solvency.rs`:**
```rust
pub fn debt_ratio(total_debt: Money, total_assets: Money) -> Result<Decimal, CasifinError>;
pub fn debt_to_equity(total_debt: Money, total_equity: Money) -> Result<Decimal, CasifinError>;
pub fn financial_leverage(total_assets: Money, total_equity: Money) -> Result<Decimal, CasifinError>;
pub fn interest_coverage(ebit: Money, interest_expense: Money) -> Result<Decimal, CasifinError>;
```

**`crates/casifin-ratios/src/profitability.rs`:**
```rust
pub fn gross_profit_margin(gross_profit: Money, revenue: Money) -> Result<Decimal, CasifinError>;
pub fn operating_profit_margin(operating_income: Money, revenue: Money) -> Result<Decimal, CasifinError>;
pub fn net_profit_margin(net_income: Money, revenue: Money) -> Result<Decimal, CasifinError>;
pub fn return_on_assets(net_income: Money, total_assets: Money) -> Result<Decimal, CasifinError>;
pub fn return_on_equity(net_income: Money, total_equity: Money) -> Result<Decimal, CasifinError>;
pub fn basic_eps(net_income: Money, preferred_dividends: Money, shares_outstanding: Decimal) -> Result<Decimal, CasifinError>;
pub fn diluted_eps(net_income: Money, preferred_dividends: Money, weighted_shares: Decimal, potential_shares: Decimal) -> Result<Decimal, CasifinError>;
```

**`crates/casifin-ratios/src/returns.rs`:**
```rust
pub fn holding_period_return(purchase_price: Money, sale_price: Money, income: Money) -> Result<Decimal, CasifinError>;
pub fn arithmetic_mean_return(returns: &[Decimal]) -> Result<Decimal, CasifinError>;
pub fn geometric_mean_return(returns: &[Decimal]) -> Result<Decimal, CasifinError>;
pub fn time_weighted_rate_of_return(period_returns: &[Decimal]) -> Result<Decimal, CasifinError>;
pub fn money_weighted_return(cash_flows: &[Money], dates: Option<&[chrono::NaiveDate]>) -> Result<Decimal, CasifinError>;
pub fn sharpe_ratio(portfolio_return: Decimal, risk_free_rate: Decimal, std_dev: Decimal) -> Result<Decimal, CasifinError>;
pub fn roys_safety_first_ratio(expected_return: Decimal, threshold: Decimal, std_dev: Decimal) -> Result<Decimal, CasifinError>;
pub fn sortino_ratio(portfolio_return: Decimal, target_return: Decimal, downside_deviation: Decimal) -> Result<Decimal, CasifinError>;
```

**`crates/casifin-ratios/src/yields.rs`:**
```rust
pub fn bank_discount_yield(face_value: Money, purchase_price: Money, days_to_maturity: u32) -> Result<Decimal, CasifinError>;
pub fn money_market_yield(face_value: Money, purchase_price: Money, days_to_maturity: u32) -> Result<Decimal, CasifinError>;
pub fn bond_equivalent_yield(semi_annual_yield: Decimal) -> Result<Decimal, CasifinError>;
pub fn holding_period_yield(purchase_price: Money, sale_price: Money, coupon: Money) -> Result<Decimal, CasifinError>;
```

**`crates/casifin-ratios/src/rates.rs`:**
```rust
pub fn effective_annual_rate(stated_rate: Decimal, compounding: casifin_core::Compounding) -> Result<Decimal, CasifinError>;
pub fn stated_from_effective(effective_rate: Decimal, compounding: casifin_core::Compounding) -> Result<Decimal, CasifinError>;
pub fn continuous_to_nominal(continuous_rate: Decimal) -> Result<Decimal, CasifinError>;
pub fn nominal_to_continuous(nominal_rate: Decimal) -> Result<Decimal, CasifinError>;
pub fn equivalent_rate(rate: casifin_core::Rate, target_compounding: casifin_core::Compounding) -> Result<casifin_core::Rate, CasifinError>;
```

**`crates/casifin-ratios/src/statistics.rs`:**
```rust
pub fn coefficient_of_variation(mean: Decimal, std_dev: Decimal) -> Result<Decimal, CasifinError>;
pub fn weighted_mean(values: &[Decimal], weights: &[Decimal]) -> Result<Decimal, CasifinError>;
pub fn harmonic_mean(values: &[Decimal]) -> Result<Decimal, CasifinError>;
pub fn sampling_error(population_std_dev: Decimal, sample_size: u32) -> Result<Decimal, CasifinError>;
pub fn standard_error(std_dev: Decimal, sample_size: u32) -> Result<Decimal, CasifinError>;
```

### 5.3 Documentation Requirements
Every function must include:
- Formula in doc comment using `text` code blocks
- CFA/CPA curriculum reference (e.g., "CFA Level I, Quantitative Methods, Reading 6")
- Complete example with assert
- Error conditions

### 5.4 Tests
At least one test per function with a known reference value.

---

## PHASE 6: CASIFIN-DEPRECIATION

Create `crates/casifin-depreciation/src/lib.rs`.

### 6.1 Trait and Implementations
```rust
use casifin_core::{Money, CasifinError};

pub trait DepreciationMethod {
    fn depreciate(&self, cost: Money, salvage: Money, life_years: u32, period: u32) -> Result<Money, CasifinError>;
    fn schedule(&self, cost: Money, salvage: Money, life_years: u32) -> Result<Vec<Money>, CasifinError>;
}

pub struct StraightLine;
pub struct DoubleDecliningBalance;

impl DepreciationMethod for StraightLine { ... }
impl DepreciationMethod for DoubleDecliningBalance { ... }
```

### 6.2 Implementation Requirements

**Straight Line:**
```text
annual_depreciation = (cost - salvage) / life_years
```
Use `checked_div`. Last period gets remainder to ensure total = cost - salvage.

**Double Declining Balance:**
```text
ddb_rate = 2 / life_years
period_depreciation = book_value_at_start * ddb_rate
```
Auto-switch to Straight Line when SL on remaining book value > DDB amount.
Invariants:
- `cost > salvage`
- `life_years > 0`
- `period <= life_years`
- Sum of schedule = cost - salvage

### 6.3 Tests
- `sl_5_year` — $10,000 cost, $2,000 salvage, 5 years = $1,600/year
- `ddb_switch` — verify auto-switch to SL occurs
- `ddb_total` — sum of all periods = cost - salvage
- `invalid_period` — period > life_years returns error

---

## PHASE 7: CASIFIN-INVENTORY

Create `crates/casifin-inventory/src/lib.rs`.

### 7.1 Types and Trait
```rust
use casifin_core::{Money, CasifinError};
use chrono::NaiveDate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InventoryLot {
    pub units: u32,
    pub unit_cost: Money,
    pub date: NaiveDate,
}

pub trait InventoryMethod {
    fn cogs(&self, lots: &[InventoryLot], units_sold: u32) -> Result<Money, CasifinError>;
    fn ending_inventory(&self, lots: &[InventoryLot], units_sold: u32) -> Result<Money, CasifinError>;
}

pub struct Fifo;
pub struct Lifo;
pub struct WeightedAverage;

impl InventoryMethod for Fifo { ... }
impl InventoryMethod for Lifo { ... }
impl InventoryMethod for WeightedAverage { ... }
```

### 7.2 Implementation Requirements

**FIFO:** Sell oldest lots first.
**LIFO:** Sell newest lots first.
**Weighted Average:** Compute average cost = total_cost / total_units, then apply.

Invariants:
- `units_sold <= total_units_in_lots`
- `cogs + ending_inventory == total_cost_of_all_lots`

### 7.3 Tests
- `fifo_cogs` — 3 lots, sell 5 units, verify oldest consumed first
- `lifo_cogs` — 3 lots, sell 5 units, verify newest consumed first
- `weighted_avg` — verify average cost applied
- `insufficient_inventory` — sell more than available returns error
- `inventory_identity` — cogs + ending = total cost

---

## PHASE 8: CASIFIN-SDK

Create `crates/casifin-sdk/src/lib.rs`.

### 8.1 Unified API
```rust
pub use casifin_core::*;
pub use casifin_tvm as tvm;
pub use casifin_cashflow as cashflow;
pub use casifin_amortization as amortization;
pub use casifin_ratios as ratios;
pub use casifin_depreciation as depreciation;
pub use casifin_inventory as inventory;

use casifin_core::{Money, Rate, Config, CasifinError};

/// Unified entry point for the casifin financial engine.
#[derive(Debug, Clone)]
pub struct Casifin {
    config: Config,
}

impl Casifin {
    pub fn new(config: Config) -> Self {
        Casifin { config }
    }

    pub fn with_defaults() -> Self {
        Casifin { config: Config::default() }
    }

    pub fn config(&self) -> Config {
        self.config
    }

    pub fn mortgage(&self, principal: Money, rate: Rate, months: u32) -> casifin_amortization::AmortizationBuilder {
        casifin_amortization::AmortizationBuilder::new(principal, rate, months)
            .with_config(self.config)
    }

    pub fn npv(&self, rate: Decimal, flows: &casifin_cashflow::CashFlowStream) -> Result<Money, CasifinError> {
        casifin_cashflow::npv(rate, flows)
    }

    pub fn irr(&self, flows: &casifin_cashflow::CashFlowStream) -> Result<Decimal, CasifinError> {
        casifin_cashflow::irr(flows, self.config)
    }

    pub fn pv(&self, rate: Rate, nper: u32, pmt: Money, fv: Money, due: PaymentDue) -> Result<Money, CasifinError> {
        casifin_tvm::pv(rate, nper, pmt, fv, due)
    }

    pub fn fv(&self, rate: Rate, nper: u32, pmt: Money, pv: Money, due: PaymentDue) -> Result<Money, CasifinError> {
        casifin_tvm::fv(rate, nper, pmt, pv, due)
    }

    pub fn pmt(&self, rate: Rate, nper: u32, pv: Money, fv: Money, due: PaymentDue) -> Result<Money, CasifinError> {
        casifin_tvm::pmt(rate, nper, pv, fv, due)
    }
}
```

---

## PHASE 9: EXAMPLES

Create these 4 example files in `examples/`:

### 9.1 `examples/tutorial_01_getting_started.rs`
```rust
use casifin_sdk::{Casifin, Money, Rate, Compounding, PaymentDue};
use rust_decimal::Decimal;

fn main() {
    let engine = Casifin::with_defaults();
    let rate = Rate::new(Decimal::new(425, 4), Compounding::Discrete(12)).unwrap();
    let pv = engine.pv(rate, 360, Money::from_decimal(Decimal::new(147582, 2)), Money::ZERO, PaymentDue::End).unwrap();
    println!("Present value: {}", pv);
}
```

### 9.2 `examples/mortgage_calculator.rs`
Full fixed vs ARM comparison with schedule printing.

### 9.3 `examples/investment_analysis.rs`
NPV/IRR for a 5-year project with uneven cash flows.

### 9.4 `examples/bond_pricing.rs`
Yield calculations using ratio functions.

**Each example must compile and run with `cargo run --example <name>`.**

---

## PHASE 10: INTEGRATION & DIFFERENTIAL TESTS

Create `tests/integration_tests.rs` and `tests/differential_tests.rs`.

### 10.1 Integration Tests
- End-to-end mortgage: build schedule, verify invariants
- Cross-crate consistency: tvm::pmt result must equal amortization base payment
- Full pipeline: create loan -> generate schedule -> compute IRR on cash flows -> should approximate rate

### 10.2 Differential Tests
Pre-compute reference values from Excel and hardcode them as constants:
```rust
const EXCEL_PV_ANNUITY: &str = "4329.476671";
const EXCEL_FV_ANNUITY: &str = "5525.631250";
const EXCEL_PMT_MORTGAGE: &str = "1475.817865";
```
Compare `casifin` output against these to 1e-9 precision using `approx` crate.

---

## PHASE 11: BENCHMARKS

Create `benches/amortization_bench.rs`:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use casifin_sdk::{Casifin, Money, Rate, Compounding};
use rust_decimal::Decimal;

fn amortization_30_year(c: &mut Criterion) {
    let engine = Casifin::with_defaults();
    let rate = Rate::new(Decimal::new(425, 4), Compounding::Discrete(12)).unwrap();
    c.bench_function("amortization_30yr", |b| {
        b.iter(|| {
            engine.mortgage(Money::from_decimal(Decimal::new(300000, 0)), rate, 360)
                .build()
                .unwrap()
        })
    });
}

criterion_group!(benches, amortization_30_year);
criterion_main!(benches);
```

---

---

## Design Philosophy (NASA/JPL Power of Ten Adapted)

Every module must obey these rules. Enforce via `clippy` lints and code review.

| Rule | Implementation in Rust |
|------|------------------------|
| **1. Simple control flow** | No `goto`, no `continue`, no `break` in nested loops. Max cyclomatic complexity 10. |
| **2. Fixed loop bounds** | All loops must have statically provable or explicitly bounded iteration counts. |
| **3. No dynamic allocation after init** | Core math uses stack-allocated `Decimal` (28-byte). Heap only for amortization schedules. |
| **4. No long functions** | Max 60 lines per function. Extract helpers aggressively. |
| **5. Minimum two assertions per function** | `debug_assert!` for preconditions, `assert!` for invariants. |
| **6. Data at smallest scope** | Pass by value (`Copy` types) or immutable reference. No `&mut` unless necessary. |
| **7. Check return values** | Every `Result` must be handled. Use `?` operator. No `.unwrap()` in library code. |
| **8. Limit preprocessor** | No `cfg` spaghetti. Feature flags are coarse-grained modules only. |
| **9. Limit pointers** | No raw pointers. Use references and `Decimal` by value. |
| **10. Compile with warnings-as-errors** | `#![deny(warnings)]` in `lib.rs`. |


**NASA-style requirements:**
- Every function must have `due: PaymentDue` enum (`Beginning`, `End`)
- Newton-Raphson solver for `rate()` must have explicit iteration cap and convergence check
- All `Decimal` operations use `checked_add`, `checked_div`, etc., mapping `None` to `CasifinError::DivisionByZero`

---

## NASA-Grade Code Patterns

### Pattern A: Defensive Arithmetic
```rust
/// # Safety
/// Uses `checked_div` to prevent division by zero. If the divisor is zero,
/// returns `CasifinError::DivisionByZero` with the operation name for diagnostics.
pub fn present_value(future_value: Money, rate: Decimal, periods: u32) -> Result<Money, CasifinError> {
    debug_assert!(rate >= Decimal::ZERO, "rate must be non-negative");
    debug_assert!(periods > 0, "periods must be positive");

    let denominator = (Decimal::ONE + rate)
        .checked_powi(periods as i64)
        .ok_or(CasifinError::ScheduleOverflow { 
            detail: "rate power overflow".to_string() 
        })?;

    let pv = future_value.0
        .checked_div(denominator)
        .ok_or(CasifinError::DivisionByZero { 
            operation: "present_value denominator" 
        })?;

    Ok(Money(pv))
}
```

### Pattern B: Builder with Invariant Checking
```rust
impl AmortizationBuilder {
    pub fn build(self) -> Result<AmortizationSchedule, CasifinError> {
        // Precondition checks
        if self.principal <= Money::ZERO {
            return Err(CasifinError::InvalidAmount(self.principal));
        }
        if self.term_months == 0 {
            return Err(CasifinError::InvalidPeriod(0));
        }

        let schedule = self.generate_schedule()?;

        // Postcondition invariant
        let final_balance = schedule.entries.last()
            .map(|e| e.balance)
            .unwrap_or(Money::ZERO);

        debug_assert!(
            final_balance.abs() <= self.config.eps,
            "amortization did not fully pay off: balance={}",
            final_balance
        );

        Ok(schedule)
    }
}
```

### Pattern C: Explicit Error Propagation
```rust
// BAD (never do this in library code):
// let result = some_decimal / other_decimal;
//
// GOOD:
let result = some_decimal
    .checked_div(other_decimal)
    .ok_or(CasifinError::DivisionByZero { operation: "interest calculation" })?;
```

---

## FINAL QUALITY GATES

Before declaring complete, run:
```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
cargo run --example tutorial_01_getting_started
cargo bench
```

All must pass without warnings or errors.
