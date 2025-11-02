use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "python", pyo3(eq, weakref, from_py_object, get_all, str))]
pub struct Fraction {
    pub num: u32,
    pub den: u32,
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[cfg_attr(feature = "python", pyo3::pymethods)]
impl Fraction {
    #[new]
    pub fn new(num: u32, den: u32) -> Self {
        assert!(den != 0, "denominator must not be zero");
        let mut f = Fraction { num, den };
        f.simplify();
        f
    }

    #[staticmethod]
    pub fn zero() -> Self {
        Fraction { num: 0, den: 1 }
    }

    #[staticmethod]
    pub fn one() -> Self {
        Fraction { num: 1, den: 1 }
    }

    #[staticmethod]
    pub fn from_int(value: u32) -> Self {
        Fraction { num: value, den: 1 }
    }

    #[staticmethod]
    fn gcd(mut a: u32, mut b: u32) -> u32 {
        while b != 0 {
            let r = a % b;
            a = b;
            b = r;
        }
        a
    }

    pub fn simplify(&mut self) {
        if self.num == 0 {
            self.den = 1;
            return;
        }
        let g = Self::gcd(self.num, self.den);
        if g > 1 {
            self.num /= g;
            self.den /= g;
        }
    }

    pub fn to_f64(&self) -> f64 {
        self.num as f64 / self.den as f64
    }
}

#[cfg(not(feature = "python"))]
impl Fraction {
    pub fn new(num: u32, den: u32) -> Self {
        assert!(den != 0, "denominator must not be zero");
        let mut f = Fraction { num, den };
        f.simplify();
        f
    }

    pub fn zero() -> Self {
        Fraction { num: 0, den: 1 }
    }

    pub fn one() -> Self {
        Fraction { num: 1, den: 1 }
    }

    pub fn from_int(value: u32) -> Self {
        Fraction { num: value, den: 1 }
    }

    fn gcd(mut a: u32, mut b: u32) -> u32 {
        while b != 0 {
            let r = a % b;
            a = b;
            b = r;
        }
        a
    }

    pub fn simplify(&mut self) {
        if self.num == 0 {
            self.den = 1;
            return;
        }
        let g = Self::gcd(self.num, self.den);
        if g > 1 {
            self.num /= g;
            self.den /= g;
        }
    }

    pub fn to_f64(&self) -> f64 {
        self.num as f64 / self.den as f64
    }
}

impl fmt::Display for Fraction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

impl Add for Fraction {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Fraction::new(self.num * rhs.den + rhs.num * self.den, self.den * rhs.den)
    }
}

impl Sub for Fraction {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        assert!(
            self.num * rhs.den >= rhs.num * self.den,
            "Result would be negative"
        );
        Fraction::new(self.num * rhs.den - rhs.num * self.den, self.den * rhs.den)
    }
}

impl Mul for Fraction {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Fraction::new(self.num * rhs.num, self.den * rhs.den)
    }
}

impl Div for Fraction {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        assert!(rhs.num != 0, "division by zero fraction");
        Fraction::new(self.num * rhs.den, self.den * rhs.num)
    }
}

// Fraction <op> integer
impl Add<u32> for Fraction {
    type Output = Fraction;
    fn add(self, rhs: u32) -> Self::Output {
        self + Fraction::from_int(rhs)
    }
}

impl Sub<u32> for Fraction {
    type Output = Fraction;
    fn sub(self, rhs: u32) -> Self::Output {
        self - Fraction::from_int(rhs)
    }
}

impl Mul<u32> for Fraction {
    type Output = Fraction;
    fn mul(self, rhs: u32) -> Self::Output {
        self * Fraction::from_int(rhs)
    }
}

impl Div<u32> for Fraction {
    type Output = Fraction;
    fn div(self, rhs: u32) -> Self::Output {
        assert!(rhs != 0, "division by zero integer");
        self / Fraction::from_int(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fraction_fraction() {
        let a = Fraction::new(1, 2);
        let b = Fraction::new(1, 3);
        assert_eq!(a + b, Fraction::new(5, 6));
        assert_eq!(a * b, Fraction::new(1, 6));
        assert_eq!(a / b, Fraction::new(3, 2));
        assert_eq!(a - Fraction::new(0, 1), Fraction::new(1, 2));
    }

    #[test]
    fn test_fraction_int() {
        let a = Fraction::new(3, 4);
        assert_eq!(a + 1, Fraction::new(7, 4));
        assert_eq!(a * 2, Fraction::new(3, 2));
        assert_eq!(a / 2, Fraction::new(3, 8));
    }

    #[test]
    fn test_fraction_chained_operations() {
        let a = Fraction::new(1, 2);
        let b = Fraction::new(1, 3);
        let c = Fraction::new(3, 4);

        let result = ((a + b) * c) / 2;
        assert_eq!(result, Fraction::new(5, 16));

        let result2 = ((a * 2) + 1) / b;
        assert_eq!(result2, Fraction::new(6, 1));
    }

    #[test]
    fn test_to_f64() {
        let f = Fraction::new(1, 4);
        assert_eq!(f.to_f64(), 0.25);
    }
}
