use action::Action;
use action::ElementLink;
pub use draw::DrawState;
pub use print::PrintState;
use random_walk::RandomWalkProcessing;
use random_walk::RandomWalkSolution;

use crate::element::CircleCP;
use crate::element::CircleCR;
use crate::element::Element;
use crate::element::LineAB;
use crate::element::LineAV;
use crate::element::MidPerpAB;
use crate::hashset2::HashMap2;
use crate::hashset2::HashSet2;
use crate::problems::ActionType;
use crate::problems::PointAndLineActionType;
use crate::problems::ProblemDefinition;
use crate::problems::ThreePointActionType;
use crate::problems::TwoPointActionType;
use crate::shape::ShapeTrait;
use crate::shape::{Point, Shape};
use crate::VecLengths;
use rayon::prelude::*;
use std::cmp::Ord;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::SystemTime;

mod action;
mod draw;
mod print;
mod random_walk;

const GIVEN: i32 = -1;
const RANDOM_WALK_LIMIT: u32 = 150000000;

#[derive(Debug)]
struct PointOrigin {
    point: Point,
    deps: u64,
    shape_origin_indices: [i32; 2],
    found_shape_mask: u32,
}
struct ShapeOrigin<'a> {
    deps: u64,
    element_link: ElementLink<'a>,
    found_shape_mask: u32,
}
impl<'a> ShapeOrigin<'a> {
    pub fn get_shape(&self) -> Shape {
        self.element_link.get_shape()
    }
}

pub struct Computation<'a> {
    problem: &'a ProblemDefinition,
    point_origins: Vec<PointOrigin>,
    shape_origins: Vec<ShapeOrigin<'a>>,
    points: HashMap2<Point, i32>,
    shapes: HashMap2<Shape, i32>,
    points_to_find: HashSet2<Point>,
    shapes_to_find: HashSet2<Shape>,
    found_points: HashSet2<Point>,
    found_shapes: HashSet2<Shape>,
    queue: BinaryHeap<Action>,
    deps_combinations: Vec<Vec<u32>>,
    deps_indices_by_hashes: HashMap<u64, Vec<i32>>,
    shape_to_find_mask_by_shape: HashMap2<Shape, u32>,
    final_found_shape: Option<Shape>,
}
impl<'a> Computation<'a> {
    pub fn new(problem: &'a ProblemDefinition) -> Self {
        let mut deps_indices_by_hashes = HashMap::new();
        deps_indices_by_hashes.insert(0, vec![0]);
        Self {
            problem,
            point_origins: vec![],
            shape_origins: vec![],
            points: HashMap2::new(),
            shapes: HashMap2::new(),
            points_to_find: HashSet2::new(),
            shapes_to_find: HashSet2::new(),
            found_points: HashSet2::new(),
            found_shapes: HashSet2::new(),
            queue: BinaryHeap::new(),
            deps_combinations: vec![vec![]],
            deps_indices_by_hashes,
            shape_to_find_mask_by_shape: HashMap2::new(),
            final_found_shape: None,
        }
    }

    fn get_deps_count(&self, deps: u64) -> u32 {
        let lower_mask = deps & ((1u64 << 40) - 1);
        if deps == lower_mask {
            lower_mask.count_ones()
        } else {
            self.deps_combinations[(deps >> 40) as usize].len_u32() + lower_mask.count_ones()
        }
    }

    fn get_combined_deps_count(&self, deps1: u64, deps2: u64) -> u32 {
        let combined_mask = deps1 | deps2;
        let lower_mask = combined_mask & ((1u64 << 40) - 1);
        if combined_mask == lower_mask {
            return lower_mask.count_ones();
        }
        let deps_vec_1 = &self.deps_combinations[(deps1 >> 40) as usize];
        let deps_vec_2 = &self.deps_combinations[(deps2 >> 40) as usize];
        let mut same_count = 0;
        for i1 in deps_vec_1 {
            for i2 in deps_vec_2 {
                if i1 == i2 {
                    same_count += 1;
                }
            }
        }
        lower_mask.count_ones() + deps_vec_1.len_u32() + deps_vec_2.len_u32() - same_count
    }

