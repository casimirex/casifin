//! Differential tests comparing casifin output against Excel/HP-12C reference values.
//!
//! These tests hard-code reference values from Excel and verify that casifin
//! produces matching results to high precision.

use std::str::FromStr;

use approx::assert_relative_eq;
use casifin_sdk::*;
use rust_decimal::{prelude::ToPrimitive, Decimal};

const EXCEL_PV_ANNUITY: &str = "4329.476670630819";
const EXCEL_FV_ANNUITY: &str = "5525.631250";
const EXCEL_PMT_MORTGAGE: &str = "1475.8196732384508";

fn decimal(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

/// Differential test: PV of $1,000/year for 5 years at 5%.
#[test]
fn diff_pv_annuity() {
    let rate = Rate::new(Decimal::new(5, 2), Compounding::Discrete(1)).unwrap();
    let result = pv(
        rate,
        5,
        Money::from_decimal(Decimal::new(1000, 0)),
        Money::ZERO,
        PaymentDue::End,
    )
    .unwrap();

    assert_relative_eq!(
        result.inner().to_f64().unwrap(),
        decimal(EXCEL_PV_ANNUITY).to_f64().unwrap(),
        epsilon = 1e-6
    );
}

/// Differential test: FV of $1,000/year for 5 years at 5%.
#[test]
fn diff_fv_annuity() {
    let rate = Rate::new(Decimal::new(5, 2), Compounding::Discrete(1)).unwrap();
    let result = fv(
        rate,
        5,
        Money::from_decimal(Decimal::new(1000, 0)),
        Money::ZERO,
        PaymentDue::End,
    )
    .unwrap();

    assert_relative_eq!(
        result.inner().to_f64().unwrap(),
        decimal(EXCEL_FV_ANNUITY).to_f64().unwrap(),
        epsilon = 1e-9
    );
}

/// Differential test: PMT on $300,000 at 4.25% for 30 years.
#[test]
fn diff_pmt_mortgage() {
    let rate = Rate::new(Decimal::new(425, 4), Compounding::Discrete(12)).unwrap();
    let result = pmt(
        rate,
        360,
        Money::from_decimal(Decimal::new(300000, 0)),
        Money::ZERO,
        PaymentDue::End,
    )
    .unwrap();

    assert_relative_eq!(
        result.inner().abs().to_f64().unwrap(),
        decimal(EXCEL_PMT_MORTGAGE).to_f64().unwrap(),
        epsilon = 1e-9
    );
}

/// Differential test: NPV of uneven cash flows at 8%.
#[test]
fn diff_npv_uneven_flows() {
    let flows = CashFlowStream::new(vec![
        CashFlow::new(Money::from_decimal(Decimal::new(-1000, 0))),
        CashFlow::new(Money::from_decimal(Decimal::new(300, 0))),
        CashFlow::new(Money::from_decimal(Decimal::new(400, 0))),
        CashFlow::new(Money::from_decimal(Decimal::new(400, 0))),
        CashFlow::new(Money::from_decimal(Decimal::new(300, 0))),
    ]);

    let result = npv(Decimal::new(8, 2), &flows).unwrap();
    let expected = Decimal::from_str("158.755158145495").unwrap();

    assert_relative_eq!(
        result.inner().to_f64().unwrap(),
        expected.to_f64().unwrap(),
        epsilon = 1e-6
    );
}

/// Differential test: IRR of uneven cash flows.
#[test]
fn diff_irr_uneven_flows() {
    let flows = CashFlowStream::new(vec![
        CashFlow::new(Money::from_decimal(Decimal::new(-1000, 0))),
        CashFlow::new(Money::from_decimal(Decimal::new(300, 0))),
        CashFlow::new(Money::from_decimal(Decimal::new(400, 0))),
        CashFlow::new(Money::from_decimal(Decimal::new(400, 0))),
        CashFlow::new(Money::from_decimal(Decimal::new(300, 0))),
    ]);

    let result = irr(&flows, Config::default()).unwrap();
    let expected = Decimal::from_str("0.14895028127375542").unwrap();

    assert_relative_eq!(
        result.to_f64().unwrap(),
        expected.to_f64().unwrap(),
        epsilon = 1e-6
    );
}
