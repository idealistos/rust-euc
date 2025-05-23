use regex::Regex;
use std::{
    fmt::{self, Display, Formatter},
    hash::{DefaultHasher, Hasher},
    str::FromStr,
};

use crate::{element::LineAB, fint::FInt, hashset2::WithTwoHashes};

#[derive(Clone, Copy)]
pub struct Point(pub FInt, pub FInt);
impl PartialEq for Point {
    fn eq(&self, x: &Point) -> bool {
        x.0 == self.0 && x.1 == self.1
    }
}
impl Eq for Point {}
impl WithTwoHashes for Point {
    fn hash1<H: Hasher>(&self, state: &mut H) {
        self.0.hash1(state);
        self.1.hash1(state);
    }

    fn hash2<H: Hasher>(&self, state: &mut H) {
        self.0.hash2(state);
        self.1.hash2(state);
    }
}
impl Display for Point {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Pt(x={},y={})", self.0, self.1)
    }
}
impl fmt::Debug for Point {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}
impl Point {
    pub fn get_hash_1(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash1(&mut hasher);
        hasher.finish()
    }

    pub fn get_hash_2(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash2(&mut hasher);
        hasher.finish()
    }

    pub fn rotated_90_pos(&self) -> Point {
        Point(self.1.negate(), self.0)
    }

    pub fn well_formed(&self) -> bool {
        self.0.precise() && self.1.precise()
    }

    pub fn distance_to(&self, point: &Point) -> FInt {
        ((self.0 - point.0).sqr() + (self.1 - point.1).sqr()).sqrt()
    }

    pub fn is_collinear(&self, point: &Point) -> bool {
        self.0 * point.1 - self.1 * point.0 == FInt::new(0.0)
    }

    fn almost_equals(&self, point: &Point) -> bool {
        self.0.almost_equals(point.0) && self.1.almost_equals(point.1)
    }
}

pub trait ShapeTrait: Display {
    fn find_intersection_points(&self, s: &Shape) -> [Option<Point>; 2];
    fn contains_point(&self, point: &Point) -> bool;
    fn well_formed(&self) -> bool;
}

#[derive(Clone, Copy, Debug)]
pub struct Line {
    pub nx: FInt,
    pub ny: FInt,
    pub d: FInt,
}
impl PartialEq for Line {
    fn eq(&self, x: &Line) -> bool {
        x.nx == self.nx && x.ny == self.ny && x.d == self.d
    }
}
impl Eq for Line {}
impl WithTwoHashes for Line {
    fn hash1<H: Hasher>(&self, state: &mut H) {
        self.nx.hash1(state);
        self.ny.hash1(state);
        self.d.hash1(state);
    }

    fn hash2<H: Hasher>(&self, state: &mut H) {
        self.nx.hash2(state);
        self.ny.hash2(state);
        self.d.hash2(state);
    }
}
impl ShapeTrait for Line {
    fn find_intersection_points(&self, shape: &Shape) -> [Option<Point>; 2] {
        match shape {
            Shape::Line(line) => [self.intersect_with_line(&line), None],
            Shape::Ray(ray) => [ray.intersect_with_line(&self), None],
            Shape::Segment(segment) => [segment.intersect_with_line(&self), None],
            Shape::Circle(circle) => self.intersect_with_circle(&circle),
        }
    }

    fn contains_point(&self, point: &Point) -> bool {
        return self.nx * point.0 + self.ny * point.1 == self.d;
    }

