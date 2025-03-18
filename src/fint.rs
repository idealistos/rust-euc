use std::{
    fmt::{Display, Formatter, Result},
    hash::Hasher,
    ops,
};

use float_next_after::NextAfter;

use crate::hashset2::WithTwoHashes;

trait NextBeforeOrAfter {
    fn inc(self) -> f64;
    fn dec(self) -> f64;
}

impl NextBeforeOrAfter for f64 {
    fn inc(self) -> f64 {
        self.next_after(std::f64::INFINITY)
    }

    fn dec(self) -> f64 {
        self.next_after(std::f64::NEG_INFINITY)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FInt(f64, f64);
impl Display for FInt {
    fn fmt(&self, f: &mut Formatter) -> Result {
        // write!(f, "{}..{}", self.0, self.1)

        let mean = 0.5 * (self.1 + self.0);
        if self.1 - self.0 > 1e-8 && self.1 - self.0 > mean.abs() * 1e-8 {
            write!(f, "{:.3} to {:.3}", self.0, self.1)
        } else {
            write!(f, "{:.3}", mean)
        }
    }
}
impl FInt {
    pub fn new(value: f64) -> FInt {
        Self::new_with_bounds(value.dec(), value.inc())
    }

    pub fn new_with_delta(value: f64, delta: f64) -> FInt {
        if delta <= 0.0 {
            panic!("Wrong interval! delta {} <= 0", delta);
        }
        Self::new_with_bounds(value - delta, value + delta)
    }

    pub fn new_with_bounds(lower: f64, upper: f64) -> FInt {
        if lower > upper {
            panic!("Wrong interval! {} > {}", lower, upper);
        }
        Self(lower, upper)
    }

    pub fn negate(self) -> FInt {
        Self::new_with_bounds(-self.1, -self.0)
    }

    pub fn inverse(self) -> FInt {
        if self.0 <= 0.0 && self.1 >= 0.0 {
            Self::new_with_bounds(f64::NAN, f64::NAN)
        } else if self.0 > 0.0 {
            Self::new_with_bounds((1.0 / self.1).dec(), (1.0 / self.0).inc())
        } else {
            self.negate().inverse().negate()
        }
    }

    pub fn sqr(self) -> FInt {
        self * self
    }

    pub fn sqrt(self) -> FInt {
        if self.0 < 0.0 {
            Self::new_with_bounds(f64::NAN, f64::NAN)
        } else {
            Self::new_with_bounds(self.0.sqrt().dec(), self.1.sqrt().inc())
        }
    }

    pub fn always_positive(self) -> bool {
        self.0 > 0.0
    }

    pub fn midpoint(self) -> f64 {
        0.5 * (self.0 + self.1)
    }
}

impl ops::Add<FInt> for FInt {
    type Output = FInt;

    fn add(self, x: FInt) -> FInt {
        Self::new_with_bounds((self.0 + x.0).dec(), (self.1 + x.1).inc())
    }
}

impl ops::Sub<FInt> for FInt {
    type Output = FInt;

    fn sub(self, x: FInt) -> FInt {
        Self::new_with_bounds((self.0 - x.1).dec(), (self.1 - x.0).inc())
    }
}

impl ops::Mul<FInt> for FInt {
    type Output = FInt;

    fn mul(self, x: FInt) -> FInt {
        if self.0 >= 0.0 {
            if x.0 >= 0.0 {
                return Self::new_with_bounds((self.0 * x.0).dec(), (self.1 * x.1).inc());
            } else if x.1 <= 0.0 {
                return Self::new_with_bounds((self.1 * x.0).dec(), (self.0 * x.1).dec());
            }
        } else if self.1 <= 0.0 {
            if x.0 >= 0.0 {
                return Self::new_with_bounds((self.0 * x.1).dec(), (self.1 * x.0).dec());
            } else if x.1 <= 0.0 {
                return Self::new_with_bounds((self.1 * x.1).dec(), (self.0 * x.0).dec());
            }
        }
        let v00 = self.0 * x.0;
        let v01 = self.0 * x.1;
        let v10 = self.1 * x.0;
        let v11 = self.1 * x.1;
        Self::new_with_bounds(
            v00.min(v01).min(v10).min(v11).dec(),
            v00.max(v01).max(v10).max(v11).inc(),
        )
    }
}

impl ops::Div<FInt> for FInt {
    type Output = FInt;

    fn div(self, x: FInt) -> FInt {
        self * x.inverse()
    }
}

impl PartialEq for FInt {
    fn eq(&self, x: &FInt) -> bool {
        !(x.1 < self.0 || x.0 > self.1)
    }
}
impl Eq for FInt {}

impl WithTwoHashes for FInt {
    fn hash1<H: Hasher>(&self, state: &mut H) {
        let m = 0.5 * (self.0 + self.1);
        let v = if m.abs() < 100.0 {
            m
        } else {
            m.abs().ln().copysign(m)
        };
        state.write_i32((1000000.0 * v) as i32);
    }

    fn hash2<H: Hasher>(&self, state: &mut H) {
        let m = 0.5 * (self.0 + self.1);
        let v = if m.abs() < 100.0 {
            m
        } else {
            m.abs().ln().copysign(m)
        };
        state.write_i32((1000000.0 * v + 0.5) as i32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let result = FInt::new(1.0) + FInt::new(1.0);
        assert!((result.0 - 2.0).abs() < 1e-14);
        assert!(result.1 - result.0 > 0.0);
    }

    #[test]
    fn test_complex_operation() {
        let result = FInt::new(1.2) / (FInt::new(1.00001) - FInt::new(0.5) * FInt::new(2.0))
            - FInt::new(120000.0);
        assert_eq!(
            format!("{:?}", result),
            "FInt(-7.447568350471557e-6, 9.872077498584987e-6)"
        );
    }
}
