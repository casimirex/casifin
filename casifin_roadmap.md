# **ROADMAP: `casifin` — Financial Computation Engine**
## *A Rust reimplementation of `finrb` with NASA-grade engineering standards*

---

## 1. Executive Summary

**Goal:** Rebuild the entire `finrb` feature set as a Rust library crate (`casifin`) plus an SDK layer, following NASA JPL's Power of Ten rules for safety-critical code, with exhaustive documentation, property-based testing, and tutorial-driven examples.

**Scope Coverage:**
- Arbitrary-precision decimal arithmetic (financial-grade, no `f64` in core)
- Time Value of Money (TVM): PV, FV, PMT, NPER, RATE
- Cash Flow Analysis: NPV, IRR, XIRR, XNPV
- Amortization Engines: Fixed-rate, Adjustable-rate (ARM), with payment modification hooks
- Interest Rate Objects: APR, APY, continuous, nominal, effective conversions
- Financial Ratio & Metric Utilities (ported from R FinCal)
- Depreciation: Straight-line, Double-declining balance
- Inventory Methods: FIFO, LIFO, Weighted Average
- Portfolio & Risk Metrics: Sharpe, TWRR, geometric mean, etc.
- Transaction-level modeling with business-day awareness

---

## 2. Design Philosophy (NASA/JPL Power of Ten Adapted)

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

---

## 3. Workspace Architecture

```text
casifin/
├── Cargo.toml                 # Workspace root
├── crates/
│   ├── casifin-core/          # Decimal math, errors, traits, primitives
│   ├── casifin-tvm/           # Time Value of Money
│   ├── casifin-cashflow/      # NPV, IRR, XIRR, XNPV
│   ├── casifin-amortization/  # Fixed & ARM amortization engines
│   ├── casifin-ratios/        # Financial ratios & metrics (FinCal port)
│   ├── casifin-depreciation/  # SL, DDB
│   ├── casifin-inventory/     # FIFO, LIFO, Weighted Avg
│   └── casifin-sdk/           # Unified API, config, builder patterns
├── examples/
│   ├── mortgage_calculator.rs
│   ├── investment_analysis.rs
│   ├── bond_pricing.rs
│   └── tutorial_01_getting_started.rs
├── benches/
│   ├── amortization_bench.rs
│   └── cashflow_bench.rs
├── tests/
│   ├── integration_tests.rs
│   └── property_tests.rs
└── docs/
    ├── ARCHITECTURE.md
    ├── API_REFERENCE.md
    ├── TUTORIALS.md
    └── CONTRIBUTING.md
```

---

## 4. Crate Specifications

### 4.1 `casifin-core` — Foundation Layer

**Responsibilities:**
- Define the `Money` newtype wrapper around `rust_decimal::Decimal`
- Define the `Rate` primitive (value, compounding frequency, convention)
- Define the global `Error` enum using `thiserror`
- Define shared traits: `FinancialCalculation`, `Schedulable`, `CashFlowStream`
- Configuration struct with builder pattern

**Key Types:**
```rust
/// A monetary value with guaranteed precision.
/// Invariant: always stores exactly 2 decimal places for display,
/// but calculations retain full precision until final rounding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Money(Decimal);

/// An interest rate with compounding metadata.
/// Invariant: rate >= 0, frequency >= 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rate {
    pub annual_rate: Decimal,        // e.g., 0.0425 for 4.25%
    pub compounding: Compounding,    // Monthly, Quarterly, Continuous, etc.
    pub convention: DayCount,        // Actual/365, 30/360, etc.
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compounding {
    Discrete(u32),   // n times per year
    Continuous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayCount {
    Actual365,
    Actual360,
    Thirty360,
    ThirtyE360,
}
```

**Error Design:**
```rust
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CasifinError {
    #[error("invalid rate: {0}. Rate must be non-negative.")]
    InvalidRate(Decimal),

    #[error("invalid period: {0}. Period must be positive.")]
    InvalidPeriod(u32),

    #[error("IRR did not converge within {max_iter} iterations (eps={eps})")]
    IrrConvergenceFailure { max_iter: u32, eps: Decimal },

    #[error("division by zero in {operation}")]
    DivisionByZero { operation: &'static str },

    #[error("amortization schedule overflow: {detail}")]
    ScheduleOverflow { detail: String },

    #[error("date out of range: {0}")]
    DateOutOfRange(String),
}
```

