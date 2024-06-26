use std::fmt::Display;

/// Simple 2d coordinate math struct inspired by glam IVec2 used in bevy.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub struct IVec2 {
    pub x: i32,
    pub y: i32,
}

/// Sugar macro
#[macro_export]
macro_rules! v {
    ($x: expr, $y: expr) => {
        IVec2::new($x, $y)
    };
}

impl IVec2 {
    pub const ZERO: IVec2 = IVec2 { x: 0, y: 0 };
    pub const ONE: IVec2 = IVec2 { x: 1, y: 1 };
    pub const X: IVec2 = IVec2 { x: 1, y: 0 };
    pub const Y: IVec2 = IVec2 { x: 0, y: 1 };
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    pub fn splat(v: i32) -> Self {
        v!(v, v)
    }
    pub fn length(&self) -> i32 {
        self.x.abs() + self.y.abs()
    }
    pub fn min(&self, min: Self) -> IVec2 {
        Self {
            x: self.x.min(min.x),
            y: self.y.min(min.y),
        }
    }
    pub fn max(&self, max: Self) -> IVec2 {
        Self {
            x: self.x.max(max.x),
            y: self.y.max(max.y),
        }
    }
    pub fn clamp(&self, min: Self, max: Self) -> IVec2 {
        Self {
            x: self.x.clamp(min.x, max.x),
            y: self.y.clamp(min.y, max.y),
        }
    }
}

impl Display for IVec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl std::ops::Add for IVec2 {
    type Output = IVec2;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Add<i32> for IVec2 {
    type Output = IVec2;

    fn add(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}

impl std::ops::Sub for IVec2 {
    type Output = IVec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Sub<i32> for IVec2 {
    type Output = IVec2;

    fn sub(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}

impl std::ops::Mul for IVec2 {
    type Output = IVec2;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl std::ops::Mul<i32> for IVec2 {
    type Output = IVec2;

    fn mul(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::Div for IVec2 {
    type Output = IVec2;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl std::ops::Div<i32> for IVec2 {
    type Output = IVec2;

    fn div(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl std::ops::Neg for IVec2 {
    type Output = IVec2;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}
