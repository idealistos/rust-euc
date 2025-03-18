use private::PrintStateHelper;

use crate::{Computation, VecLengths, GIVEN};

pub trait PrintState {
    fn print_shapes(&mut self, only_deps_of_shape_index: Option<i32>);
    fn print_state(&mut self);
}
impl<'a> PrintState for Computation<'a> {
    fn print_shapes(&mut self, only_deps_of_shape_index: Option<i32>) {
        let deps_to_match = match only_deps_of_shape_index {
            None => None,
            Some(shape_index) => Some(self.shape_origins[shape_index as usize].deps),
        };
        for i in 0..self.shape_origins.len_i32() {
            let origin = &self.shape_origins[i as usize];
            match deps_to_match {
                Some(deps) => {
                    if self.combine_deps(deps, origin.deps, None) != deps {
                        continue;
                    }
                }
                _ => (),
            }
            let origin = &self.shape_origins[i as usize];
            let from_part = if origin.point_origin_indices[0] == GIVEN {
                "".to_string()
            } else {
                format!(
                    " from {} and {} ({})",
                    self.get_point_name(origin.point_origin_indices[0]),
                    self.get_point_name(origin.point_origin_indices[1]),
                    origin.element_or_ref,
                )
            };
            let origin = &self.shape_origins[i as usize];
            let found_part = if self.shapes_to_find.contains(origin.shape) {
                " (!!!)"
            } else {
                ""
            };
            println!(
                "{}: {} {} [{} actions: {:b} ({:b})]{}{}",
                i,
                self.get_shape_name(i),
                origin.shape,
                self.get_deps_count(origin.deps),
                origin.deps,
                origin.found_shape_mask,
                from_part,
                found_part,
            );
        }
    }

    fn print_state(&mut self) {
        println!("--- State --- ");
        for i in 0..self.point_origins.len_i32() {
            if i > 300 {
                break;
            }
            let origin = &self.point_origins[i as usize];
            let found_part = if self.found_points.contains(origin.point) {
                " (!!!)"
            } else {
                ""
            };
            println!(
                "{}: {} {} [{} actions: {:b} ({:b})] {}",
                i,
                self.get_point_name(i),
                origin.point,
                self.get_deps_count(origin.deps),
                origin.deps,
                origin.found_shape_mask,
                found_part,
            );
        }
        println!("");
        self.print_shapes(None);
        println!("");
        for point in self.points_to_find.as_vector() {
            println!("Point yet to find: {}", point,);
        }
        for shape in self.shapes_to_find.as_vector() {
            println!("Shape yet to find: {}", shape,);
        }
        let mut queue_copy = self.queue.clone().into_sorted_vec();
        queue_copy.reverse();
        println!("Actions in queue: {}", queue_copy.len());
        for (i, action) in queue_copy.iter().enumerate() {
            if i < 20 {
                println!(
                    "Points {} and {}, shape = {}: priority = {}",
                    action.point_index_1, action.point_index_2, action.shape, action.priority
                );
            }
        }
    }
}

mod private {
    use crate::{shape::Shape, Computation, GIVEN};

    pub trait PrintStateHelper {
        fn get_shape_name(&self, shape_index: i32) -> String;
        fn get_point_name(&self, point_index: i32) -> String;
    }
    impl<'a> PrintStateHelper for Computation<'a> {
        fn get_shape_name(&self, shape_index: i32) -> String {
            let origin = &self.shape_origins[shape_index as usize];
            let prefix = if origin.point_origin_indices[0] == GIVEN {
                "Given"
            } else {
                ""
            };
            let name = match origin.shape {
                Shape::Line(_line) => "Line",
                Shape::Ray(_ray) => "Ray",
                Shape::Circle(_circle) => "Circle",
            };
            return format!("{prefix}{name}{shape_index}");
        }

        fn get_point_name(&self, point_index: i32) -> String {
            let origin = &self.point_origins[point_index as usize];
            if origin.shape_origin_indices[0] == GIVEN {
                format!("GivenPoint{}", point_index)
            } else {
                format!(
                    "x/{}/{}",
                    self.get_shape_name(origin.shape_origin_indices[0]),
                    self.get_shape_name(origin.shape_origin_indices[1])
                )
            }
        }
    }
}