    #[allow(dead_code)]
    fn get_combined_three_deps_count(&self, deps1: u64, deps2: u64, deps3: u64) -> u32 {
        let combined_mask = deps1 | deps2 | deps3;
        let lower_mask = combined_mask & ((1u64 << 40) - 1);
        if combined_mask == lower_mask {
            return lower_mask.count_ones();
        }
        let deps_vec_1 = &self.deps_combinations[(deps1 >> 40) as usize];
        let deps_vec_2 = &self.deps_combinations[(deps2 >> 40) as usize];
        let deps_vec_3 = &self.deps_combinations[(deps3 >> 40) as usize];
        let mut same_two_count = 0;
        let mut all_three_count = 0;
        for i1 in deps_vec_1 {
            for i2 in deps_vec_2 {
                if i1 == i2 {
                    same_two_count += 1;
                    for i3 in deps_vec_3 {
                        if i3 == i1 {
                            all_three_count += 1
                        }
                    }
                }
            }
        }
        for i1 in deps_vec_1 {
            for i2 in deps_vec_3 {
                if i1 == i2 {
                    same_two_count += 1;
                }
            }
        }
        for i1 in deps_vec_2 {
            for i2 in deps_vec_3 {
                if i1 == i2 {
                    same_two_count += 1;
                }
            }
        }
        lower_mask.count_ones() + deps_vec_1.len_u32() + deps_vec_2.len_u32() + deps_vec_3.len_u32()
            - same_two_count
            + all_three_count
    }

    pub fn combine_deps(&mut self, deps1: u64, deps2: u64, index: Option<i32>) -> u64 {
        let mut buffer: [u32; 1000] = [0; 1000];
        let index_mask = match index {
            Some(value) if value < 40 => 1 << value,
            Some(_value) => 1 << 40,
            None => 0,
        };
        let combined_mask = deps1 | deps2 | index_mask;
        let lower_mask = combined_mask & ((1u64 << 40) - 1);
        if combined_mask == lower_mask {
            return lower_mask;
        }
        if (deps1 >> 40) == (deps2 >> 40) && index_mask < (1 << 40) {
            return lower_mask | (deps1 & !((1u64 << 40) - 1));
        }
        let deps_vec_1 = &self.deps_combinations[(deps1 >> 40) as usize];
        let deps_vec_2 = &self.deps_combinations[(deps2 >> 40) as usize];
        let mut size = deps_vec_1.len() + deps_vec_2.len();
        buffer[0..deps_vec_1.len()].copy_from_slice(deps_vec_1.as_slice());
        buffer[deps_vec_1.len()..size].copy_from_slice(deps_vec_2.as_slice());
        if index.is_some() {
            buffer[size] = (index.unwrap() as u32) - 39u32;
            size += 1;
        }
        buffer[0..size].sort_unstable();
        let mut sum = 0u32;
        let mut sum2 = 0u32;
        let mut j = 0;
        for i in 0..size {
            if i == 0 || buffer[i] != buffer[i - 1] {
                sum += buffer[i];
                sum2 += buffer[i] * buffer[i];
                buffer[j] = buffer[i];
                j += 1;
            }
        }
        let hash = ((sum as u64) << 32u64) | (sum2 as u64);
        let deps_indices = self.deps_indices_by_hashes.get(&hash);
        let index = match deps_indices {
            None => {
                let new_index = self.deps_combinations.len_i32();
                self.deps_indices_by_hashes.insert(hash, vec![new_index]);
                self.deps_combinations.push(buffer[0..j].to_vec());
                new_index
            }
            Some(indices) => {
                let mut found_index = -1;
                for index in indices {
                    if buffer[0..j] == self.deps_combinations[*index as usize] {
                        found_index = *index;
                        break;
                    }
                }
                if found_index < 0 {
                    let new_index = self.deps_combinations.len_i32();
                    self.deps_indices_by_hashes
                        .get_mut(&hash)
                        .unwrap()
                        .push(new_index);
                    self.deps_combinations.push(buffer[0..j].to_vec());
                    new_index
                } else {
                    found_index
                }
            }
        };
        lower_mask | ((index as u64) << 40)
    }

    fn requires_deps(&self, action_deps: &[u64; 3], deps: u64) -> bool {
        if deps == (deps & ((1u64 << 40) - 1)) {
            (action_deps[0] & deps) == deps
                || (action_deps[1] & deps) == deps
                || (action_deps[2] & deps) == deps
        } else {
            for i in 0..3 {
                if self.get_combined_deps_count(action_deps[i], deps)
                    == self.get_deps_count(action_deps[i])
                {
                    return true;
                }
            }
            false
        }
    }

    fn get_solution_deps_list(&self) -> HashSet<u64> {
        let mut deps_list: HashSet<u64> = HashSet::new();
        for i in 0..self.shape_origins.len_i32() {
            if self
                .shape_to_find_mask_by_shape
                .contains_key(self.shape_origins[i as usize].get_shape())
            {
                deps_list.insert(self.shape_origins[i as usize].deps);
            }
        }
        for i in 0..self.point_origins.len_i32() {
            if self
                .found_points
                .contains(self.point_origins[i as usize].point)
            {
                deps_list.insert(self.point_origins[i as usize].deps);
            }
        }
        deps_list
    }

