use std::collections::HashSet;
use std::fs::read_to_string;
use std::str::FromStr;

use crate::shape::Shape;
use crate::Computation;
use crate::FInt;
use crate::VecLengths;
use private::*;
use svg::Document;

pub trait DrawState {
    fn draw_state(&mut self, filename: String, hw: f64, only_included_in_deps: HashSet<u64>);
    fn draw_solution(&mut self, filename: String, hw: f64);
    fn draw_shapes(shapes: &Vec<Shape>, filename: String, hw: f64);
    fn draw_shapes_from_file(input_filename: String, filename: String, hw: f64);
}
impl<'a> DrawState for Computation<'a> {
    fn draw_state(&mut self, filename: String, hw: f64, only_included_in_deps: HashSet<u64>) {
        let colors = [
            "darkgray",
            "blue",
            "green",
            "red",
            "purple",
            "brown",
            "deepskyblue",
            "darkcyan",
            "maroon",
            "lightpink",
        ];
        let mut document = Document::new().set("viewBox", (0, 0, 800, 800));

        for i in 0..self.shape_origins.len_i32() {
            let mut include = only_included_in_deps.is_empty();
            for deps in &only_included_in_deps {
                let origin = &self.shape_origins[i as usize];
                if self.combine_deps(*deps, origin.deps, None) == *deps {
                    include = true;
                    break;
                }
            }
            if !include {
                continue;
            }
            let shape_origin = &self.shape_origins[i as usize];
            let deps_count = self.get_deps_count(shape_origin.deps);
            let color = colors[(deps_count % 10) as usize];
            let stroke_width = if deps_count <= 2 {
                3
            } else {
                (deps_count / 10) + 1
            };
            document = match shape_origin.get_shape() {
                Shape::Line(line) => document.add(
                    svg::node::element::Line::new()
                        .set(
                            "x1",
                            Self::to_svg(line.nx * line.d - FInt::new(3.0 * hw) * line.ny, hw),
                        )
                        .set(
                            "y1",
                            Self::to_svg_flip(line.ny * line.d + FInt::new(3.0 * hw) * line.nx, hw),
                        )
                        .set(
                            "x2",
                            Self::to_svg(line.nx * line.d + FInt::new(3.0 * hw) * line.ny, hw),
                        )
                        .set(
                            "y2",
                            Self::to_svg_flip(line.ny * line.d - FInt::new(3.0 * hw) * line.nx, hw),
                        )
                        .set("fill", "none")
                        .set("stroke", color)
                        .set("stroke-width", stroke_width),
                ),
                Shape::Ray(ray) => document.add(
                    svg::node::element::Line::new()
                        .set("x1", Self::to_svg(ray.a.0, hw))
                        .set("y1", Self::to_svg_flip(ray.a.1, hw))
                        .set(
                            "x2",
                            Self::to_svg(ray.a.0 + FInt::new(3.0 * hw) * ray.v.0, hw),
                        )
                        .set(
                            "y2",
                            Self::to_svg_flip(ray.a.1 + FInt::new(3.0 * hw) * ray.v.1, hw),
                        )
                        .set("fill", "none")
                        .set("stroke", color)
                        .set("stroke-width", stroke_width),
                ),
                Shape::Circle(circle) => document.add(
                    svg::node::element::Circle::new()
                        .set("cx", Self::to_svg(circle.c.0, hw))
                        .set("cy", Self::to_svg_flip(circle.c.1, hw))
                        .set("r", Self::to_svg(circle.r2.sqrt() - FInt::new(hw), hw))
                        .set("fill", "none")
                        .set("stroke", color)
                        .set("stroke-width", stroke_width),
                ),
            };
        }
        for i in 0..self.point_origins.len_i32() {
            let mut include = false;
            for deps in &only_included_in_deps {
                let origin = &self.point_origins[i as usize];
                if self.combine_deps(*deps, origin.deps, None) == *deps {
                    include = true;
                    break;
                }
            }
            if !include {
                continue;
            }
            let point_origin = &self.point_origins[i as usize];
            let deps_count = self.get_deps_count(point_origin.deps);
            let color = colors[(deps_count % 10) as usize];
            document = document.add(
                svg::node::element::Circle::new()
                    .set("cx", Self::to_svg(point_origin.point.0, hw))
                    .set("cy", Self::to_svg_flip(point_origin.point.1, hw))
                    .set("r", 2)
                    .set("fill", "black")
                    .set("stroke", color)
                    .set("stroke-width", 2),
            );
        }
        svg::save(filename, &document).unwrap();
    }