    fn well_formed(&self) -> bool {
        self.nx.precise() && self.ny.precise() && self.d.precise()
    }
}
impl Display for Line {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Line(nx={},ny={},d={})", self.nx, self.ny, self.d)
    }
}
impl FromStr for Line {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Example: Line(nx=0.600,ny=-0.800,d=1.000)
        let regex = Regex::new(r"Line\(nx=([0-9.e-]+),ny=([0-9.e-]+),d=([0-9.e-]+)\)").unwrap();
        let captures = regex.captures(s).ok_or("Wrong format: ".to_string() + s)?;
        let nx: f64 = captures.get(1).unwrap().as_str().parse().unwrap();
        let ny: f64 = captures.get(2).unwrap().as_str().parse().unwrap();
        let d: f64 = captures.get(3).unwrap().as_str().parse().unwrap();
        Ok(Self {
            nx: FInt::new(nx),
            ny: FInt::new(ny),
            d: FInt::new(d),
        })
    }
}
impl Line {
    fn intersect_with_line(&self, line: &Line) -> Option<Point> {
        let den = self.nx * line.ny - line.nx * self.ny;
        if den == FInt::new(0.0) {
            return None;
        }
        let nom1 = self.d * line.ny - line.d * self.ny;
        let nom2 = self.d * line.nx - line.d * self.nx;
        let inv_den = den.inverse();
        Some(Point(nom1 * inv_den, nom2.negate() * inv_den))
    }

    fn intersect_with_circle(&self, circle: &Circle) -> [Option<Point>; 2] {
        // nx x' + ny y' = d' = d - nx cx - ny cy
        // x'^2 + y'^2 = r^2
        // y'^2 (nx^2 + ny^2) - 2 d' ny y' + d'^2 - nx^2 r^2 = 0
        // D = nx^2 (r^2 n^2 - d'^2)
        let n2 = self.nx.sqr() + self.ny.sqr();
        if n2 == FInt::new(0.0) {
            return [None, None];
        }
        let n2_inv = n2.inverse();
        let dp = self.d - self.nx * circle.c.0 - self.ny * circle.c.1;
        let det = circle.r2 * n2 - dp.sqr();
        if det == FInt::new(0.0) {
            return [
                Some(Point(
                    dp * self.nx * n2_inv + circle.c.0,
                    dp * self.ny * n2_inv + circle.c.1,
                )),
                None,
            ];
        }
        if !det.always_positive() {
            return [None, None];
        }
        let det_sqrt = det.sqrt();
        [
            Some(Point(
                (dp * self.nx + det_sqrt * self.ny) * n2_inv + circle.c.0,
                (dp * self.ny - det_sqrt * self.nx) * n2_inv + circle.c.1,
            )),
            Some(Point(
                (dp * self.nx - det_sqrt * self.ny) * n2_inv + circle.c.0,
                (dp * self.ny + det_sqrt * self.nx) * n2_inv + circle.c.1,
            )),
        ]
    }

    fn get_direction(&self) -> Option<Point> {
        Some(Point(self.ny.negate(), self.nx))
    }

