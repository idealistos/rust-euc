use draw::DrawState;
use element::CircleCP;
use element::Element;
use element::LineAB;
use element::MidPerpAB;
use fint::FInt;
use hashset2::HashMap2;
use hashset2::HashSet2;
use hashset2::WithTwoHashes;
use print::PrintState;
use problems::ActionType;
use problems::ProblemDefinition;
// use rayon::prelude::*;
use shape::{Point, Shape};
use std::cmp::{Ord, Ordering};
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::fmt::Display;
use std::time::SystemTime;

mod draw;
mod element;
mod fint;
mod hashset2;
mod print;
mod problems;
mod shape;

const GIVEN: i32 = -1;

#[allow(unused_macros)]
macro_rules! box_array {
    ($val:expr ; $len:expr) => {{
        // Use a generic function so that the pointer cast remains type-safe
        fn vec_to_boxed_array<T>(vec: Vec<T>) -> Box<[T; $len]> {
            let boxed_slice = vec.into_boxed_slice();

            let ptr = ::std::boxed::Box::into_raw(boxed_slice) as *mut [T; $len];

            unsafe { Box::from_raw(ptr) }
        }

        vec_to_boxed_array(vec![$val; $len])
    }};
}

trait VecLengths {
    fn len_u32(&self) -> u32;
    fn len_i32(&self) -> i32;
}
impl<K: WithTwoHashes, V: Copy> VecLengths for HashMap2<K, V> {
    fn len_u32(&self) -> u32 {
        self.len() as u32
    }
    fn len_i32(&self) -> i32 {
        self.len() as i32
    }
}
impl<T> VecLengths for Vec<T> {
    fn len_u32(&self) -> u32 {
        self.len() as u32
    }
    fn len_i32(&self) -> i32 {
        self.len() as i32
    }
}

enum GivenOrNewElement<'a> {
    GivenElement(&'a Element),
    NewElement(Element),
}
impl<'a> Display for GivenOrNewElement<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GivenOrNewElement::GivenElement(e) => {
                let name: &'static str = (*e).into();
                write!(f, "{}", name)
            }
            GivenOrNewElement::NewElement(e) => {
                let name: &'static str = e.into();
                write!(f, "{}", name)
            }
        }
    }
}

#[derive(Debug)]
struct PointOrigin {
    point: Point,
    deps: u64,
    shape_origin_indices: [i32; 2],
    found_shape_mask: u32,
}

