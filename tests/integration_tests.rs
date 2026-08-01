//! Integration tests for the casifin financial computation engine.
//!
//! These tests verify end-to-end functionality across multiple crates.

use casifin_sdk::*;
use rust_decimal::Decimal;

/// End-to-end mortgage calculation vs known reference values.
#[test]
fn test_mortgage_vs_excel_reference() {
    let casifin = Casifin::with_default_config();

    // Standard 30-year fixed mortgage: $200,000 at 6%
    let principal = Money::from(200_000);
    let rate = Rate::new(
        Decimal::new(6, 2),
        Compounding::MONTHLY,
        DayCount::Actual365,
    )
    .unwrap();

    let schedule = casifin.mortgage(principal, rate, 360).build().unwrap();

    // Excel PMT(6%/12, 360, -200000) = $1,199.10
    let monthly_payment = schedule.entries[0].payment;
    assert!(monthly_payment > Money::from(1199));
    assert!(monthly_payment < Money::from(1200));

    // Total payments should be 360
    assert_eq!(schedule.entries.len(), 360);

    // Final balance should be near zero
    let final_balance = schedule.entries.last().unwrap().balance;
    assert!(final_balance.abs() <= Money::from(1));

    // Total principal should equal original principal
    let principal_diff = (schedule.total_principal - principal).abs();
    assert!(principal_diff <= Money::from(1));
}

/// Full NPV/IRR pipeline test.
#[test]
fn test_npv_irr_pipeline() {
    let casifin = Casifin::with_default_config();

    // Investment: -$10,000 initial, $3,000/year for 5 years
    let flows = CashFlowStream::from_vec(vec![
        Money::from(-10_000),
        Money::from(3_000),
        Money::from(3_000),
        Money::from(3_000),
        Money::from(3_000),
        Money::from(3_000),
    ]);

    // NPV at 10% should be positive
    let npv = casifin.npv(Decimal::new(10, 2), &flows).unwrap();
    assert!(npv > Money::ZERO, "NPV should be positive for this investment");

    // IRR should be > 10%
    let irr = casifin.irr(&flows).unwrap();
    assert!(
        irr > Decimal::new(10, 2),
        "IRR should exceed 10% for this investment"
    );

    // NPV at 0% should equal sum of cash flows
    let npv_zero = casifin.npv(Decimal::ZERO, &flows).unwrap();
    let sum: Money = vec![
        Money::from(-10_000),
        Money::from(3_000),
        Money::from(3_000),
        Money::from(3_000),
        Money::from(3_000),
        Money::from(3_000),
    ]
    .into_iter()
    .sum();
    assert_eq!(npv_zero, sum);
}

/// ARM schedule vs known reference.
#[test]
fn test_arm_schedule_reference() {
    let casifin = Casifin::with_default_config();

    let principal = Money::from(300_000);
    let initial_rate = Rate::new(
        Decimal::new(5, 2),
        Compounding::MONTHLY,
        DayCount::Actual365,
    )
    .unwrap();

    let adj_rate = Rate::new(
        Decimal::new(7, 2),
        Compounding::MONTHLY,
        DayCount::Actual365,
    )
    .unwrap();

    let caps = RateCaps::new(
        Decimal::new(2, 2), // 2% periodic cap
        Decimal::new(5, 2), // 5% lifetime cap
    );

    let arm = casifin
        .arm(principal, initial_rate, 360)
        .with_adjustment(61, adj_rate)
        .with_caps(caps)
        .build()
        .unwrap();

    // Should have entries
    assert!(!arm.schedule.entries.is_empty());

    // First payment should be at 5% rate
    let first_payment = arm.schedule.entries[0].payment;
    assert!(first_payment > Money::from(1600));
    assert!(first_payment < Money::from(1700));

    // Total principal should equal original
    let principal_diff = (arm.schedule.total_principal - principal).abs();
    assert!(principal_diff <= Money::from(1));
}

/// Depreciation total equals depreciable base.
#[test]
fn test_depreciation_total_equals_base() {
    let cost = Money::from(10_000);
    let salvage = Money::from(1_000);
    let life = 5u32;

    let sl = StraightLine;
    let schedule = sl.schedule(cost, salvage, life).unwrap();
    let total: Money = schedule.iter().copied().sum();
    assert_eq!(total, cost - salvage);

    let ddb = DoubleDecliningBalance;
    let schedule = ddb.schedule(cost, salvage, life).unwrap();
    let total: Money = schedule.iter().copied().sum();
    let diff = (total - (cost - salvage)).abs();
    assert!(diff <= Money::from(10));
}

/// Inventory methods produce consistent total value.
#[test]
fn test_inventory_total_consistency() {
    use chrono::NaiveDate;

    let lots = vec![
        InventoryLot::new(100, Money::from(10), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        InventoryLot::new(100, Money::from(12), NaiveDate::from_ymd_opt(2024, 2, 1).unwrap()),
        InventoryLot::new(100, Money::from(14), NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()),
    ];

    let total_value: Money = lots.iter().map(|l| l.total_value()).sum();

    let fifo = Fifo;
    let lifo = Lifo;
    let wa = WeightedAverage;

    let fifo_cogs = fifo.cogs(&lots, 150).unwrap();
    let fifo_ending = fifo.ending_inventory(&lots, 150).unwrap();
    assert_eq!(fifo_cogs + fifo_ending, total_value);

    let lifo_cogs = lifo.cogs(&lots, 150).unwrap();
    let lifo_ending = lifo.ending_inventory(&lots, 150).unwrap();
    assert_eq!(lifo_cogs + lifo_ending, total_value);

    let wa_cogs = wa.cogs(&lots, 150).unwrap();
    let wa_ending = wa.ending_inventory(&lots, 150).unwrap();
    assert_eq!(wa_cogs + wa_ending, total_value);
}

/// TVM round-trip: PV -> FV -> PV should be consistent.
#[test]
fn test_tvm_round_trip() {
    let rate = Decimal::new(5, 2);
    let nper = 10u32;
    let pmt = Money::from(100);
    let pv_original = Money::from(1000);

    // PV -> FV
    let fv = fv(rate, nper, pmt, pv_original, PaymentDue::End).unwrap();

    // FV -> PV should give back original
    let pv_back = pv(rate, nper, pmt, fv, PaymentDue::End).unwrap();

    let diff = (pv_back - pv_original).abs();
    assert!(diff <= Money::from(1));
}

/// Ratio functions handle edge cases.
#[test]
fn test_ratio_edge_cases() {
    use casifin_sdk::ratios::liquidity;

    // Division by zero should return error
    let result = liquidity::current_ratio(Money::from(100), Money::ZERO);
    assert!(result.is_err());

    // Normal case
    let result = liquidity::current_ratio(Money::from(500), Money::from(250)).unwrap();
    assert_eq!(result, Decimal::from(2));
}
