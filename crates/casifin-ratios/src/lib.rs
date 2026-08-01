//! Financial ratios and metrics for the casifin financial computation engine.
//!
//! This crate provides the following ratio categories:
//! - Liquidity ratios (current, quick, cash, defensive interval)
//! - Solvency ratios (debt, debt-to-equity, leverage, interest coverage)
//! - Profitability ratios (margins, ROA, ROE, EPS)
//! - Return metrics (HPR, mean returns, TWRR, MWRR, Sharpe, Sortino)
//! - Yield metrics (bank discount, money market, bond equivalent, holding period)
//! - Rate conversions (EAR, stated, continuous, equivalent)
//! - Statistical utilities (CV, weighted mean, harmonic mean, standard error)

#![deny(warnings)]

pub mod liquidity;
pub mod profitability;
pub mod rates;
pub mod returns;
pub mod solvency;
pub mod statistics;
pub mod yields;

pub use liquidity::*;
pub use profitability::*;
pub use rates::*;
pub use returns::*;
pub use solvency::*;
pub use statistics::*;
pub use yields::*;
