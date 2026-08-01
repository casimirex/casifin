# Contributing to casifin

Thank you for considering contributing to `casifin`! This document provides guidelines for contributing.

## Design Principles

Follow the NASA JPL Power of Ten rules:

1. **Simple control flow** - No `goto`, max cyclomatic complexity 10
2. **Fixed loop bounds** - All loops must have bounded iterations
3. **No dynamic allocation after init** - Core math uses stack-allocated types
4. **No long functions** - Max 60 lines per function
5. **Minimum two assertions** - `debug_assert!` for preconditions
6. **Data at smallest scope** - Pass by value or immutable reference
7. **Check return values** - Every `Result` must be handled
8. **Limited preprocessor** - No `cfg` spaghetti
9. **No raw pointers** - Use references only
10. **Warnings as errors** - `#![deny(warnings)]`

## Code Style

### Error Handling

```rust
// GOOD: Explicit error handling
let result = value
    .checked_div(other)
    .ok_or(CasifinError::DivisionByZero { operation: "example" })?;

// BAD: Never do this in library code
let result = value / other;  // Can panic!
```

### Function Documentation

Every public function needs:

```rust
/// Computes the present value of an annuity.
///
/// # Formula
/// ```text
/// PV = PMT * (1 - (1 + r)^-n) / r
/// ```
///
/// # Arguments
/// * `rate` - The interest rate per period
/// * `nper` - Total number of payment periods
/// * `pmt` - Payment made each period
/// * `fv` - Future value (cash balance after last payment)
/// * `due` - Payment timing (Beginning/End)
///
/// # Returns
/// `Ok(Money)` containing the present value, or `Err(CasifinError)` if:
/// - `rate` is negative
/// - `nper` is zero
/// - Division by zero occurs
///
/// # Example
/// ```
/// use casifin_sdk::{pv, PaymentDue, Money};
/// use rust_decimal::Decimal;
///
/// let rate = Decimal::new(5, 2);
/// let pmt = Money::from(1000);
/// let result = pv(rate, 10, pmt, Money::ZERO, PaymentDue::End).unwrap();
/// ```
pub fn pv(...) -> Result<Money, CasifinError> { ... }
```

### Testing

Every function gets:

1. **Happy path** - Known reference value
2. **Boundary** - Zero, negative inputs
3. **Error paths** - Validation rejection

```rust
#[test]
fn test_pv_ordinary_annuity() {
    let rate = Decimal::new(5, 2);
    let pmt = Money::from(1000);
    let nper = 10u32;
    let result = pv(rate, nper, pmt, Money::ZERO, PaymentDue::End).unwrap();
    assert!(result > Money::from(7000));
    assert!(result < Money::from(8000));
}

#[test]
fn test_pv_zero_rate() {
    let rate = Decimal::ZERO;
    let pmt = Money::from(100);
    let nper = 10u32;
    let result = pv(rate, nper, pmt, Money::ZERO, PaymentDue::End).unwrap();
    assert_eq!(result, Money::from(1000));
}

#[test]
fn test_pv_negative_rate_rejected() {
    let rate = Decimal::new(-5, 2);
    let result = pv(rate, 10, Money::from(100), Money::ZERO, PaymentDue::End);
    assert!(matches!(result, Err(CasifinError::InvalidRate(_))));
}
```

## Pull Request Process

### Before Submitting

- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes (100% green)
- [ ] `cargo fmt --check` passes
- [ ] Every public item has doc comments
- [ ] No `.unwrap()` or `.expect()` in library code
- [ ] Property tests added for numeric functions
- [ ] Benchmarks added for iterative algorithms

### PR Title Format

```
feat: Add XNPV with date support
fix: Correct IRR convergence check
docs: Improve rate function examples
refactor: Simplify derivative_rate function
perf: Optimize amortization schedule generation
```

## Adding a New Function

1. **Add to appropriate crate** based on functionality
2. **Write comprehensive docs** with formula, arguments, returns, example
3. **Implement with checked arithmetic** - no unchecked operations
4. **Add unit tests** - happy path, boundaries, errors
5. **Add property tests** if applicable
6. **Re-export from SDK** if public API
7. **Update API_REFERENCE.md**

Example:

```rust
// In crates/casifin-tvm/src/lib.rs

/// Computes present value of a perpetuity.
///
/// # Formula
/// ```text
/// PV = PMT / r
/// ```
///
/// # Arguments
/// * `rate` - Discount rate (must be positive)
/// * `pmt` - Periodic payment
///
/// # Returns
/// `Ok(Money)` or `Err(CasifinError::InvalidRate)` if rate <= 0
pub fn pv_perpetuity(rate: Decimal, pmt: Money) -> Result<Money, CasifinError> {
    debug_assert!(rate > Decimal::ZERO, "rate must be positive");

    if rate <= Decimal::ZERO {
        return Err(CasifinError::InvalidRate(rate));
    }

    pmt.checked_div_decimal(rate)
        .ok_or(CasifinError::DivisionByZero { operation: "pv_perpetuity" })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pv_perpetuity() {
        let rate = Decimal::new(5, 2);
        let pmt = Money::from(100);
        let result = pv_perpetuity(rate, pmt).unwrap();
        assert_eq!(result, Money::from(2000));
    }

    #[test]
    fn test_pv_perpetuity_zero_rate() {
        let result = pv_perpetuity(Decimal::ZERO, Money::from(100));
        assert!(matches!(result, Err(CasifinError::InvalidRate(_))));
    }
}
```

## Releasing

1. Update version in `Cargo.toml`
2. Run `cargo publish --dry-run` for each crate
3. Create release tag
4. Publish to crates.io

## Questions?

Open an issue for discussion. We welcome all contributions!