    fn almost_equals(&self, line: &Line) -> bool {
        (self.nx.almost_equals(line.nx)
            && self.ny.almost_equals(line.ny)
            && self.d.almost_equals(line.d))
            || (self.nx.almost_equals(line.nx.negate())
                && self.ny.almost_equals(line.ny.negate())
                && self.d.almost_equals(line.d.negate()))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Circle {
    pub c: Point,
    pub r2: FInt,
}
impl PartialEq for Circle {
    fn eq(&self, x: &Circle) -> bool {
        x.c == self.c && x.r2 == self.r2
    }
}
impl Eq for Circle {}
impl WithTwoHashes for Circle {
    fn hash1<H: Hasher>(&self, state: &mut H) {
        self.c.hash1(state);
        self.r2.hash1(state);
    }

    fn hash2<H: Hasher>(&self, state: &mut H) {
        self.c.hash2(state);
        self.r2.hash2(state);
    }
}
impl ShapeTrait for Circle {
    fn find_intersection_points(&self, shape: &Shape) -> [Option<Point>; 2] {
        match shape {
            Shape::Line(line) => line.intersect_with_circle(self),
            Shape::Ray(ray) => ray.intersect_with_circle(self),
            Shape::Segment(segment) => segment.intersect_with_circle(&self),
            Shape::Circle(circle) => self.intersect_with_circle(&circle),
        }
    }

    fn contains_point(&self, point: &Point) -> bool {
        return (point.0 - self.c.0).sqr() + (point.1 - self.c.1).sqr() == self.r2;
    }

    fn well_formed(&self) -> bool {
        self.c.well_formed() && self.r2.precise()
    }
}
impl Display for Circle {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Circle(c.x={},c.y={},r2={})",
            self.c.0, self.c.1, self.r2
        )
    }
}
impl FromStr for Circle {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Example: Circle(c.x=0.600,c.y=-0.800,r2=1.000)
        let regex =
            Regex::new(r"Circle\(c\.x=([0-9.e-]+),c\.y=([0-9.e-]+),r2=([0-9.e-]+)\)").unwrap();
        let captures = regex.captures(s).ok_or("Wrong format: ".to_string() + s)?;
        let cx: f64 = captures.get(1).unwrap().as_str().parse().unwrap();
        let cy: f64 = captures.get(2).unwrap().as_str().parse().unwrap();
        let r2: f64 = captures.get(3).unwrap().as_str().parse().unwrap();
        Ok(Self {
            c: Point(FInt::new(cx), FInt::new(cy)),
            r2: FInt::new(r2),
        })
    }
}
impl Circle {
    fn intersect_with_circle(&self, circle: &Circle) -> [Option<Point>; 2] {
        let cx = circle.c.0 - self.c.0;
        let cy = circle.c.1 - self.c.1;
        let cn = cx * cx + cy * cy;
        if cn == FInt::new(0.0) {
            return [None, None];
        }
        let m = (circle.r2 - self.r2 - cx.sqr() - cy.sqr()) * FInt::new(0.5);
        let det = cn * self.r2 - m.sqr();
        let cn_inv_neg = cn.inverse().negate();
        if det == FInt::new(0.0) {
            return [
                Some(Point(
                    m * cx * cn_inv_neg + self.c.0,
                    m * cy * cn_inv_neg + self.c.1,
                )),
                None,
            ];
        }
        if !det.always_positive() {
            return [None, None];
        }
        let sign = FInt::new(if cy.always_positive() { 1.0 } else { -1.0 });
        let det_sqrt = det.sqrt() * sign;
        return [
            Some(Point(
                (m * cx - det_sqrt * cy) * cn_inv_neg + self.c.0,
                (m * cy + det_sqrt * cx) * cn_inv_neg + self.c.1,
            )),
            Some(Point(
                (m * cx + det_sqrt * cy) * cn_inv_neg + self.c.0,
                (m * cy - det_sqrt * cx) * cn_inv_neg + self.c.1,
            )),
        ];
    }

    fn almost_equals(&self, circle: &Circle) -> bool {
        self.c.almost_equals(&circle.c) && self.r2.almost_equals(circle.r2)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub a: Point,
    pub v: Point,
}
impl PartialEq for Ray {
    fn eq(&self, x: &Ray) -> bool {
        x.a == self.a && x.v == self.v
    }
}
impl Eq for Ray {}
impl WithTwoHashes for Ray {
    fn hash1<H: Hasher>(&self, state: &mut H) {
        self.a.hash1(state);
        self.v.hash1(state);
    }

    fn hash2<H: Hasher>(&self, state: &mut H) {
        self.a.hash2(state);
        self.v.hash2(state);
    }
}
impl ShapeTrait for Ray {
    fn find_intersection_points(&self, shape: &Shape) -> [Option<Point>; 2] {
        match shape {
            Shape::Line(line) => [self.intersect_with_line(&line), None],
            Shape::Ray(ray) => [self.intersect_with_ray(&ray), None],
            Shape::Segment(segment) => [segment.intersect_with_ray(&self), None],
            Shape::Circle(circle) => self.intersect_with_circle(&circle),
        }
    }

    fn contains_point(&self, point: &Point) -> bool {
        let line = self.as_line();
        line.contains_point(point)
            && !((self.a.0 - point.0) * self.v.0 + (self.a.1 - point.1) * self.v.1)
                .always_positive()
    }

