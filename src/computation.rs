pub use draw::DrawState;
pub use print::PrintState;
use random_walk::RandomWalkProcessing;
use random_walk::RandomWalkSolution;

use crate::element::CircleCP;
use crate::element::Element;
use crate::element::LineAB;
use crate::element::LineAV;
use crate::element::MidPerpAB;
use crate::hashset2::HashMap2;
use crate::hashset2::HashSet2;
use crate::problems::ActionType;
use crate::problems::PointAndLineActionType;
use crate::problems::ProblemDefinition;
use crate::problems::TwoPointActionType;
use crate::shape::ShapeTrait;
use crate::shape::{Point, Shape};
use crate::VecLengths;
use rayon::prelude::*;
use std::cmp::{Ord, Ordering};
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::time::SystemTime;

mod draw;
mod print;
mod random_walk;

const GIVEN: i32 = -1;
const RANDOM_WALK_LIMIT: u32 = 500000000;

enum GivenOrNewElement<'a> {
    GivenElement { element: &'a Element, shape: Shape },
    TwoPointElement { element: Element, action: Action },
    PointAndLineElement { element: Element, action: Action },
}
impl<'a> Display for GivenOrNewElement<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GivenOrNewElement::GivenElement { element, shape: _ } => {
                let name: &'static str = (*element).into();
                write!(f, "{}", name)
            }
            GivenOrNewElement::TwoPointElement { element, action: _ } => {
                let name: &'static str = element.into();
                write!(f, "{}", name)
            }
            GivenOrNewElement::PointAndLineElement { element, action: _ } => {
                let name: &'static str = element.into();
                write!(f, "{}", name)
            }
        }
    }
}
impl<'a> GivenOrNewElement<'a> {
    pub fn get_shape(&self) -> Shape {
        match &self {
            GivenOrNewElement::GivenElement { element: _, shape } => *shape,
            GivenOrNewElement::TwoPointElement { element: _, action } => action.shape,
            GivenOrNewElement::PointAndLineElement { element: _, action } => action.shape,
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
    deps: u64,
    element_or_ref: GivenOrNewElement<'a>,
    found_shape_mask: u32,
}
impl<'a> ShapeOrigin<'a> {
    pub fn get_shape(&self) -> Shape {
        self.element_or_ref.get_shape()
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct Action {
    priority: i32,
    point_index_1: i32,
    point_index_2: i32,
    extra_index: i32,
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
                .then(self.extra_index.cmp(&other.extra_index).reverse())
                .then(self.action_type.cmp(&other.action_type).reverse()),
        )
    }
}
impl Ord for Action {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl Action {
    fn compute_priority(&self, comp: &Computation) -> i32 {
        self.action_type.compute_priority(
            comp,
            self.point_index_1,
            self.point_index_2,
            self.extra_index,
            &self.shape,
            self.deps_count,
        )
    }

    fn get_action_index(&self) -> usize {
        match self.action_type {
            ActionType::TwoPointActionType(value) => value as usize,
            ActionType::PointAndLineActionType(value) => value as usize,
        }
    }
}

trait PriorityComputation {
    fn compute_priority(
        self,
        comp: &Computation,
        point_index_1: i32,
        point_index_2: i32,
        extra_index: i32,
        shape: &Shape,
        deps_count: u32,
    ) -> i32;
}
impl PriorityComputation for ActionType {
    fn compute_priority(
        self,
        comp: &Computation,
        point_index_1: i32,
        point_index_2: i32,
        extra_index: i32,
        shape: &Shape,
        deps_count: u32,
    ) -> i32 {
        match self {
            Self::TwoPointActionType(value) => {
                value.compute_priority(comp, point_index_1, point_index_2, -1, shape, deps_count)
            }
            Self::PointAndLineActionType(value) => {
                value.compute_priority(comp, point_index_1, -1, extra_index, shape, deps_count)
            }
        }
    }
}
impl PriorityComputation for TwoPointActionType {
    // Priority rules for an action (point1 + point2 + action number (0-2)):
    // - if a point is in points_to_find, +1
    // - if a point lies on a shape in an unregistered shapes_to_find, +5
    // - if the resulting shape is in shapes_to_find, +10
    // - if the resulting shape passes through an unregistered point in points_to_find, +5
    // Base priority = 2 * (num_actions - (dep count of point1) - (dep count of point2))
    fn compute_priority(
        self,
        comp: &Computation,
        point_index_1: i32,
        point_index_2: i32,
        _extra_index: i32,
        shape: &Shape,
        deps_count: u32,
    ) -> i32 {
        match comp.problem.random_walk_at_n_actions {
            Some(n) => {
                if deps_count >= n - 1 {
                    return -1;
                }
            }
            None => (),
        }
        let origin1 = &comp.point_origins[point_index_1 as usize];
        let origin2 = &comp.point_origins[point_index_2 as usize];
        let mut found_shape_mask = origin1.found_shape_mask | origin2.found_shape_mask;
        match comp.shape_to_find_mask_by_shape.get(*shape) {
            Some(mask) => {
                found_shape_mask |= mask;
            }
            None => (),
        }
        let reserved = comp.shape_to_find_mask_by_shape.len_u32() - found_shape_mask.count_ones();
        if deps_count + reserved >= comp.problem.action_count {
            return -1;
        }
        let mut priority: i32 = 2 * ((comp.problem.action_count as i32) - (deps_count as i32));
        if deps_count <= 2 && comp.problem.prioritize_low_action_count_shapes {
            priority += (3 - (deps_count as i32)) * 50;
        }
        if comp.found_points.contains(origin1.point) {
            priority += 1;
        }
        if comp.found_points.contains(origin2.point) {
            priority += 1;
        }
        for shape in &comp.shapes_to_find {
            if shape.contains_point(&origin1.point) {
                priority += 5;
            }
            if shape.contains_point(&origin2.point) {
                priority += 5;
            }
        }
        if comp.shapes_to_find.contains(*shape) {
            priority += 20;
        }
        for point in &comp.points_to_find {
            if shape.contains_point(&point) {
                priority += 5;
            }
        }
        priority
    }
}
impl PriorityComputation for PointAndLineActionType {
    // Priority rules for a "point + line" action:
    // - if the point is in points_to_find, +1
    // - if the line is in shapes_to_find, +1
    // - if the point lies on a shape in an unregistered shapes_to_find, +5
    // - if the resulting shape is in shapes_to_find, +10
    // - if the resulting shape passes through an unregistered point in points_to_find, +5
    // Base priority = 2 * (num_actions - (dep count of point) - (dep count of line))
    fn compute_priority(
        self,
        comp: &Computation,
        point_index_1: i32,
        _point_index_2: i32,
        extra_index: i32,
        shape: &Shape,
        deps_count: u32,
    ) -> i32 {
        match comp.problem.random_walk_at_n_actions {
            Some(n) => {
                if deps_count >= n {
                    return -1;
                }
            }
            None => (),
        }
        let point_origin = &comp.point_origins[point_index_1 as usize];
        let line_origin = &comp.shape_origins[extra_index as usize];
        let mut found_shape_mask = point_origin.found_shape_mask | line_origin.found_shape_mask;
        match comp.shape_to_find_mask_by_shape.get(*shape) {
            Some(mask) => {
                found_shape_mask |= mask;
            }
            None => (),
        }
        let reserved = comp.shape_to_find_mask_by_shape.len_u32() - found_shape_mask.count_ones();
        if deps_count + reserved >= comp.problem.action_count {
            return -1;
        }
        let mut priority: i32 = 2 * ((comp.problem.action_count as i32) - (deps_count as i32));
        if deps_count <= 2 && comp.problem.prioritize_low_action_count_shapes {
            priority += (3 - (deps_count as i32)) * 50;
        }
        if comp.found_points.contains(point_origin.point) {
            priority += 1;
        }
        if comp.found_shapes.contains(line_origin.get_shape()) {
            priority += 1;
        }
        for shape in &comp.shapes_to_find {
            if shape.contains_point(&point_origin.point) {
                priority += 5;
            }
        }
        for point in &comp.points_to_find {
            let line_shape = line_origin.get_shape();
            if line_shape.contains_point(point) {
                priority += 5;
            }
        }
        if comp.shapes_to_find.contains(*shape) {
            priority += 20;
        }
        for point in &comp.points_to_find {
            if shape.contains_point(&point) {
                priority += 5;
            }
        }
        priority
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

    fn get_action_deps(&self, action: &Action) -> [u64; 3] {
        match action.action_type {
            ActionType::TwoPointActionType(_) => [
                self.point_origins[action.point_index_1 as usize].deps,
                self.point_origins[action.point_index_2 as usize].deps,
                0,
            ],
            ActionType::PointAndLineActionType(_) => [
                self.point_origins[action.point_index_1 as usize].deps,
                self.shape_origins[action.extra_index as usize].deps,
                0,
            ],
        }
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

    fn check_action_two_points(
        &self,
        i1: i32,
        i2: i32,
    ) -> [Option<Action>; TwoPointActionType::Last as usize] {
        let point_origin_1 = &self.point_origins[i1 as usize];
        let point_origin_2 = &self.point_origins[i2 as usize];
        let deps_count = self.get_combined_deps_count(point_origin_1.deps, point_origin_2.deps);
        let found_shape_count =
            (point_origin_1.found_shape_mask | point_origin_2.found_shape_mask).count_ones();
        let reserved = self.shape_to_find_mask_by_shape.len_u32() - found_shape_count;
        const NONE: Option<Action> = None;
        let mut results = [NONE; TwoPointActionType::Last as usize];
        if deps_count + reserved > self.problem.action_count {
            return results;
        }
        for action_type in self.problem.action_types {
            let maybe_action = match action_type {
                ActionType::TwoPointActionType(two_point_action_type) => {
                    let element = Self::create_two_point_element(
                        &self.point_origins[i1 as usize].point,
                        &self.point_origins[i2 as usize].point,
                        *two_point_action_type,
                    );
                    let new_shape = element.get_shape().unwrap();
                    if self.shapes.contains_key(new_shape) {
                        None
                    } else {
                        Some(Action {
                            priority: 0,
                            point_index_1: i1,
                            point_index_2: i2,
                            extra_index: -1,
                            shape: new_shape,
                            action_type: *action_type,
                            deps_count,
                        })
                    }
                }
                _ => None,
            };
            match maybe_action {
                Some(mut action) => {
                    let priority = action.compute_priority(self);
                    if priority > 0 {
                        action.priority = priority;
                        let index = action.get_action_index();
                        results[index] = Some(action);
                    }
                }
                None => (),
            }
        }
        results
    }

    fn check_action_point_and_line(
        &self,
        i_point: i32,
        i_line: i32,
    ) -> [Option<Action>; PointAndLineActionType::Last as usize] {
        let point_origin = &self.point_origins[i_point as usize];
        let line_origin = &self.shape_origins[i_line as usize];
        let deps_count = self.get_combined_deps_count(point_origin.deps, line_origin.deps);
        let found_shape_count =
            (point_origin.found_shape_mask | line_origin.found_shape_mask).count_ones();
        let reserved = self.shape_to_find_mask_by_shape.len_u32() - found_shape_count;
        const NONE: Option<Action> = None;
        let mut results = [NONE; PointAndLineActionType::Last as usize];
        if deps_count + reserved > self.problem.action_count {
            return results;
        }
        for action_type in self.problem.action_types {
            let maybe_action = match action_type {
                ActionType::PointAndLineActionType(point_and_line_action_type) => {
                    let element = Self::create_point_and_line_element(
                        &self.point_origins[i_point as usize].point,
                        &self.shape_origins[i_line as usize].get_shape(),
                        *point_and_line_action_type,
                    );
                    let new_shape = element.get_shape().unwrap();
                    if self.shapes.contains_key(new_shape) {
                        None
                    } else {
                        Some(Action {
                            priority: 0,
                            point_index_1: i_point,
                            point_index_2: -1,
                            extra_index: i_line,
                            shape: new_shape,
                            action_type: *action_type,
                            deps_count,
                        })
                    }
                }
                _ => None,
            };
            match maybe_action {
                Some(mut action) => {
                    let priority = action.compute_priority(self);
                    if priority > 0 {
                        action.priority = priority;
                        let index = action.get_action_index();
                        results[index] = Some(action);
                    }
                }
                None => (),
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
            let maybe_actions = self.check_action_two_points(i, index);
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
                let maybe_actions = self.check_action_point_and_line(index, i);
                for maybe_action in maybe_actions {
                    match maybe_action {
                        Some(action) => self.queue.push(action),
                        None => (),
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

    fn register_shape(&mut self, element_or_ref: GivenOrNewElement<'a>) {
        let shape = element_or_ref.get_shape();
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
        match &element_or_ref {
            GivenOrNewElement::GivenElement { .. } => (),
            GivenOrNewElement::TwoPointElement { element: _, action } => {
                let point_origin_1 = &self.point_origins[action.point_index_1 as usize];
                let point_origin_2 = &self.point_origins[action.point_index_2 as usize];
                combined_deps_with_index =
                    self.combine_deps(point_origin_1.deps, point_origin_2.deps, Some(index));
                let point_origin_1 = &self.point_origins[action.point_index_1 as usize];
                let point_origin_2 = &self.point_origins[action.point_index_2 as usize];
                found_shape_mask =
                    point_origin_1.found_shape_mask | point_origin_2.found_shape_mask;
                match self.shape_to_find_mask_by_shape.get(shape) {
                    None => (),
                    Some(mask) => {
                        found_shape_mask |= mask;
                    }
                }
            }
            GivenOrNewElement::PointAndLineElement { element: _, action } => {
                let point_origin = &self.point_origins[action.point_index_1 as usize];
                let line_origin = &self.shape_origins[action.extra_index as usize];
                combined_deps_with_index =
                    self.combine_deps(point_origin.deps, line_origin.deps, Some(index));
                let point_origin = &self.point_origins[action.point_index_1 as usize];
                let line_origin = &self.shape_origins[action.extra_index as usize];
                found_shape_mask = point_origin.found_shape_mask | line_origin.found_shape_mask;
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
            element_or_ref,
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
                    let maybe_actions = self.check_action_point_and_line(i, index);
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
            _ => self.register_shape(GivenOrNewElement::GivenElement {
                element,
                shape: element.get_shape().unwrap(),
            }),
        };
    }

    fn create_two_point_element(
        point1: &Point,
        point2: &Point,
        action_type: TwoPointActionType,
    ) -> Element {
        match action_type {
            TwoPointActionType::Line => Element::LineAB(LineAB {
                a: *point1,
                b: *point2,
            }),
            TwoPointActionType::Circle12 => Element::CircleCP(CircleCP {
                c: *point1,
                p: *point2,
            }),
            TwoPointActionType::Circle21 => Element::CircleCP(CircleCP {
                c: *point2,
                p: *point1,
            }),
            TwoPointActionType::MidPerp => Element::MidPerpAB(MidPerpAB {
                a: *point1,
                b: *point2,
            }),
            TwoPointActionType::Last => panic!("Can't happen"),
        }
    }

    fn create_point_and_line_element(
        point: &Point,
        line: &Shape,
        action_type: PointAndLineActionType,
    ) -> Element {
        match action_type {
            PointAndLineActionType::Perp => Element::LineAV(LineAV {
                a: *point,
                v: line.get_direction().unwrap().rotated_90_pos(),
            }),
            PointAndLineActionType::Par => Element::LineAV(LineAV {
                a: *point,
                v: line.get_direction().unwrap(),
            }),
            PointAndLineActionType::Last => panic!("Can't happen"),
        }
    }

    fn create_and_register_element(&mut self, action: Action) {
        let element_or_ref = match action.action_type {
            ActionType::TwoPointActionType(two_point_action_type) => {
                GivenOrNewElement::TwoPointElement {
                    element: Self::create_two_point_element(
                        &self.point_origins[action.point_index_1 as usize].point,
                        &self.point_origins[action.point_index_2 as usize].point,
                        two_point_action_type,
                    ),
                    action,
                }
            }
            ActionType::PointAndLineActionType(point_and_line_action_type) => {
                GivenOrNewElement::PointAndLineElement {
                    element: Self::create_point_and_line_element(
                        &self.point_origins[action.point_index_1 as usize].point,
                        &self.shape_origins[action.extra_index as usize].get_shape(),
                        point_and_line_action_type,
                    ),
                    action,
                }
            }
        };
        self.register_shape(element_or_ref);
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
                // self.print_state();
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
            self.create_and_register_element(action);
            if self.found_points.len() == self.points_to_find.len()
                && self.found_shapes.len() == self.shapes_to_find.len()
            {
                println!("=== Printing solution! ===");
                self.print_solution();
                self.draw_solution("solution.svg".to_string(), 5.0);
                println!("Finished in {} seconds", time.elapsed().unwrap().as_secs());
                return;
            }
            // if i == 10 || i == 30 || i == 120 || i == 300 {
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
