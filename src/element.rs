use strum_macros::IntoStaticStr;

use crate::{
    fint::FInt,
    shape::{Circle, Line, Point, Ray, Shape, ShapeTrait},
};
extern crate strum;

#[derive(Debug)]
pub struct LineAB {
    pub a: Point,
    pub b: Point,
}
impl LineAB {
    pub fn get_shape(&self) -> Line {
        // (x - x0) / dx = (y - y0) / dy
        let dx = self.b.0 - self.a.0;
        let dy = self.b.1 - self.a.1;
        let n = dx.sqr() + dy.sqr();
        let n_sqrt_inv = n.sqrt().inverse();
        let nx = dy * n_sqrt_inv;
        let minus_ny = dx * n_sqrt_inv;
        let d = (self.a.0 * dy - self.a.1 * dx) * n_sqrt_inv;
        let sign_ok = dy.always_positive() || (dy == FInt::new(0.0) && !dx.always_positive());
        if sign_ok {
            Line {
                nx,
                ny: minus_ny.negate(),
                d,
            }
        } else {
            Line {
                nx: nx.negate(),
                ny: minus_ny,
                d: d.negate(),
            }
        }
    }
}

#[derive(Debug)]
pub struct LineAV {
    pub a: Point,
    pub v: Point,
}
impl LineAV {
    pub fn get_shape(&self) -> Line {
        // (x - x0) / dx = (y - y0) / dy
        LineAB {
            a: self.a,
            b: Point(self.a.0 + self.v.0, self.a.1 + self.v.1),
        }
        .get_shape()
    }
}

#[derive(Debug)]
pub struct CircleCP {
    pub c: Point,
    pub p: Point,
}
impl CircleCP {
    pub fn get_shape(&self) -> Circle {
        let r2 = (self.p.0 - self.c.0).sqr() + (self.p.1 - self.c.1).sqr();
        Circle { c: self.c, r2 }
    }
}

#[derive(Debug)]
pub struct CircleCR {
    pub c: Point,
    pub r: FInt,
}
impl CircleCR {
    pub fn get_shape(&self) -> Circle {
        Circle {
            c: self.c,
            r2: self.r.sqr(),
        }
    }
}

#[derive(Debug)]
pub struct RayAV {
    pub a: Point,
    pub v: Point,
}
impl RayAV {
    pub fn get_shape(&self) -> Ray {
        let d_inv = (self.v.0.sqr() + self.v.1.sqr()).sqrt().inverse();
        Ray {
            a: self.a,
            v: Point(self.v.0 * d_inv, self.v.1 * d_inv),
        }
    }
}

#[derive(Debug)]
pub struct MidPerpAB {
    pub a: Point,
    pub b: Point,
}
impl MidPerpAB {
    pub fn get_shape(&self) -> Line {
        let p_mid = Point(
            (self.a.0 + self.b.0) * FInt::new(0.5),
            (self.a.1 + self.b.1) * FInt::new(0.5),
        );
        let v = Point(self.a.1 - self.b.1, self.b.0 - self.a.0);
        return LineAV { a: p_mid, v }.get_shape();
    }
}

#[derive(Debug, IntoStaticStr)]
pub enum Element {
    Point(Point),
    LineAB(LineAB),
    LineAV(LineAV),
    RayAV(RayAV),
    CircleCP(CircleCP),
    CircleCR(CircleCR),
    MidPerpAB(MidPerpAB),
    // SegmentAB(SegmentAB),
}
impl Element {
    pub fn get_shape(&self) -> Option<Shape> {
        match self {
            Element::Point(_point) => None,
            Element::LineAB(line_ab) => Some(Shape::Line(line_ab.get_shape())),
            Element::LineAV(line_av) => Some(Shape::Line(line_av.get_shape())),
            Element::RayAV(ray_av) => Some(Shape::Ray(ray_av.get_shape())),
            Element::CircleCP(circle_cp) => Some(Shape::Circle(circle_cp.get_shape())),
            Element::CircleCR(circle_cr) => Some(Shape::Circle(circle_cr.get_shape())),
            Element::MidPerpAB(mid_perp_ab) => Some(Shape::Line(mid_perp_ab.get_shape())),
        }
    }

    pub fn get_point_priority(&self, inspected_point: &Point) -> i32 {
        match self {
            Element::Point(point) => {
                if *point == *inspected_point {
                    10
                } else {
                    0
                }
            }
            Element::LineAB(line_ab) => {
                if line_ab.a == *inspected_point || line_ab.b == *inspected_point {
                    10
                } else if line_ab.get_shape().contains_point(inspected_point) {
                    2
                } else {
                    0
                }
            }
            Element::LineAV(line_av) => {
                if line_av.a == *inspected_point {
                    10
                } else if line_av.get_shape().contains_point(inspected_point) {
                    5
                } else {
                    0
                }
            }
            Element::RayAV(ray_av) => {
                if ray_av.a == *inspected_point {
                    10
                } else if ray_av.get_shape().contains_point(inspected_point) {
                    5
                } else {
                    0
                }
            }
            Element::CircleCP(circle_cp) => {
                if circle_cp.c == *inspected_point {
                    10
                } else if circle_cp.get_shape().contains_point(inspected_point) {
                    5
                } else {
                    0
                }
            }
            Element::CircleCR(circle_cr) => {
                if circle_cr.c == *inspected_point {
                    10
                } else if circle_cr.get_shape().contains_point(inspected_point) {
                    5
                } else {
                    0
                }
            }
            Element::MidPerpAB(mid_perp_ab) => {
                if mid_perp_ab.get_shape().contains_point(inspected_point) {
                    2
                } else {
                    0
                }
            }
        }
    }
}
