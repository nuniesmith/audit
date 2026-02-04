//! Sample module for testing documentation generation

use std::fmt;

/// A simple calculator for basic arithmetic operations
pub struct Calculator {
    /// The current value stored in the calculator
    value: f64,
}

impl Calculator {
    /// Create a new calculator with an initial value of 0
    ///
    /// # Examples
    ///
    /// ```
    /// let calc = Calculator::new();
    /// assert_eq!(calc.value(), 0.0);
    /// ```
    pub fn new() -> Self {
        Self { value: 0.0 }
    }

    /// Create a calculator with a specific starting value
    ///
    /// # Arguments
    ///
    /// * `initial` - The starting value for the calculator
    ///
    /// # Examples
    ///
    /// ```
    /// let calc = Calculator::with_value(10.0);
    /// assert_eq!(calc.value(), 10.0);
    /// ```
    pub fn with_value(initial: f64) -> Self {
        Self { value: initial }
    }

    /// Add a number to the current value
    ///
    /// # Arguments
    ///
    /// * `n` - The number to add
    ///
    /// # Returns
    ///
    /// The new value after addition
    ///
    /// # Examples
    ///
    /// ```
    /// let mut calc = Calculator::new();
    /// calc.add(5.0);
    /// assert_eq!(calc.value(), 5.0);
    /// ```
    pub fn add(&mut self, n: f64) -> f64 {
        self.value += n;
        self.value
    }

    /// Subtract a number from the current value
    ///
    /// # Arguments
    ///
    /// * `n` - The number to subtract
    ///
    /// # Returns
    ///
    /// The new value after subtraction
    pub fn subtract(&mut self, n: f64) -> f64 {
        self.value -= n;
        self.value
    }

    /// Get the current value
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Reset the calculator to zero
    pub fn reset(&mut self) {
        self.value = 0.0;
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Calculator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Calculator({})", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let calc = Calculator::new();
        assert_eq!(calc.value(), 0.0);
    }

    #[test]
    fn test_add() {
        let mut calc = Calculator::new();
        calc.add(5.0);
        assert_eq!(calc.value(), 5.0);
    }

    #[test]
    fn test_subtract() {
        let mut calc = Calculator::with_value(10.0);
        calc.subtract(3.0);
        assert_eq!(calc.value(), 7.0);
    }
}
