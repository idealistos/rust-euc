use crate::draw::private::DrawStateHelper;
use crate::Computation;
use crate::FInt;
use crate::Shape;
use crate::VecLengths;
use svg::Document;

pub trait DrawState {
    fn draw_state(&mut self, filename: String, hw: f64, only_deps_of_shape_index: Option<i32>);
}

impl<'a> DrawState for Computation<'a> {
    fn draw_state(&mut self, filename: String, hw: f64, only_deps_of_shape_index: Option<i32>) {
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

        let deps_to_match = match only_deps_of_shape_index {
            None => None,
            Some(shape_index) => Some(self.shape_origins[shape_index as usize].deps),
        };
        for i in 0..self.shape_origins.len_i32() {
            let shape_origin = &self.shape_origins[i as usize];
            match deps_to_match {
                Some(deps) => {
                    if self.combine_deps(deps, shape_origin.deps, None) != deps {
                        continue;
                    }
                }
                _ => (),
            }
            let shape_origin = &self.shape_origins[i as usize];
            let deps_count = self.get_deps_count(shape_origin.deps);
            let color = colors[(deps_count % 10) as usize];
            let stroke_width = if deps_count <= 2 {
                3
            } else {
                (deps_count / 10) + 1
            };
            document = match shape_origin.shape {
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
            let point_origin = &self.point_origins[i as usize];
            match deps_to_match {
                Some(deps) => {
                    if self.combine_deps(deps, point_origin.deps, None) != deps {
                        continue;
                    }
                }
                _ => (),
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
}

mod private {
    use crate::Computation;
    use crate::FInt;

    pub trait DrawStateHelper {
        fn to_svg(value: FInt, hw: f64) -> i32;
        fn to_svg_flip(value: FInt, hw: f64) -> i32;
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
    }
}
