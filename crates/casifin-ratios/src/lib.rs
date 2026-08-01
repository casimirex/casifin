//! Financial ratios and metrics (FinCal port) for the casifin financial computation engine.

#![deny(warnings)]

use casifin_core::{CasifinError, Money};
use rust_decimal::{Decimal, MathematicalOps};

/// Liquidity ratios.
pub mod liquidity {
    use super::*;

    /// Current Ratio = Current Assets / Current Liabilities
    ///
    /// Measures a company's ability to pay short-term obligations.
    pub fn current_ratio(
        current_assets: Money,
        current_liabilities: Money,
    ) -> Result<Decimal, CasifinError> {
        if current_liabilities.is_zero() {
            return Err(CasifinError::DivisionByZero {
                operation: "current_ratio",
            });
        }
        current_assets
            .inner()
            .checked_div(current_liabilities.inner())
            .ok_or(CasifinError::DivisionByZero {
                operation: "current_ratio",
            })
    }

    /// Quick Ratio = (Cash + Marketable Securities + Receivables) / Current Liabilities
    ///
    /// Also known as the acid-test ratio.
    pub fn quick_ratio(
        cash: Money,
        marketable_securities: Money,
        receivables: Money,
        current_liabilities: Money,
    ) -> Result<Decimal, CasifinError> {
        if current_liabilities.is_zero() {
            return Err(CasifinError::DivisionByZero {
                operation: "quick_ratio",
            });
        }
        let quick_assets = cash + marketable_securities + receivables;
        quick_assets
            .inner()
            .checked_div(current_liabilities.inner())
            .ok_or(CasifinError::DivisionByZero {
                operation: "quick_ratio",
            })
    }

    /// Cash Ratio = Cash / Current Liabilities
    ///
    /// Most conservative liquidity measure.
    pub fn cash_ratio(cash: Money, current_liabilities: Money) -> Result<Decimal, CasifinError> {
        if current_liabilities.is_zero() {
            return Err(CasifinError::DivisionByZero {
                operation: "cash_ratio",
            });
        }
        cash.inner()
            .checked_div(current_liabilities.inner())
            .ok_or(CasifinError::DivisionByZero {
                operation: "cash_ratio",
            })
    }
}

/// Solvency ratios.
pub mod solvency {
    use super::*;

    /// Debt Ratio = Total Debt / Total Assets
    pub fn debt_ratio(total_debt: Money, total_assets: Money) -> Result<Decimal, CasifinError> {
        if total_assets.is_zero() {
            return Err(CasifinError::DivisionByZero {
                operation: "debt_ratio",
            });
        }
        total_debt
            .inner()
            .checked_div(total_assets.inner())
            .ok_or(CasifinError::DivisionByZero {
                operation: "debt_ratio",
            })
    }

    /// Debt-to-Equity Ratio = Total Debt / Total Equity
    pub fn debt_to_equity(total_debt: Money, total_equity: Money) -> Result<Decimal, CasifinError> {
        if total_equity.is_zero() {
            return Err(CasifinError::DivisionByZero {
                operation: "debt_to_equity",
            });
        }
        total_debt
            .inner()
            .checked_div(total_equity.inner())
            .ok_or(CasifinError::DivisionByZero {
                operation: "debt_to_equity",
            })
    }

    /// Financial Leverage = Total Assets / Total Equity
    pub fn financial_leverage(
        total_assets: Money,
        total_equity: Money,
    ) -> Result<Decimal, CasifinError> {
        if total_equity.is_zero() {
            return Err(CasifinError::DivisionByZero {
                operation: "financial_leverage",
            });
        }
        total_assets
            .inner()
            .checked_div(total_equity.inner())
            .ok_or(CasifinError::DivisionByZero {
                operation: "financial_leverage",
            })
    }
}

/// Profitability ratios.
pub mod profitability {
    use super::*;

    /// Gross Profit Margin = Gross Profit / Revenue
    pub fn gross_profit_margin(
        gross_profit: Money,
        revenue: Money,
    ) -> Result<Decimal, CasifinError> {
        if revenue.is_zero() {
            return Err(CasifinError::DivisionByZero {
                operation: "gross_profit_margin",
            });
        }
        gross_profit
            .inner()
            .checked_div(revenue.inner())
            .ok_or(CasifinError::DivisionByZero {
                operation: "gross_profit_margin",
            })
    }

