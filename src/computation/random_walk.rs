use std::{mem::transmute, sync::RwLock};

use super::*;
use rand::{rng, Rng};

const NEW_SHAPE_MULTIPLIER: u32 = 4;

pub struct RandomWalkSolution {}

pub struct RandomWalkParent {
    pt_index_counts: Vec<u32>,
    given_shape_count: u32,
    action_count: u32,
    action_types: &'static [ActionType],
    fixed_points: Vec<Point>,
    shapes_to_find: HashSet2<Shape>,
    points_to_find: HashSet2<Point>,
    solution_found: RwLock<bool>,
}

// "Initial shapes": all given shapes + 1 shape with each dep_count (1,.., "rw at N" - 1)
// - shapes with dep_count = 1 .. N - 2: registered
// - shapes with dep_count = N - 1: from "actions" (not registered)

// Random number format: (pt1_index[i], pt2_index[i], i_action) with i = N, N + 1,.., L = "action count"
// where pt_index identifies the point:
//   - pt_index < fixed_points.len(): fixed point
//   - otherwise, it is (shape1_index, delta) + fixed_points.len():
//      - delta: 1..(#shapes - 1), with #shapes = n1 + 1 + (i - N)
//      - shape1_index:
//         - shape1_index <= n1 (n1 is initial #shapes - 1): index in shapes
//         - otherwise, the index in shapes is n1 + (shape1_index - n1) % (#shapes - i)

// Example: initial #shapes = 5 (3 given + "1" + "2"), added "3" and "4", n1 = 4, M = 5, N = 3
// 0 -> 0, 1 -> 1, 2 -> 2, 3 -> 3, 4 -> 4, 5 -> 5, 6 -> 6, 7 -> 4,.., 16 -> 4, 17 -> 5, 18 -> 6
// Total count: n1 + (#shapes - n1) * M
pub struct RandomWalk<'a> {
    random_walk_index: u32,
    parent: &'a RandomWalkParent,
    initial_shapes: Vec<Shape>,
}
impl<'a> RandomWalk<'a> {
    pub fn run_iterations(&self, limit: u32) -> Option<RandomWalkSolution> {
        if self.random_walk_index % 500 == 0 {
            println!(
                "Running random walk {} with limit = {}",
                self.random_walk_index, limit
            );
        }
        let mut shapes = [self.initial_shapes[0]; 20];
        for i in 0..self.initial_shapes.len() {
            shapes[i] = self.initial_shapes[i];
        }
        let action_type_count = self.parent.action_types.len() as u32;
        let mut shapes_to_find_set = HashSet2::new();
        for shape_to_find in &self.parent.shapes_to_find {
            shapes_to_find_set.insert(*shape_to_find);
        }
        for _iteration in 0..limit {
            {
                if *self.parent.solution_found.read().unwrap() {
                    break;
                }
            }
            // shapes:
            //   - given: 0.."given" (size: "initial" - N + 1)
            //   - common to all walks in "self": "given".."initial" (size: N - 1)
            //   - found so far: up to "initial".."initial" + L - N + 1 (size: L - N + 1)
            let i0 = self.initial_shapes.len_u32();
            let mut i = i0;
            let chosen_count = i0 - self.parent.given_shape_count; // N - 1
            let i_max = i0 + self.parent.action_count - chosen_count;
            while i < i_max {
                let n = self.parent.pt_index_counts[(i - i0) as usize];
                let rw_choice_count = n * n * action_type_count;
                let rw_choice = rng().random_range(0..rw_choice_count);
                // println!("Generating {}-th shape, random value: {}", i, rw_choice);
                let i_action = rw_choice % action_type_count;
                let point_index_1 = (rw_choice / action_type_count) % n;
                let point_index_2 = (rw_choice / (n * action_type_count)) % n;
                if point_index_1 == point_index_2 {
                    continue;
                }
                let points = self.get_point_pair(point_index_1, point_index_2, &shapes, i);
                // println!("Found points: {:?}", points);
                match points {
                    Some(points) if points[0].well_formed() && points[1].well_formed() => {
                        match self.get_shape(&points, i_action) {
                            Some(shape) if shape.well_formed() => {
                                let mut has_same = false;
                                for i1 in 0..i {
                                    if shapes[i1 as usize] == shape {
                                        has_same = true;
                                        break;
                                    }
                                }
                                if has_same {
                                    continue;
                                }
                                // println!("Adding shape {}", shape);
                                shapes[i as usize] = shape;
                            }
                            _ => continue,
                        }
                    }
                    _ => continue,
                }
                i += 1;
            }
            if i == i_max {
                let check = if self.parent.shapes_to_find.len() > 0 {
                    // Not quite correct - it is possible that the last shape isn't in shapes_to_find
                    // but it is necessary because it defines a point in points_to_find
                    shapes_to_find_set.contains(shapes[(i - 1) as usize])
                } else {
                    self.parent
                        .points_to_find
                        .iter()
                        .any(|point| shapes[(i - 1) as usize].contains_point(point))
                };
                if check {
                    println!("Candidate found");
                    for i in 0..i_max {
                        println!("  - {}", shapes[i as usize]);
                    }
                    let mut new_shapes_set = HashSet2::new();
                    for i1 in self.parent.given_shape_count..i_max {
                        new_shapes_set.insert(shapes[i1 as usize]);
                    }
                    let mut all_found = self
                        .parent
                        .shapes_to_find
                        .iter()
                        .all(|shape| new_shapes_set.contains(*shape));
                    if all_found && self.parent.points_to_find.len() > 0 {
                        // Each point in points_to_find should belong to 2 shapes
                        all_found = self.parent.points_to_find.iter().all(|point| {
                            let count = (0..i)
                                .filter(|i1| shapes[*i1 as usize].contains_point(point))
                                .count();
                            return count >= 2;
                        });
                    }
                    if all_found {
                        println!("Solution found!");
                        for i_shape in self.parent.given_shape_count..i_max {
                            println!("  - {}", shapes[i_shape as usize]);
                        }
                        *self.parent.solution_found.write().unwrap() = true;
                        return Some(RandomWalkSolution {});
                    }
                }
            }
        }
        None
    }

