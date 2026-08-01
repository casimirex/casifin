//! Error types for the casifin financial computation engine.

use rust_decimal::Decimal;
use thiserror::Error;

/// The error type for all casifin operations.
///
/// This enum covers all possible error conditions that can occur
/// during financial calculations.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CasifinError {
    /// The rate provided is invalid (negative).
    #[error("invalid rate: {0}. Rate must be non-negative.")]
    InvalidRate(Decimal),

    /// The period provided is invalid (zero or negative).
    #[error("invalid period: {0}. Period must be positive.")]
    InvalidPeriod(u32),

    /// The IRR solver did not converge within the specified iterations.
    #[error("IRR did not converge within {max_iter} iterations (eps={eps})")]
    IrrConvergenceFailure {
        /// Maximum iterations allowed
        max_iter: u32,
        /// Convergence threshold
        eps: Decimal,
    },

    /// Division by zero occurred during a calculation.
    #[error("division by zero in {operation}")]
    DivisionByZero {
        /// The operation where division by zero occurred
        operation: &'static str,
    },

    /// The amortization schedule overflowed or failed to converge.
    #[error("amortization schedule overflow: {detail}")]
    ScheduleOverflow {
        /// Details about the overflow
        detail: String,
    },

    /// A date value is outside the valid range.
    #[error("date out of range: {0}")]
    DateOutOfRange(String),

    /// An invalid monetary amount was provided.
    #[error("invalid amount: {0}. Amount must be non-negative.")]
    InvalidAmount(crate::Money),

    /// The number of periods is invalid.
    #[error("invalid number of periods: {0}. Must be positive.")]
    InvalidNper(i64),

    /// Payment amount is invalid.
    #[error("invalid payment: {0}. Payment cannot be zero.")]
    InvalidPayment(Decimal),

    /// Cash flow stream is empty.
    #[error("cash flow stream cannot be empty")]
    EmptyCashFlowStream,

    /// XIRR requires at least one positive and one negative cash flow.
    #[error("XIRR requires at least one positive and one negative cash flow")]
    XirrSignRequirement,

    /// Invalid day count convention for the operation.
    #[error("invalid day count convention for this operation")]
    InvalidDayCountConvention,

    /// Rate conversion failed.
    #[error("rate conversion failed: {0}")]
    RateConversionFailure(String),

    /// Depreciation calculation error.
    #[error("depreciation error: {0}")]
    DepreciationError(String),

    /// Inventory calculation error.
    #[error("inventory error: {0}")]
    InventoryError(String),
}
