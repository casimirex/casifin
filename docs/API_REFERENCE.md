# API Reference

## Main Entry Point

### Casifin

The unified API for all financial calculations.

```rust
pub struct Casifin {
    config: Config,
}

impl Casifin {
    pub fn new(config: Config) -> Self;
    pub fn with_default_config() -> Self;
    
    // Mortgage/Amortization
    pub fn mortgage(&self, principal: Money, rate: Rate, term_months: u32) 
        -> AmortizationBuilder;
    pub fn arm(&self, principal: Money, initial_rate: Rate, term_months: u32)
        -> AdjustableRateBuilder;
    
    // Cash Flow
    pub fn npv(&self, rate: Decimal, flows: &CashFlowStream) -> Result<Money, CasifinError>;
    pub fn irr(&self, flows: &CashFlowStream) -> Result<Decimal, CasifinError>;
    pub fn xnpv(&self, rate: Decimal, flows: &CashFlowStream) -> Result<Money, CasifinError>;
    pub fn xirr(&self, flows: &CashFlowStream) -> Result<Decimal, CasifinError>;
    
    // TVM
    pub fn pv(...) -> Result<Money, CasifinError>;
    pub fn fv(...) -> Result<Money, CasifinError>;
    pub fn pmt(...) -> Result<Money, CasifinError>;
    pub fn nper(...) -> Result<u32, CasifinError>;
    pub fn rate(...) -> Result<Decimal, CasifinError>;
}
```

## Core Types

### Money

```rust
pub struct Money(Decimal);

impl Money {
    pub const ZERO: Self;
    pub const ONE: Self;
    pub fn new(value: Decimal) -> Self;
    pub fn from_cents(cents: i64) -> Self;
    pub fn from<T: Into<Decimal>>(value: T) -> Self;
    pub fn inner(&self) -> Decimal;
    pub fn abs(&self) -> Self;
    pub fn checked_add(&self, other: Self) -> Option<Self>;
    pub fn checked_sub(&self, other: Self) -> Option<Self>;
    pub fn checked_mul(&self, other: Self) -> Option<Self>;
    pub fn checked_div(&self, other: Self) -> Option<Self>;
    pub fn checked_div_decimal(&self, other: Decimal) -> Option<Self>;
    pub fn is_zero(&self) -> bool;
    pub fn round_to_cents(&self) -> Self;
}
```

### Rate

```rust
pub struct Rate {
    pub annual_rate: Decimal,
    pub compounding: Compounding,
    pub convention: DayCount,
}

impl Rate {
    pub fn new(
        annual_rate: Decimal,
        compounding: Compounding,
        convention: DayCount,
    ) -> Result<Self, CasifinError>;
    
    pub fn periodic_rate(&self, periods_per_year: u32) -> Result<Decimal, CasifinError>;
    pub fn effective_annual_rate(&self) -> Result<Decimal, CasifinError>;
    pub fn rate_per_period(&self, payments_per_year: u32) -> Result<Decimal, CasifinError>;
}
```

### Compounding

```rust
pub enum Compounding {
    Discrete(u32),
    Continuous,
}

impl Compounding {
    pub const MONTHLY: Self;
    pub const QUARTERLY: Self;
    pub const SEMI_ANNUAL: Self;
    pub const ANNUAL: Self;
    pub const DAILY: Self;
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

### Config

```rust
pub struct Config {
    pub eps: Decimal,
    pub max_iterations: u32,
    pub guess: Decimal,
    pub business_days_only: bool,
    pub periodic_compound: bool,
}