    /// Net Profit Margin = Net Income / Revenue
    pub fn net_profit_margin(net_income: Money, revenue: Money) -> Result<Decimal, CasifinError> {
        if revenue.is_zero() {
            return Err(CasifinError::DivisionByZero {
                operation: "net_profit_margin",
            });
        }
        net_income
            .inner()
            .checked_div(revenue.inner())
            .ok_or(CasifinError::DivisionByZero {
                operation: "net_profit_margin",
            })
    }

    /// Basic EPS = (Net Income - Preferred Dividends) / Weighted Average Shares
    pub fn basic_eps(
        net_income: Money,
        preferred_dividends: Money,
        shares: Decimal,
    ) -> Result<Decimal, CasifinError> {
        if shares.is_zero() {
            return Err(CasifinError::DivisionByZero {
                operation: "basic_eps",
            });
        }
        let numerator = net_income.inner() - preferred_dividends.inner();
        numerator
            .checked_div(shares)
            .ok_or(CasifinError::DivisionByZero {
                operation: "basic_eps",
            })
    }

    /// Diluted EPS = (Net Income - Preferred Dividends) / (Weighted Shares + Potential Shares)
    pub fn diluted_eps(
        net_income: Money,
        preferred_dividends: Money,
        weighted_shares: Decimal,
        potential_shares: Decimal,
    ) -> Result<Decimal, CasifinError> {
        let total_shares = weighted_shares + potential_shares;
        if total_shares.is_zero() {
            return Err(CasifinError::DivisionByZero {
                operation: "diluted_eps",
            });
        }
        let numerator = net_income.inner() - preferred_dividends.inner();
        numerator
            .checked_div(total_shares)
            .ok_or(CasifinError::DivisionByZero {
                operation: "diluted_eps",
            })
    }
}

/// Return metrics.
pub mod returns {
    use super::*;

    /// Holding Period Return = (Sale Price - Purchase Price + Dividends) / Purchase Price
    pub fn holding_period_return(
        purchase_price: Money,
        sale_price: Money,
        dividends: Money,
    ) -> Result<Decimal, CasifinError> {
        if purchase_price.is_zero() {
            return Err(CasifinError::DivisionByZero {
                operation: "holding_period_return",
            });
        }
        let gain = sale_price - purchase_price + dividends;
        gain.inner()
            .checked_div(purchase_price.inner())
            .ok_or(CasifinError::DivisionByZero {
                operation: "holding_period_return",
            })
    }

    /// Geometric Mean Return
    ///
    /// # Formula
    /// ```text
    /// GM = ((1 + r1) * (1 + r2) * ... * (1 + rn))^(1/n) - 1
    /// ```
    pub fn geometric_mean_return(returns: &[Decimal]) -> Result<Decimal, CasifinError> {
        if returns.is_empty() {
            return Err(CasifinError::EmptyCashFlowStream);
        }

        let one = Decimal::ONE;
        let n = Decimal::from(returns.len());
        let mut product = Decimal::ONE;

        for &r in returns {
            product = product
                .checked_mul(one + r)
                .ok_or(CasifinError::ScheduleOverflow {
                    detail: "geometric_mean_return: overflow".to_string(),
                })?;
        }

        // nth root using power
        let result = product.powd(Decimal::ONE / n);
        Ok(result - one)
    }

    /// Time-Weighted Rate of Return (TWRR)
    ///
    /// # Formula
    /// ```text
    /// TWRR = (1 + r1) * (1 + r2) * ... * (1 + rn) - 1
    /// ```
    pub fn time_weighted_rate_of_return(
        period_returns: &[Decimal],
    ) -> Result<Decimal, CasifinError> {
        if period_returns.is_empty() {
            return Err(CasifinError::EmptyCashFlowStream);
        }

        let one = Decimal::ONE;
        let mut twrr = Decimal::ONE;

        for &r in period_returns {
            twrr = twrr
                .checked_mul(one + r)
                .ok_or(CasifinError::ScheduleOverflow {
                    detail: "twrr: overflow".to_string(),
                })?;
        }

        Ok(twrr - one)
    }

    /// Money-Weighted Rate of Return (MWRR).
    ///
    /// Also known as the Internal Rate of Return (IRR) of a portfolio.
    /// Computes the discount rate that makes the NPV of all cash flows equal to zero.
    ///
    /// # Formula
    /// ```text
    /// 0 = Σ CF_t / (1 + MWRR)^t
    /// ```
    ///
    /// # Arguments
    /// * `cash_flows` - The cash flow stream (negative = investment, positive = withdrawal/return)
    ///
    /// # Returns
    /// `Ok(Decimal)` containing the MWRR, or `Err(CasifinError)` if the stream is empty
    /// or lacks mixed signs.
    pub fn money_weighted_return(
        cash_flows: &casifin_cashflow::CashFlowStream,
    ) -> Result<Decimal, CasifinError> {
        casifin_cashflow::irr(cash_flows, Decimal::new(1, 1), 1000, Decimal::new(1, 12))
    }