    fn well_formed(&self) -> bool {
        self.a.well_formed() && self.v.well_formed()
    }
}
impl Display for Ray {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Ray(a={},v={})", self.a, self.v)
    }
}
impl FromStr for Ray {
    type Err = String;
    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Err("Not supported".to_string())
    }
}
impl Ray {
    fn as_line(&self) -> Line {
        if (FInt::new(0.0) - self.v.1).always_positive() {
            Line {
                nx: FInt::new(0.0) - self.v.1,
                ny: self.v.0,
                d: self.a.1 * self.v.0 - self.a.0 * self.v.1,
            }
        } else if self.v.1.always_positive() {
            Line {
                nx: self.v.1,
                ny: FInt::new(0.0) - self.v.0,
                d: self.a.0 * self.v.1 - self.a.1 * self.v.0,
            }
        } else if self.v.0.always_positive() {
            Line {
                nx: FInt::new(0.0) - self.v.1,
                ny: self.v.0,
                d: self.a.1 * self.v.0 - self.a.0 * self.v.1,
            }
        } else {
            Line {
                nx: self.v.1,
                ny: FInt::new(0.0) - self.v.0,
                d: self.a.0 * self.v.1 - self.a.1 * self.v.0,
            }
        }
    }

    fn intersect_with_line(&self, line: &Line) -> Option<Point> {
        let line1 = self.as_line();
        let point = line1.intersect_with_line(line)?;
        if self.contains_point(&point) {
            Some(point)
        } else {
            None
        }
    }

    fn intersect_with_ray(&self, ray: &Ray) -> Option<Point> {
        let line1 = self.as_line();
        let point = line1.intersect_with_line(&ray.as_line())?;
        if self.contains_point(&point) && ray.contains_point(&point) {
            Some(point)
        } else {
            None
        }
    }

    fn intersect_with_circle(&self, circle: &Circle) -> [Option<Point>; 2] {
        let line1 = self.as_line();
        let points = line1.intersect_with_circle(circle);
        let point1 = match points[0] {
            None => None,
            Some(point) => {
                if self.contains_point(&point) {
                    Some(point)
                } else {
                    None
                }
            }
        };
        let point2 = match points[1] {
            None => None,
            Some(point) => {
                if self.contains_point(&point) {
                    Some(point)
                } else {
                    None
                }
            }
        };
        if point1.is_none() {
            [point2, point1]
        } else {
            [point1, point2]
        }
    }

    fn get_direction(&self) -> Option<Point> {
        Some(self.v)
    }

    // True if the directions match or the endpoint is in the ray ( = point + t * v for some positive t)
    fn intersects_with_collinear_ray(&self, point: &Point, v: &Point) -> bool {
        let proj = point.0 * v.0 + point.1 * v.1;
        let proj1 = self.a.0 * v.0 + self.a.1 * v.1;
        return (proj1 - proj).always_positive()
            || (v.0 * self.v.0 + v.1 * self.v.1).always_positive();
    }

    fn almost_equals(&self, ray: &Ray) -> bool {
        self.a.almost_equals(&ray.a) && self.v.almost_equals(&ray.v)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Segment {
    pub a: Point,
    pub b: Point,
}
impl PartialEq for Segment {
    fn eq(&self, x: &Segment) -> bool {
        x.a == self.a && x.b == self.b
    }
}
impl Eq for Segment {}
impl WithTwoHashes for Segment {
    fn hash1<H: Hasher>(&self, state: &mut H) {
        self.a.hash1(state);
        self.b.hash1(state);
    }

    fn hash2<H: Hasher>(&self, state: &mut H) {
        self.a.hash2(state);
        self.b.hash2(state);
    }
}
impl ShapeTrait for Segment {
    fn find_intersection_points(&self, shape: &Shape) -> [Option<Point>; 2] {
        match shape {
            Shape::Line(line) => [self.intersect_with_line(&line), None],
            Shape::Ray(ray) => [self.intersect_with_ray(&ray), None],
            Shape::Segment(segment) => [self.intersect_with_segment(&segment), None],
            Shape::Circle(circle) => self.intersect_with_circle(&circle),
        }
    }

    fn contains_point(&self, point: &Point) -> bool {
        let line = self.as_line();
        line.contains_point(point)
            && !((self.a.0 - point.0) * (self.b.0 - point.0)
                + (self.a.1 - point.1) * (self.b.1 - point.1))
                .always_positive()
    }

    fn well_formed(&self) -> bool {
        self.a.well_formed() && self.b.well_formed()
    }
}
impl Display for Segment {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Segment(a={},b={})", self.a, self.b)
    }
}
impl FromStr for Segment {
    type Err = String;
    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Err("Not supported".to_string())
    }
}
impl Segment {
    fn as_line(&self) -> Line {
        LineAB {
            a: self.a,
            b: self.b,
        }
        .get_shape()
    }