    fn register_point(&mut self, point: Point, shape_origin_indices: [i32; 2]) {
        if self.points.contains_key(point) {
            return;
        }
        if self.points_to_find.contains(point) {
            self.found_points.insert(point);
            self.points_to_find.slow_remove(point);
            if self.points_to_find.len() == 0 && self.shapes_to_find.len() == 0 {
                println!("Solution possibly found!");
            }
        }
        let mut found_shape_mask = 0;
        let deps1 = if shape_origin_indices[0] >= 0 {
            let shape_origin = &self.shape_origins[shape_origin_indices[0] as usize];
            found_shape_mask |= shape_origin.found_shape_mask;
            shape_origin.deps
        } else {
            0
        };
        let deps2 = if shape_origin_indices[1] >= 0 {
            let shape_origin = &self.shape_origins[shape_origin_indices[1] as usize];
            found_shape_mask |= shape_origin.found_shape_mask;
            shape_origin.deps
        } else {
            0
        };
        let combined_deps = self.combine_deps(deps1, deps2, None);
        if self.get_deps_count(combined_deps) > self.problem.action_count {
            println!("Shouldn't happen");
            return;
        }
        self.points
            .insert_if_new(point, self.point_origins.len_i32());
        let index = self.point_origins.len_i32();
        self.point_origins.push(PointOrigin {
            point,
            deps: combined_deps,
            shape_origin_indices,
            found_shape_mask,
        });
        for i in 0..index {
            let maybe_actions = Action::check_action_two_points(self, i, index);
            for maybe_action in maybe_actions {
                match maybe_action {
                    Some(action) => self.queue.push(action),
                    None => (),
                }
            }
        }
        if self.problem.has_point_and_line_actions() {
            for i in 0..self.shape_origins.len_i32() {
                let shape1 = self.shape_origins[i as usize].get_shape();
                match shape1 {
                    Shape::Line(_) => (),
                    _ => continue,
                }
                let maybe_actions = Action::check_action_point_and_line(self, index, i);
                for maybe_action in maybe_actions {
                    match maybe_action {
                        Some(action) => self.queue.push(action),
                        None => (),
                    }
                }
            }
        }
        if self.problem.has_three_point_actions() {
            for i1 in 0..index {
                for i2 in (i1 + 1)..index {
                    let maybe_actions = Action::check_action_three_points(self, i1, i2, index);
                    for maybe_action in maybe_actions {
                        match maybe_action {
                            Some(action) => self.queue.push(action),
                            None => (),
                        }
                    }
                }
            }
        }

        if self.queue.len() > 100000000 {
            println!("Reducing queue size");
            let mut new_queue = BinaryHeap::new();
            for _i in 1..50000 {
                new_queue.push(self.queue.pop().unwrap());
            }
            self.queue = new_queue;
        }
    }

    fn register_shape(&mut self, element_link: ElementLink<'a>) {
        let shape = element_link.get_shape();
        if self.shapes.contains_key(shape)
            && (self.final_found_shape.is_none() || shape != self.final_found_shape.unwrap())
        {
            // Actually one needs to verify whether new dependencies are less/different
            return;
        }
        if self.shapes_to_find.contains(shape) {
            self.found_shapes.insert(shape);
            println!("{}", self.shapes_to_find.len());
            for shape1 in &self.shapes_to_find {
                println!("To find: {}", shape1);
            }
            self.shapes_to_find.slow_remove(shape);
            println!("Shape found: {}", shape);
            println!(
                "{} {}",
                self.points_to_find.len(),
                self.shapes_to_find.len()
            );
            if self.points_to_find.len() == 0 && self.shapes_to_find.len() == 0 {
                println!("Solution possibly found!");
                if self.final_found_shape.is_none() {
                    self.final_found_shape = Some(shape);
                }
            }
        }
        let index = self.shape_origins.len_i32();
        let (combined_deps_with_index, found_shape_mask) = match &element_link {
            ElementLink::GivenElement { .. } => (0, 0),
            ElementLink::Action(action) => action.process(self, index),
        };
        self.shapes.insert_if_new(shape, index);
        self.shape_origins.push(ShapeOrigin {
            element_link,
            deps: combined_deps_with_index,
            found_shape_mask,
        });
        for i in 0..index {
            let shape_origin = &self.shape_origins[i as usize];
            let deps_count =
                self.get_combined_deps_count(combined_deps_with_index, shape_origin.deps);
            let combined_mask = shape_origin.found_shape_mask | found_shape_mask;
            let reserved = self.shape_to_find_mask_by_shape.len_u32() - combined_mask.count_ones();
            if deps_count + reserved <= self.problem.action_count {
                let maybe_points = self.shape_origins[i as usize]
                    .get_shape()
                    .find_intersection_points(&shape);
                for maybe_point in maybe_points {
                    match maybe_point {
                        Some(point) => self.register_point(point, [i, index]),
                        None => (),
                    }
                }
            }
        }
        if matches!(shape, Shape::Line { .. }) && self.problem.has_point_and_line_actions() {
            for i in 0..self.point_origins.len_i32() {
                let point_origin = &self.point_origins[i as usize];
                let deps_count =
                    self.get_combined_deps_count(combined_deps_with_index, point_origin.deps);
                let combined_mask = point_origin.found_shape_mask | found_shape_mask;
                let reserved =
                    self.shape_to_find_mask_by_shape.len_u32() - combined_mask.count_ones();
                if deps_count + reserved <= self.problem.action_count {
                    let maybe_actions = Action::check_action_point_and_line(self, i, index);
                    for maybe_action in maybe_actions {
                        match maybe_action {
                            Some(action) => self.queue.push(action),
                            None => (),
                        }
                    }
                }
            }
        }
    }