**Configuration:**
```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub eps: Decimal,              // Default: 1e-12
    pub max_iterations: u32,       // Default: 1000
    pub guess: Decimal,            // Default: 0.1
    pub business_days_only: bool,
    pub periodic_compound: bool,
}

impl Default for Config { ... }
impl Config { pub fn builder() -> ConfigBuilder { ... } }
```

---

### 4.2 `casifin-tvm` — Time Value of Money

**Functions (all return `Result<Money, CasifinError>` or `Result<Decimal, CasifinError>`):**

| Function | Formula | Validation |
|----------|---------|------------|
| `pv(rate, nper, pmt, fv, due)` | PV of annuity | rate >= 0, nper > 0 |
| `fv(rate, nper, pmt, pv, due)` | FV of annuity | rate >= 0, nper > 0 |
| `pmt(rate, nper, pv, fv, due)` | Payment | rate >= 0, nper > 0, pv != 0 |
| `nper(rate, pmt, pv, fv, due)` | Number of periods | rate >= 0, pmt != 0 |
| `rate(nper, pmt, pv, fv, due, guess)` | Rate per period | Newton-Raphson iteration, bounded |
| `pv_perpetuity(rate, pmt)` | PV = pmt / rate | rate > 0 |
| `fv_uneven_cashflows(rate, flows)` | Sigma CF_t / (1+r)^t | rate > -1 |

**NASA-style requirements:**
- Every function must have `due: PaymentDue` enum (`Beginning`, `End`)
- Newton-Raphson solver for `rate()` must have explicit iteration cap and convergence check
- All `Decimal` operations use `checked_add`, `checked_div`, etc., mapping `None` to `CasifinError::DivisionByZero`

---

### 4.3 `casifin-cashflow` — Cash Flow Analysis

**Core Types:**
```rust
#[derive(Debug, Clone)]
pub struct CashFlow {
    pub amount: Money,
    pub date: Option<NaiveDate>,   // None = implicit period index
}

pub struct CashFlowStream(Vec<CashFlow>);
```

**Functions:**

| Function | Description | Algorithm |
|----------|-------------|-----------|
| `npv(rate, stream)` | Net Present Value | Direct summation |
| `irr(stream, guess, config)` | Internal Rate of Return | Newton-Raphson / bisection hybrid |
| `xnpv(rate, stream)` | NPV with dates | Actual day count discounting |
| `xirr(stream, guess, config)` | IRR with dates | Newton-Raphson with date-weighted derivatives |

**Convergence Requirements:**
- IRR solver must fallback from Newton-Raphson to bisection if derivative approaches zero
- Must return `Err(CasifinError::IrrConvergenceFailure)` with diagnostics after `max_iterations`
- XIRR requires at least one positive and one negative cash flow (precondition assert)

---

### 4.4 `casifin-amortization` — Loan Amortization Engine

**Core Types:**
```rust
#[derive(Debug, Clone)]
pub struct AmortizationEntry {
    pub period: u32,
    pub payment: Money,
    pub principal: Money,
    pub interest: Money,
    pub balance: Money,
}

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
    payment_modifier: Option<Box<dyn Fn(u32, Money) -> Money>>,
}
```

**Builders:**
```rust
let schedule = AmortizationBuilder::new(principal, rate, 360)
    .with_payment_modifier(|period, base_payment| base_payment - Money::from(150))
    .build()?;
```

**ARM Support:**
```rust
pub struct AdjustableRateSchedule {
    pub initial_rate: Rate,
    pub adjustment_periods: Vec<(u32, Rate)>,  // (start_period, new_rate)
    pub caps: Option<RateCaps>,                  // periodic, lifetime
}
```

**Invariants (enforced at build time):**
- `principal > 0`
- `rate.annual_rate >= 0`
- `term_months > 0`
- Final balance must equal zero within `config.eps`
- Each period: `payment == principal_payment + interest_payment` within `config.eps`

---

### 4.5 `casifin-ratios` — Financial Metrics (FinCal Port)

Organize into modules by category:

