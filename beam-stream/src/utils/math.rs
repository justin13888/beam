use ffmpeg_next as ffmpeg;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Rational {
    inner: ffmpeg::Rational,
}

impl Rational {
    pub fn inner(&self) -> &ffmpeg::Rational {
        &self.inner
    }

    /// Create a new rational number
    pub fn new(numerator: i32, denominator: i32) -> Self {
        Rational {
            inner: ffmpeg::Rational::new(numerator, denominator),
        }
    }

    /// Get the numerator
    pub fn numerator(&self) -> i32 {
        self.inner.numerator()
    }

    /// Get the denominator
    pub fn denominator(&self) -> i32 {
        self.inner.denominator()
    }

    /// Convert to floating point value
    pub fn to_f64(&self) -> f64 {
        f64::from(self.inner)
    }

    /// Convert to floating point value (f32)
    pub fn to_f32(&self) -> f32 {
        self.to_f64() as f32
    }

    /// Check if this rational is zero
    pub fn is_zero(&self) -> bool {
        self.numerator() == 0
    }

    /// Check if this rational is valid (denominator is not zero)
    pub fn is_valid(&self) -> bool {
        self.denominator() != 0
    }

    /// Get the reciprocal of this rational
    pub fn reciprocal(&self) -> Option<Self> {
        if self.numerator() != 0 {
            Some(Rational::new(self.denominator(), self.numerator()))
        } else {
            None
        }
    }

    /// Reduce the rational to its simplest form
    pub fn reduce(&self) -> Self {
        let gcd = gcd(self.numerator().abs(), self.denominator().abs());
        if gcd > 1 {
            Rational::new(self.numerator() / gcd, self.denominator() / gcd)
        } else {
            *self
        }
    }

    /// Format as a string with custom precision
    pub fn format_decimal(&self, precision: usize) -> String {
        format!("{:.precision$}", self.to_f64(), precision = precision)
    }

    /// Get a human-readable representation
    pub fn display(&self) -> String {
        if self.denominator() == 1 {
            self.numerator().to_string()
        } else if self.is_valid() {
            format!("{}/{}", self.numerator(), self.denominator())
        } else {
            "Invalid".to_string()
        }
    }
}

impl From<ffmpeg::Rational> for Rational {
    fn from(rational: ffmpeg::Rational) -> Self {
        Rational { inner: rational }
    }
}

impl From<f64> for Rational {
    fn from(value: f64) -> Self {
        // Convert float to rational with reasonable precision
        let denominator = 1000000; // Use 6 decimal places precision
        let numerator = (value * denominator as f64).round() as i32;
        Rational::new(numerator, denominator).reduce()
    }
}

impl From<Rational> for f64 {
    fn from(rational: Rational) -> Self {
        rational.to_f64()
    }
}

impl std::fmt::Display for Rational {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// Calculate the greatest common divisor of two numbers
fn gcd(a: i32, b: i32) -> i32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rational_creation() {
        let r = Rational::new(3, 4);
        assert_eq!(r.numerator(), 3);
        assert_eq!(r.denominator(), 4);
        assert_eq!(r.to_f64(), 0.75);
    }

    #[test]
    fn test_rational_reduce() {
        let r = Rational::new(6, 8);
        let reduced = r.reduce();
        assert_eq!(reduced.numerator(), 3);
        assert_eq!(reduced.denominator(), 4);
    }

    #[test]
    fn test_rational_from_f64() {
        let r = Rational::from(0.75);
        let f = r.to_f64();
        assert!((f - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_rational_reciprocal() {
        let r = Rational::new(3, 4);
        let reciprocal = r.reciprocal().unwrap();
        assert_eq!(reciprocal.numerator(), 4);
        assert_eq!(reciprocal.denominator(), 3);
    }
}