    fn intersect_with_line(&self, line: &Line) -> Option<Point> {
        let line1 = self.as_line();
        let point = line1.intersect_with_line(line)?;
        if self.contains_point(&point) {
            Some(point)
        } else {
            None
        }
    }

    fn intersect_with_ray(&self, ray: &Ray) -> Option<Point> {
        let line1 = self.as_line();
        let point = line1.intersect_with_line(&ray.as_line())?;
        if self.contains_point(&point) && ray.contains_point(&point) {
            Some(point)
        } else {
            None
        }
    }

    fn intersect_with_segment(&self, segment: &Segment) -> Option<Point> {
        let line1 = self.as_line();
        let point = line1.intersect_with_line(&segment.as_line())?;
        if self.contains_point(&point) && segment.contains_point(&point) {
            Some(point)
        } else {
            None
        }
    }

    fn intersect_with_circle(&self, circle: &Circle) -> [Option<Point>; 2] {
        let line1 = self.as_line();
        let points = line1.intersect_with_circle(circle);
        let point1 = match points[0] {
            None => None,
            Some(point) => {
                if self.contains_point(&point) {
                    Some(point)
                } else {
                    None
                }
            }
        };
        let point2 = match points[1] {
            None => None,
            Some(point) => {
                if self.contains_point(&point) {
                    Some(point)
                } else {
                    None
                }
            }
        };
        if point1.is_none() {
            [point2, point1]
        } else {
            [point1, point2]
        }
    }

    fn get_direction(&self) -> Option<Point> {
        self.as_line().get_direction()
    }

    // True if at least one endpoint is in the ray ( = point + t * v for some positive t)
    fn intersects_with_collinear_ray(&self, point: &Point, v: &Point) -> bool {
        let proj = point.0 * v.0 + point.1 * v.1;
        let proj1 = self.a.0 * v.0 + self.a.1 * v.1;
        let proj2 = self.b.0 * v.0 + self.b.1 * v.1;
        return (proj1 - proj).always_positive() || (proj2 - proj).always_positive();
    }

    // We don't take into account that a and b can be reversed
    fn almost_equals(&self, segment: &Segment) -> bool {
        self.a.almost_equals(&segment.a) && self.b.almost_equals(&segment.b)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Shape {
    Line(Line),
    Ray(Ray),
    Segment(Segment),
    Circle(Circle),
}
impl WithTwoHashes for Shape {
    fn hash1<H: Hasher>(&self, state: &mut H) {
        match self {
            Shape::Line(line) => line.hash1(state),
            Shape::Ray(ray) => ray.hash1(state),
            Shape::Segment(segment) => segment.hash1(state),
            Shape::Circle(circle) => circle.hash1(state),
        }
    }

    fn hash2<H: Hasher>(&self, state: &mut H) {
        match self {
            Shape::Line(line) => line.hash2(state),
            Shape::Ray(ray) => ray.hash2(state),
            Shape::Segment(segment) => segment.hash2(state),
            Shape::Circle(circle) => circle.hash2(state),
        }
    }
}
impl Display for Shape {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Shape::Line(line) => line.fmt(f),
            Shape::Ray(ray) => ray.fmt(f),
            Shape::Segment(segment) => segment.fmt(f),
            Shape::Circle(circle) => circle.fmt(f),
        }
    }
}
impl ShapeTrait for Shape {
    fn find_intersection_points(&self, shape: &Shape) -> [Option<Point>; 2] {
        match self {
            Shape::Line(line) => line.find_intersection_points(shape),
            Shape::Ray(ray) => ray.find_intersection_points(shape),
            Shape::Segment(segment) => segment.find_intersection_points(shape),
            Shape::Circle(circle) => circle.find_intersection_points(shape),
        }
    }

