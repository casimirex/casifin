//! Integration tests for the casifin financial computation engine.
//!
//! These tests verify end-to-end functionality across multiple crates.

use casifin_sdk::*;
use rust_decimal::Decimal;

/// End-to-end mortgage calculation vs known reference values.
#[test]
fn test_mortgage_vs_excel_reference() {
    let casifin = Casifin::with_defaults();

    // Standard 30-year fixed mortgage: $200,000 at 6%
    let principal = Money::from(200_000);
    let rate = Rate::new(Decimal::new(6, 2), Compounding::Discrete(12))
        .unwrap()
        .with_convention(DayCount::Actual365);

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
    let casifin = Casifin::with_defaults();

    // Investment: -$10,000 initial, $3,000/year for 5 years
    let flows = CashFlowStream::new(vec![
        CashFlow::new(Money::from(-10_000)),
        CashFlow::new(Money::from(3_000)),
        CashFlow::new(Money::from(3_000)),
        CashFlow::new(Money::from(3_000)),
        CashFlow::new(Money::from(3_000)),
        CashFlow::new(Money::from(3_000)),
    ]);

    // NPV at 10% should be positive
    let npv = casifin.npv(Decimal::new(10, 2), &flows).unwrap();
    assert!(
        npv > Money::ZERO,
        "NPV should be positive for this investment"
    );

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
        InventoryLot::new(
            100,
            Money::from(10),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        ),
        InventoryLot::new(
            100,
            Money::from(12),
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
        ),
        InventoryLot::new(
            100,
            Money::from(14),
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
        ),
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
    let rate = Rate::new(Decimal::new(5, 2), Compounding::Discrete(1))
        .unwrap()
        .with_convention(DayCount::Actual365);
    let nper = 10u32;
    let pv_original = Money::from(1000);

    // Lump-sum round-trip (PMT=0): FV = PV * (1+r)^n, PV = FV * (1+r)^-n
    let fv = fv(rate, nper, Money::ZERO, pv_original, PaymentDue::End).unwrap();
    let pv_back = pv(rate, nper, Money::ZERO, fv, PaymentDue::End).unwrap();

    let diff = (pv_back - pv_original).abs();
    assert!(
        diff <= Money::from(1),
        "Round-trip diff too large: {}",
        diff
    );
}

/// Cross-crate consistency: TVM PMT equals amortization base payment.
#[test]
fn test_pmt_amortization_consistency() {
    let casifin = Casifin::with_defaults();
    let principal = Money::from(200_000);
    let rate = Rate::new(Decimal::new(6, 2), Compounding::Discrete(12)).unwrap();

    let tvm_pmt = casifin
        .pmt(rate, 360, principal, Money::ZERO, PaymentDue::End)
        .unwrap();
    let schedule = casifin.mortgage(principal, rate, 360).build().unwrap();

    let base_payment = schedule.entries[0].payment;
    let diff = (tvm_pmt.abs() - base_payment).abs();
    assert!(
        diff <= Money::from(1),
        "TVM PMT {} != base payment {}",
        tvm_pmt,
        base_payment
    );
}

/// Full pipeline: loan -> schedule -> cash flows -> IRR approximates loan rate.
#[test]
fn test_loan_schedule_irr_pipeline() {
    let casifin = Casifin::with_defaults();
    let principal = Money::from(100_000);
    let annual_rate = Decimal::new(5, 2);
    let rate = Rate::new(annual_rate, Compounding::Discrete(12)).unwrap();

    // Use a short term to keep the IRR solver well-conditioned.
    let schedule = casifin.mortgage(principal, rate, 12).build().unwrap();

    let mut flows = vec![CashFlow::new(-principal)];
    for entry in &schedule.entries {
        flows.push(CashFlow::new(entry.payment));
    }
    let stream = CashFlowStream::new(flows);

    let irr = casifin.irr(&stream).unwrap();
    let monthly_rate = annual_rate / Decimal::from(12);
    let diff = (irr - monthly_rate).abs();
    assert!(
        diff < Decimal::new(1, 4),
        "IRR {} does not approximate monthly rate {}",
        irr,
        monthly_rate
    );
}

/// Ratio functions handle edge cases and known values.
#[test]
fn test_ratio_edge_cases() {
    // Division by zero should return error
    let result = current_ratio(Money::from(100), Money::ZERO);
    assert!(result.is_err());

    // Normal case via glob re-export
    let result = current_ratio(Money::from(500), Money::from(250)).unwrap();
    assert_eq!(result, Decimal::from(2));
}