    pub fn get_point_pair(
        &self,
        point_index_1: u32,
        point_index_2: u32,
        shapes: &[Shape],
        index: u32,
    ) -> Option<[Point; 2]> {
        let point1 = self.get_point(point_index_1, shapes, index)?;
        let point2 = self.get_point(point_index_2, shapes, index)?;
        if point1 == point2 {
            None
        } else {
            Some([point1, point2])
        }
    }

    pub fn get_point(&self, point_index: u32, shapes: &[Shape], index: u32) -> Option<Point> {
        if point_index < self.parent.fixed_points.len_u32() {
            Some(self.parent.fixed_points[point_index as usize])
        } else {
            let i0 = self.initial_shapes.len_u32();
            let pt_index_count = self.parent.pt_index_counts[(index - i0) as usize];
            let intersection_index = point_index - self.parent.fixed_points.len_u32();
            let two_shapes_index = intersection_index / 2;
            let offset = two_shapes_index % (index - 1);
            let shape_index_with_multiplier = two_shapes_index / (index - 1);
            let value = i0 - 1 + (index - (i0 - 1)) * NEW_SHAPE_MULTIPLIER;
            assert_eq!(
                pt_index_count,
                self.parent.fixed_points.len_u32() + value * (index - 1) * 2
            );
            let shape_index_1 = if shape_index_with_multiplier < i0 - 1 {
                shape_index_with_multiplier
            } else {
                i0 - 1 + (shape_index_with_multiplier - (i0 - 1)) % (index - (i0 - 1))
            };
            let shape_index_2 = (shape_index_1 + offset + 1) % index;
            let shape1 = shapes[shape_index_1 as usize];
            let shape2 = shapes[shape_index_2 as usize];
            if shape1 == shape2 {
                return None;
            }
            let intersection_points = shape1.find_intersection_points(&shape2);
            intersection_points[(intersection_index % 2) as usize]
        }
    }

    pub fn get_shape(&self, points: &[Point; 2], i_action: u32) -> Option<Shape> {
        let action_type = unsafe { transmute(i_action as i8) };
        Computation::create_two_point_element(&points[0], &points[1], action_type).get_shape()
    }
}

