//! Unified SDK interface for the casifin financial computation engine.
//!
//! This crate re-exports all sub-crates and provides a single entry point
//! for consumers of the casifin library.

#![deny(warnings)]

// Re-export all sub-crates
pub use casifin_amortization as amortization;
pub use casifin_amortization::{AmortizationBuilder, AmortizationEntry, AmortizationSchedule};
pub use casifin_cashflow as cashflow;
pub use casifin_cashflow::{irr, npv, xirr, xnpv, CashFlow, CashFlowStream};
pub use casifin_core::{
    CasifinError, Compounding, Config, ConfigBuilder, DayCount, FinancialCalculation, Money,
    PaymentDue, Rate, Schedulable,
};
pub use casifin_depreciation as depreciation;
pub use casifin_depreciation::{DepreciationMethod, DoubleDecliningBalance, StraightLine};
pub use casifin_inventory as inventory;
pub use casifin_inventory::{Fifo, InventoryLot, InventoryMethod, Lifo, WeightedAverage};
pub use casifin_ratios as ratios;
pub use casifin_ratios::*;
pub use casifin_tvm as tvm;
pub use casifin_tvm::{fv, fv_uneven_cashflows, nper, pmt, pv, pv_perpetuity, rate};
use rust_decimal::Decimal;

/// The main entry point for casifin consumers.
///
/// This struct provides a unified API for all financial calculations.
///
/// # Example
/// ```
/// use casifin_sdk::{Casifin, Money, Rate, Compounding, DayCount};
/// use rust_decimal::Decimal;
///
/// let casifin = Casifin::with_defaults();
///
/// // Calculate a mortgage payment
/// let principal = Money::from(200000);
/// let rate = Rate::new(Decimal::new(6, 2), Compounding::Discrete(12))
///     .unwrap()
///     .with_convention(DayCount::Actual365);
/// let schedule = casifin.mortgage(principal, rate, 360).build().unwrap();
/// ```
pub struct Casifin {
    config: Config,
}

impl Casifin {
    /// Creates a new `Casifin` with the specified configuration.
    pub fn new(config: Config) -> Self {
        Casifin { config }
    }

    /// Creates a `Casifin` with default configuration.
    pub fn with_defaults() -> Self {
        Casifin::new(Config::default())
    }

    /// Returns the configuration.
    pub fn config(&self) -> Config {
        self.config
    }

    /// Creates an amortization builder for a fixed-rate loan.
    ///
    /// # Arguments
    /// * `principal` - The loan amount
    /// * `rate` - The annual interest rate
    /// * `term_months` - The loan term in months
    pub fn mortgage(
        &self,
        principal: Money,
        rate: Rate,
        term_months: u32,
    ) -> amortization::AmortizationBuilder {
        amortization::AmortizationBuilder::new(principal, rate, term_months)
            .with_config(self.config)
    }

    /// Computes the Net Present Value of a cash flow stream.
    ///
    /// # Formula
    /// ```text
    /// NPV = Σ CF_t / (1 + r)^t
    /// ```
    ///
    /// # Arguments
    /// * `rate` - The discount rate per period
    /// * `flows` - The cash flow stream
    ///
    /// # Returns
    /// `Ok(Money)` containing the NPV, or `Err(CasifinError)` on invalid input.
    pub fn npv(&self, rate: Decimal, flows: &CashFlowStream) -> Result<Money, CasifinError> {
        npv(rate, flows)
    }

    /// Computes the Internal Rate of Return using the configured solver settings.
    ///
    /// Uses a hybrid Newton-Raphson / bisection solver.
    ///
    /// # Arguments
    /// * `flows` - The cash flow stream (must have mixed signs)
    ///
    /// # Returns
    /// `Ok(Decimal)` containing the IRR, or `Err(CasifinError)` if the solver
    /// does not converge.
    pub fn irr(&self, flows: &CashFlowStream) -> Result<Decimal, CasifinError> {
        irr(flows, self.config)
    }

    /// Computes the Net Present Value with actual dates (XNPV).
    ///
    /// # Formula
    /// ```text
    /// XNPV = Σ CF_t / (1 + r)^(days_t / 365)
    /// ```
    ///
    /// # Arguments
    /// * `rate` - The annual discount rate
    /// * `flows` - The dated cash flow stream
    ///
    /// # Returns
    /// `Ok(Money)` containing the XNPV, or `Err(CasifinError)` on invalid input.
    pub fn xnpv(&self, rate: Decimal, flows: &CashFlowStream) -> Result<Money, CasifinError> {
        xnpv(rate, flows)
    }

    /// Computes the Internal Rate of Return with actual dates (XIRR).
    ///
    /// Uses a hybrid Newton-Raphson / bisection solver with date-weighted derivatives.
    ///
    /// # Arguments
    /// * `flows` - The dated cash flow stream (must have mixed signs)
    ///
    /// # Returns
    /// `Ok(Decimal)` containing the XIRR, or `Err(CasifinError)` if the solver
    /// does not converge.
    pub fn xirr(&self, flows: &CashFlowStream) -> Result<Decimal, CasifinError> {
        xirr(flows, self.config)
    }