```rust
pub mod liquidity {
    pub fn current_ratio(current_assets: Money, current_liabilities: Money) -> Result<Decimal, ...>;
    pub fn quick_ratio(cash: Money, marketable_securities: Money, receivables: Money, current_liabilities: Money) -> Result<Decimal, ...>;
    pub fn cash_ratio(cash: Money, current_liabilities: Money) -> Result<Decimal, ...>;
}

pub mod solvency {
    pub fn debt_ratio(total_debt: Money, total_assets: Money) -> Result<Decimal, ...>;
    pub fn debt_to_equity(total_debt: Money, total_equity: Money) -> Result<Decimal, ...>;
    pub fn financial_leverage(total_assets: Money, total_equity: Money) -> Result<Decimal, ...>;
}

pub mod profitability {
    pub fn gross_profit_margin(gross_profit: Money, revenue: Money) -> Result<Decimal, ...>;
    pub fn net_profit_margin(net_income: Money, revenue: Money) -> Result<Decimal, ...>;
    pub fn basic_eps(net_income: Money, preferred_dividends: Money, shares: Decimal) -> Result<Decimal, ...>;
    pub fn diluted_eps(net_income: Money, preferred_dividends: Money, weighted_shares: Decimal, potential_shares: Decimal) -> Result<Decimal, ...>;
}

pub mod returns {
    pub fn holding_period_return(purchase_price: Money, sale_price: Money, dividends: Money) -> Result<Decimal, ...>;
    pub fn geometric_mean_return(returns: &[Decimal]) -> Result<Decimal, ...>;
    pub fn money_weighted_return(cash_flows: CashFlowStream) -> Result<Decimal, ...>;
    pub fn time_weighted_rate_of_return(period_returns: &[Decimal]) -> Result<Decimal, ...>;
    pub fn sharpe_ratio(portfolio_return: Decimal, risk_free_rate: Decimal, std_dev: Decimal) -> Result<Decimal, ...>;
    pub fn roys_safety_first_ratio(expected_return: Decimal, threshold: Decimal, std_dev: Decimal) -> Result<Decimal, ...>;
}

pub mod yields {
    pub fn bank_discount_yield(face_value: Money, purchase_price: Money, days_to_maturity: u32) -> Result<Decimal, ...>;
    pub fn money_market_yield(face_value: Money, purchase_price: Money, days_to_maturity: u32) -> Result<Decimal, ...>;
    pub fn bond_equivalent_yield(semi_annual_yield: Decimal) -> Result<Decimal, ...>;
}

pub mod rates {
    pub fn effective_annual_rate(stated_rate: Decimal, compounding: Compounding) -> Result<Decimal, ...>;
    pub fn stated_from_effective(effective_rate: Decimal, compounding: Compounding) -> Result<Decimal, ...>;
    pub fn continuous_to_nominal(continuous_rate: Decimal) -> Result<Decimal, ...>;
    pub fn nominal_to_continuous(nominal_rate: Decimal) -> Result<Decimal, ...>;
    pub fn holding_period_to_effective(hpr: Decimal, periods_per_year: u32) -> Result<Decimal, ...>;
    pub fn equivalent_rate(rate: Rate, target_compounding: Compounding) -> Result<Rate, ...>;
}

pub mod statistics {
    pub fn coefficient_of_variance(mean: Decimal, std_dev: Decimal) -> Result<Decimal, ...>;
    pub fn weighted_mean(values: &[Decimal], weights: &[Decimal]) -> Result<Decimal, ...>;
    pub fn harmonic_mean(values: &[Decimal]) -> Result<Decimal, ...>;
    pub fn sampling_error(population_std_dev: Decimal, sample_size: u32) -> Result<Decimal, ...>;
}
```

---

### 4.6 `casifin-depreciation` — Asset Depreciation

```rust
pub trait DepreciationMethod {
    fn depreciate(&self, cost: Money, salvage: Money, life_years: u32, period: u32) -> Result<Money, CasifinError>;
    fn schedule(&self, cost: Money, salvage: Money, life_years: u32) -> Result<Vec<Money>, CasifinError>;
}

pub struct StraightLine;
pub struct DoubleDecliningBalance;
// Future: SumOfYearsDigits, UnitsOfProduction
```

**Invariants:**
- `cost > salvage`
- `life_years > 0`
- `period <= life_years`
- DDB switches to SL when SL produces higher depreciation (auto-conversion)

---

### 4.7 `casifin-inventory` — Inventory Costing

```rust
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
```

---

### 4.8 `casifin-sdk` — Unified Developer Interface

This crate re-exports all sub-crates with a unified, ergonomic API.

