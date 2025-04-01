use std::{fmt, mem::transmute, sync::RwLock};

use crate::{computation::action::ElementLink, fint::FInt};

use super::*;
use rand::{rng, Rng};

const NEW_SHAPE_MULTIPLIER: u32 = 4;

pub struct RandomWalkSolution {
    pub shapes: Vec<Shape>,
}

pub struct RandomWalkParent<'a> {
    problem: &'a ProblemDefinition,
    pt_index_counts: Vec<u32>,
    given_shape_count: u32,
    fixed_points: Vec<Point>,
    shapes_to_find: Vec<Shape>,
    points_to_find: HashSet2<Point>,
    solution_found: RwLock<bool>,
    actions: Vec<ActionType>,
}

#[derive(Clone, Copy, Debug)]
enum FSupportState {
    NeedBoth,       // Line: no points have two shapes passing through it
    NeedOne(usize), // Line: one point with two shapes passing through it found; circle: see NeedBoth for line
    AllFound, // line: two points with two shapes each found; circle: a point with two shapes found
}

struct FData {
    f_supports: [Point; 50],
    f_alt_lines: [Option<Shape>; 50],
    size: usize,
    f_state_1: FSupportState,
    f_state_2: FSupportState,
}
impl fmt::Debug for FData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FData")
            .field("f_supports", &&self.f_supports[0..self.size])
            .field("size", &self.size)
            .field("f_state_1", &self.f_state_1)
            .field("f_state_2", &self.f_state_2)
            .finish()
    }
}
impl FData {
    const fn new() -> Self {
        Self {
            f_supports: [Point(FInt::zero(), FInt::zero()); 50],
            f_alt_lines: [None; 50],
            size: 0,
            f_state_1: FSupportState::AllFound,
            f_state_2: FSupportState::AllFound,
        }
    }

    fn reset_to(&mut self, initial: &FData) {
        self.size = initial.size;
        self.f_state_1 = initial.f_state_1;
        self.f_state_2 = initial.f_state_2;
    }

    fn update(&mut self, f_shape: &Shape, shape: &Shape) {
        if !matches!(self.f_state_1, FSupportState::AllFound) {
            let f_support_to_ignore = if let FSupportState::NeedOne(i) = self.f_state_1 {
                i
            } else {
                usize::MAX
            };
            let direction = shape.get_direction();
            let points = f_shape.find_intersection_points(shape);
            for j in 0..2 {
                if let Some(point) = points[j] {
                    if !point.well_formed() {
                        println!("Ignoring {} because it isn't well-formed", point);
                        continue;
                    }
                    let i_found = self.f_supports[0..self.size]
                        .iter()
                        .position(|&pt| pt == point)
                        .unwrap_or(usize::MAX);

                    if i_found != usize::MAX {
                        if i_found != f_support_to_ignore
                            && !(direction
                                .map(|d| {
                                    self.f_alt_lines[i_found]
                                        .map(|alt_line| {
                                            alt_line.get_direction().unwrap().is_collinear(&d)
                                        })
                                        .unwrap_or(false)
                                })
                                .unwrap_or(false))
                        {
                            self.f_state_1 = match self.f_state_1 {
                                FSupportState::AllFound => FSupportState::AllFound,
                                FSupportState::NeedOne(_) => {
                                    // println!("Two supports found for {f_shape}");
                                    // println!("Shape: {shape}");
                                    // println!("Alt line: {:?}", self.f_alt_lines[i_found]);
                                    FSupportState::AllFound
                                }
                                FSupportState::NeedBoth => FSupportState::NeedOne(i_found),
                            };
                        }
                    } else {
                        self.f_supports[self.size] = point;
                        match shape {
                            Shape::Ray(_) | Shape::Segment(_) => {
                                self.f_alt_lines[self.size] = Some(*shape);
                            }
                            _ => (),
                        }
                        self.size += 1;
                    }
                }
            }
        }
        match f_shape {
            Shape::Circle(circle)
                if !matches!(self.f_state_2, FSupportState::AllFound)
                    && shape.contains_point(&circle.c) =>
            {
                self.f_state_2 = match self.f_state_2 {
                    FSupportState::AllFound => FSupportState::AllFound,
                    FSupportState::NeedOne(_) => FSupportState::AllFound,
                    FSupportState::NeedBoth => FSupportState::NeedOne(usize::MAX),
                };
            }
            _ => (),
        }
    }
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
    parent: &'a RandomWalkParent<'a>,
    initial_shapes: Vec<Shape>,
}
impl<'a> RandomWalk<'a> {
    fn choose_random_shape_to_add(
        &self,
        shapes: &[Shape],
        n: u32,
        added_shape_count: u32,
    ) -> Option<Shape> {
        let action_type_count = self.parent.problem.action_types.len() as u32;
        let rw_choice_count = n * n * action_type_count;
        let rw_choice = rng().random_range(0..rw_choice_count);
        // println!("Generating {}-th shape, random value: {}", i, rw_choice);
        let i_action = rw_choice % action_type_count;
        let point_index_1 = (rw_choice / action_type_count) % n;
        let point_index_2 = (rw_choice / (n * action_type_count)) % n;
        if point_index_1 == point_index_2 {
            return None;
        }
        let points = self.get_point_pair(point_index_1, point_index_2, &shapes, added_shape_count);
        // println!("Found points: {:?}", points);
        match points {
            Some(points) if points[0].well_formed() && points[1].well_formed() => {
                match self.get_shape(&points, i_action) {
                    Some(shape) if shape.well_formed() => {
                        let mut has_same = false;
                        for i1 in 0..added_shape_count {
                            if shapes[i1 as usize].almost_equals(&shape) {
                                has_same = true;
                                break;
                            }
                        }
                        if has_same {
                            return None;
                        }
                        // println!("Adding shape {}", shape);
                        return Some(shape);
                    }
                    _ => return None,
                }
            }
            _ => return None,
        }
    }

