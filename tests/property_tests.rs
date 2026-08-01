//! Property-based tests for the casifin financial computation engine.
//!
//! Uses proptest to verify invariants across random inputs.

use casifin_sdk::*;
use proptest::prelude::*;
use rust_decimal::Decimal;

/// NPV at zero rate equals sum of cash flows.
proptest! {
    #[test]
    fn npv_at_zero_rate_equals_sum_of_flows(
        flows in prop::collection::vec(any::<i64>(), 1..20)
    ) {
        let money_flows: Vec<Money> = flows
            .into_iter()
            .map(|v| Money::from(v))
            .collect();
        let sum: Money = money_flows.iter().copied().sum();
        let stream = CashFlowStream::from_vec(money_flows);

        let npv_result = npv(Decimal::ZERO, &stream).unwrap();
        assert_eq!(npv_result, sum);
    }
}

/// PV at zero rate: PV = sum of payments + FV.
proptest! {
    #[test]
    fn pv_at_zero_rate_equals_sum(
        nper in 1u32..100u32,
        pmt in any::<i64>(),
        fv in any::<i64>(),
    ) {
        let pmt_money = Money::from(pmt);
        let fv_money = Money::from(fv);

        let result = pv(Decimal::ZERO, nper, pmt_money, fv_money, PaymentDue::End).unwrap();
        let expected = pmt_money * Decimal::from(nper) + fv_money;
        assert_eq!(result, expected);
    }
}

/// FV at zero rate: FV = sum of payments + PV.
proptest! {
    #[test]
    fn fv_at_zero_rate_equals_sum(
        nper in 1u32..100u32,
        pmt in any::<i64>(),
        pv in any::<i64>(),
    ) {
        let pmt_money = Money::from(pmt);
        let pv_money = Money::from(pv);

        let result = fv(Decimal::ZERO, nper, pmt_money, pv_money, PaymentDue::End).unwrap();
        let expected = pmt_money * Decimal::from(nper) + pv_money;
        assert_eq!(result, expected);
    }
}

/// Depreciation schedule total equals depreciable base.
proptest! {
    #[test]
    fn straight_line_total_equals_depreciable_base(
        cost in 1000i64..100000i64,
        salvage in 0i64..500i64,
        life in 1u32..20u32,
    ) {
        let cost_money = Money::from(cost);
        let salvage_money = Money::from(salvage);

        if cost_money <= salvage_money {
            return Ok(());
        }

        let sl = StraightLine;
        if let Ok(schedule) = sl.schedule(cost_money, salvage_money, life) {
            let total: Money = schedule.iter().copied().sum();
            assert_eq!(total, cost_money - salvage_money);
        }
    }
}

/// Current ratio is positive for positive inputs.
proptest! {
    #[test]
    fn current_ratio_positive_for_positive_inputs(
        assets in 1i64..1000000i64,
        liabilities in 1i64..1000000i64,
    ) {
        let ratio = casifin_sdk::ratios::liquidity::current_ratio(
            Money::from(assets),
            Money::from(liabilities),
        ).unwrap();
        assert!(ratio > Decimal::ZERO);
    }
}

/// Money arithmetic is consistent.
proptest! {
    #[test]
    fn money_add_sub_consistent(
        a in any::<i64>(),
        b in any::<i64>(),
    ) {
        let ma = Money::from(a);
        let mb = Money::from(b);

        // (a + b) - b == a
        let result = (ma + mb) - mb;
        assert_eq!(result, ma);

        // a + b == b + a
        assert_eq!(ma + mb, mb + ma);
    }
}