    fn draw_solution(&mut self, filename: String, hw: f64) {
        let deps_list = self.get_solution_deps_list();
        self.draw_state(filename, hw, deps_list)
    }

    fn draw_shapes_from_file(input_filename: String, filename: String, hw: f64) {
        let shapes: Vec<Shape> = read_to_string(input_filename)
            .unwrap()
            .lines()
            .filter_map(|s| Shape::from_str(s).ok())
            .collect();
        Self::draw_shapes(&shapes, filename, hw);
    }

    fn draw_shapes(shapes: &Vec<Shape>, filename: String, hw: f64) {
        let colors = [
            "darkgray",
            "blue",
            "green",
            "red",
            "purple",
            "brown",
            "deepskyblue",
            "darkcyan",
            "maroon",
            "lightpink",
        ];
        let mut document = Document::new().set("viewBox", (0, 0, 800, 800));

        for i in 0..shapes.len() {
            let color = colors[(i % 10) as usize];
            let stroke_width = if i <= shapes.len() - 3 { 3 } else { 2 };

            document = Self::draw_shape(document, &shapes[i], hw, color, stroke_width);
        }
        svg::save(filename, &document).unwrap();
    }
}

mod private {
    use svg::Document;

    use crate::shape::Shape;
    use crate::Computation;
    use crate::FInt;

    pub trait DrawStateHelper {
        fn to_svg(value: FInt, hw: f64) -> i32;
        fn to_svg_flip(value: FInt, hw: f64) -> i32;
        fn draw_shape(
            document: Document,
            shape: &Shape,
            hw: f64,
            color: &str,
            stroke_width: i32,
        ) -> Document;
    }
    impl<'a> DrawStateHelper for Computation<'a> {
        fn to_svg(value: FInt, hw: f64) -> i32 {
            // (-hw, hw) -> (0, 800)
            ((value.midpoint() + hw) * (400.0 / hw)) as i32
        }

        fn to_svg_flip(value: FInt, hw: f64) -> i32 {
            // (-hw, hw) -> (800, 0)
            800 - Self::to_svg(value, hw)
        }

        fn draw_shape(
            document: Document,
            shape: &Shape,
            hw: f64,
            color: &str,
            stroke_width: i32,
        ) -> Document {
            match shape {
                Shape::Line(line) => document.add(
                    svg::node::element::Line::new()
                        .set(
                            "x1",
                            Self::to_svg(line.nx * line.d - FInt::new(3.0 * hw) * line.ny, hw),
                        )
                        .set(
                            "y1",
                            Self::to_svg_flip(line.ny * line.d + FInt::new(3.0 * hw) * line.nx, hw),
                        )
                        .set(
                            "x2",
                            Self::to_svg(line.nx * line.d + FInt::new(3.0 * hw) * line.ny, hw),
                        )
                        .set(
                            "y2",
                            Self::to_svg_flip(line.ny * line.d - FInt::new(3.0 * hw) * line.nx, hw),
                        )
                        .set("fill", "none")
                        .set("stroke", color)
                        .set("stroke-width", stroke_width),
                ),
                Shape::Ray(ray) => document.add(
                    svg::node::element::Line::new()
                        .set("x1", Self::to_svg(ray.a.0, hw))
                        .set("y1", Self::to_svg_flip(ray.a.1, hw))
                        .set(
                            "x2",
                            Self::to_svg(ray.a.0 + FInt::new(3.0 * hw) * ray.v.0, hw),
                        )
                        .set(
                            "y2",
                            Self::to_svg_flip(ray.a.1 + FInt::new(3.0 * hw) * ray.v.1, hw),
                        )
                        .set("fill", "none")
                        .set("stroke", color)
                        .set("stroke-width", stroke_width),
                ),
                Shape::Circle(circle) => document.add(
                    svg::node::element::Circle::new()
                        .set("cx", Self::to_svg(circle.c.0, hw))
                        .set("cy", Self::to_svg_flip(circle.c.1, hw))
                        .set("r", Self::to_svg(circle.r2.sqrt() - FInt::new(hw), hw))
                        .set("fill", "none")
                        .set("stroke", color)
                        .set("stroke-width", stroke_width),
                ),
            }
        }
    }
}