    fn initialize_supports(&self, f_data_list: &mut [FData], initial_f_mask: u32) {
        for (f_index, f_shape) in self.parent.shapes_to_find.iter().enumerate() {
            let f_data = &mut f_data_list[f_index];
            match f_shape {
                Shape::Circle(_) => {
                    f_data.f_state_1 = FSupportState::NeedOne(usize::MAX);
                    f_data.f_state_2 = FSupportState::NeedBoth;
                }
                _ => {
                    f_data.f_state_1 = FSupportState::NeedBoth;
                    f_data.f_state_2 = FSupportState::AllFound;
                }
            }
            if initial_f_mask & (1 << f_index) != 0 {
                for shape in &self.initial_shapes {
                    f_data.update(f_shape, shape);
                }
            }
        }
    }

    fn get_first_found_shape_index_with_supports(
        &self,
        f_mask: u32,
        f_data_list: &[FData],
    ) -> Option<u32> {
        for f_index in 0..self.parent.shapes_to_find.len_u32() {
            if f_mask & (1 << f_index) != 0
                && matches!(
                    f_data_list[f_index as usize].f_state_1,
                    FSupportState::AllFound
                )
                && matches!(
                    f_data_list[f_index as usize].f_state_2,
                    FSupportState::AllFound
                )
            {
                return Some(f_index);
            }
        }
        None
    }