pub trait RandomWalkProcessing<'a> {
    fn create_random_walk_parent(&self) -> RandomWalkParent;

    fn prepare_random_walks(
        &self,
        random_walk_parent: &'a RandomWalkParent,
        rw_queue: Vec<Action>,
    ) -> Vec<RandomWalk>;
}
impl<'a> RandomWalkProcessing<'a> for Computation<'a> {
    fn create_random_walk_parent(&self) -> RandomWalkParent {
        let mut fixed_points = Vec::new();
        for point_origin in &self.point_origins {
            if point_origin.deps == 0 {
                fixed_points.push(point_origin.point);
            }
        }
        let mut given_shapes = Vec::new();
        for shape_origin in &self.shape_origins {
            match shape_origin.element_or_ref {
                GivenOrNewElement::GivenElement { shape, .. } => {
                    given_shapes.push(shape);
                }
                _ => (),
            }
        }
        let mut shapes_to_find = HashSet2::new();
        self.shapes_to_find.iter().for_each(|shape| {
            shapes_to_find.insert(*shape);
        });
        self.found_shapes.iter().for_each(|shape| {
            shapes_to_find.insert(*shape);
        });
        let mut points_to_find = HashSet2::new();
        self.points_to_find.iter().for_each(|point| {
            points_to_find.insert(*point);
        });
        self.found_points.iter().for_each(|point| {
            points_to_find.insert(*point);
        });
        let mut pt_index_counts = Vec::new();
        let n = self.problem.random_walk_at_n_actions.unwrap();
        let i0 = given_shapes.len_u32() + n - 1;
        let i_max = given_shapes.len_u32() + self.problem.action_count;
        for i in i0..i_max {
            // #shapes at each iteration: deps = N -> n1 + 1,.., deps = L -> n1 + 1 + (L - N)
            // pt_index_counts: deps = N -> n1 + M,.., deps = L -> n1 + M * (L - N + 1)
            let value = (i0 - 1) + NEW_SHAPE_MULTIPLIER * (i - (i0 - 1));
            pt_index_counts.push(fixed_points.len_u32() + value * (i - 1) * 2);
        }
        RandomWalkParent {
            pt_index_counts,
            given_shape_count: given_shapes.len_u32(),
            action_count: self.problem.action_count,
            action_types: self.problem.action_types,
            fixed_points,
            shapes_to_find,
            points_to_find,
            solution_found: RwLock::new(false),
        }
    }

    fn prepare_random_walks(
        &self,
        random_walk_parent: &'a RandomWalkParent,
        rw_queue: Vec<Action>,
    ) -> Vec<RandomWalk> {
        println!("Preparing random walks for {} actions", rw_queue.len());
        let mut shapes_seen_so_far = HashSet2::new();
        let mut given_shapes = Vec::new();
        for shape_origin in &self.shape_origins {
            let deps_count = match &shape_origin.element_or_ref {
                GivenOrNewElement::GivenElement { .. } => 0,
                GivenOrNewElement::TwoPointElement { element: _, action } => action.deps_count + 1,
                GivenOrNewElement::PointAndLineElement { element: _, action } => {
                    action.deps_count + 1
                }
            };
            if deps_count <= self.problem.random_walk_at_n_actions.unwrap() - 2 {
                shapes_seen_so_far.insert(shape_origin.get_shape());
            }
            if deps_count == 0 {
                given_shapes.push(shape_origin.get_shape());
            }
        }

        let mut random_walks = Vec::new();
        let mut deps = Vec::new();
        let mut last_shapes = Vec::new();
        for action in rw_queue {
            if shapes_seen_so_far.contains(action.shape) {
                continue;
            }
            shapes_seen_so_far.insert(action.shape);
            random_walks.push(RandomWalk {
                random_walk_index: random_walks.len_u32(),
                parent: random_walk_parent,
                initial_shapes: given_shapes.clone(),
            });
            deps.push(self.get_action_deps(&action));
            last_shapes.push(action.shape);
        }
        shapes_seen_so_far = HashSet2::new();
        for i in 0..self.shape_origins.len() {
            let shape_origin = &self.shape_origins[i];
            if matches!(
                shape_origin.element_or_ref,
                GivenOrNewElement::GivenElement { .. }
            ) {
                continue;
            }
            if shapes_seen_so_far.contains(shape_origin.get_shape()) {
                continue;
            }
            shapes_seen_so_far.insert(shape_origin.get_shape());
            let shape_deps = shape_origin.deps;
            for j in 0..random_walks.len() {
                if self.requires_deps(&deps[j], shape_deps) {
                    let shape_origin = &self.shape_origins[i];
                    random_walks[j]
                        .initial_shapes
                        .push(shape_origin.get_shape());
                }
            }
        }
        for i in 0..random_walks.len() {
            random_walks[i].initial_shapes.push(last_shapes[i]);
        }
        println!("Initialized {} random walks", random_walks.len());
        let freqs =
            random_walks
                .iter()
                .fold(HashMap::new(), |mut map: HashMap<usize, i32>, value| {
                    *map.entry(value.initial_shapes.len()).or_default() += 1;
                    map
                });
        println!("{:?}", freqs);
        println!("Some random walk:",);
        for shape in &random_walks[random_walks.len() / 2].initial_shapes {
            println!("  - {}", shape);
        }
        random_walks
    }
}
