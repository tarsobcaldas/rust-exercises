use std::f64::consts::PI;
// use clap::{ValueEnum};

#[derive(Debug, Clone)]
pub enum TwoDShape {
    Square {
        side: f64,
    },
    Circle {
        radius: f64,
    },
    Triangle {
        base: f64,
        height: f64,
        side2: f64,
        side3: f64,
    },
    Rectangle {
        width: f64,
        height: f64,
    },
}

#[derive(Debug, Clone)]
pub enum ThreeDShape {
    Sphere {
        radius: f64,
    },
    Cilinder {
        radius: f64,
        height: f64,
    },
    Cone {
        radius: f64,
        height: f64,
    },
    Cube { 
        side: f64,
    },
    Tetrahedron {
        side: f64,
    },
}

#[derive(Debug, Clone)]
pub enum Shape {
    TwoD(TwoDShape),
    ThreeD(ThreeDShape),
}

impl TwoDShape {
    pub fn area(&self) -> f64 {
        use TwoDShape::*;
        match self {
            Square { side } => side * side,
            Circle { radius } => PI * radius * radius,
            Triangle { base, height, side2: _, side3: _ } => 0.5 * base * height,
            Rectangle { width, height } => width * height,
        }
    }

    pub fn perimeter(&self) -> f64 {
        use TwoDShape::*;
        match self {
            Square { side } => 4.0 * side,
            Circle { radius } => 2.0 * PI * radius,
            Rectangle { width, height } => 2.0 * (width + height),
            Triangle { base, side2, side3, height: _ } => base + side2 + side3,
        }
    }
}

impl ThreeDShape {
    pub fn volume(&self) -> f64 {
        use ThreeDShape::*;
        match self {
            Sphere { radius } => 4.0 / 3.0 * PI * radius * radius * radius,
            Cilinder { radius, height } => PI * radius * radius * height,
            Cone { radius, height } => 1.0 / 3.0 * PI * radius * radius * height,
            Cube { side } => side * side * side,
            Tetrahedron { side } => side * side * side / 6.0 * 2.0_f64.sqrt(),
        }
    }

    pub fn surface_area(&self) -> f64 {
        use ThreeDShape::*;
        match self {
            Sphere { radius } => 4.0 * PI * radius * radius,
            Cilinder { radius, height } => 2.0 * PI * radius * height + 2.0 * PI * radius * radius,
            Cone { radius, height } => PI * radius * (radius + (radius.powi(2) + height.powi(2)).sqrt()),
            Cube { side } => 6.0 * side * side,
            Tetrahedron { side } => 3.0_f64.sqrt() * side * side,
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    NotA2DShape,
    NotA3DShape,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use ErrorKind::*;
        match self {
            NotA2DShape => write!(f, "Not a 2D shape"),
            NotA3DShape => write!(f, "Not a 3D shape"),
        }
    }
}

impl Shape {
    pub fn area(&self) -> Result<f64, ErrorKind> {
        use Shape::*;
        match self {
            TwoD(s) => Ok(s.area()),
            ThreeD(s) => Ok(s.surface_area()),
        }
    }

    pub fn perimeter(&self) -> Result<f64, ErrorKind> {
        use ErrorKind::*;
        use Shape::*;
        match self {
            TwoD(s) => Ok(s.perimeter()),
            ThreeD(_) => Err(NotA2DShape),
        }
    }

    pub fn volume(&self) -> Result<f64, ErrorKind> {
        use ErrorKind::*;
        use Shape::*;
        match self {
            TwoD(_) => Err(NotA3DShape),
            ThreeD(s) => Ok(s.volume()),
        }
    }
}