    pub fn run_iterations(&self, limit: u32) -> Option<RandomWalkSolution> {
        if self.random_walk_index % 500 == 0 {
            println!(
                "Running random walk {} with limit = {}",
                self.random_walk_index, limit
            );
        }
        if self.initial_shapes.len_u32()
            != self.parent.given_shape_count + self.parent.problem.random_walk_at_n_actions.unwrap()
                - 1
        {
            println!("Initial shapes mismatch");
            return None;
        }
        let mut shapes = [self.initial_shapes[0]; 20];
        for i in 0..self.initial_shapes.len() {
            shapes[i] = self.initial_shapes[i];
        }
        let mut initial_f_mask = 0;
        for (f_index, f_shape) in self.parent.shapes_to_find.iter().enumerate() {
            let mut found = false;
            for shape1 in &self.initial_shapes {
                if *shape1 == *f_shape {
                    found = true;
                    break;
                }
            }
            if !found {
                initial_f_mask |= 1u32 << f_index;
            }
        }
        let mut f_data_list = [const { FData::new() }; 20];
        let mut f_initial_data_list = [const { FData::new() }; 20];
        if self.parent.problem.track_supports_in_rw {
            self.initialize_supports(&mut f_data_list, initial_f_mask);
        }
        for f_index in 0..self.parent.shapes_to_find.len() {
            f_initial_data_list[f_index].reset_to(&f_data_list[f_index]);
        }
        for _iteration in 0..limit {
            {
                if *self.parent.solution_found.read().unwrap() {
                    break;
                }
            }
            let mut f_mask = initial_f_mask;
            if self.parent.problem.track_supports_in_rw {
                for f_index in 0..self.parent.shapes_to_find.len() {
                    f_data_list[f_index].reset_to(&f_initial_data_list[f_index]);
                }
            }
            // shapes:
            //   - given: 0.."given" (size: "initial" - N + 1)
            //   - common to all walks in "self": "given".."initial" (size: N - 1)
            //   - found so far: up to "initial".."initial" + L - N + 1 (size: L - N + 1)
            let i0 = self.initial_shapes.len_u32();
            let mut i = i0;
            let chosen_count = i0 - self.parent.given_shape_count; // N - 1
            let i_max = i0 + self.parent.problem.action_count - chosen_count;
            let mut i_retry = 0;
            while i < i_max && i_retry < i_max - i0 + 10 {
                i_retry += 1;
                let f_index = if self.parent.problem.track_supports_in_rw {
                    self.get_first_found_shape_index_with_supports(f_mask, &f_data_list)
                } else {
                    None
                };
                let mut maybe_shape =
                    f_index.map(|f_index| self.parent.shapes_to_find[f_index as usize]);
                if maybe_shape.is_none() {
                    let to_find = f_mask.count_ones();
                    if i_max - i < to_find
                        || (self.parent.problem.track_supports_in_rw && i_max - i == to_find)
                    {
                        break;
                    }
                    let n = self.parent.pt_index_counts[(i - i0) as usize];
                    maybe_shape = self.choose_random_shape_to_add(&shapes, n, i);
                }
                if let Some(shape) = maybe_shape {
                    match f_index {
                        Some(f_index) => {
                            f_mask ^= 1 << f_index;
                        }
                        None if !self.parent.problem.track_supports_in_rw => {
                            for (f_index, f_shape) in self.parent.shapes_to_find.iter().enumerate()
                            {
                                if f_mask & (1 << f_index) != 0 && shape == *f_shape {
                                    f_mask ^= 1 << f_index;
                                    break;
                                }
                            }
                            if i_max - i == f_mask.count_ones() {
                                break;
                            }
                        }
                        _ => (),
                    }
                    shapes[i as usize] = shape;
                    i += 1;
                    if self.parent.problem.track_supports_in_rw {
                        for (f_index, f_shape) in self.parent.shapes_to_find.iter().enumerate() {
                            if f_mask & (1 << f_index) != 0 {
                                f_data_list[f_index].update(f_shape, &shape);
                            }
                        }
                    }
                } else {
                    continue;
                }
            }
            if i == i_max {
                // println!("Candidate found");
                // for i in 0..i_max {
                //     println!("  - {}", shapes[i as usize]);
                // }
                let mut all_found = true;
                if self.parent.points_to_find.len() > 0 {
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
                    return Some(RandomWalkSolution {
                        shapes: shapes[0..(i_max as usize)].to_vec(),
                    });
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
        added_shape_count: u32,
    ) -> Option<[Point; 2]> {
        let point1 = self.get_point(point_index_1, shapes, added_shape_count)?;
        let point2 = self.get_point(point_index_2, shapes, added_shape_count)?;
        if point1 == point2 {
            None
        } else {
            Some([point1, point2])
        }
    }

    pub fn get_point(
        &self,
        point_index: u32,
        shapes: &[Shape],
        added_shape_count: u32,
    ) -> Option<Point> {
        if point_index < self.parent.fixed_points.len_u32() {
            Some(self.parent.fixed_points[point_index as usize])
        } else {
            let i0 = self.initial_shapes.len_u32();
            let pt_index_count = self.parent.pt_index_counts[(added_shape_count - i0) as usize];
            let intersection_index = point_index - self.parent.fixed_points.len_u32();
            let two_shapes_index = intersection_index / 2;
            let offset = two_shapes_index % (added_shape_count - 1);
            let shape_index_with_multiplier = two_shapes_index / (added_shape_count - 1);
            let value = i0 - 1 + (added_shape_count - (i0 - 1)) * NEW_SHAPE_MULTIPLIER;
            assert_eq!(
                pt_index_count,
                self.parent.fixed_points.len_u32() + value * (added_shape_count - 1) * 2
            );
            let shape_index_1 = if shape_index_with_multiplier < i0 - 1 {
                shape_index_with_multiplier
            } else {
                i0 - 1 + (shape_index_with_multiplier - (i0 - 1)) % (added_shape_count - (i0 - 1))
            };
            let shape_index_2 = (shape_index_1 + offset + 1) % added_shape_count;
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
        match self.parent.actions[i_action as usize] {
            ActionType::TwoPointActionType(two_point_action_type) => {
                Action::create_two_point_element(&points[0], &points[1], two_point_action_type)
                    .get_shape()
            }
            _ => panic!(),
        }
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
            match shape_origin.element_link {
                ElementLink::GivenElement { shape, .. } => {
                    given_shapes.push(shape);
                }
                ElementLink::Action(_) => (),
            }
        }
        let mut shapes_to_find = Vec::new();
        self.found_shapes.iter().for_each(|shape| {
            shapes_to_find.push(*shape);
        });
        self.shapes_to_find.iter().for_each(|shape| {
            shapes_to_find.push(*shape);
        });
        let mut points_to_find = HashSet2::new();
        self.found_points.iter().for_each(|point| {
            points_to_find.insert(*point);
        });
        self.points_to_find.iter().for_each(|point| {
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
        let actions: Vec<ActionType> = self.problem.action_types.iter().map(|&x| x).collect();
        RandomWalkParent {
            problem: &self.problem,
            pt_index_counts,
            given_shape_count: given_shapes.len_u32(),
            fixed_points,
            shapes_to_find,
            points_to_find,
            solution_found: RwLock::new(false),
            actions,
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
            let deps_count = match &shape_origin.element_link {
                ElementLink::GivenElement { .. } => 0,
                ElementLink::Action(action) => action.deps_count + 1,
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
            deps.push(action.get_action_deps(&self));
            last_shapes.push(action.shape);
        }
        for i in 0..self.shape_origins.len() {
            let shape_origin = &self.shape_origins[i];
            if matches!(shape_origin.element_link, ElementLink::GivenElement { .. }) {
                continue;
            }
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
        for shape in &random_walks[43].initial_shapes {
            println!("  - {}", shape);
        }
        random_walks
    }
}