impl Config {
    pub fn new() -> Self;
    pub fn builder() -> ConfigBuilder;
    pub fn with_eps(self, eps: Decimal) -> Self;
    pub fn with_max_iterations(self, max_iterations: u32) -> Self;
    pub fn with_guess(self, guess: Decimal) -> Self;
}
```

### CasifinError

```rust
pub enum CasifinError {
    InvalidRate(Decimal),
    InvalidPeriod(u32),
    IrrConvergenceFailure { max_iter: u32, eps: Decimal },
    DivisionByZero { operation: &'static str },
    ScheduleOverflow { detail: String },
    DateOutOfRange(String),
    InvalidAmount(Money),
    InvalidNper(i64),
    InvalidPayment(Decimal),
    EmptyCashFlowStream,
    XirrSignRequirement,
    // ... and more
}
```

## Time Value of Money (casifin_tvm)

```rust
pub fn pv(
    rate: Decimal,
    nper: u32,
    pmt: Money,
    fv: Money,
    due: PaymentDue,
) -> Result<Money, CasifinError>;

pub fn fv(
    rate: Decimal,
    nper: u32,
    pmt: Money,
    pv: Money,
    due: PaymentDue,
) -> Result<Money, CasifinError>;

pub fn pmt(
    rate: Decimal,
    nper: u32,
    pv: Money,
    fv: Money,
    due: PaymentDue,
) -> Result<Money, CasifinError>;

pub fn nper(
    rate: Decimal,
    pmt: Money,
    pv: Money,
    fv: Money,
    due: PaymentDue,
) -> Result<u32, CasifinError>;

pub fn rate(
    nper: u32,
    pmt: Money,
    pv: Money,
    fv: Money,
    due: PaymentDue,
    guess: Decimal,
    max_iter: u32,
    eps: Decimal,
) -> Result<Decimal, CasifinError>;

pub fn pv_perpetuity(
    rate: Decimal,
    pmt: Money,
) -> Result<Money, CasifinError>;

pub fn fv_uneven_cashflows(
    rate: Decimal,
    flows: &[Money],
) -> Result<Money, CasifinError>;
```

## Cash Flow Analysis (casifin_cashflow)

```rust
pub struct CashFlow {
    pub amount: Money,
    pub date: Option<NaiveDate>,
}

pub struct CashFlowStream(Vec<CashFlow>);

impl CashFlowStream {
    pub fn new(flows: Vec<CashFlow>) -> Self;
    pub fn from_vec(flows: Vec<Money>) -> Self;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn get(&self, index: usize) -> Option<&CashFlow>;
    pub fn iter(&self) -> impl Iterator<Item = &CashFlow>;
    pub fn has_mixed_signs(&self) -> bool;
}

pub fn npv(
    rate: Decimal,
    stream: &CashFlowStream,
) -> Result<Money, CasifinError>;

pub fn irr(
    stream: &CashFlowStream,
    guess: Decimal,
    max_iter: u32,
    eps: Decimal,
) -> Result<Decimal, CasifinError>;

pub fn xnpv(
    rate: Decimal,
    stream: &CashFlowStream,
) -> Result<Money, CasifinError>;

pub fn xirr(
    stream: &CashFlowStream,
    guess: Decimal,
    max_iter: u32,
    eps: Decimal,
) -> Result<Decimal, CasifinError>;
```

## Amortization (casifin_amortization)

```rust
pub struct AmortizationEntry {
    pub period: u32,
    pub payment: Money,
    pub principal: Money,
    pub interest: Money,
    pub balance: Money,
    pub date: Option<NaiveDate>,
}

pub struct AmortizationSchedule {
    pub entries: Vec<AmortizationEntry>,
    pub total_payments: Money,
    pub total_interest: Money,
    pub total_principal: Money,
}

pub struct AmortizationBuilder {
    pub fn new(principal: Money, rate: Rate, term_months: u32) -> Self;
    pub fn with_payment_modifier<F>(self, modifier: F) -> Self
        where F: Fn(u32, Money) -> Money + 'static;
    pub fn with_30_360(self) -> Self;
    pub fn with_start_date(self, date: NaiveDate) -> Self;
    pub fn build(self) -> Result<AmortizationSchedule, CasifinError>;
}

pub struct RateCaps {
    pub periodic_cap: Decimal,
    pub lifetime_cap: Decimal,
    pub initial_cap: Option<Decimal>,
}

pub struct AdjustableRateSchedule {
    pub initial_rate: Rate,
    pub adjustments: Vec<(u32, Rate)>,
    pub caps: Option<RateCaps>,
    pub schedule: AmortizationSchedule,
}
```

## Financial Ratios (casifin_ratios)

### Liquidity
```rust
pub fn current_ratio(
    current_assets: Money,
    current_liabilities: Money,
) -> Result<Decimal, CasifinError>;

pub fn quick_ratio(
    cash: Money,
    marketable_securities: Money,
    receivables: Money,
    current_liabilities: Money,
) -> Result<Decimal, CasifinError>;

pub fn cash_ratio(
    cash: Money,
    current_liabilities: Money,
) -> Result<Decimal, CasifinError>;
```

### Solvency
```rust
pub fn debt_ratio(...) -> Result<Decimal, CasifinError>;
pub fn debt_to_equity(...) -> Result<Decimal, CasifinError>;
pub fn financial_leverage(...) -> Result<Decimal, CasifinError>;
```

### Profitability
```rust
pub fn gross_profit_margin(...) -> Result<Decimal, CasifinError>;
pub fn net_profit_margin(...) -> Result<Decimal, CasifinError>;
pub fn basic_eps(...) -> Result<Decimal, CasifinError>;
pub fn diluted_eps(...) -> Result<Decimal, CasifinError>;
```

### Returns
```rust
pub fn holding_period_return(...) -> Result<Decimal, CasifinError>;
pub fn geometric_mean_return(...) -> Result<Decimal, CasifinError>;
pub fn time_weighted_rate_of_return(...) -> Result<Decimal, CasifinError>;
pub fn sharpe_ratio(...) -> Result<Decimal, CasifinError>;
pub fn roys_safety_first_ratio(...) -> Result<Decimal, CasifinError>;
```

### Yields
```rust
pub fn bank_discount_yield(...) -> Result<Decimal, CasifinError>;
pub fn money_market_yield(...) -> Result<Decimal, CasifinError>;
pub fn bond_equivalent_yield(...) -> Result<Decimal, CasifinError>;
```

### Rates
```rust
pub fn effective_annual_rate(...) -> Result<Decimal, CasifinError>;
pub fn stated_from_effective(...) -> Result<Decimal, CasifinError>;
pub fn nominal_to_continuous(...) -> Result<Decimal, CasifinError>;
pub fn continuous_to_nominal(...) -> Result<Decimal, CasifinError>;
```

### Statistics
```rust
pub fn coefficient_of_variance(...) -> Result<Decimal, CasifinError>;
pub fn weighted_mean(...) -> Result<Decimal, CasifinError>;
pub fn harmonic_mean(...) -> Result<Decimal, CasifinError>;
pub fn sampling_error(...) -> Result<Decimal, CasifinError>;
```

## Depreciation (casifin_depreciation)

```rust
pub trait DepreciationMethod {
    fn depreciate(
        &self,
        cost: Money,
        salvage: Money,
        life_years: u32,
        period: u32,
    ) -> Result<Money, CasifinError>;
    
    fn schedule(
        &self,
        cost: Money,
        salvage: Money,
        life_years: u32,
    ) -> Result<Vec<Money>, CasifinError>;
}

pub struct StraightLine;
pub struct DoubleDecliningBalance;
```

## Inventory (casifin_inventory)

```rust
pub struct InventoryLot {
    pub units: u32,
    pub unit_cost: Money,
    pub date: NaiveDate,
}

pub trait InventoryMethod {
    fn cogs(
        &self,
        lots: &[InventoryLot],
        units_sold: u32,
    ) -> Result<Money, CasifinError>;
    
    fn ending_inventory(
        &self,
        lots: &[InventoryLot],
        units_sold: u32,
    ) -> Result<Money, CasifinError>;
}

pub struct Fifo;
pub struct Lifo;
pub struct WeightedAverage;
```
