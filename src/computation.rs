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
const RANDOM_WALK_LIMIT: u32 = 50000000;

#[derive(Debug)]
struct PointOrigin {
    point: Point,
    deps: u64,
    shape_origin_indices: [i32; 2],
    found_shape_mask: u32,
    next: i32,
}
struct ShapeOrigin<'a> {
    deps: u64,
    element_link: ElementLink<'a>,
    found_shape_mask: u32,
    next: i32,
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
    solution_deps: Option<u64>,
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
            solution_deps: None,
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

    fn _get_deps_combined_with_index_count(&self, deps: u64, index: i32) -> u32 {
        if index < 40 {
            self.get_deps_count(deps | (1 << index))
        } else if deps < (1u64 << 40) {
            self.get_deps_count(deps) + 1
        } else {
            let deps_vec = &self.deps_combinations[(deps >> 40) as usize];
            if deps_vec.contains(&(index as u32)) {
                self.get_deps_count(deps)
            } else {
                self.get_deps_count(deps) + 1
            }
        }
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

    fn get_index_of_deps_combination(&mut self, high_indices: &[u32]) -> i32 {
        let mut sum = 0u32;
        let mut sum2 = 0u32;
        for i in high_indices {
            sum += *i;
            sum2 += *i * *i;
        }
        let hash = ((sum as u64) << 32u64) | (sum2 as u64);
        let deps_indices = self.deps_indices_by_hashes.get(&hash);
        match deps_indices {
            None => {
                let new_index = self.deps_combinations.len_i32();
                self.deps_indices_by_hashes.insert(hash, vec![new_index]);
                self.deps_combinations.push(high_indices.to_vec());
                new_index
            }
            Some(indices) => {
                let mut found_index = -1;
                for index in indices {
                    if high_indices == self.deps_combinations[*index as usize] {
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
                    self.deps_combinations.push(high_indices.to_vec());
                    new_index
                } else {
                    found_index
                }
            }
        }
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
        let mut j = 0;
        for i in 0..size {
            if i == 0 || buffer[i] != buffer[i - 1] {
                buffer[j] = buffer[i];
                j += 1;
            }
        }

        let index = self.get_index_of_deps_combination(&buffer[0..j]);
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

    fn check_action_and_add_to_results(
        &self,
        maybe_action: Option<Action>,
        results: &mut [Option<Action>],
    ) {
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

    // Order bits of all_deps and use indices corresponding to deps as bits for the result
    fn compress(&self, deps: u64, all_deps: u64) -> u64 {
        let deps_combination = &self.deps_combinations[(deps >> 40) as usize];
        let index_set: HashSet<u32> = HashSet::from_iter(deps_combination.iter().cloned());
        let mut index = 0;
        let mut result = 0u64;
        for i in 0..40 {
            if (all_deps & (1 << i)) != 0 {
                if deps & (1 << i) != 0 {
                    result |= 1 << index;
                }
                index += 1;
            }
        }
        for i in &self.deps_combinations[(all_deps >> 40) as usize] {
            if index_set.contains(i) {
                result |= 1 << index;
            }
            index += 1;
        }
        result
    }

    fn decompress(&mut self, union: u64, all_deps: u64) -> u64 {
        let mut high_indices = [0; 100];
        let mut high_index_count = 0;
        let mut index = 0;
        let mut lower_mask = 0u64;
        for i in 0..40 {
            if (all_deps & (1 << i)) != 0 {
                if union & (1 << index) != 0 {
                    lower_mask |= 1 << i;
                }
                index += 1;
            }
        }
        for i in &self.deps_combinations[(all_deps >> 40) as usize] {
            if union & (1 << index) != 0 {
                high_indices[high_index_count] = *i;
                high_index_count += 1;
            }
            index += 1;
        }
        let dep_combination_index =
            self.get_index_of_deps_combination(&high_indices[0..high_index_count]);
        ((dep_combination_index as u64) << 40) | lower_mask
    }

    fn find_shortest_deps_union(deps_lists: &Vec<Vec<u64>>, union_so_far: u64, index: u32) -> u64 {
        if index == deps_lists.len_u32() {
            return union_so_far;
        }
        let mut min_len = 64;
        let mut best_deps = 0;
        for deps in &deps_lists[index as usize] {
            let to_check =
                Self::find_shortest_deps_union(deps_lists, union_so_far | deps, index + 1);
            let len = to_check.count_ones();
            if len < min_len {
                min_len = len;
                best_deps = to_check;
            }
        }
        best_deps
    }

    fn check_multimatch_solution_found(&mut self) {
        if self.points_to_find.len() > 0 || self.shapes_to_find.len() > 0 {
            return;
        }
        let mut all_relevant_deps = 0;
        let points_copy: Vec<_> = self.found_points.iter().cloned().collect();
        for point in &points_copy {
            let mut index = self.points.get(*point).unwrap();
            while index >= 0 {
                let point_origin_deps = self.point_origins[index as usize].deps;
                all_relevant_deps = self.combine_deps(all_relevant_deps, point_origin_deps, None);
                index = self.point_origins[index as usize].next;
            }
        }
        let shapes_copy: Vec<_> = self.found_shapes.iter().cloned().collect();
        for shape in &shapes_copy {
            let mut index = self.shapes.get(*shape).unwrap();
            while index >= 0 {
                let shape_origin_deps = self.shape_origins[index as usize].deps;
                all_relevant_deps = self.combine_deps(all_relevant_deps, shape_origin_deps, None);
                index = self.shape_origins[index as usize].next;
            }
        }
        // println!(
        //     "All deps: {:b} (count: {})",
        //     all_relevant_deps,
        //     self.get_deps_count(all_relevant_deps)
        // );
        let mut deps_lists = Vec::new();
        for point in &points_copy {
            let mut index = self.points.get(*point).unwrap();
            let mut deps_list = Vec::new();
            while index >= 0 {
                let point_origin_deps = self.point_origins[index as usize].deps;
                deps_list.push(self.compress(point_origin_deps, all_relevant_deps));
                index = self.point_origins[index as usize].next;
            }
            deps_lists.push(deps_list);
        }
        for shape in &shapes_copy {
            let mut index = self.shapes.get(*shape).unwrap();
            let mut deps_list = Vec::new();
            while index >= 0 {
                let shape_origin_deps = self.shape_origins[index as usize].deps;
                deps_list.push(self.compress(shape_origin_deps, all_relevant_deps));
                index = self.shape_origins[index as usize].next;
            }
            deps_lists.push(deps_list);
        }
        // for deps_list in &deps_lists {
        //     println!("--");
        //     for deps in deps_list {
        //         println!("{:b}", deps);
        //     }
        // }
        let shortest_union = Self::find_shortest_deps_union(&deps_lists, 0, 0);
        println!(
            "Deps in the shortest union: {}",
            shortest_union.count_ones()
        );

        if shortest_union.count_ones() <= self.problem.action_count {
            self.solution_deps = Some(self.decompress(shortest_union, all_relevant_deps));
            println!("Solution deps: {:b}", self.solution_deps.unwrap());
        }
    }

    fn update_point_seen_before(&mut self, point: Point, index: i32, deps: u64) -> bool {
        let mut i = self.points.get(point).unwrap();
        while i >= 0 {
            let point_origin = &self.point_origins[i as usize];
            if self.get_combined_deps_count(point_origin.deps, deps) == self.get_deps_count(deps) {
                return false;
            }
            let point_origin = &mut self.point_origins[i as usize];
            i = point_origin.next;
            if i < 0 {
                // println!(
                //     "Point seen multiple times: {} (deps: {:b} / {:b})",
                //     point, point_origin.deps, deps
                // );
                point_origin.next = index;
            }
        }
        true
    }

    fn register_point(&mut self, point: Point, shape_origin_indices: [i32; 2]) {
        let seen_before = self.points.contains_key(point);
        if seen_before && !self.problem.multimatch {
            return;
        }
        if !seen_before && self.points_to_find.contains(point) {
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
            println!(
                "Shouldn't happen: deps count {} is above action count {}",
                self.get_deps_count(combined_deps),
                self.problem.action_count
            );
            return;
        }
        let index = self.point_origins.len_i32();
        if seen_before && !self.update_point_seen_before(point, index, combined_deps) {
            return;
        }
        self.points.insert_if_new(point, index);
        self.point_origins.push(PointOrigin {
            point,
            deps: combined_deps,
            shape_origin_indices,
            found_shape_mask,
            next: -1,
        });
        if self.problem.multimatch && self.found_points.contains(point) {
            self.check_multimatch_solution_found();
        }
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
                if shape1.get_direction().is_none() {
                    continue;
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
        if self.problem.has_two_point_and_line_actions() {
            for i1 in 0..index {
                for i2 in 0..self.shape_origins.len_i32() {
                    let shape1 = self.shape_origins[i2 as usize].get_shape();
                    if shape1.get_direction().is_none() {
                        continue;
                    }
                    let maybe_actions =
                        Action::check_action_two_point_and_line(self, i1, index, i2);
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

    fn update_shape_seen_before(&mut self, shape: Shape, index: i32, deps: u64) -> bool {
        let i0 = self.shapes.get(shape).unwrap();
        let mut i = i0;
        // println!("Shape seen before, i0: {}, index: {}", i0, index);
        while i >= 0 {
            let shape_origin = &self.shape_origins[i as usize];
            // println!(
            //     "Comparing deps: {:b} (i = {}), new: {:b}, equal: {}",
            //     shape_origin.deps,
            //     i,
            //     deps,
            //     deps == shape_origin.deps
            // );
            if self.get_combined_deps_count(shape_origin.deps, deps) == self.get_deps_count(deps) {
                return false;
            }
            let shape_origin = &mut self.shape_origins[i as usize];
            i = shape_origin.next;
            if i < 0 {
                // println!(
                //     "Shape seen multiple times: {} (deps: {:b} / {:b})",
                //     shape, shape_origin.deps, deps
                // );
                shape_origin.next = index;
            }
        }
        // println!("Adding {}", index);
        true
    }

    fn register_shape(&mut self, element_link: ElementLink<'a>) {
        let shape = element_link.get_shape();
        let previous_index = self.shapes.get(shape);
        let seen_before = previous_index.is_some();
        if seen_before && !self.problem.multimatch {
            return;
        }
        if !seen_before && self.shapes_to_find.contains(shape) {
            self.found_shapes.insert(shape);
            self.shapes_to_find.slow_remove(shape);
            if self.points_to_find.len() == 0 && self.shapes_to_find.len() == 0 {
                println!("Solution possibly found!");
            }
        }
        let current_index = self.shape_origins.len_i32();
        let index = previous_index.unwrap_or(current_index);
        let (combined_deps_with_index, found_shape_mask) = match &element_link {
            ElementLink::GivenElement { .. } => (0, 0),
            ElementLink::Action(action) => action.process(self, index),
        };
        if seen_before
            && !self.update_shape_seen_before(shape, current_index, combined_deps_with_index)
        {
            return;
        }
        if !seen_before {
            self.shapes.insert_if_new(shape, index);
        }
        let saved_as_index = self.shape_origins.len_i32();
        self.shape_origins.push(ShapeOrigin {
            element_link,
            deps: combined_deps_with_index,
            found_shape_mask,
            next: -1,
        });
        if self.found_shapes.contains(shape) {
            self.check_multimatch_solution_found();
        }
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
                        Some(point) => self.register_point(point, [i, saved_as_index]),
                        None => (),
                    }
                }
            }
        }
        if shape.get_direction().is_some() && self.problem.has_point_and_line_actions() {
            for i in 0..self.point_origins.len_i32() {
                let maybe_actions = Action::check_action_point_and_line(self, i, index);
                for maybe_action in maybe_actions {
                    match maybe_action {
                        Some(action) => self.queue.push(action),
                        None => (),
                    }
                }
            }
        }
        if shape.get_direction().is_some() && self.problem.has_two_point_and_line_actions() {
            for i1 in 0..self.point_origins.len_i32() {
                for i2 in (i1 + 1)..self.point_origins.len_i32() {
                    let maybe_actions =
                        Action::check_action_two_point_and_line(self, i1, i2, index);
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
            self.register_shape(ElementLink::Action(action));
            if self.solution_deps.is_some() {
                println!("=== Printing solution! ===");
                self.print_solution();
                self.draw_solution("solution.svg".to_string(), 5.0);
                println!(
                    "Solution found in {} seconds",
                    time.elapsed().unwrap().as_secs()
                );
                if !self.problem.find_all_solutions {
                    return;
                }
                self.solution_deps = None;
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
                    "Loop {}; points: {}, shapes: {} ({}), found: {}+{}, queue: {}, deps: {}, p: {}, time: {}",
                    i,
                    self.point_origins.len(),
                    self.shape_origins.len(),
                    self.shapes.len(),
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
            let rw_results: Vec<RandomWalkSolution> = random_walks
                .par_iter()
                .filter_map(|rw| rw.run_iterations(limit))
                .collect();
            println!(
                "Ended random walks, time: {}",
                time.elapsed().unwrap().as_secs(),
            );
            for i in 0..rw_results.len_u32() {
                Self::draw_shapes(
                    &rw_results[i as usize].shapes,
                    format!("rw_solution_{}.svg", i),
                    5.0,
                );
            }
        }
    }
}