```rust
// casifin-sdk/src/lib.rs
pub use casifin_core::{Money, Rate, Config, Compounding, DayCount, CasifinError};
pub use casifin_tvm as tvm;
pub use casifin_cashflow as cashflow;
pub use casifin_amortization as amortization;
pub use casifin_ratios as ratios;
pub use casifin_depreciation as depreciation;
pub use casifin_inventory as inventory;

/// The main entry point for SDK consumers.
pub struct Casifin {
    config: Config,
}

impl Casifin {
    pub fn new(config: Config) -> Self;
    pub fn with_default_config() -> Self;

    // Convenience methods that delegate to sub-crates
    pub fn mortgage(&self, principal: Money, rate: Rate, months: u32) -> amortization::AmortizationBuilder;
    pub fn npv(&self, rate: Rate, flows: cashflow::CashFlowStream) -> Result<Money, CasifinError>;
    pub fn irr(&self, flows: cashflow::CashFlowStream) -> Result<Decimal, CasifinError>;
}
```

---

## 5. Phase-by-Phase Implementation Roadmap

### **Phase 0: Bootstrap & Tooling** (Day 1)
1. `cargo new --lib casifin` and create workspace `Cargo.toml`
2. Create all crate directories under `crates/`
3. Add shared dependencies to workspace `Cargo.toml`:
   ```toml
   [workspace.dependencies]
   rust_decimal = { version = "1.35", features = ["maths"] }
   chrono = "1.0"
   thiserror = "1.0"
   serde = { version = "1.0", features = ["derive"] }
   ```
4. Set up CI: GitHub Actions with `cargo clippy -- -D warnings`, `cargo test`, `cargo fmt --check`
5. Add `rustfmt.toml`:
   ```toml
   max_width = 100
   fn_single_line = false
   format_strings = true
   ```
6. Add `.clippy.toml`:
   ```toml
   cognitive-complexity-threshold = 10
   too-many-arguments-threshold = 5
   ```

### **Phase 1: Core Foundation** (Days 2–4)
**Deliverable:** `casifin-core` compiles, all types have full doc comments, unit tests pass.

- Implement `Money` newtype with `From<Decimal>`, `Display`, arithmetic ops
- Implement `Rate`, `Compounding`, `DayCount` with validation
- Implement `Config` and `ConfigBuilder`
- Implement `CasifinError` with `thiserror`
- Write unit tests for every validation path (negative rate, zero period, etc.)
- **Target:** 100% branch coverage on `casifin-core`

### **Phase 2: TVM Engine** (Days 5–7)
**Deliverable:** All TVM functions pass against Excel/HP-12C reference values.

- Implement `pv`, `fv`, `pmt`, `nper`, `rate`
- Implement `pv_perpetuity`, `fv_uneven_cashflows`
- Create reference test suite with known values (e.g., PV of $1000 at 5% for 10 years = $613.91)
- Implement Newton-Raphson solver for `rate()` with bisection fallback
- **Benchmark:** Compare against `rust_decimal` raw operations

### **Phase 3: Cash Flow Analysis** (Days 8–10)
**Deliverable:** NPV/IRR/XIRR match Ruby `finrb` output to 12 decimal places.

- Implement `npv` with `CashFlowStream`
- Implement `irr` with hybrid Newton/bisection
- Implement `xnpv` and `xirr` with `chrono` date handling
- Test against Ruby `finrb` reference data (extract from gem test suite)
- Add property-based tests with `proptest`: NPV at 0% rate = sum of cash flows

### **Phase 4: Amortization Engine** (Days 11–14)
**Deliverable:** Fixed-rate and ARM schedules with payment modification hooks.

- Implement `AmortizationBuilder` with fluent API
- Implement fixed-rate schedule generation (30/360, Actual/365)
- Implement ARM with rate adjustment vectors
- Implement payment modifier closure support
- Verify invariants: balance reaches zero, total principal = original principal
- Add schedule serialization with `serde`

### **Phase 5: Ratios & Metrics** (Days 15–18)
**Deliverable:** All FinCal-ported functions with documentation linking to CFA curriculum.

- Port all liquidity, solvency, profitability functions
- Port all return, yield, rate conversion functions
- Port all statistical utility functions
- Each function gets a doc comment with:
  - Formula in LaTeX
  - CFA/CPA reference
  - Example usage
  - Edge case behavior

### **Phase 6: Depreciation & Inventory** (Days 19–20)
**Deliverable:** SL, DDB, FIFO, LIFO, Weighted Average.

- Implement trait-based depreciation
- Implement trait-based inventory costing
- DDB auto-switch to SL verification
- Inventory lot tracking with dates

### **Phase 7: SDK & Ergonomics** (Days 21–22)
**Deliverable:** Single `Casifin` struct as entry point.

- Re-export all crates
- Implement convenience methods
- Add `serde` support for all public types
- Create `examples/` directory with 4 complete tutorials

