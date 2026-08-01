//! Global configuration for casifin calculations.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Global configuration for casifin calculations.
///
/// This struct holds configuration parameters that affect numerical
/// algorithms throughout the library.
///
/// # Fields
/// * `eps` - Convergence threshold for iterative algorithms (default: 1e-12)
/// * `max_iterations` - Maximum iterations for solvers (default: 1000)
/// * `guess` - Initial guess for iterative solvers (default: 0.1)
/// * `business_days_only` - Whether to use business days only (default: false)
/// * `periodic_compound` - Whether to use periodic compounding (default: true)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Convergence threshold for iterative algorithms.
    pub eps: Decimal,
    /// Maximum iterations for numerical solvers.
    pub max_iterations: u32,
    /// Initial guess for iterative solvers.
    pub guess: Decimal,
    /// Whether to use business days only for date calculations.
    pub business_days_only: bool,
    /// Whether to use periodic compounding.
    pub periodic_compound: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            eps: Decimal::new(1, 12), // 1e-12
            max_iterations: 1000,
            guess: Decimal::new(1, 1), // 0.1
            business_days_only: false,
            periodic_compound: true,
        }
    }
}

impl Config {
    /// Creates a new `Config` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ConfigBuilder` for constructing a `Config`.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    /// Creates a `Config` with the specified convergence threshold.
    ///
    /// # Arguments
    /// * `eps` - The convergence threshold (e.g., 1e-12)
    pub fn with_eps(mut self, eps: Decimal) -> Self {
        self.eps = eps;
        self
    }

    /// Creates a `Config` with the specified maximum iterations.
    ///
    /// # Arguments
    /// * `max_iterations` - The maximum number of iterations
    pub fn with_max_iterations(mut self, max_iterations: u32) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    /// Creates a `Config` with the specified initial guess.
    ///
    /// # Arguments
    /// * `guess` - The initial guess for solvers
    pub fn with_guess(mut self, guess: Decimal) -> Self {
        self.guess = guess;
        self
    }

    /// Creates a `Config` with business days only enabled.
    pub fn with_business_days_only(mut self, business_days_only: bool) -> Self {
        self.business_days_only = business_days_only;
        self
    }

    /// Creates a `Config` with periodic compounding setting.
    pub fn with_periodic_compound(mut self, periodic_compound: bool) -> Self {
        self.periodic_compound = periodic_compound;
        self
    }
}

/// Builder for constructing `Config` instances.
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    eps: Option<Decimal>,
    max_iterations: Option<u32>,
    guess: Option<Decimal>,
    business_days_only: Option<bool>,
    periodic_compound: Option<bool>,
}

impl ConfigBuilder {
    /// Creates a new `ConfigBuilder` with no fields set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the convergence threshold.
    pub fn eps(mut self, eps: Decimal) -> Self {
        self.eps = Some(eps);
        self
    }

    /// Sets the maximum iterations.
    pub fn max_iterations(mut self, max_iterations: u32) -> Self {
        self.max_iterations = Some(max_iterations);
        self
    }

    /// Sets the initial guess.
    pub fn guess(mut self, guess: Decimal) -> Self {
        self.guess = Some(guess);
        self
    }

    /// Sets business days only.
    pub fn business_days_only(mut self, business_days_only: bool) -> Self {
        self.business_days_only = Some(business_days_only);
        self
    }

    /// Sets periodic compounding.
    pub fn periodic_compound(mut self, periodic_compound: bool) -> Self {
        self.periodic_compound = Some(periodic_compound);
        self
    }

    /// Builds the `Config` instance.
    pub fn build(self) -> Config {
        let default = Config::default();
        Config {
            eps: self.eps.unwrap_or(default.eps),
            max_iterations: self.max_iterations.unwrap_or(default.max_iterations),
            guess: self.guess.unwrap_or(default.guess),
            business_days_only: self
                .business_days_only
                .unwrap_or(default.business_days_only),
            periodic_compound: self.periodic_compound.unwrap_or(default.periodic_compound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.eps, Decimal::new(1, 12));
        assert_eq!(config.max_iterations, 1000);
        assert_eq!(config.guess, Decimal::new(1, 1));
        assert!(!config.business_days_only);
        assert!(config.periodic_compound);
    }

    #[test]
    fn test_config_builder() {
        let config = Config::builder()
            .eps(Decimal::new(1, 15))
            .max_iterations(500)
            .build();
        assert_eq!(config.eps, Decimal::new(1, 15));
        assert_eq!(config.max_iterations, 500);
    }

    #[test]
    fn test_config_with_methods() {
        let config = Config::new()
            .with_eps(Decimal::new(1, 10))
            .with_max_iterations(100);
        assert_eq!(config.eps, Decimal::new(1, 10));
        assert_eq!(config.max_iterations, 100);
    }
}