    /// Sharpe Ratio = (Portfolio Return - Risk-Free Rate) / Standard Deviation
    pub fn sharpe_ratio(
        portfolio_return: Decimal,
        risk_free_rate: Decimal,
        std_dev: Decimal,
    ) -> Result<Decimal, CasifinError> {
        if std_dev.is_zero() {
            return Err(CasifinError::DivisionByZero {
                operation: "sharpe_ratio",
            });
        }
        let excess_return = portfolio_return - risk_free_rate;
        excess_return
            .checked_div(std_dev)
            .ok_or(CasifinError::DivisionByZero {
                operation: "sharpe_ratio",
            })
    }

    /// Roy's Safety-First Ratio = (Expected Return - Threshold Return) / Standard Deviation
    pub fn roys_safety_first_ratio(
        expected_return: Decimal,
        threshold: Decimal,
        std_dev: Decimal,
    ) -> Result<Decimal, CasifinError> {
        if std_dev.is_zero() {
            return Err(CasifinError::DivisionByZero {
                operation: "roys_safety_first_ratio",
            });
        }
        let excess = expected_return - threshold;
        excess
            .checked_div(std_dev)
            .ok_or(CasifinError::DivisionByZero {
                operation: "roys_safety_first_ratio",
            })
    }
}

/// Yield metrics.
pub mod yields {
    use super::*;

    /// Bank Discount Yield = ((Face Value - Purchase Price) / Face Value) * (360 / Days to
    /// Maturity)
    pub fn bank_discount_yield(
        face_value: Money,
        purchase_price: Money,
        days_to_maturity: u32,
    ) -> Result<Decimal, CasifinError> {
        if face_value.is_zero() || days_to_maturity == 0 {
            return Err(CasifinError::DivisionByZero {
                operation: "bank_discount_yield",
            });
        }
        let discount = face_value - purchase_price;
        let days_factor = Decimal::from(360) / Decimal::from(days_to_maturity);
        discount
            .inner()
            .checked_div(face_value.inner())
            .map(|d| d * days_factor)
            .ok_or(CasifinError::DivisionByZero {
                operation: "bank_discount_yield",
            })
    }

    /// Money Market Yield (CD Equivalent Yield)
    ///
    /// # Formula
    /// ```text
    /// MMY = ((Face Value - Purchase Price) / Purchase Price) * (360 / Days to Maturity)
    /// ```
    pub fn money_market_yield(
        face_value: Money,
        purchase_price: Money,
        days_to_maturity: u32,
    ) -> Result<Decimal, CasifinError> {
        if purchase_price.is_zero() || days_to_maturity == 0 {
            return Err(CasifinError::DivisionByZero {
                operation: "money_market_yield",
            });
        }
        let discount = face_value - purchase_price;
        let days_factor = Decimal::from(360) / Decimal::from(days_to_maturity);
        discount
            .inner()
            .checked_div(purchase_price.inner())
            .map(|d| d * days_factor)
            .ok_or(CasifinError::DivisionByZero {
                operation: "money_market_yield",
            })
    }

    /// Bond Equivalent Yield = Semi-Annual Yield * 2
    pub fn bond_equivalent_yield(semi_annual_yield: Decimal) -> Result<Decimal, CasifinError> {
        Ok(semi_annual_yield * Decimal::from(2))
    }
}

/// Rate conversions.
pub mod rates {
    use casifin_core::Compounding;

    use super::*;

    /// Effective Annual Rate (EAR) from stated rate
    ///
    /// # Formula
    /// ```text
    /// EAR = (1 + r/n)^n - 1
    /// ```
    pub fn effective_annual_rate(
        stated_rate: Decimal,
        compounding: Compounding,
    ) -> Result<Decimal, CasifinError> {
        if stated_rate < Decimal::ZERO {
            return Err(CasifinError::InvalidRate(stated_rate));
        }

        let one = Decimal::ONE;
        match compounding {
            Compounding::Discrete(n) => {
                let periodic = stated_rate.checked_div(Decimal::from(n)).ok_or(
                    CasifinError::DivisionByZero {
                        operation: "effective_annual_rate periodic",
                    },
                )?;
                let power = (one + periodic).checked_powd(Decimal::from(n)).ok_or(
                    CasifinError::ScheduleOverflow {
                        detail: "effective_annual_rate power overflow".to_string(),
                    },
                )?;
                Ok(power - one)
            }
            Compounding::Continuous => {
                // e^r - 1
                Ok(Decimal::E.powd(stated_rate) - one)
            }
        }
    }