### **Phase 8: Documentation & Tutorials** (Days 23–25)
**Deliverable:** `docs/` folder with architecture docs and 4 tutorials.

- `ARCHITECTURE.md`: Module diagram, data flow, error handling strategy
- `API_REFERENCE.md`: Auto-generated via `cargo doc`, plus manual supplement
- `TUTORIALS.md`:
  1. Getting Started: Your first mortgage calculation
  2. Investment Analysis: NPV/IRR for a real estate deal
  3. ARM Modeling: Adjustable rate loan with caps
  4. Portfolio Metrics: Sharpe ratio and TWRR

### **Phase 9: Hardening** (Days 26–28)
**Deliverable:** Production-ready crate.

- Property-based testing with `proptest` across all numeric functions
- Fuzz testing for amortization inputs
- Benchmarks with `criterion.rs`
- Audit: `cargo audit`, `cargo geiger` (unsafe code scan)
- Publish dry-run: `cargo publish --dry-run` for each crate

---

## 6. NASA-Grade Code Patterns

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

## 7. Testing Strategy

### Unit Tests
Every function gets:
1. Happy path with known reference value
2. Zero boundary
3. Negative input rejection
4. Overflow/underflow handling

### Integration Tests
In `tests/integration_tests.rs`:
- End-to-end mortgage calculation vs. Excel PMT/PMT/PPMT
- ARM schedule vs. known reference
- Full NPV/IRR pipeline

### Property Tests (proptest)
```rust
proptest! {
    #[test]
    fn npv_at_zero_rate_equals_sum_of_flows(flows in vec(any::<i64>(), 1..100)) {
        let money_flows: Vec<Money> = flows.into_iter().map(Money::from_cents).collect();
        let stream = CashFlowStream::from_vec(money_flows);
        let npv = xnpv(Decimal::ZERO, stream).unwrap();
        let sum: Money = money_flows.iter().copied().sum();
        prop_assert!(npv.almost_eq(sum, eps()));
    }
}
```

### Fuzz Tests
```rust
// fuzz/fuzz_targets/amortization.rs
fuzz_target!(|data: &[u8]| {
    if let Ok((principal, rate, months)) = parse_fuzz_input(data) {
        let _ = AmortizationBuilder::new(principal, rate, months).build();
    }
});
```

---

## 8. Documentation Standards

Every public item must follow this template:

