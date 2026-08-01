//! Shared traits for the casifin financial computation engine.

use crate::{CasifinError, Money};

/// A trait for financial calculations that produce a monetary result.
///
/// Implementors of this trait are financial calculations that can be
/// computed to produce a `Money` value.
pub trait FinancialCalculation {
    /// Computes the result of this calculation.
    ///
    /// # Returns
    /// `Ok(Money)` if the calculation succeeds, `Err(CasifinError)` otherwise.
    fn compute(&self) -> Result<Money, CasifinError>;
}

/// A trait for items that can be scheduled (e.g., payments, cash flows).
///
/// Implementors are financial items that occur at specific times.
pub trait Schedulable {
    /// Returns the period number for this item.
    fn period(&self) -> u32;

    /// Returns the scheduled amount.
    fn amount(&self) -> Money;
}

/// A trait for streams of cash flows.
///
/// Implementors provide access to a sequence of cash flows.
pub trait CashFlowStream {
    /// Returns the number of cash flows in this stream.
    fn len(&self) -> usize;

    /// Returns `true` if the stream is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the cash flow at the specified index.
    fn cash_flow(&self, index: usize) -> Option<&dyn Schedulable>;

    /// Returns an iterator over the cash flows.
    fn iter(&self) -> Box<dyn Iterator<Item = &dyn Schedulable> + '_>;
}