    fn contains_point(&self, point: &Point) -> bool {
        match self {
            Shape::Line(line) => line.contains_point(point),
            Shape::Ray(ray) => ray.contains_point(point),
            Shape::Segment(segment) => segment.contains_point(point),
            Shape::Circle(circle) => circle.contains_point(point),
        }
    }

    fn well_formed(&self) -> bool {
        match self {
            Shape::Line(line) => line.well_formed(),
            Shape::Ray(ray) => ray.well_formed(),
            Shape::Segment(segment) => segment.well_formed(),
            Shape::Circle(circle) => circle.well_formed(),
        }
    }
}
impl FromStr for Shape {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pos = s.find('(').ok_or("No '('".to_string())?;
        let head = &s[..pos];
        match head {
            "Line" => Line::from_str(s).map(|line| Shape::Line(line)),
            "Circle" => Circle::from_str(s).map(|circle| Shape::Circle(circle)),
            "Ray" => Ray::from_str(s).map(|ray| Shape::Ray(ray)),
            "Segment" => Segment::from_str(s).map(|segment| Shape::Segment(segment)),
            _ => Err("Wrong head: {}".to_string() + head),
        }
    }
}
impl Shape {
    pub fn get_direction(&self) -> Option<Point> {
        match self {
            Shape::Line(line) => line.get_direction(),
            Shape::Ray(ray) => ray.get_direction(),
            Shape::Segment(segment) => segment.get_direction(),
            Shape::Circle(_circle) => None,
        }
    }

    pub fn intersects_with_collinear_ray(&self, point: &Point, v: &Point) -> bool {
        match self {
            Shape::Line(_line) => true,
            Shape::Ray(ray) => ray.intersects_with_collinear_ray(point, v),
            Shape::Segment(segment) => segment.intersects_with_collinear_ray(point, v),
            Shape::Circle(_circle) => false,
        }
    }

    pub fn almost_equals(&self, shape: &Shape) -> bool {
        match (self, shape) {
            (Shape::Line(x), Shape::Line(y)) => x.almost_equals(y),
            (Shape::Circle(x), Shape::Circle(y)) => x.almost_equals(y),
            (Shape::Ray(x), Shape::Ray(y)) => x.almost_equals(y),
            (Shape::Segment(x), Shape::Segment(y)) => x.almost_equals(y),
            _ => false,
        }
    }
}

mod tests {
    #[cfg(test)]
    use super::*;

    #[test]
    fn test_from_str_for_line() {
        let line = Line {
            nx: FInt::new(0.6),
            ny: FInt::new(0.8),
            d: FInt::new(1.4),
        };
        let as_str = format!("{}", line);
        assert_eq!(as_str, "Line(nx=0.600,ny=0.800,d=1.400)".to_string());
        let from_str = Line::from_str(as_str.as_str()).unwrap();
        assert_eq!(from_str, line);
    }

    #[test]
    fn test_from_str_for_circle() {
        let circle = Circle {
            c: Point(FInt::new(0.6), FInt::new(0.8)),
            r2: FInt::new(1.4),
        };
        let as_str = format!("{}", circle);
        assert_eq!(as_str, "Circle(c.x=0.600,c.y=0.800,r2=1.400)".to_string());
        let from_str = Circle::from_str(as_str.as_str()).unwrap();
        assert_eq!(from_str, circle);
    }

    #[test]
    fn test_line_intersection() {
        let line1 = Line {
            nx: FInt::new(0.6),
            ny: FInt::new(0.8),
            d: FInt::new(1.4),
        };
        let line2 = Line {
            nx: FInt::new(1.0),
            ny: FInt::new(0.0),
            d: FInt::new(1.0),
        };
        let points = line1.find_intersection_points(&Shape::Line(line2));
        assert_eq!(points, [Some(Point(FInt::new(1.0), FInt::new(1.0))), None])
    }

