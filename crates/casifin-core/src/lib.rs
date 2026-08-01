//! Core types and primitives for the casifin financial computation engine.
//!
//! This crate provides the foundational types used throughout casifin:
//! - [`Money`] - A monetary value wrapper around `rust_decimal::Decimal`
//! - [`Rate`] - An interest rate with compounding metadata
//! - [`CasifinError`] - The error type for all casifin operations
//! - [`Config`] - Global configuration for calculations

#![deny(warnings)]

pub mod config;
pub mod error;
pub mod money;
pub mod rate;
pub mod traits;

pub use config::{Config, ConfigBuilder};
pub use error::CasifinError;
pub use money::Money;
pub use rate::{Compounding, DayCount, Rate};
pub use traits::{CashFlowStream, FinancialCalculation, Schedulable};
