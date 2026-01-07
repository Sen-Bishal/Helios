// TODO : Conversion function in a diff unit module?

use std::ops::{Add, Sub, Mul, Div, Neg};
use std::fmt;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Distance(i128);

// Conversion constants
const PICOMETERS_PER_NANOMETER: i128 = 1_000;
const PICOMETERS_PER_MICROMETER: i128 = 1_000_000;
const PICOMETERS_PER_MILLIMETER: i128 = 1_000_000_000;
const PICOMETERS_PER_METER: i128 = 1_000_000_000_000;

impl Distance {
    /// Create from picometers
    #[inline]
    pub const fn from_picometers(pm: i128) -> Self {
        Self(pm)
    }

    /// Create from nanometers
    #[inline]
    pub const fn from_nanometers(nm: i128) -> Self {
        Self(nm * PICOMETERS_PER_NANOMETER)
    }

    /// Create from micrometers
    #[inline]
    pub const fn from_micrometers(um: i128) -> Self {
        Self(um * PICOMETERS_PER_MICROMETER)
    }

    /// Create from millimeters
    #[inline]
    pub const fn from_millimeters(mm: i128) -> Self {
        Self(mm * PICOMETERS_PER_MILLIMETER)
    }

    /// Create from meters
    #[inline]
    pub const fn from_meters(m: i128) -> Self {
        Self(m * PICOMETERS_PER_METER)
    }

    /// Create from floating-point meters (for initialization only)
    #[inline]
    pub fn from_meters_f64(m: f64) -> Self {
        Self((m * PICOMETERS_PER_METER as f64) as i128)
    }

    /// Convert to picometers
    #[inline]
    pub const fn as_picometers(&self) -> i128 {
        self.0
    }

    #[inline]
    pub fn as_meters_f64(&self) -> f64 {
        self.0 as f64 / PICOMETERS_PER_METER as f64
    }
    #[inline]
    pub fn as_nanometers_f64(&self) -> f64 {
        self.0 as f64 / PICOMETERS_PER_NANOMETER as f64
    }
    pub const ZERO: Distance = Distance(0);
    pub const ONE_METER: Distance = Distance(PICOMETERS_PER_METER);
    pub const ONE_NANOMETER: Distance = Distance(PICOMETERS_PER_NANOMETER);
    #[inline]
    pub const fn abs(&self) -> Self {
        if self.0 < 0 {
            Self(-self.0)
        } else {
            *self
        }
    }
}
impl Add for Distance {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}
impl Sub for Distance {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}
impl Neg for Distance {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self(-self.0)
    }
}
impl Mul<i128> for Distance {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: i128) -> Self {
        Self(self.0 * rhs)
    }
}

impl Div<i128> for Distance {
    type Output = Self;
    #[inline]
    fn div(self, rhs: i128) -> Self {
        Self(self.0 / rhs)
    }
}

impl fmt::Display for Distance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let meters = self.as_meters_f64();
        if meters.abs() >= 1.0 {
            write!(f, "{:.12} m", meters)
        } else if meters.abs() >= 1e-3 {
            write!(f, "{:.9} mm", meters * 1e3)
        } else if meters.abs() >= 1e-6 {
            write!(f, "{:.6} μm", meters * 1e6)
        } else if meters.abs() >= 1e-9 {
            write!(f, "{:.3} nm", meters * 1e9)
        } else {
            write!(f, "{} pm", self.0)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position3D {
    pub x: Distance,
    pub y: Distance,
    pub z: Distance,
}

impl Position3D {
    pub const fn new(x: Distance, y: Distance, z: Distance) -> Self {
        Self { x, y, z }
    }

    pub const fn zero() -> Self {
        Self {
            x: Distance::ZERO,
            y: Distance::ZERO,
            z: Distance::ZERO,
        }
    }

    pub fn to_vec3(&self) -> glam::Vec3 {
        glam::Vec3::new(
            self.x.as_meters_f64() as f32,
            self.y.as_meters_f64() as f32,
            self.z.as_meters_f64() as f32,
        )
    }

    pub fn from_vec3(v: glam::Vec3) -> Self {
        Self {
            x: Distance::from_meters_f64(v.x as f64),
            y: Distance::from_meters_f64(v.y as f64),
            z: Distance::from_meters_f64(v.z as f64),
        }
    }
    pub fn distance_to(&self, other: &Position3D) -> Distance {
        let dx = (self.x.as_meters_f64() - other.x.as_meters_f64()).powi(2);
        let dy = (self.y.as_meters_f64() - other.y.as_meters_f64()).powi(2);
        let dz = (self.z.as_meters_f64() - other.z.as_meters_f64()).powi(2);
        Distance::from_meters_f64((dx + dy + dz).sqrt())
    }
}

impl Add for Position3D {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Position3D {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_precision() {
        let d1 = Distance::from_meters(1);
        let d2 = Distance::from_picometers(1);
        let sum = d1 + d2;
        assert_eq!(sum.as_picometers(), 1_000_000_000_001);
    }

    #[test]
    fn test_distance_conversion() {
        let nm = Distance::from_nanometers(13);
        assert_eq!(nm.as_nanometers_f64(), 13.0);
    }

    #[test]
    fn test_position_operations() {
        let p1 = Position3D::new(
            Distance::from_nanometers(100),
            Distance::from_nanometers(200),
            Distance::from_nanometers(300),
        );
        let p2 = Position3D::new(
            Distance::from_nanometers(50),
            Distance::from_nanometers(50),
            Distance::from_nanometers(50),
        );
        
        let diff = p1 - p2;
        assert_eq!(diff.x.as_nanometers_f64(), 50.0);
    }
}