    #[test]
    fn test_line_and_circle_intersection() {
        let line = Line {
            nx: FInt::new(1.0),
            ny: FInt::new(0.0),
            d: FInt::new(3.0),
        };
        let circle = Circle {
            c: Point(FInt::new(0.0), FInt::new(0.0)),
            r2: FInt::new(25.0),
        };
        let points = line.find_intersection_points(&Shape::Circle(circle));
        assert_eq!(
            points,
            [
                Some(Point(FInt::new(3.0), FInt::new(-4.0))),
                Some(Point(FInt::new(3.0), FInt::new(4.0)))
            ]
        )
    }

    #[test]
    fn test_two_circles_intersection() {
        let circle1 = Circle {
            c: Point(FInt::new(0.0), FInt::new(0.0)),
            r2: FInt::new(25.0),
        };
        let circle2 = Circle {
            c: Point(FInt::new(-5.0), FInt::new(-2.0)),
            r2: FInt::new(100.0),
        };
        let points = circle1.find_intersection_points(&Shape::Circle(circle2));
        assert_eq!(
            points,
            [
                Some(Point(FInt::new(143.0 / 29.0), FInt::new(-24.0 / 29.0))),
                Some(Point(FInt::new(3.0), FInt::new(4.0))),
            ]
        );
        let points_reversed = circle2.find_intersection_points(&Shape::Circle(circle1));
        assert_eq!(
            points_reversed,
            [
                Some(Point(FInt::new(143.0 / 29.0), FInt::new(-24.0 / 29.0))),
                Some(Point(FInt::new(3.0), FInt::new(4.0))),
            ]
        );
    }

    #[test]
    fn line_contains_point() {
        let line = Line {
            nx: FInt::new(0.6),
            ny: FInt::new(0.8),
            d: FInt::new(1.4),
        };
        assert_eq!(
            line.contains_point(&Point(FInt::new(1.0), FInt::new(1.0))),
            true
        );
        assert_eq!(
            line.contains_point(&Point(FInt::new(0.0), FInt::new(0.0))),
            false
        );
    }

    #[test]
    fn circle_contains_point() {
        let circle = Circle {
            c: Point(FInt::new(0.0), FInt::new(0.0)),
            r2: FInt::new(25.0),
        };
        assert_eq!(
            circle.contains_point(&Point(FInt::new(3.0), FInt::new(4.0))),
            true
        );
        assert_eq!(
            circle.contains_point(&Point(FInt::new(0.0), FInt::new(0.0))),
            false
        );
    }

    #[test]
    fn parallel_lines() {
        let line1 = Line {
            nx: FInt::new_with_bounds(0.49999999999999806, 0.5000000000000017),
            ny: FInt::new_with_bounds(-0.8660254037844396, -0.8660254037844383),
            d: FInt::new_with_bounds(0.8660254037844365, 0.8660254037844412),
        };
        let line2 = Line {
            nx: FInt::new_with_bounds(0.4999999999999999, 0.49999999999999994),
            ny: FInt::new_with_bounds(-0.866025403784439, -0.8660254037844385),
            d: FInt::new_with_bounds(-2e-323, 1.5e-323),
        };
        let point = line1.intersect_with_line(&line2);
        assert_eq!(point, None);
    }

    #[test]
    fn test_two_circles_intersect_in_one_point() {
        let c1 = Point(FInt::new(-1.0), FInt::new(0.0));
        let c2 = Point(FInt::new(-2.2345), FInt::new(-1.0));
        let circle1 = Circle {
            c: c1,
            r2: FInt::new(1.2345 * 1.2345 + 1.0),
        };
        let circle2 = Circle {
            c: c2,
            r2: FInt::new(2.469 * 2.469 + 4.0),
        };
        let points = circle1.find_intersection_points(&Shape::Circle(circle2));
        assert_eq!(points[1], None);
        assert_eq!(points[0], Some(Point(FInt::new(0.2345), FInt::new(1.0))));
        let point = points[0].unwrap();
        let d = (point.0 - c1.0) * (point.0 - c1.0) + (point.1 - c1.1) * (point.1 - c1.1);
        assert_eq!(d, circle1.r2);
        let d = (point.0 - c2.0) * (point.0 - c2.0) + (point.1 - c2.1) * (point.1 - c2.1);
        assert_eq!(d, circle2.r2);
    }
}