    /// Stated Rate from Effective Annual Rate
    pub fn stated_from_effective(
        effective_rate: Decimal,
        compounding: Compounding,
    ) -> Result<Decimal, CasifinError> {
        let one = Decimal::ONE;
        match compounding {
            Compounding::Discrete(n) => {
                let nth_root = (one + effective_rate).powd(one / Decimal::from(n));
                Ok((nth_root - one) * Decimal::from(n))
            }
            Compounding::Continuous => {
                // ln(1 + r)
                Ok(ln_approx(one + effective_rate))
            }
        }
    }

    /// Continuous Rate from Nominal Rate
    pub fn nominal_to_continuous(nominal_rate: Decimal) -> Result<Decimal, CasifinError> {
        if nominal_rate < Decimal::ZERO {
            return Err(CasifinError::InvalidRate(nominal_rate));
        }
        Ok(Decimal::E.powd(nominal_rate) - Decimal::ONE)
    }

    /// Nominal Rate from Continuous Rate
    pub fn continuous_to_nominal(continuous_rate: Decimal) -> Result<Decimal, CasifinError> {
        if continuous_rate < Decimal::NEGATIVE_ONE {
            return Err(CasifinError::InvalidRate(continuous_rate));
        }
        Ok(ln_approx(Decimal::ONE + continuous_rate))
    }

    /// Holding Period Return to Effective Annual Rate
    pub fn holding_period_to_effective(
        hpr: Decimal,
        periods_per_year: u32,
    ) -> Result<Decimal, CasifinError> {
        if periods_per_year == 0 {
            return Err(CasifinError::DivisionByZero {
                operation: "holding_period_to_effective",
            });
        }
        let one = Decimal::ONE;
        let power = (one + hpr).powd(one / Decimal::from(periods_per_year));
        Ok(power - one)
    }

    /// Equivalent Rate: converts a rate from one compounding frequency to another.
    ///
    /// # Formula
    /// ```text
    /// r_target = (1 + r_source/n_source)^(n_source/n_target) - 1
    /// ```
    ///
    /// # Arguments
    /// * `rate` - The source rate with its compounding
    /// * `target_compounding` - The desired compounding frequency
    ///
    /// # Returns
    /// `Ok(Rate)` with the equivalent rate, or `Err(CasifinError)` on failure.
    pub fn equivalent_rate(
        rate: &casifin_core::Rate,
        target_compounding: Compounding,
    ) -> Result<casifin_core::Rate, CasifinError> {
        let effective = effective_annual_rate(rate.annual_rate, rate.compounding)?;
        let stated = stated_from_effective(effective, target_compounding)?;
        casifin_core::Rate::new(stated, target_compounding, rate.convention)
    }

    /// Approximate natural logarithm.
    fn ln_approx(x: Decimal) -> Decimal {
        if x <= Decimal::ZERO {
            return Decimal::ZERO;
        }
        let y = (x - Decimal::ONE) / (x + Decimal::ONE);
        let y2 = y * y;
        let y3 = y2 * y;
        let y5 = y3 * y2;
        y - y3 / Decimal::from(3) + y5 / Decimal::from(5)
    }
}

/// Statistical utilities.
pub mod statistics {
    use super::*;

    /// Coefficient of Variation = Standard Deviation / Mean
    pub fn coefficient_of_variance(
        mean: Decimal,
        std_dev: Decimal,
    ) -> Result<Decimal, CasifinError> {
        if mean.is_zero() {
            return Err(CasifinError::DivisionByZero {
                operation: "coefficient_of_variance",
            });
        }
        std_dev
            .checked_div(mean)
            .ok_or(CasifinError::DivisionByZero {
                operation: "coefficient_of_variance",
            })
    }