struct ShapeOrigin<'a> {
    shape: Shape,
    deps: u64,
    element_or_ref: GivenOrNewElement<'a>,
    point_origin_indices: [i32; 2],
    found_shape_mask: u32,
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct Action {
    priority: i32,
    point_index_1: i32,
    point_index_2: i32,
    action_type: ActionType,
    shape: Shape,
    deps_count: u32,
}
impl PartialOrd for Action {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.priority
                .cmp(&other.priority)
                .then(self.point_index_1.cmp(&other.point_index_1).reverse())
                .then(self.point_index_2.cmp(&other.point_index_2).reverse())
                .then(self.action_type.cmp(&other.action_type).reverse()),
        )
    }
}
impl Ord for Action {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
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
}
impl<'a> Computation<'a> {
    fn new(problem: &'a ProblemDefinition) -> Self {
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

    fn combine_deps(&mut self, deps1: u64, deps2: u64, index: Option<i32>) -> u64 {
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

    fn check_action(&self, i1: i32, i2: i32) -> [Option<Action>; ActionType::Last as usize] {
        let point_origin_1 = &self.point_origins[i1 as usize];
        let point_origin_2 = &self.point_origins[i2 as usize];
        let deps_count = self.get_combined_deps_count(point_origin_1.deps, point_origin_2.deps);
        let found_shape_count =
            (point_origin_1.found_shape_mask | point_origin_2.found_shape_mask).count_ones();
        let reserved = self.shape_to_find_mask_by_shape.len_u32() - found_shape_count;
        const NONE: Option<Action> = None;
        let mut results = [NONE; ActionType::Last as usize];
        if deps_count + reserved > self.problem.action_count {
            return results;
        }
        for action_type in &self.problem.action_types {
            let element = self.create_element(i1, i2, *action_type);
            let new_shape = element.get_shape().unwrap();
            if self.shapes.contains_key(new_shape) {
                continue;
            }
            let priority = self.compute_priority(i1, i2, &new_shape, deps_count);
            if priority > 0 {
                let action = Action {
                    priority,
                    point_index_1: i1,
                    point_index_2: i2,
                    shape: new_shape,
                    action_type: *action_type,
                    deps_count,
                };
                results[*action_type as usize] = Some(action);
            }
        }
        results
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
        // let actions: Vec<Action> = (0..index)
        //     .into_par_iter()
        //     .map(|i| self.check_action(i, index))
        //     .flatten()
        //     .filter_map(|x| x)
        //     .collect();
        // actions
        //     .into_iter()
        //     .for_each(|action| self.queue.push(action));
        for i in 0..index {
            let maybe_actions = self.check_action(i, index);
            for maybe_action in maybe_actions {
                match maybe_action {
                    Some(action) => self.queue.push(action),
                    None => (),
                }
            }
        }
        if self.queue.len() > 100000 {
            println!("Reducing queue size");
            let mut new_queue = BinaryHeap::new();
            for _i in 1..50000 {
                new_queue.push(self.queue.pop().unwrap());
            }
            self.queue = new_queue;
        }
    }

    fn register_shape(
        &mut self,
        shape: Shape,
        element_or_ref: GivenOrNewElement<'a>,
        point_origin_indices: [i32; 2],
    ) {
        if self.shapes.contains_key(shape) {
            // Actually one needs to verify whether new dependencies are less/different
            return;
        }
        if self.shapes_to_find.contains(shape) {
            self.found_shapes.insert(shape);
            self.shapes_to_find.slow_remove(shape);
            if self.points_to_find.len() == 0 && self.shapes_to_find.len() == 0 {
                println!("Solution possibly found!");
            }
        }
        let index = self.shape_origins.len_i32();
        let mut combined_deps_with_index = 0;
        let mut found_shape_mask = 0;
        match element_or_ref {
            GivenOrNewElement::GivenElement(_) => (),
            GivenOrNewElement::NewElement(_) => {
                let point_origin_1 = &self.point_origins[point_origin_indices[0] as usize];
                let point_origin_2 = &self.point_origins[point_origin_indices[1] as usize];
                combined_deps_with_index =
                    self.combine_deps(point_origin_1.deps, point_origin_2.deps, Some(index));
                let point_origin_1 = &self.point_origins[point_origin_indices[0] as usize];
                let point_origin_2 = &self.point_origins[point_origin_indices[1] as usize];
                found_shape_mask =
                    point_origin_1.found_shape_mask | point_origin_2.found_shape_mask;
                match self.shape_to_find_mask_by_shape.get(shape) {
                    None => (),
                    Some(mask) => {
                        found_shape_mask |= mask;
                    }
                }
            }
        };
        self.shapes.insert_if_new(shape, index);
        self.shape_origins.push(ShapeOrigin {
            shape,
            element_or_ref,
            deps: combined_deps_with_index,
            point_origin_indices,
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
                    .shape
                    .as_trait()
                    .find_intersection_points(&shape);
                for maybe_point in maybe_points {
                    match maybe_point {
                        Some(point) => self.register_point(point, [i, index]),
                        None => (),
                    }
                }
            }
        }
    }

    fn register_given_element(&mut self, element: &'a Element) {
        match element {
            Element::Point(point) => self.register_point(*point, [GIVEN, GIVEN]),
            _ => self.register_shape(
                element.get_shape().unwrap(),
                GivenOrNewElement::GivenElement(element),
                [GIVEN, GIVEN],
            ),
        };
    }

    fn create_element(
        &self,
        point_index_1: i32,
        point_index_2: i32,
        action_type: ActionType,
    ) -> Element {
        match action_type {
            ActionType::Line => Element::LineAB(LineAB {
                a: self.point_origins[point_index_1 as usize].point,
                b: self.point_origins[point_index_2 as usize].point,
            }),
            ActionType::Circle12 => Element::CircleCP(CircleCP {
                c: self.point_origins[point_index_1 as usize].point,
                p: self.point_origins[point_index_2 as usize].point,
            }),
            ActionType::Circle21 => Element::CircleCP(CircleCP {
                c: self.point_origins[point_index_2 as usize].point,
                p: self.point_origins[point_index_1 as usize].point,
            }),
            ActionType::MidPerp => Element::MidPerpAB(MidPerpAB {
                a: self.point_origins[point_index_1 as usize].point,
                b: self.point_origins[point_index_2 as usize].point,
            }),
            _ => panic!("Can't happen"),
        }
    }

    fn create_and_register_element(&mut self, action: &Action) {
        let element = self.create_element(
            action.point_index_1,
            action.point_index_2,
            action.action_type,
        );
        self.register_shape(
            action.shape,
            GivenOrNewElement::NewElement(element),
            [action.point_index_1, action.point_index_2],
        );
    }

    fn initialize_queue(&mut self) {
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

    // Priority rules for an action (point1 + point2 + action number (0-2)):
    // - if a point is in points_to_find, +1
    // - if a point lies on a shape in an unregistered shapes_to_find, +5
    // - if the resulting shape is in shapes_to_find, +10
    // - if the resulting shape passes through an unregistered point in points_to_find, +5
    // Base priority = 2 * (num_actions - (dep count of point1) - (dep count of point2))
    fn compute_priority(
        &self,
        point_index_1: i32,
        point_index_2: i32,
        shape: &Shape,
        deps_count: u32,
    ) -> i32 {
        let origin1 = &self.point_origins[point_index_1 as usize];
        let origin2 = &self.point_origins[point_index_2 as usize];
        let mut found_shape_mask = origin1.found_shape_mask | origin2.found_shape_mask;
        match self.shape_to_find_mask_by_shape.get(*shape) {
            Some(mask) => {
                found_shape_mask |= mask;
            }
            None => (),
        }
        let reserved = self.shape_to_find_mask_by_shape.len_u32() - found_shape_mask.count_ones();
        if deps_count + reserved >= self.problem.action_count {
            return -1;
        }
        let mut priority: i32 = 2 * ((self.problem.action_count as i32) - (deps_count as i32));
        if self.found_points.contains(origin1.point) {
            priority += 1;
        }
        if self.found_points.contains(origin2.point) {
            priority += 1;
        }
        for shape in &self.shapes_to_find {
            let shape_obj = shape.as_trait();
            if shape_obj.contains_point(&origin1.point) {
                priority += 5;
            }
            if shape_obj.contains_point(&origin2.point) {
                priority += 5;
            }
        }
        if self.shapes_to_find.contains(*shape) {
            priority += 20;
        }
        for point in &self.points_to_find {
            let shape_obj = shape.as_trait();
            if shape_obj.contains_point(&point) {
                priority += 5;
            }
        }
        // if deps_count <= 0 {
        //     priority += 30;
        // }
        // for point in &self.found_points {
        //     let shape_obj = shape.as_trait();
        //     if shape_obj.contains_point(&point) {
        //         priority += 3;
        //     }
        // }
        priority
    }

    fn solve(&mut self) {
        let time = SystemTime::now();
        for i in 0..1000000 {
            if self.queue.is_empty() {
                println!("All actions explored");
                return;
            }
            let mut action = self.queue.pop().unwrap();
            let priority = self.compute_priority(
                action.point_index_1,
                action.point_index_2,
                &action.shape,
                action.deps_count,
            );
            if priority < 0 {
                println!("Skipping action");
                continue;
            }
            if priority != action.priority {
                action.priority = priority;
                self.queue.push(action);
                continue;
            }
            self.create_and_register_element(&action);
            if self.found_points.len() == self.points_to_find.len()
                && self.found_shapes.len() == self.shapes_to_find.len()
            {
                println!("=== Printing solution! ===");
                self.print_state();
                self.print_shapes(Some(self.shape_origins.len_i32() - 1));
                self.draw_state(
                    "solution.svg".to_string(),
                    5.0,
                    Some(self.shape_origins.len_i32() - 1),
                );
                println!("Finished in {} seconds", time.elapsed().unwrap().as_secs());
                return;
            }
            if i == 10 || i == 30 || i == 120 || i == 300 {
                self.print_state();
                self.draw_state(format!("image{}.svg", i), 5.0, None);
            }
            if i % 10 == 0 {
                println!(
                    "Loop {}; points: {} ({}), shapes: {} ({}), found points: {}, found shapes: {}, queue size: {}, deps: {}/{}, priority: {}, time: {}",
                    i,
                    self.point_origins.len(),
                    self.points.len(),
                    self.shape_origins.len(),
                    self.shapes.len(),
                    self.found_points.len(),
                    self.found_shapes.len(),
                    self.queue.len(),
                    self.deps_combinations.len(),
                    self.deps_indices_by_hashes.len(),
                    self.queue.peek().unwrap().priority,
                    time.elapsed().unwrap().as_secs(),
                );
            }
        }
    }
}

struct Main();

impl Main {
    fn compute() {
        let problem = ProblemDefinition::get_problem();
        // let problem = Self::midpoint_problem_1_3();
        let mut computation = Computation::new(&problem);

        computation.initialize_queue();
        computation.print_state();
        // computation.queue.push((10, 2, 3));
        // computation.queue.push((11, 3, 32));
        // computation.queue.push((1, 300, 2));
        // println!("{:?}", computation.queue.pop());
        // println!("{:?}", computation.queue.pop());
        // println!("{:?}", computation.queue.pop());

        // computation.register_element(Element::LineAB(LineAB { a: p1, b: p2 }));
        println!("Running...");
        computation.solve();

        println!("Finished");
    }

    fn test() {
        FInt::new(2.0);
    }
}

fn main() {
    Main::test();
    Main::compute();
}