    fn register_given_element(&mut self, element: &'a Element) {
        match element {
            Element::Point(point) => self.register_point(*point, [GIVEN, GIVEN]),
            _ => self.register_shape(ElementLink::GivenElement {
                element,
                shape: element.get_shape().unwrap(),
            }),
        };
    }

    pub fn initialize_queue(&mut self) {
        for element in &self.problem.given_elements {
            self.register_given_element(element);
        }
        for (i, element) in self.problem.elements_to_find.iter().enumerate() {
            match element {
                Element::Point(point) => {
                    self.points_to_find.insert(*point);
                }
                _ => {
                    let shape = element.get_shape().unwrap();
                    self.shapes_to_find.insert(shape);
                    self.shape_to_find_mask_by_shape
                        .insert_if_new(shape, 1 << i);
                }
            };
        }
    }

    pub fn solve(&'a mut self) {
        let time = SystemTime::now();
        let mut rw_queue = Vec::new();
        for i in 0..1000000 {
            if self.queue.is_empty() {
                self.draw_state("final.svg".to_string(), 5.0, HashSet::new());
                println!("All actions explored");
                break;
            }
            let mut action = self.queue.pop().unwrap();
            match self.problem.random_walk_at_n_actions {
                Some(n) => {
                    if action.deps_count == n - 2 {
                        rw_queue.push(action);
                        continue;
                    }
                }
                None => (),
            }
            let priority = action.compute_priority(self);
            if priority < 0 {
                println!("Skipping action");
                continue;
            }
            if priority != action.priority {
                action.priority = priority;
                self.queue.push(action);
                continue;
            }
            let shape = action.shape;
            self.register_shape(ElementLink::Action(action));
            if self.final_found_shape.is_some() && shape == self.final_found_shape.unwrap() {
                println!("=== Printing solution! ===");
                self.print_solution();
                self.draw_solution("solution.svg".to_string(), 5.0);
                println!(
                    "Solution found in {} seconds",
                    time.elapsed().unwrap().as_secs()
                );
                // return;
            }
            // if i == 10 {
            //     self.print_state();
            //     self.draw_state(format!("image{}.svg", i), 5.0, HashSet::new());
            // }
            if i % 10 == 0 {
                if false {
                    println!(
                        "Loop {} ({}/{})",
                        i,
                        self.point_origins.len(),
                        self.shape_origins.len()
                    );
                }
                println!(
                    "Loop {}; points: {}, shapes: {}, found: {}+{}, queue: {}, deps: {}, p: {}, time: {}",
                    i,
                    self.point_origins.len(),
                    self.shape_origins.len(),
                    self.found_points.len(),
                    self.found_shapes.len(),
                    self.queue.len(),
                    self.deps_combinations.len(),
                    if self.queue.is_empty() { 0 } else { self.queue.peek().unwrap().priority},
                    time.elapsed().unwrap().as_secs(),
                );
            }
        }
        if !rw_queue.is_empty() {
            println!("Found {} elements in the random walk queue", rw_queue.len());
            let random_walk_parent = self.create_random_walk_parent();
            let random_walks = self.prepare_random_walks(&random_walk_parent, rw_queue);
            println!(
                "Starting random walks, time: {}",
                time.elapsed().unwrap().as_secs(),
            );
            let limit = RANDOM_WALK_LIMIT / random_walks.len_u32();
            let _rw_results: Vec<Option<RandomWalkSolution>> = random_walks
                .par_iter()
                .map(|rw| rw.run_iterations(limit))
                .collect();
            println!(
                "Ended random walks, time: {}",
                time.elapsed().unwrap().as_secs(),
            );
        }
    }
}