    /// Weighted Mean
    ///
    /// # Formula
    /// ```text
    /// WM = Σ(wi * xi) / Σwi
    /// ```
    pub fn weighted_mean(values: &[Decimal], weights: &[Decimal]) -> Result<Decimal, CasifinError> {
        if values.len() != weights.len() {
            return Err(CasifinError::InventoryError(
                "values and weights must have same length".to_string(),
            ));
        }

        let mut weighted_sum = Decimal::ZERO;
        let mut weight_sum = Decimal::ZERO;

        for (&v, &w) in values.iter().zip(weights.iter()) {
            weighted_sum += v * w;
            weight_sum += w;
        }

        if weight_sum.is_zero() {
            return Err(CasifinError::DivisionByZero {
                operation: "weighted_mean",
            });
        }

        weighted_sum
            .checked_div(weight_sum)
            .ok_or(CasifinError::DivisionByZero {
                operation: "weighted_mean",
            })
    }

    /// Harmonic Mean
    ///
    /// # Formula
    /// ```text
    /// HM = n / (Σ(1/xi))
    /// ```
    pub fn harmonic_mean(values: &[Decimal]) -> Result<Decimal, CasifinError> {
        if values.is_empty() {
            return Err(CasifinError::EmptyCashFlowStream);
        }

        let n = Decimal::from(values.len());
        let mut reciprocal_sum = Decimal::ZERO;

        for &v in values {
            if v.is_zero() {
                return Err(CasifinError::DivisionByZero {
                    operation: "harmonic_mean",
                });
            }
            reciprocal_sum += Decimal::ONE / v;
        }

        n.checked_div(reciprocal_sum)
            .ok_or(CasifinError::DivisionByZero {
                operation: "harmonic_mean",
            })
    }

    /// Sampling Error (Standard Error of the Mean)
    ///
    /// # Formula
    /// ```text
    /// SE = σ / √n
    /// ```
    pub fn sampling_error(
        population_std_dev: Decimal,
        sample_size: u32,
    ) -> Result<Decimal, CasifinError> {
        if sample_size == 0 {
            return Err(CasifinError::DivisionByZero {
                operation: "sampling_error",
            });
        }

        let n = Decimal::from(sample_size);
        let sqrt_n = n.sqrt().unwrap_or(Decimal::ONE);

        population_std_dev
            .checked_div(sqrt_n)
            .ok_or(CasifinError::DivisionByZero {
                operation: "sampling_error",
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_ratio() {
        let assets = Money::from(500000);
        let liabilities = Money::from(250000);
        let ratio = liquidity::current_ratio(assets, liabilities).unwrap();
        assert_eq!(ratio, Decimal::from(2));
    }

    #[test]
    fn test_debt_ratio() {
        let debt = Money::from(100000);
        let assets = Money::from(500000);
        let ratio = solvency::debt_ratio(debt, assets).unwrap();
        assert_eq!(ratio, Decimal::new(2, 1)); // 0.2
    }

    #[test]
    fn test_gross_profit_margin() {
        let gross_profit = Money::from(300000);
        let revenue = Money::from(1000000);
        let margin = profitability::gross_profit_margin(gross_profit, revenue).unwrap();
        assert_eq!(margin, Decimal::new(3, 1)); // 0.3
    }

    #[test]
    fn test_geometric_mean_return() {
        let returns = vec![
            Decimal::new(1, 1),  // 10%
            Decimal::new(5, 2),  // 5%
            Decimal::new(15, 2), // 15%
        ];
        let gm = returns::geometric_mean_return(&returns).unwrap();
        assert!(gm > Decimal::new(9, 2)); // > 9%
        assert!(gm < Decimal::new(11, 2)); // < 11%
    }

    #[test]
    fn test_sharpe_ratio() {
        let portfolio_return = Decimal::new(12, 2); // 12%
        let risk_free = Decimal::new(3, 2); // 3%
        let std_dev = Decimal::new(15, 2); // 15%
        let sharpe = returns::sharpe_ratio(portfolio_return, risk_free, std_dev).unwrap();
        assert_eq!(sharpe, Decimal::new(6, 1)); // 0.6
    }

    #[test]
    fn test_effective_annual_rate() {
        let stated = Decimal::new(12, 2); // 12%
        let ear = rates::effective_annual_rate(stated, casifin_core::Compounding::MONTHLY).unwrap();
        // (1 + 0.01)^12 - 1 ≈ 0.1268
        assert!(ear > Decimal::new(12, 2));
        assert!(ear < Decimal::new(13, 2));
    }

    #[test]
    fn test_harmonic_mean() {
        let values = vec![Decimal::from(2), Decimal::from(4), Decimal::from(8)];
        let hm = statistics::harmonic_mean(&values).unwrap();
        // HM = 3 / (1/2 + 1/4 + 1/8) = 3 / 0.875 = 3.43
        assert!(hm > Decimal::from(3));
        assert!(hm < Decimal::from(4));
    }
}