```rust
/// Computes the present value of an ordinary annuity.
///
/// # Formula
///
/// ```text
///         PMT * (1 - (1 + r)^-n)
/// PV = ---------------------------
///                r
/// ```
///
/// Where:
/// - `PMT` = periodic payment
/// - `r` = interest rate per period
/// - `n` = total number of periods
///
/// # Arguments
///
/// * `rate` - The interest rate per period as a decimal (e.g., 0.05 for 5%)
/// * `nper` - The total number of payment periods
/// * `pmt`  - The payment made each period
/// * `fv`   - The future value (cash balance after last payment)
/// * `due`  - Whether payments are due at the beginning or end of each period
///
/// # Returns
///
/// Returns `Ok(Money)` containing the present value, or `Err(CasifinError)` if:
/// - `rate` is negative
/// - `nper` is zero
/// - Division by zero occurs (rate = 0 and pmt = 0)
///
/// # Example
///
/// ```
/// use casifin_sdk::{tvm, Money, Rate, Compounding, PaymentDue};
///
/// let rate = Rate::new(Decimal::new(5, 2), Compounding::Monthly).unwrap();
/// let pv = tvm::pv(rate, 360, Money::from(1200), Money::ZERO, PaymentDue::End).unwrap();
/// assert_eq!(pv, Money::from("223538.51"));
/// ```
///
/// # References
///
/// - CFA Institute, *Quantitative Methods*, Reading 6
/// - Ruby `finrb` gem, `Finance::TVM.pv`
///
/// # Panics
///
/// This function does not panic. All error conditions return `Result::Err`.
pub fn pv(...) -> Result<Money, CasifinError> { ... }
```

---

## 9. Claude Code Implementation Prompts

Save these as `.claude/prompts/` or feed them sequentially to Claude Code:

### Prompt 1: Bootstrap
```
Create a Rust workspace named `casifin` with 8 crates under `crates/`.
Set up rustfmt.toml with max_width=100, clippy.toml with cognitive-complexity-threshold=10.
Add workspace dependencies: rust_decimal, chrono, thiserror, serde.
Create GitHub Actions CI that runs clippy -- -D warnings, cargo test, and cargo fmt --check.
Do not write any business logic yet.
```

### Prompt 2: Core Types
```
In `crates/casifin-core/src/lib.rs`, implement:
1. `Money` newtype wrapping `Decimal` with `From`, `Display`, `Add`, `Sub`, `Mul`, `Div`
2. `Rate` struct with `annual_rate: Decimal`, `compounding: Compounding`, `convention: DayCount`
3. `Compounding` enum: `Discrete(u32)`, `Continuous`
4. `DayCount` enum: `Actual365`, `Actual360`, `Thirty360`
5. `CasifinError` enum with `thiserror` covering: InvalidRate, InvalidPeriod, DivisionByZero, IrrConvergenceFailure, ScheduleOverflow
6. `Config` struct with `eps`, `max_iterations`, `guess`, and a `ConfigBuilder`
All types must derive `Debug, Clone`. Money must derive `Copy`.
Every public function needs full doc comments with formula, arguments, returns, example, and error conditions.
Write unit tests for every validation path.
```

### Prompt 3: TVM Functions
```
In `crates/casifin-tvm/src/lib.rs`, implement:
pv, fv, pmt, nper, rate, pv_perpetuity, fv_uneven_cashflows
Use only `checked_add`, `checked_div`, etc. from rust_decimal.
`rate()` must use a hybrid Newton-Raphson / bisection solver with max 1000 iterations.
Return CasifinError on any arithmetic failure or non-convergence.
Add integration tests comparing against Excel reference values to 12 decimal places.
```

### Prompt 4: Cash Flow
```
In `crates/casifin-cashflow/src/lib.rs`, implement:
npv, irr, xnpv, xirr
Use `CashFlow` struct with `amount: Money` and `date: Option<NaiveDate>`.
XIRR must use actual day count for discounting.
IRR must fallback to bisection if Newton derivative approaches zero.
Test against Ruby finrb reference data (I will provide sample data).
```

### Prompt 5: Amortization
```
In `crates/casifin-amortization/src/lib.rs`, implement:
AmortizationBuilder with fluent API for fixed-rate loans.
Support 30/360 and Actual/365 day count conventions.
Support payment modifiers via `Fn(u32, Money) -> Money` closure.
Generate AmortizationSchedule with entries for each period.
Verify invariant: final balance == 0 within eps.
Add ARM support with rate adjustment vectors.
```

### Prompt 6: Ratios & Metrics
```
In `crates/casifin-ratios/src/`, create modules: liquidity, solvency, profitability, returns, yields, rates, statistics.
Port every function from the Ruby finrb Utils class.
Each function must include the formula in doc comments and a CFA/CPA reference.
Use only checked arithmetic.
```

### Prompt 7: Depreciation & Inventory
```
In `crates/casifin-depreciation/src/lib.rs`, implement StraightLine and DoubleDecliningBalance as traits.
In `crates/casifin-inventory/src/lib.rs`, implement Fifo, Lifo, WeightedAverage as traits.
DDB must auto-switch to SL when SL produces higher depreciation.
```

### Prompt 8: SDK & Examples
```
In `crates/casifin-sdk/src/lib.rs`, create a unified `Casifin` struct that re-exports all sub-crates.
Create 4 examples in `examples/`:
1. mortgage_calculator.rs - Fixed vs ARM comparison
2. investment_analysis.rs - NPV/IRR for a 5-year project
3. bond_pricing.rs - Yield calculations
4. tutorial_01_getting_started.rs - Basic TVM
Each example must be a complete, compilable program with comments.
```

---

## 10. Dependency Manifest

```toml
# Workspace Cargo.toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.dependencies]
rust_decimal = { version = "1.35", features = ["maths", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }

# Dev dependencies
criterion = "0.5"
proptest = "1.4"
approx = "0.5"
```

---

## 11. Quality Gates (Definition of Done)

Before any PR merges:
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes with 100% of unit tests green
- [ ] `cargo fmt --check` passes
- [ ] Every public item has doc comments with formula, example, and error conditions
- [ ] No `.unwrap()` or `.expect()` in library code (only in tests/examples)
- [ ] `cargo audit` reports zero vulnerabilities
- [ ] Property tests added for any function with numeric inputs
- [ ] Benchmark added for any O(n) or iterative algorithm

---

This roadmap gives Claude Code (or any developer) a complete, unambiguous specification to build `casifin` from an empty directory to a production-grade financial computation SDK. Each phase is independently verifiable, and the NASA-style constraints ensure the code remains maintainable, correct, and auditable.
