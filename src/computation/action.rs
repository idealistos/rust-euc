use std::{cmp::Ordering, fmt::Display};

use crate::element::BisectorCVV;

use super::*;

pub enum ElementLink<'a> {
    GivenElement { element: &'a Element, shape: Shape },
    Action(Action),
}
impl<'a> Display for ElementLink<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElementLink::GivenElement { element, shape: _ } => {
                let name: &'static str = (*element).into();
                write!(f, "{}", name)
            }
            ElementLink::Action(action) => {
                write!(f, "{:?}", action.action_type)
            }
        }
    }
}
impl<'a> ElementLink<'a> {
    pub fn get_shape(&self) -> Shape {
        match &self {
            ElementLink::GivenElement { element: _, shape } => *shape,
            ElementLink::Action(action) => action.shape,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Action {
    pub priority: i32,
    pub deps_count: u32,
    pub shape: Shape,
    pub action_type: ActionType,
    point_index_1: i32,
    point_index_2: i32,
    extra_index: i32,
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
    pub fn create_two_point_element(
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

    fn create_three_point_element(
        point1: &Point,
        point2: &Point,
        point3: &Point,
        action_type: ThreePointActionType,
    ) -> Element {
        match action_type {
            ThreePointActionType::CircleCAB => Element::CircleCR(CircleCR {
                c: *point1,
                r: point2.distance_to(point3),
            }),
            ThreePointActionType::CircleACB => Element::CircleCR(CircleCR {
                c: *point2,
                r: point1.distance_to(point3),
            }),
            ThreePointActionType::CircleABC => Element::CircleCR(CircleCR {
                c: *point3,
                r: point1.distance_to(point2),
            }),
            ThreePointActionType::BisectorCAB => Element::BisectorCVV(BisectorCVV {
                c: *point1,
                v1: Point(point2.0 - point1.0, point2.1 - point1.1),
                v2: Point(point3.0 - point1.0, point3.1 - point1.1),
            }),
            ThreePointActionType::BisectorACB => Element::BisectorCVV(BisectorCVV {
                c: *point2,
                v1: Point(point1.0 - point2.0, point1.1 - point2.1),
                v2: Point(point3.0 - point2.0, point3.1 - point2.1),
            }),
            ThreePointActionType::BisectorABC => Element::BisectorCVV(BisectorCVV {
                c: *point3,
                v1: Point(point1.0 - point3.0, point1.1 - point3.1),
                v2: Point(point2.0 - point3.0, point2.1 - point3.1),
            }),
            ThreePointActionType::Last => panic!("Can't happen"),
        }
    }

    pub fn process(&self, comp: &mut Computation, index: i32) -> (u64, u32) {
        let combined_deps_with_index;
        let mut found_shape_mask;
        match self.action_type {
            ActionType::TwoPointActionType(_) => {
                let point_origin_1 = &comp.point_origins[self.point_index_1 as usize];
                let point_origin_2 = &comp.point_origins[self.point_index_2 as usize];
                combined_deps_with_index =
                    comp.combine_deps(point_origin_1.deps, point_origin_2.deps, Some(index));
                let point_origin_1 = &comp.point_origins[self.point_index_1 as usize];
                let point_origin_2 = &comp.point_origins[self.point_index_2 as usize];
                found_shape_mask =
                    point_origin_1.found_shape_mask | point_origin_2.found_shape_mask;
            }
            ActionType::PointAndLineActionType(_) => {
                let point_origin = &comp.point_origins[self.point_index_1 as usize];
                let line_origin = &comp.shape_origins[self.extra_index as usize];
                combined_deps_with_index =
                    comp.combine_deps(point_origin.deps, line_origin.deps, Some(index));
                let point_origin = &comp.point_origins[self.point_index_1 as usize];
                let line_origin = &comp.shape_origins[self.extra_index as usize];
                found_shape_mask = point_origin.found_shape_mask | line_origin.found_shape_mask;
            }
            ActionType::ThreePointActionType(_) => {
                let point_origin_1 = &comp.point_origins[self.point_index_1 as usize];
                let point_origin_2 = &comp.point_origins[self.point_index_2 as usize];
                let deps_temp =
                    comp.combine_deps(point_origin_1.deps, point_origin_2.deps, Some(index));
                let point_origin_3 = &comp.point_origins[self.extra_index as usize];
                combined_deps_with_index = comp.combine_deps(deps_temp, point_origin_3.deps, None);
                let point_origin_1 = &comp.point_origins[self.point_index_1 as usize];
                let point_origin_2 = &comp.point_origins[self.point_index_2 as usize];
                let point_origin_3 = &comp.point_origins[self.extra_index as usize];
                found_shape_mask = point_origin_1.found_shape_mask
                    | point_origin_2.found_shape_mask
                    | point_origin_3.found_shape_mask;
            }
        }
        match comp.shape_to_find_mask_by_shape.get(self.shape) {
            None => (),
            Some(mask) => {
                found_shape_mask |= mask;
            }
        }
        (combined_deps_with_index, found_shape_mask)
    }

    pub fn check_action_two_points(
        comp: &Computation,
        i1: i32,
        i2: i32,
    ) -> [Option<Self>; TwoPointActionType::Last as usize] {
        let point_origin_1 = &comp.point_origins[i1 as usize];
        let point_origin_2 = &comp.point_origins[i2 as usize];
        let deps_count = comp.get_combined_deps_count(point_origin_1.deps, point_origin_2.deps);
        let found_shape_count =
            (point_origin_1.found_shape_mask | point_origin_2.found_shape_mask).count_ones();
        let reserved = comp.shape_to_find_mask_by_shape.len_u32() - found_shape_count;
        const NONE: Option<Action> = None;
        let mut results = [NONE; TwoPointActionType::Last as usize];
        if deps_count + reserved > comp.problem.action_count {
            return results;
        }
        for action_type in comp.problem.action_types {
            let maybe_action = match action_type {
                ActionType::TwoPointActionType(two_point_action_type) => {
                    let element = Self::create_two_point_element(
                        &comp.point_origins[i1 as usize].point,
                        &comp.point_origins[i2 as usize].point,
                        *two_point_action_type,
                    );
                    let new_shape = element.get_shape().unwrap();
                    if comp.shapes.contains_key(new_shape) {
                        None
                    } else {
                        Some(Action {
                            priority: 0,
                            deps_count,
                            shape: new_shape,
                            action_type: *action_type,
                            point_index_1: i1,
                            point_index_2: i2,
                            extra_index: -1,
                        })
                    }
                }
                _ => None,
            };
            match maybe_action {
                Some(mut action) => {
                    let priority = action.compute_priority(comp);
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

    pub fn check_action_point_and_line(
        comp: &Computation,
        i_point: i32,
        i_line: i32,
    ) -> [Option<Self>; PointAndLineActionType::Last as usize] {
        let point_origin = &comp.point_origins[i_point as usize];
        let line_origin = &comp.shape_origins[i_line as usize];
        let deps_count = comp.get_combined_deps_count(point_origin.deps, line_origin.deps);
        let found_shape_count =
            (point_origin.found_shape_mask | line_origin.found_shape_mask).count_ones();
        let reserved = comp.shape_to_find_mask_by_shape.len_u32() - found_shape_count;
        const NONE: Option<Action> = None;
        let mut results = [NONE; PointAndLineActionType::Last as usize];
        if deps_count + reserved > comp.problem.action_count {
            return results;
        }
        for action_type in comp.problem.action_types {
            let maybe_action = match action_type {
                ActionType::PointAndLineActionType(point_and_line_action_type) => {
                    let element = Self::create_point_and_line_element(
                        &comp.point_origins[i_point as usize].point,
                        &comp.shape_origins[i_line as usize].get_shape(),
                        *point_and_line_action_type,
                    );
                    let new_shape = element.get_shape().unwrap();
                    if comp.shapes.contains_key(new_shape) {
                        None
                    } else {
                        Some(Action {
                            priority: 0,
                            deps_count,
                            shape: new_shape,
                            action_type: *action_type,
                            point_index_1: i_point,
                            point_index_2: -1,
                            extra_index: i_line,
                        })
                    }
                }
                _ => None,
            };
            match maybe_action {
                Some(mut action) => {
                    let priority = action.compute_priority(comp);
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

    pub fn check_action_three_points(
        comp: &Computation,
        i1: i32,
        i2: i32,
        i3: i32,
    ) -> [Option<Self>; ThreePointActionType::Last as usize] {
        let point_origin_1 = &comp.point_origins[i1 as usize];
        let point_origin_2 = &comp.point_origins[i2 as usize];
        let point_origin_3 = &comp.point_origins[i3 as usize];
        let deps_count = comp.get_combined_three_deps_count(
            point_origin_1.deps,
            point_origin_2.deps,
            point_origin_3.deps,
        );
        let found_shape_count = (point_origin_1.found_shape_mask
            | point_origin_2.found_shape_mask
            | point_origin_3.found_shape_mask)
            .count_ones();
        let reserved = comp.shape_to_find_mask_by_shape.len_u32() - found_shape_count;
        const NONE: Option<Action> = None;
        let mut results = [NONE; ThreePointActionType::Last as usize];
        if deps_count + reserved > comp.problem.action_count {
            return results;
        }
        for action_type in comp.problem.action_types {
            let maybe_action = match action_type {
                ActionType::ThreePointActionType(two_point_action_type) => {
                    let element = Self::create_three_point_element(
                        &comp.point_origins[i1 as usize].point,
                        &comp.point_origins[i2 as usize].point,
                        &comp.point_origins[i3 as usize].point,
                        *two_point_action_type,
                    );
                    let new_shape = element.get_shape().unwrap();
                    if comp.shapes.contains_key(new_shape) {
                        None
                    } else {
                        Some(Action {
                            priority: 0,
                            deps_count,
                            shape: new_shape,
                            action_type: *action_type,
                            point_index_1: i1,
                            point_index_2: i2,
                            extra_index: i3,
                        })
                    }
                }
                _ => None,
            };
            match maybe_action {
                Some(mut action) => {
                    let priority = action.compute_priority(comp);
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

    pub fn compute_priority(&self, comp: &Computation) -> i32 {
        self.action_type.compute_priority(
            comp,
            self.point_index_1,
            self.point_index_2,
            self.extra_index,
            &self.shape,
            self.deps_count,
        )
    }

    pub fn get_point_indices(&self) -> Vec<i32> {
        match self.action_type {
            ActionType::TwoPointActionType(_) => vec![self.point_index_1, self.point_index_2],
            ActionType::PointAndLineActionType(_) => vec![self.point_index_1],
            ActionType::ThreePointActionType(_) => {
                vec![self.point_index_1, self.point_index_2, self.extra_index]
            }
        }
    }

    pub fn get_shape_indices(&self) -> Vec<i32> {
        match self.action_type {
            ActionType::TwoPointActionType(_) => vec![],
            ActionType::PointAndLineActionType(_) => vec![self.extra_index],
            ActionType::ThreePointActionType(_) => vec![],
        }
    }

    pub fn get_action_deps(&self, comp: &Computation) -> [u64; 3] {
        match self.action_type {
            ActionType::TwoPointActionType(_) => [
                comp.point_origins[self.point_index_1 as usize].deps,
                comp.point_origins[self.point_index_2 as usize].deps,
                0,
            ],
            ActionType::PointAndLineActionType(_) => [
                comp.point_origins[self.point_index_1 as usize].deps,
                comp.shape_origins[self.extra_index as usize].deps,
                0,
            ],
            ActionType::ThreePointActionType(_) => [
                comp.point_origins[self.point_index_1 as usize].deps,
                comp.point_origins[self.point_index_2 as usize].deps,
                comp.point_origins[self.extra_index as usize].deps,
            ],
        }
    }

    fn get_action_index(&self) -> usize {
        match self.action_type {
            ActionType::TwoPointActionType(value) => value as usize,
            ActionType::PointAndLineActionType(value) => value as usize,
            ActionType::ThreePointActionType(value) => value as usize,
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
            Self::ThreePointActionType(value) => value.compute_priority(
                comp,
                point_index_1,
                point_index_2,
                extra_index,
                shape,
                deps_count,
            ),
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
impl PriorityComputation for ThreePointActionType {
    // Priority rules for an action (point1 + point2 + point3 + action number (0-2)):
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
        extra_index: i32,
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
        let origin3 = &comp.point_origins[extra_index as usize];
        let mut found_shape_mask =
            origin1.found_shape_mask | origin2.found_shape_mask | origin3.found_shape_mask;
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
        if comp.found_points.contains(origin3.point) {
            priority += 1;
        }
        for shape in &comp.shapes_to_find {
            if shape.contains_point(&origin1.point) {
                priority += 5;
            }
            if shape.contains_point(&origin2.point) {
                priority += 5;
            }
            if shape.contains_point(&origin3.point) {
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
