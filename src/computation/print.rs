use std::collections::HashSet;

use private::*;

use crate::{Computation, VecLengths};

pub trait PrintState {
    fn print_state(&mut self);
    fn print_solution(&mut self);
}
impl<'a> PrintState for Computation<'a> {
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
                "{}: {} {} [{} actions: {} ({:b})] {}",
                i,
                self.get_point_name(i),
                origin.point,
                self.get_deps_count(origin.deps),
                self.print_deps(origin.deps),
                origin.found_shape_mask,
                found_part,
            );
        }
        println!("");
        self.print_shapes(HashSet::new());
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
        // for (i, action) in queue_copy.iter().enumerate() {
        //     if i < 100 {
        //         let mut indices = action.get_point_indices();
        //         let mut shape_indices = action.get_shape_indices();
        //         indices.append(&mut shape_indices);
        //         println!(
        //             "Indices {:?}, shape = {}: priority = {}, type: {:?}",
        //             indices, action.shape, action.priority, action.action_type,
        //         );
        //     }
        // }
    }

    fn print_solution(&mut self) {
        // self.print_state();
        let deps_list = match self.solution_deps {
            None => self.get_solution_deps_list(),
            Some(deps) => HashSet::from([deps]),
        };
        self.print_shapes(deps_list)
    }
}

mod private {
    use std::collections::HashSet;

    use crate::computation::{action::ElementLink, GIVEN};
    use crate::{shape::Shape, Computation, VecLengths};

    pub trait PrintStateHelper {
        fn get_shape_name(&self, shape_index: i32) -> String;
        fn get_point_name(&self, point_index: i32) -> String;
        fn print_shapes(&mut self, only_included_in_deps: HashSet<u64>);
    }
    impl<'a> PrintStateHelper for Computation<'a> {
        fn get_shape_name(&self, shape_index: i32) -> String {
            let origin = &self.shape_origins[shape_index as usize];
            let prefix = match &origin.element_link {
                ElementLink::GivenElement { .. } => "Given",
                ElementLink::Action(_) => "",
            };
            let name = match origin.get_shape() {
                Shape::Line(_line) => "Line",
                Shape::Ray(_ray) => "Ray",
                Shape::Segment(_segment) => "Segment",
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

        fn print_shapes(&mut self, only_included_in_deps: HashSet<u64>) {
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
                let origin = &self.shape_origins[i as usize];
                let from_part = match &origin.element_link {
                    ElementLink::GivenElement { .. } => "".to_string(),
                    ElementLink::Action(action) => {
                        let mut names: Vec<String> = action
                            .get_point_indices()
                            .into_iter()
                            .map(|i| self.get_point_name(i))
                            .collect();
                        let mut shape_names: Vec<String> = action
                            .get_shape_indices()
                            .into_iter()
                            .map(|i| self.get_shape_name(i))
                            .collect();
                        names.append(&mut shape_names);

                        let point_names_str = if names.len() == 2 {
                            format!("{} and {}", names[0], names[1])
                        } else {
                            format!("{}, {}, and {}", names[0], names[1], names[2])
                        };
                        format!(
                            "[pri = {}] from {} ({})",
                            action.priority, point_names_str, origin.element_link,
                        )
                    }
                };
                let origin = &self.shape_origins[i as usize];
                let found_part = if self.shapes_to_find.contains(origin.get_shape()) {
                    " (!!!)"
                } else {
                    ""
                };
                println!(
                    "{}: {} {} [{} actions: {} ({:b})]{}{}",
                    i,
                    self.get_shape_name(i),
                    origin.get_shape(),
                    self.get_deps_count(origin.deps),
                    self.print_deps(origin.deps),
                    origin.found_shape_mask,
                    from_part,
                    found_part,
                );
            }
        }
    }
}