    /// Computes the present value of an annuity.
    ///
    /// # Arguments
    /// * `rate` - The interest rate
    /// * `nper` - The total number of payment periods
    /// * `pmt` - The payment made each period
    /// * `fv` - The future value (cash balance after last payment)
    /// * `due` - Whether payments are due at beginning or end of period
    ///
    /// # Returns
    /// `Ok(Money)` containing the present value, or `Err(CasifinError)` on invalid input.
    #[allow(clippy::too_many_arguments)]
    pub fn pv(
        &self,
        rate: Rate,
        nper: u32,
        pmt: Money,
        fv: Money,
        due: PaymentDue,
    ) -> Result<Money, CasifinError> {
        pv(rate, nper, pmt, fv, due)
    }

    /// Computes the future value of an annuity.
    ///
    /// # Arguments
    /// * `rate` - The interest rate
    /// * `nper` - The total number of payment periods
    /// * `pmt` - The payment made each period
    /// * `pv` - The present value (initial investment)
    /// * `due` - Whether payments are due at beginning or end of period
    ///
    /// # Returns
    /// `Ok(Money)` containing the future value, or `Err(CasifinError)` on invalid input.
    #[allow(clippy::too_many_arguments)]
    pub fn fv(
        &self,
        rate: Rate,
        nper: u32,
        pmt: Money,
        pv: Money,
        due: PaymentDue,
    ) -> Result<Money, CasifinError> {
        fv(rate, nper, pmt, pv, due)
    }

    /// Computes the payment amount for an annuity.
    ///
    /// # Arguments
    /// * `rate` - The interest rate
    /// * `nper` - The total number of payment periods
    /// * `pv` - The present value (loan amount or investment)
    /// * `fv` - The future value (desired balance after last payment)
    /// * `due` - Whether payments are due at beginning or end of period
    ///
    /// # Returns
    /// `Ok(Money)` containing the payment amount, or `Err(CasifinError)` on invalid input.
    #[allow(clippy::too_many_arguments)]
    pub fn pmt(
        &self,
        rate: Rate,
        nper: u32,
        pv: Money,
        fv: Money,
        due: PaymentDue,
    ) -> Result<Money, CasifinError> {
        pmt(rate, nper, pv, fv, due)
    }

    /// Computes the number of periods required to reach a future value.
    ///
    /// # Arguments
    /// * `rate` - The interest rate (must be positive)
    /// * `pmt` - The payment made each period (must be non-zero)
    /// * `pv` - The present value
    /// * `fv` - The future value
    /// * `due` - Whether payments are due at beginning or end of period
    ///
    /// # Returns
    /// `Ok(Decimal)` containing the number of periods, or `Err(CasifinError)`
    /// on invalid input.
    #[allow(clippy::too_many_arguments)]
    pub fn nper(
        &self,
        rate: Rate,
        pmt: Money,
        pv: Money,
        fv: Money,
        due: PaymentDue,
    ) -> Result<Decimal, CasifinError> {
        nper(rate, pmt, pv, fv, due)
    }

    /// Computes the interest rate per period for an annuity.
    ///
    /// Uses a hybrid Newton-Raphson / bisection solver with the configured settings.
    ///
    /// # Arguments
    /// * `nper` - The total number of payment periods
    /// * `pmt` - The payment made each period
    /// * `pv` - The present value
    /// * `fv` - The future value
    /// * `due` - Whether payments are due at beginning or end of period
    ///
    /// # Returns
    /// `Ok(Decimal)` containing the rate per period, or `Err(CasifinError)` if the
    /// solver does not converge.
    #[allow(clippy::too_many_arguments)]
    pub fn rate(
        &self,
        nper: u32,
        pmt: Money,
        pv: Money,
        fv: Money,
        due: PaymentDue,
    ) -> Result<Decimal, CasifinError> {
        rate(nper, pmt, pv, fv, due, None, self.config)
    }
}

impl Default for Casifin {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_casifin_default_config() {
        let casifin = Casifin::with_defaults();
        assert_eq!(casifin.config().max_iterations, 1000);
    }

    #[test]
    fn test_casifin_mortgage() {
        let casifin = Casifin::with_defaults();
        let principal = Money::from(200000);
        let rate = Rate::new(Decimal::new(6, 2), Compounding::Discrete(12))
            .unwrap()
            .with_convention(DayCount::Actual365);

        let schedule = casifin.mortgage(principal, rate, 360).build().unwrap();
        assert_eq!(schedule.entries.len(), 360);
    }

    #[test]
    fn test_casifin_npv() {
        let casifin = Casifin::with_defaults();
        let flows = CashFlowStream::new(vec![
            CashFlow::new(Money::from(-1000)),
            CashFlow::new(Money::from(500)),
            CashFlow::new(Money::from(500)),
            CashFlow::new(Money::from(500)),
        ]);

        let rate = Decimal::new(10, 2);
        let npv_result = casifin.npv(rate, &flows).unwrap();
        assert!(npv_result > Money::from_decimal(Decimal::new(200, 0)));
    }

    #[test]
    fn test_casifin_irr() {
        let casifin = Casifin::with_defaults();
        let flows = CashFlowStream::new(vec![
            CashFlow::new(Money::from(-1000)),
            CashFlow::new(Money::from(500)),
            CashFlow::new(Money::from(500)),
            CashFlow::new(Money::from(500)),
        ]);

        let irr_result = casifin.irr(&flows);
        assert!(irr_result.is_ok());
    }
}
