use crate::{
    element::{CircleCP, CircleCR, Element, LineAB, LineAV, RayAV, SegmentAB},
    fint::FInt,
    shape::{Point, Shape, ShapeTrait},
};

fn pt(x: f64, y: f64) -> Point {
    Point(FInt::new(x), FInt::new(y))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TwoPointActionType {
    Line,
    Circle12,
    Circle21,
    MidPerp,
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PointAndLineActionType {
    Perp,
    Par,
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreePointActionType {
    CircleCAB,
    CircleACB,
    CircleABC,
    BisectorCAB,
    BisectorACB,
    BisectorABC,
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TwoPointAndLineActionType {
    BisectorPosCAL, // "Positive" means that line.get_direction() is used (not -line.get_direction())
    BisectorPosACL,
    BisectorNegCAL,
    BisectorNegACL,
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionType {
    TwoPointActionType(TwoPointActionType),
    PointAndLineActionType(PointAndLineActionType),
    ThreePointActionType(ThreePointActionType),
    TwoPointAndLineActionType(TwoPointAndLineActionType),
}
impl ActionType {
    const LINE: Self = ActionType::TwoPointActionType(TwoPointActionType::Line);
    const CIRCLE12: Self = ActionType::TwoPointActionType(TwoPointActionType::Circle12);
    const CIRCLE21: Self = ActionType::TwoPointActionType(TwoPointActionType::Circle21);
    const MID_PERP: Self = ActionType::TwoPointActionType(TwoPointActionType::MidPerp);
    const PERP: Self = ActionType::PointAndLineActionType(PointAndLineActionType::Perp);
    const PAR: Self = ActionType::PointAndLineActionType(PointAndLineActionType::Par);
    const CIRCLE_CAB: Self = ActionType::ThreePointActionType(ThreePointActionType::CircleCAB);
    const CIRCLE_ACB: Self = ActionType::ThreePointActionType(ThreePointActionType::CircleACB);
    const CIRCLE_ABC: Self = ActionType::ThreePointActionType(ThreePointActionType::CircleABC);
    const BISECTOR_CAB: Self = ActionType::ThreePointActionType(ThreePointActionType::BisectorCAB);
    const BISECTOR_ACB: Self = ActionType::ThreePointActionType(ThreePointActionType::BisectorACB);
    const BISECTOR_ABC: Self = ActionType::ThreePointActionType(ThreePointActionType::BisectorABC);
    const BISECTOR_POS_CAL: Self =
        ActionType::TwoPointAndLineActionType(TwoPointAndLineActionType::BisectorPosCAL);
    const BISECTOR_POS_ACL: Self =
        ActionType::TwoPointAndLineActionType(TwoPointAndLineActionType::BisectorPosACL);
    const BISECTOR_NEG_CAL: Self =
        ActionType::TwoPointAndLineActionType(TwoPointAndLineActionType::BisectorNegCAL);
    const BISECTOR_NEG_ACL: Self =
        ActionType::TwoPointAndLineActionType(TwoPointAndLineActionType::BisectorNegACL);
}

pub struct ProblemDefinition {
    pub given_elements: Vec<Element>,
    pub elements_to_find: Vec<Element>,
    pub action_count: u32,
    pub action_types: &'static [ActionType],
    pub random_walk_at_n_actions: Option<u32>,
    pub prioritize_low_action_count_shapes: bool,
    pub multimatch: bool,
    pub find_all_solutions: bool,
    pub track_supports_in_rw: bool,
}
#[allow(dead_code)]
impl ProblemDefinition {
    const BASIC: ProblemDefinition = ProblemDefinition {
        given_elements: vec![],
        elements_to_find: vec![],
        action_count: 0,
        action_types: &[ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
        random_walk_at_n_actions: None,
        prioritize_low_action_count_shapes: true,
        multimatch: false,
        find_all_solutions: false,
        track_supports_in_rw: false,
    };

    const LIMITED_ADVANCED: ProblemDefinition = ProblemDefinition {
        given_elements: vec![],
        elements_to_find: vec![],
        action_count: 0,
        action_types: &[
            ActionType::LINE,
            ActionType::CIRCLE12,
            ActionType::CIRCLE21,
            ActionType::MID_PERP,
            ActionType::PERP,
        ],
        random_walk_at_n_actions: None,
        prioritize_low_action_count_shapes: true,
        multimatch: false,
        find_all_solutions: false,
        track_supports_in_rw: false,
    };

    const ADVANCED: ProblemDefinition = ProblemDefinition {
        given_elements: vec![],
        elements_to_find: vec![],
        action_count: 0,
        action_types: &[
            ActionType::LINE,
            ActionType::CIRCLE12,
            ActionType::CIRCLE21,
            ActionType::MID_PERP,
            ActionType::PERP,
            ActionType::PAR,
        ],
        random_walk_at_n_actions: None,
        prioritize_low_action_count_shapes: true,
        multimatch: false,
        find_all_solutions: false,
        track_supports_in_rw: false,
    };

    const FULL_WITHOUT_BISECTOR: ProblemDefinition = ProblemDefinition {
        given_elements: vec![],
        elements_to_find: vec![],
        action_count: 0,
        action_types: &[
            ActionType::LINE,
            ActionType::CIRCLE12,
            ActionType::CIRCLE21,
            ActionType::MID_PERP,
            ActionType::PERP,
            ActionType::PAR,
            ActionType::CIRCLE_CAB,
            ActionType::CIRCLE_ACB,
            ActionType::CIRCLE_ABC,
        ],
        random_walk_at_n_actions: None,
        prioritize_low_action_count_shapes: true,
        multimatch: false,
        find_all_solutions: false,
        track_supports_in_rw: false,
    };

    const FULL: ProblemDefinition = ProblemDefinition {
        given_elements: vec![],
        elements_to_find: vec![],
        action_count: 0,
        action_types: &[
            ActionType::LINE,
            ActionType::CIRCLE12,
            ActionType::CIRCLE21,
            ActionType::MID_PERP,
            ActionType::PERP,
            ActionType::PAR,
            ActionType::CIRCLE_CAB,
            ActionType::CIRCLE_ACB,
            ActionType::CIRCLE_ABC,
            ActionType::BISECTOR_CAB,
            ActionType::BISECTOR_ACB,
            ActionType::BISECTOR_ABC,
            ActionType::BISECTOR_POS_CAL,
            ActionType::BISECTOR_POS_ACL,
            ActionType::BISECTOR_NEG_CAL,
            ActionType::BISECTOR_NEG_ACL,
        ],
        random_walk_at_n_actions: None,
        prioritize_low_action_count_shapes: true,
        multimatch: false,
        find_all_solutions: false,
        track_supports_in_rw: false,
    };

    pub fn has_point_and_line_actions(&self) -> bool {
        self.action_types
            .iter()
            .any(|action_type| matches!(action_type, ActionType::PointAndLineActionType(_)))
    }

    pub fn has_three_point_actions(&self) -> bool {
        self.action_types
            .iter()
            .any(|action_type| matches!(action_type, ActionType::ThreePointActionType(_)))
    }

    pub fn has_two_point_and_line_actions(&self) -> bool {
        self.action_types
            .iter()
            .any(|action_type| matches!(action_type, ActionType::TwoPointAndLineActionType(_)))
    }

    // Easy problem, solved
    fn midpoint_problem_1_3() -> ProblemDefinition {
        let p1 = pt(-1.0, 0.0);
        let p2 = pt(1.0, 0.0);
        let px = pt(0.0, 0.0);
        ProblemDefinition {
            given_elements: vec![Element::Point(p1), Element::Point(p2)],
            elements_to_find: vec![Element::Point(px)],
            action_count: 4,
            ..Self::BASIC
        }
    }

    fn midpoint_problem_1_3_with_midperp() -> ProblemDefinition {
        let p1 = pt(-1.0, 0.0);
        let p2 = pt(1.0, 0.0);
        let px = pt(0.0, 0.0);
        ProblemDefinition {
            given_elements: vec![Element::Point(p1), Element::Point(p2)],
            elements_to_find: vec![Element::Point(px)],
            action_count: 2,
            action_types: &[
                ActionType::LINE,
                ActionType::CIRCLE12,
                ActionType::CIRCLE21,
                ActionType::MID_PERP,
            ],
            ..Self::BASIC
        }
    }

    // Apparently can't be solved in 7 actions
    fn inscribed_square_1_7() -> ProblemDefinition {
        let c = pt(0.0, 0.0);
        let p0 = pt(0.0, -1.0);
        let p1 = pt(1.0, 0.0);
        let p2 = pt(0.0, 1.0);
        let p3 = pt(-1.0, 0.0);

        ProblemDefinition {
            given_elements: vec![
                Element::CircleCP(CircleCP { c, p: p0 }),
                Element::Point(p0),
                Element::Point(c),
            ],
            elements_to_find: vec![
                Element::LineAB(LineAB { a: p0, b: p1 }),
                Element::LineAB(LineAB { a: p1, b: p2 }),
                Element::LineAB(LineAB { a: p2, b: p3 }),
                Element::LineAB(LineAB { a: p3, b: p0 }),
            ],
            action_count: 7,
            multimatch: true,
            ..Self::BASIC
        }
    }

    // Apparently can't be solved in 7 actions
    fn inscribed_square_problem_1_7_extended() -> ProblemDefinition {
        let c = pt(0.0, 0.0);
        let p0 = pt(0.0, -1.0);
        let p1 = pt(1.0, 0.0);
        let p2 = pt(0.0, 1.0);
        let p3 = pt(-1.0, 0.0);

        ProblemDefinition {
            given_elements: vec![
                Element::CircleCP(CircleCP { c, p: p0 }),
                Element::Point(p0),
                Element::Point(c),
            ],
            elements_to_find: vec![
                Element::LineAB(LineAB { a: p0, b: p1 }),
                Element::LineAB(LineAB { a: p1, b: p2 }),
                Element::LineAB(LineAB { a: p2, b: p3 }),
                Element::LineAB(LineAB { a: p3, b: p0 }),
                Element::Point(p1),
                Element::Point(p2),
                Element::Point(p3),
            ],
            action_count: 8,
            ..Self::BASIC
        }
    }

    fn tangent_to_circle_at_point_2_8() -> ProblemDefinition {
        let c = pt(0.0, 0.0);
        let p0 = pt(1.0, 0.0);
        let v = pt(0.0, 1.0);
        let p_additional = pt((1.0 - 0.23f64 * 0.23).sqrt(), 0.23);

        ProblemDefinition {
            given_elements: vec![
                Element::CircleCP(CircleCP { c, p: p0 }),
                Element::Point(p0),
                Element::Point(c),
                Element::Point(p_additional),
            ],
            elements_to_find: vec![Element::LineAV(LineAV { a: p0, v })],
            action_count: 3, // Looks like it can't be solved in 3 actions
            multimatch: true,
            ..Self::BASIC
        }
    }

    fn tangent_to_circle_at_point_2_8_with_perp() -> ProblemDefinition {
        let c = pt(0.0, 0.0);
        let p0 = pt(1.0, 0.0);
        let v = pt(0.0, 1.0);

        ProblemDefinition {
            given_elements: vec![
                Element::CircleCP(CircleCP { c, p: p0 }),
                Element::Point(p0),
                Element::Point(c),
            ],
            elements_to_find: vec![Element::LineAV(LineAV { a: p0, v })],
            action_count: 2,
            action_types: &[
                ActionType::LINE,
                ActionType::CIRCLE12,
                ActionType::CIRCLE21,
                ActionType::PERP,
            ],
            ..Self::BASIC
        }
    }

    fn equilateral_triangle_in_circle_problem_4_4() -> ProblemDefinition {
        let c = pt(0.0, 0.0);
        let p = pt(1.0, 0.0);
        let p_additional = pt(0.2, 0.96f64.sqrt());
        let xp1 = pt(-0.5, 0.75f64.sqrt());
        let xp2 = pt(-0.5, -(0.75f64.sqrt()));
        ProblemDefinition {
            given_elements: vec![
                Element::CircleCP(CircleCP { c, p }),
                Element::Point(p),
                Element::Point(p_additional),
            ],
            elements_to_find: vec![
                Element::LineAB(LineAB { a: xp1, b: p }),
                Element::LineAB(LineAB { a: xp2, b: p }),
                Element::LineAB(LineAB { a: xp1, b: xp2 }),
                Element::Point(xp1),
                Element::Point(xp2),
            ],
            action_count: 6, // Unclear whether it can be solved in 6 actions
            multimatch: true,
            ..Self::BASIC
        }
    }

    fn equilateral_triangle_in_circle_problem_4_4_mm_mod() -> ProblemDefinition {
        let c = pt(0.0, 0.0);
        let p = pt(1.0, 0.0);
        // let p2 = pt(0.1, 0.99f64.sqrt());
        let p_additional = pt(0.6213, -(1.0f64 - 0.6213f64 * 0.6213f64).sqrt());
        let xp1 = pt(-0.5, 0.75f64.sqrt());
        let xp2 = pt(-0.5, -(0.75f64.sqrt()));
        ProblemDefinition {
            given_elements: vec![
                Element::CircleCP(CircleCP { c, p }),
                Element::Point(p),
                Element::Point(p_additional),
            ],
            elements_to_find: vec![
                // Element::LineAB(LineAB { a: xp1, b: p }),
                // Element::LineAB(LineAB { a: xp2, b: p }),
                Element::LineAB(LineAB { a: xp1, b: xp2 }),
                // Element::Point(xp1),
                // Element::Point(xp2),
            ],
            action_count: 5,
            multimatch: true,
            find_all_solutions: true,
            ..Self::BASIC
        }
    }

    fn equilateral_triangle_in_circle_problem_4_4_adv() -> ProblemDefinition {
        let c = pt(0.0, 0.0);
        let p = pt(1.0, 0.0);
        let p2 = pt(0.6, 0.8);
        let xp1 = pt(-0.5, 0.75f64.sqrt());
        let xp2 = pt(-0.5, -(0.75f64.sqrt()));
        ProblemDefinition {
            given_elements: vec![
                Element::CircleCP(CircleCP { c, p }),
                Element::Point(p),
                Element::Point(p2),
            ],
            elements_to_find: vec![
                Element::LineAB(LineAB { a: xp1, b: p }),
                Element::LineAB(LineAB { a: xp2, b: p }),
                Element::LineAB(LineAB { a: xp1, b: xp2 }),
                Element::Point(xp1),
                Element::Point(xp2),
            ],
            action_count: 5,
            ..Self::LIMITED_ADVANCED
        }
    }

    fn angle_of_60_4_2() -> ProblemDefinition {
        let p0 = pt(0.0, 0.75f64.sqrt());
        let p = pt(0.23456, 0.0);
        let v = pt(1.0, 0.0);
        let xp = pt(-0.5, 0.0);

        ProblemDefinition {
            given_elements: vec![
                Element::LineAV(LineAV { a: p, v }),
                Element::Point(p0),
                Element::Point(p),
            ],
            elements_to_find: vec![Element::LineAB(LineAB { a: p0, b: xp }), Element::Point(xp)],
            action_count: 4,
            ..Self::BASIC
        }
    }

    fn angle_of_60_4_2_adv() -> ProblemDefinition {
        let p0 = pt(0.0, 0.75f64.sqrt());
        let p = pt(0.23456, 0.0);
        let v = pt(1.0, 0.0);
        let xp = pt(-0.5, 0.0);

        ProblemDefinition {
            given_elements: vec![
                Element::LineAV(LineAV { a: p, v }),
                Element::Point(p0),
                Element::Point(p),
            ],
            elements_to_find: vec![Element::LineAB(LineAB { a: p0, b: xp }), Element::Point(xp)],
            action_count: 3,
            ..Self::LIMITED_ADVANCED
        }
    }

    // Solved
    fn circumscribed_equilateral_triangle_4_3() -> ProblemDefinition {
        let c = pt(0.0, 0.0);
        let p = pt(0.0, -1.0);
        let x1 = pt(0.0, 2.0);
        let x2 = pt(-(3f64.sqrt()), -1.0);
        let x3 = pt(3f64.sqrt(), -1.0);

        ProblemDefinition {
            given_elements: vec![
                Element::CircleCP(CircleCP { c, p }),
                Element::Point(c),
                Element::Point(p),
            ],
            elements_to_find: vec![
                Element::LineAB(LineAB { a: x1, b: x2 }),
                Element::LineAB(LineAB { a: x2, b: x3 }),
                Element::LineAB(LineAB { a: x3, b: x1 }),
                Element::Point(x1),
                Element::Point(x2),
                Element::Point(x3),
            ],
            action_count: 6,
            ..Self::BASIC
        }
    }

    fn square_by_opposite_midpoints_4_9_adv() -> ProblemDefinition {
        let p1 = pt(-0.5, 0.0);
        let p2 = pt(0.5, 0.0);
        let ux1 = pt(-0.5, 0.5);
        let ux2 = pt(0.5, 0.5);
        let lx1 = pt(-0.5, -0.5);
        let lx2 = pt(0.5, -0.5);

        ProblemDefinition {
            given_elements: vec![Element::Point(p1), Element::Point(p2)],
            elements_to_find: vec![
                Element::LineAB(LineAB { a: ux1, b: ux2 }),
                Element::LineAB(LineAB { a: lx1, b: lx2 }),
                Element::LineAB(LineAB { a: ux1, b: lx1 }),
                Element::LineAB(LineAB { a: ux2, b: lx2 }),
            ],
            action_count: 6,
            ..Self::LIMITED_ADVANCED
        }
    }

    // Solved
    fn line_equidistant_from_two_points_5_3() -> ProblemDefinition {
        let p1 = pt(-1.0, 0.0);
        let p2 = pt(1.0, 0.0);
        let p3 = pt(0.2345, 1.0);
        let v = pt(1.0, 0.0);

        ProblemDefinition {
            given_elements: vec![Element::Point(p1), Element::Point(p2), Element::Point(p3)],
            elements_to_find: vec![Element::LineAV(LineAV { a: p3, v })],
            action_count: 4,
            prioritize_low_action_count_shapes: false,
            ..Self::BASIC
        }
    }

    // Solved - required reducing the number of actions by 1
    // (not all shapes are constructed)
    fn shift_angle_5_6() -> ProblemDefinition {
        let p1 = pt(0.0, 0.0);
        let p2 = pt(3.0, 1.2345);
        let v1 = pt(1.0, 0.0);
        let v2 = pt(0.6, 0.8);

        ProblemDefinition {
            given_elements: vec![
                Element::RayAV(RayAV { a: p1, v: v1 }),
                Element::RayAV(RayAV { a: p1, v: v2 }),
                Element::Point(p1),
                Element::Point(p2),
            ],
            elements_to_find: vec![
                Element::LineAV(LineAV { a: p2, v: v1 }),
                Element::LineAV(LineAV { a: p2, v: v2 }),
            ],
            action_count: 5,
            ..Self::BASIC
        }
    }

    // Finishes in 159 seconds with prioritize_low_action_count_shapes = true, and in 13 seconds with false
    fn line_equidistant_from_two_lines_5_7() -> ProblemDefinition {
        let p1 = pt(0.0, 0.0);
        let p2 = pt(0.2345, 2.0);
        let v = pt(1.0, 0.0);
        let px = pt(0.0, 1.0);

        ProblemDefinition {
            given_elements: vec![
                Element::LineAV(LineAV { a: p1, v }),
                Element::LineAV(LineAV { a: p2, v }),
                Element::Point(p1),
                Element::Point(p2),
            ],
            elements_to_find: vec![Element::LineAV(LineAV { a: px, v })],
            action_count: 5,
            prioritize_low_action_count_shapes: false,
            ..Self::BASIC
        }
    }

    fn line_equidistant_from_two_lines_5_7_rw() -> ProblemDefinition {
        let p1 = pt(0.0, 0.0);
        let p2 = pt(0.2345, 2.0);
        let v = pt(1.0, 0.0);
        let px = pt(0.0, 1.0);

        ProblemDefinition {
            given_elements: vec![
                Element::LineAV(LineAV { a: p1, v }),
                Element::LineAV(LineAV { a: p2, v }),
                Element::Point(p1),
                Element::Point(p2),
            ],
            elements_to_find: vec![Element::LineAV(LineAV { a: px, v })],
            action_count: 5,
            random_walk_at_n_actions: Some(4),
            ..Self::BASIC
        }
    }

    fn circumscribed_square_5_8() -> ProblemDefinition {
        let c = pt(0.0, 0.0);
        let p = pt(0.0, 1.0);
        let p2 = pt(0.36482, -2.12345);
        let v = pt(1.0, 0.0);
        let px1 = pt(1.0, 1.0);
        let px2 = pt(1.0, -1.0);
        let px3 = pt(-1.0, -1.0);
        let px4 = pt(-1.0, 1.0);

        ProblemDefinition {
            given_elements: vec![
                Element::CircleCP(CircleCP { c, p }),
                Element::LineAV(LineAV { a: p2, v }),
                Element::Point(c),
                Element::Point(p2),
            ],
            elements_to_find: vec![
                Element::LineAB(LineAB { a: px1, b: px2 }),
                Element::LineAB(LineAB { a: px2, b: px3 }),
                Element::LineAB(LineAB { a: px3, b: px4 }),
                Element::LineAB(LineAB { a: px4, b: px1 }),
            ],
            action_count: 11,
            multimatch: true,
            ..Self::BASIC
        }
    }

    fn circumscribed_square_5_8_rw() -> ProblemDefinition {
        let c = pt(0.0, 0.0);
        let p = pt(0.0, 1.0);
        let p2 = pt(0.36482, -2.12345);
        let v = pt(1.0, 0.0);
        let px1 = pt(1.0, 1.0);
        let px2 = pt(1.0, -1.0);
        let px3 = pt(-1.0, -1.0);
        let px4 = pt(-1.0, 1.0);

        ProblemDefinition {
            given_elements: vec![
                Element::CircleCP(CircleCP { c, p }),
                Element::LineAV(LineAV { a: p2, v }),
                Element::Point(c),
                Element::Point(p2),
            ],
            elements_to_find: vec![
                Element::LineAB(LineAB { a: px1, b: px2 }),
                Element::LineAB(LineAB { a: px2, b: px3 }),
                Element::LineAB(LineAB { a: px3, b: px4 }),
                Element::LineAB(LineAB { a: px4, b: px1 }),
                Element::Point(px1),
                Element::Point(px2),
                Element::Point(px3),
                Element::Point(px4),
            ],
            action_count: 11,
            random_walk_at_n_actions: Some(4),
            ..Self::BASIC
        }
    }

    fn circumscribed_square_5_8_mm_mod() -> ProblemDefinition {
        let c = pt(0.0, 0.0);
        let p = pt(0.0, 1.0);
        let p2 = pt(0.31482564, -3.5234576);
        let v = pt(1.0, 0.0);
        let px1 = pt(1.0, 1.0);
        let _px2 = pt(1.0, -1.0);
        let _px3 = pt(-1.0, -1.0);
        let _px4 = pt(-1.0, 1.0);

        ProblemDefinition {
            given_elements: vec![
                Element::CircleCP(CircleCP { c, p }),
                Element::LineAV(LineAV { a: p2, v }),
                Element::Point(c),
                Element::Point(p2),
            ],
            elements_to_find: vec![
                // Element::LineAB(LineAB { a: px1, b: px2 }),
                // Element::LineAB(LineAB { a: px2, b: px3 }),
                // Element::LineAB(LineAB { a: px3, b: px4 }),
                // Element::LineAB(LineAB { a: px4, b: px1 }),
                Element::Point(px1),
                // Element::Point(px2),
                // Element::Point(px3),
                // Element::Point(px4),
            ],
            action_count: 7,
            // multimatch: true,
            // find_all_solutions: true,
            random_walk_at_n_actions: Some(4),
            // prioritize_low_action_count_shapes: false,
            ..Self::BASIC
        }
    }

    fn circle_tangent_to_square_side_5_10() -> ProblemDefinition {
        let p1 = pt(1.0, 1.0);
        let p2 = pt(1.0, -1.0);
        let p3 = pt(-1.0, -1.0);
        let p4 = pt(-1.0, 1.0);

        let px = pt(0.0, -0.25);

        ProblemDefinition {
            given_elements: vec![
                Element::LineAB(LineAB { a: p1, b: p2 }),
                Element::LineAB(LineAB { a: p2, b: p3 }),
                Element::LineAB(LineAB { a: p3, b: p4 }),
                Element::LineAB(LineAB { a: p4, b: p1 }),
                Element::Point(p1),
                Element::Point(p2),
                Element::Point(p3),
                Element::Point(p4),
            ],
            elements_to_find: vec![
                Element::CircleCP(CircleCP { c: px, p: p1 }),
                Element::Point(px),
            ],
            action_count: 6,
            ..Self::BASIC
        }
    }

    fn circle_tangent_to_square_side_5_10_rw_mod() -> ProblemDefinition {
        let p1 = pt(1.0, 1.0);
        let p2 = pt(1.0, -1.0);
        let p3 = pt(-1.0, -1.0);
        let p4 = pt(-1.0, 1.0);

        let px = pt(0.0, -0.25);

        ProblemDefinition {
            given_elements: vec![
                Element::LineAB(LineAB { a: p1, b: p2 }),
                Element::LineAB(LineAB { a: p2, b: p3 }),
                Element::LineAB(LineAB { a: p3, b: p4 }),
                Element::LineAB(LineAB { a: p4, b: p1 }),
                // Element::Point(p1),
                // Element::Point(p2),
                // Element::Point(p3),
                // Element::Point(p4),
            ],
            elements_to_find: vec![
                // Element::CircleCP(CircleCP { c: px, p: p1 }),
                Element::Point(px),
            ],
            action_count: 5,
            multimatch: true,
            random_walk_at_n_actions: Some(4),
            ..Self::BASIC
        }
    }

    fn square_in_square_5_9() -> ProblemDefinition {
        let p1 = pt(1.0, 1.0);
        let p2 = pt(1.0, -1.0);
        let p3 = pt(-1.0, -1.0);
        let p4 = pt(-1.0, 1.0);

        let px1 = pt(1.0, 0.823);
        let px2 = pt(0.823, -1.0);
        let px3 = pt(-1.0, -0.823);
        let px4 = pt(-0.823, 1.0);

        ProblemDefinition {
            given_elements: vec![
                Element::LineAB(LineAB { a: p1, b: p2 }),
                Element::LineAB(LineAB { a: p2, b: p3 }),
                Element::LineAB(LineAB { a: p3, b: p4 }),
                Element::LineAB(LineAB { a: p4, b: p1 }),
                Element::Point(p1),
                Element::Point(p2),
                Element::Point(p3),
                Element::Point(p4),
                Element::Point(px1),
            ],
            elements_to_find: vec![
                Element::LineAB(LineAB { a: px1, b: px2 }),
                Element::LineAB(LineAB { a: px2, b: px3 }),
                Element::LineAB(LineAB { a: px3, b: px4 }),
                Element::LineAB(LineAB { a: px4, b: px1 }),
                Element::Point(px2),
                Element::Point(px3),
                Element::Point(px4),
            ],
            action_count: 7,
            ..Self::BASIC
        }
    }

    fn point_reflection_6_1() -> ProblemDefinition {
        let pc = pt(0.0, 0.0);
        let p1 = pt(1.0, 1.0);
        let p2 = pt(1.12345, 0.0);
        let px1 = pt(-1.0, -1.0);
        let px2 = pt(-1.12345, 0.0);
        ProblemDefinition {
            given_elements: vec![
                Element::LineAB(LineAB { a: p1, b: p2 }),
                Element::Point(p1),
                Element::Point(p2),
                Element::Point(pc),
            ],
            elements_to_find: vec![
                Element::LineAB(LineAB { a: px1, b: px2 }),
                Element::Point(px1),
                Element::Point(px2),
            ],
            action_count: 5,
            ..Self::BASIC
        }
    }

    fn copy_segment_6_3() -> ProblemDefinition {
        let p1 = pt(-1.0, 0.0);
        let p2 = pt(0.0, 0.0);
        let p2p = pt(2.3456, 0.0);
        let p1px = pt(3.3456, 0.0);
        ProblemDefinition {
            given_elements: vec![
                Element::LineAB(LineAB { a: p1, b: p2 }),
                Element::Point(p1),
                Element::Point(p2),
                Element::Point(p2p),
            ],
            elements_to_find: vec![
                Element::CircleCP(CircleCP { c: p2p, p: p1px }),
                Element::Point(p1px),
            ],
            action_count: 5,
            ..Self::BASIC
        }
    }

    fn symmetry_of_four_lines_6_10() -> ProblemDefinition {
        let o = pt(0.0, 0.0);
        let v1 = pt(1.0, 0.0);
        let v2 = pt(0.6, 0.8);
        let v3 = pt(-1.0, 1.0);
        let vx = pt(-0.2, 1.4);
        let pt = pt(1.0, 0.0);
        ProblemDefinition {
            given_elements: vec![
                Element::LineAV(LineAV { a: o, v: v1 }),
                Element::LineAV(LineAV { a: o, v: v2 }),
                Element::LineAV(LineAV { a: o, v: v3 }),
                Element::Point(pt),
            ],
            elements_to_find: vec![Element::LineAV(LineAV { a: o, v: vx })],
            action_count: 4,
            ..Self::BASIC
        }
    }

    fn symmetry_of_four_lines_6_10_adv() -> ProblemDefinition {
        let o = pt(0.0, 0.0);
        let v1 = pt(1.0, 0.0);
        let v2 = pt(0.6, 0.8);
        let v3 = pt(-1.0, 1.01);
        let vx = pt(-1.0 - 0.04, 7.03);
        let pt = pt(1.0, 0.0);
        ProblemDefinition {
            given_elements: vec![
                Element::LineAV(LineAV { a: o, v: v1 }),
                Element::LineAV(LineAV { a: o, v: v2 }),
                Element::LineAV(LineAV { a: o, v: v3 }),
                Element::Point(pt),
            ],
            elements_to_find: vec![Element::LineAV(LineAV { a: o, v: vx })],
            action_count: 3,
            ..Self::LIMITED_ADVANCED
        }
    }

    fn parallelogram_by_three_midpoints_6_11() -> ProblemDefinition {
        let xp1 = pt(0.0, 0.0);
        let xp2 = pt(0.6, 0.8);
        let xp3 = pt(1.62, 0.8);
        let xp4 = pt(1.02, 0.0);
        let m23 = pt(1.11, 0.8);
        let m34 = pt(1.32, 0.4);
        let m41 = pt(0.51, 0.0);
        ProblemDefinition {
            given_elements: vec![
                Element::Point(m23),
                Element::Point(m34),
                Element::Point(m41),
            ],
            elements_to_find: vec![
                Element::LineAB(LineAB { a: xp1, b: xp2 }),
                Element::LineAB(LineAB { a: xp2, b: xp3 }),
                Element::LineAB(LineAB { a: xp3, b: xp4 }),
                Element::LineAB(LineAB { a: xp4, b: xp1 }),
            ],
            action_count: 10,
            ..Self::BASIC
        }
    }

    // Solution in more actions than allowed:
    // 0: Line0 Line(nx=0.885,ny=0.465,d=1.355) [1 actions: 1 (0)][pri = 174] from GivenPoint0 and GivenPoint1 (TwoPointActionType(Line))
    // 1: Circle1 Circle(c.x=1.110,c.y=0.800,r2=0.204) [1 actions: 10 (0)][pri = 174] from GivenPoint0 and GivenPoint1 (TwoPointActionType(Circle12))
    // 2: Circle2 Circle(c.x=1.320,c.y=0.400,r2=0.204) [1 actions: 100 (0)][pri = 174] from GivenPoint0 and GivenPoint1 (TwoPointActionType(Circle21))
    // 7: Line7 Line(nx=0.800,ny=-0.600,d=0.408) [1 actions: 10000000 (0)][pri = 174] from GivenPoint0 and GivenPoint2 (TwoPointActionType(Line))
    // 11: Line11 Line(nx=0.443,ny=-0.897,d=0.226) [1 actions: 100000000000 (0)][pri = 174] from GivenPoint1 and GivenPoint2 (TwoPointActionType(Line))
    // 12: Circle12 Circle(c.x=1.320,c.y=0.400,r2=0.816) [1 actions: 1000000000000 (0)][pri = 174] from GivenPoint1 and GivenPoint2 (TwoPointActionType(Circle12))
    // 13: Circle13 Circle(c.x=0.510,c.y=0.000,r2=0.816) [1 actions: 10000000000000 (0)][pri = 174] from GivenPoint1 and GivenPoint2 (TwoPointActionType(Circle21))
    // 15: Line15 Line(nx=0.800,ny=-0.600,d=0.816) [2 actions: 1000000010000000 (100)][pri = 137] from GivenPoint1 and Line7 (PointAndLineActionType(Par))
    // 27: Line27 Line(nx=0.000,ny=1.000,d=0.800) [3 actions: 1000000000000001100000000000 (10)][pri = 90] from GivenPoint0 and x/Line11/Circle12 (TwoPointActionType(Line))
    // 28: Line28 Line(nx=0.000,ny=1.000,d=0.000) [3 actions: 10000000000000000000000000101 (1000)][pri = 90] from GivenPoint2 and x/Line0/Circle2 (TwoPointActionType(Line))
    // 1831: Line1831 Line(nx=0.800,ny=-0.600,d=-0.000) [4 actions: 111000000000000000000000000000000000010000000000011 (1)][pri = 38] from x/Line0/Circle1 and x/Circle1/Circle13 (TwoPointActionType(Line))
    // Finished in 4699 seconds
    fn parallelogram_by_three_midpoints_6_11_adv() -> ProblemDefinition {
        let xp1 = pt(0.0, 0.0);
        let xp2 = pt(0.6, 0.8);
        let xp3 = pt(1.62, 0.8);
        let xp4 = pt(1.02, 0.0);
        let m23 = pt(1.11, 0.8);
        let m34 = pt(1.32, 0.4);
        let m41 = pt(0.51, 0.0);
        ProblemDefinition {
            given_elements: vec![
                Element::Point(m23),
                Element::Point(m34),
                Element::Point(m41),
            ],
            elements_to_find: vec![
                Element::LineAB(LineAB { a: xp1, b: xp2 }),
                Element::LineAB(LineAB { a: xp2, b: xp3 }),
                Element::LineAB(LineAB { a: xp3, b: xp4 }),
                Element::LineAB(LineAB { a: xp4, b: xp1 }),
            ],
            action_count: 7,
            multimatch: true,
            prioritize_low_action_count_shapes: false,
            ..Self::FULL_WITHOUT_BISECTOR
        }
    }

    fn parallelogram_by_three_midpoints_6_11_full_partial() -> ProblemDefinition {
        let xp1 = pt(0.0, 0.0);
        let xp2 = pt(0.6, 0.8);
        let _xp3 = pt(1.62, 0.8);
        let _xp4 = pt(1.02, 0.0);
        let m23 = pt(1.11, 0.8);
        let m34 = pt(1.32, 0.4);
        let m41 = pt(0.51, 0.0);
        ProblemDefinition {
            given_elements: vec![
                Element::Point(m23),
                Element::Point(m34),
                Element::Point(m41),
            ],
            elements_to_find: vec![Element::LineAB(LineAB { a: xp1, b: xp2 })],
            action_count: 4,
            prioritize_low_action_count_shapes: false,
            ..Self::FULL_WITHOUT_BISECTOR
        }
    }

    fn annulus_7_2() -> ProblemDefinition {
        let c = pt(0.0, 0.0);
        let p = pt(1.0, 0.0);
        ProblemDefinition {
            given_elements: vec![
                Element::Point(c),
                Element::Point(p),
                Element::CircleCP(CircleCP { c, p }),
            ],
            elements_to_find: vec![Element::CircleCR(CircleCR {
                c,
                r: FInt::new(0.5).sqrt(),
            })],
            action_count: 5,
            ..Self::BASIC
        }
    }

    fn annulus_7_2_adv() -> ProblemDefinition {
        let c = pt(0.0, 0.0);
        let p = pt(1.0, 0.0);
        ProblemDefinition {
            given_elements: vec![
                Element::Point(c),
                Element::Point(p),
                Element::CircleCP(CircleCP { c, p }),
            ],
            elements_to_find: vec![Element::CircleCR(CircleCR {
                c,
                r: FInt::new(0.5).sqrt(),
            })],
            action_count: 4,
            ..Self::LIMITED_ADVANCED
        }
    }

    fn angle_of_75_7_3_full() -> ProblemDefinition {
        let c = pt(0.0, 0.0);
        let p1 = pt(1.2345, 0.0);
        // tan 75 = sqrt((1 - cos 150) / (1 + cos 150)) = (1 + r3 / 2) / (1/2)
        let px = pt(1.0, 2.0 + 3.0f64.sqrt());
        ProblemDefinition {
            given_elements: vec![
                Element::RayAV(RayAV { a: c, v: p1 }),
                Element::Point(c),
                Element::Point(p1),
            ],
            elements_to_find: vec![Element::LineAB(LineAB { a: c, b: px })],
            action_count: 3,
            multimatch: true,
            prioritize_low_action_count_shapes: false,
            ..Self::FULL
        }
    }

    fn angle_isosceles_7_10() -> ProblemDefinition {
        let c = pt(0.0, 0.0);
        let ax = pt(1.0, 0.0);
        let bx = pt(0.6, 0.8);
        let p = pt(1.0 - 0.4 * 0.2345, 0.8 * 0.2345);
        ProblemDefinition {
            given_elements: vec![
                Element::RayAV(RayAV { a: c, v: ax }),
                Element::RayAV(RayAV { a: c, v: bx }),
                Element::Point(p),
            ],
            elements_to_find: vec![Element::LineAB(LineAB { a: ax, b: bx })],
            action_count: 5,
            ..Self::BASIC
        }
    }

    fn herons_problem_7_5() -> ProblemDefinition {
        let c = pt(0.12345, 0.0);
        let v = pt(1.0, 0.0);
        let x = pt(0.0, 0.0);
        let p1 = pt(0.6, 0.8);
        let p2 = pt(-0.72, 0.96);
        ProblemDefinition {
            given_elements: vec![
                Element::LineAV(LineAV { a: c, v }),
                Element::Point(p1),
                Element::Point(p2),
                Element::Point(c),
            ],
            elements_to_find: vec![
                Element::LineAB(LineAB { a: x, b: p1 }),
                Element::LineAB(LineAB { a: x, b: p2 }),
            ],
            action_count: 4,
            ..Self::BASIC
        }
    }

    fn inscribed_circle_7_7_with_prep() -> ProblemDefinition {
        // Heronian triangle (21, 20, 13), area = 126, h = 12.6
        // r = 2S / (a1 + a2 + a3) = 14/3
        // x-distance from b: (AB + BC - AC) / 2 = (20 + 21 - 13) / 2 = 14
        let a = pt(0.0, 1.26);
        let b = pt(-1.68, 0.0);
        let c = pt(0.32, 0.0);

        let cx = pt(1.4 - 1.68, 1.4 / 3.0);
        let c_side = pt(0.32 - 1.3, 0.0);

        ProblemDefinition {
            given_elements: vec![
                Element::SegmentAB(SegmentAB { a, b }),
                Element::SegmentAB(SegmentAB { a, b: c }),
                Element::SegmentAB(SegmentAB { a: c, b }),
                Element::CircleCP(CircleCP { c, p: a }),
                Element::CircleCP(CircleCP { c: c_side, p: a }),
                Element::CircleCP(CircleCP { c: a, p: c_side }),
            ],
            elements_to_find: vec![Element::CircleCR(CircleCR {
                c: cx,
                r: FInt::new(1.4 / 3.0),
            })],
            action_count: 2,
            ..Self::BASIC
        }
    }

    fn circle_tangent_to_three_lines_7_8() -> ProblemDefinition {
        let cx = pt(0.0, 0.0);
        let p = pt(0.0, -1.0);
        let v = pt(1.0, 0.0);
        let v2 = pt(0.8, 0.6);
        let px1 = pt(-0.6, 0.8);
        let px2 = pt(0.6, -0.8);
        ProblemDefinition {
            given_elements: vec![
                Element::LineAV(LineAV { a: p, v }),
                Element::LineAV(LineAV { a: px1, v: v2 }),
                Element::LineAV(LineAV { a: px2, v: v2 }),
            ],
            elements_to_find: vec![Element::CircleCP(CircleCP { c: cx, p })],
            action_count: 6,
            ..Self::BASIC
        }
    }

    // Solution (for this particular case, cannot be generalized):
    // - Line(nx=0.000,ny=1.000,d=-1.000)
    // - Line(nx=0.600,ny=-0.800,d=-1.000)
    // - Line(nx=0.600,ny=-0.800,d=1.000)
    // - Circle(c.x=-3.000,c.y=-1.000,r2=11.111)
    // - Circle(c.x=0.333,c.y=-1.000,r2=4.444)
    // - Line(nx=0.236,ny=-0.972,d=-0.525)
    // - Line(nx=0.949,ny=0.316,d=0.000)
    // - Circle(c.x=-0.167,c.y=0.500,r2=0.278)
    // - Circle(c.x=0.000,c.y=0.000,r2=1.000)
    fn circle_tangent_to_three_lines_7_8_rw() -> ProblemDefinition {
        let cx = pt(0.0, 0.0);
        let p = pt(0.0, -1.0);
        let v = pt(1.0, 0.0);
        let v2 = pt(0.8, 0.6);
        let px1 = pt(-0.6, 0.8);
        let px2 = pt(0.6, -0.8);
        ProblemDefinition {
            given_elements: vec![
                Element::LineAV(LineAV { a: p, v }),
                Element::LineAV(LineAV { a: px1, v: v2 }),
                Element::LineAV(LineAV { a: px2, v: v2 }),
            ],
            elements_to_find: vec![Element::CircleCP(CircleCP { c: cx, p })],
            action_count: 6,
            random_walk_at_n_actions: Some(4),
            ..Self::BASIC
        }
    }

    fn circle_tangent_to_three_lines_7_8_rw_alt() -> ProblemDefinition {
        let cos = 0.814237;
        let sin = (1.0f64 - cos * cos).sqrt();
        let cx = pt(0.0, 0.0);
        let p = pt(0.0, -1.0);
        let v = pt(1.0, 0.0);
        let v2 = pt(cos, sin);
        let px1 = pt(-sin, cos);
        let px2 = pt(sin, -cos);
        let _p_additional = pt(-0.12345, -1.0);
        ProblemDefinition {
            given_elements: vec![
                Element::LineAV(LineAV { a: p, v }),
                Element::LineAV(LineAV { a: px1, v: v2 }),
                Element::LineAV(LineAV { a: px2, v: v2 }),
                // Element::Point(p_additional),
            ],
            elements_to_find: vec![Element::CircleCP(CircleCP { c: cx, p })],
            action_count: 6,
            random_walk_at_n_actions: Some(4),
            multimatch: true,
            ..Self::BASIC
        }
    }

    fn circle_tangent_to_three_lines_7_8_rw_mod() -> ProblemDefinition {
        let cos = 0.814237;
        let sin = (1.0f64 - cos * cos).sqrt();
        let cx = pt(0.0, 0.0);
        let p = pt(0.0, -1.0);
        let v = pt(1.0, 0.0);
        let v2 = pt(cos, sin);
        let px1 = pt(-sin, cos);
        let px2 = pt(sin, -cos);
        let line = Shape::Line(LineAV { a: p, v }.get_shape());
        let line1 = Shape::Line(LineAV { a: px1, v: v2 }.get_shape());
        let line2 = Shape::Line(LineAV { a: px2, v: v2 }.get_shape());
        let pt1 = line1.find_intersection_points(&line)[0].unwrap();
        let pt2 = line2.find_intersection_points(&line)[0].unwrap();
        let circle = Shape::Circle(CircleCP { c: pt1, p: pt2 }.get_shape());
        let pt3 = circle.find_intersection_points(&line1)[1].unwrap();

        ProblemDefinition {
            given_elements: vec![
                Element::LineAV(LineAV { a: p, v }),
                Element::LineAV(LineAV { a: px1, v: v2 }),
                Element::LineAV(LineAV { a: px2, v: v2 }),
                Element::CircleCP(CircleCP { c: pt1, p: pt2 }),
                Element::LineAB(LineAB { a: pt2, b: pt3 }),
            ],
            elements_to_find: vec![Element::CircleCP(CircleCP { c: cx, p })],
            action_count: 4,
            random_walk_at_n_actions: Some(3),
            ..Self::BASIC
        }
    }

    fn segment_by_midpoint_7_9_adv() -> ProblemDefinition {
        let c = pt(0.0, 0.0);
        let p = pt(1.0, 0.5);
        let p1 = pt(1.0, 0.0);
        let px = pt(1.234, 1.0);
        ProblemDefinition {
            given_elements: vec![
                Element::LineAB(LineAB { a: c, b: p1 }),
                Element::LineAB(LineAB { a: c, b: px }),
                Element::Point(p),
            ],
            elements_to_find: vec![Element::LineAB(LineAB { a: p, b: px })],
            action_count: 3,
            ..Self::ADVANCED
        }
    }

    fn perimeter_bisector_8_1_adv() -> ProblemDefinition {
        // From this Heronian triangle: 7, 15, 20; area = 42
        // d(a, b) = 0.35; d(a, c) = 0.75; d(b, c) = 1.0
        // Doesn't work with this one :)
        // let a = pt(0.0, 0.21);
        // let b = pt(0.28, 0.0);
        // let c = pt(-0.72, 0.0);
        // let px = pt(-0.42, 0.0);

        // From another Heronian triangle: 6, 25, 29; area = 60
        // d(a, b) = 0.6, d(a, c) = 2.9, d(b, c) = 2.5
        let a = pt(0.0, 0.48);
        let b = pt(0.36, 0.0);
        let c = pt(2.86, 0.0);
        let px = pt(2.76, 0.0);
        ProblemDefinition {
            given_elements: vec![
                Element::LineAB(LineAB { a, b }),
                Element::LineAB(LineAB { a, b: c }),
                Element::LineAB(LineAB { a: c, b }),
            ],
            elements_to_find: vec![Element::LineAB(LineAB { a, b: px })],
            action_count: 4,
            action_types: &[
                ActionType::LINE,
                ActionType::CIRCLE12,
                ActionType::CIRCLE21,
                ActionType::MID_PERP,
                ActionType::CIRCLE_CAB,
                ActionType::CIRCLE_ACB,
                ActionType::CIRCLE_ABC,
            ],
            ..Self::BASIC
        }
    }

    fn angle_54_trisection_8_2_mm() -> ProblemDefinition {
        // tan 54 = sqrt(1 + 2 / sqrt(5))
        // tan 18 = sqrt(1 - 2 / sqrt(5))
        // tan 36 = sqrt(5 - 2 sqrt(5))
        let c = pt(0.0, 0.0);
        let a = pt(1.0, 0.0);
        let b = pt(1.0, (1.0 + 2.0 / 5.0f64.sqrt()).sqrt());
        let px1 = pt(1.0, (1.0 - 2.0 / 5.0f64.sqrt()).sqrt());
        let px2 = pt(1.0, (5.0 - 2.0 * 5.0f64.sqrt()).sqrt());

        ProblemDefinition {
            given_elements: vec![
                Element::RayAV(RayAV { a: c, v: a }),
                Element::RayAV(RayAV { a: c, v: b }),
                Element::Point(c),
                Element::Point(a),
            ],
            elements_to_find: vec![
                Element::LineAB(LineAB { a: c, b: px1 }),
                Element::LineAB(LineAB { a: c, b: px2 }),
            ],
            action_count: 5,
            multimatch: true,
            // find_all_solutions: true,
            ..Self::BASIC
        }
    }

    fn interior_angles_8_3() -> ProblemDefinition {
        let c = pt(0.0, 0.0);
        let v1 = pt(1.0, 0.57743);
        let v2 = pt(1.0, -0.57743);
        let p = pt(2.0, 2.1341342);
        let vx = pt(0.0, 1.0);
        ProblemDefinition {
            given_elements: vec![
                Element::LineAV(LineAV { a: c, v: v1 }),
                Element::LineAV(LineAV { a: c, v: v2 }),
                Element::Point(p),
            ],
            elements_to_find: vec![Element::LineAV(LineAV { a: p, v: vx })],
            action_count: 4,
            multimatch: true,
            ..Self::BASIC
        }
    }

    fn regular_octagon_8_4() -> ProblemDefinition {
        let kh = 0.5f64.sqrt();
        let p1 = pt(0.0, 0.0);
        let p2 = pt(1.0, 0.0);
        let px3 = pt(1.0 + kh, kh);
        let px4 = pt(1.0 + kh, 1.0 + kh);
        let px5 = pt(1.0, 1.0 + 2.0 * kh);
        let px6 = pt(0.0, 1.0 + 2.0 * kh);
        let px7 = pt(-kh, 1.0 + kh);
        let px8 = pt(-kh, kh);

        ProblemDefinition {
            given_elements: vec![
                Element::SegmentAB(SegmentAB { a: p1, b: p2 }),
                Element::Point(p1),
                Element::Point(p2),
            ],
            elements_to_find: vec![
                Element::LineAB(LineAB { a: p2, b: px3 }),
                Element::LineAB(LineAB { a: px3, b: px4 }),
                Element::LineAB(LineAB { a: px4, b: px5 }),
                Element::LineAB(LineAB { a: px5, b: px6 }),
                Element::LineAB(LineAB { a: px6, b: px7 }),
                Element::LineAB(LineAB { a: px7, b: px8 }),
                Element::LineAB(LineAB { a: px8, b: p1 }),
            ],
            action_count: 13,
            multimatch: true,
            prioritize_low_action_count_shapes: false,
            ..Self::BASIC
        }
    }

    fn regular_octagon_8_4_adv() -> ProblemDefinition {
        let kh = 0.5f64.sqrt();
        let p1 = pt(0.0, 0.0);
        let p2 = pt(1.0, 0.0);
        let px3 = pt(1.0 + kh, kh);
        let px4 = pt(1.0 + kh, 1.0 + kh);
        let px5 = pt(1.0, 1.0 + 2.0 * kh);
        let px6 = pt(0.0, 1.0 + 2.0 * kh);
        let px7 = pt(-kh, 1.0 + kh);
        let px8 = pt(-kh, kh);

        ProblemDefinition {
            given_elements: vec![
                Element::SegmentAB(SegmentAB { a: p1, b: p2 }),
                Element::Point(p1),
                Element::Point(p2),
            ],
            elements_to_find: vec![
                Element::LineAB(LineAB { a: p2, b: px3 }),
                Element::LineAB(LineAB { a: px3, b: px4 }),
                Element::LineAB(LineAB { a: px4, b: px5 }),
                Element::LineAB(LineAB { a: px5, b: px6 }),
                Element::LineAB(LineAB { a: px6, b: px7 }),
                Element::LineAB(LineAB { a: px7, b: px8 }),
                Element::LineAB(LineAB { a: px8, b: p1 }),
                // Element::Point(Point(FInt::new(0.12390123), FInt::new(0.12930193))),
            ],
            action_count: 9,
            multimatch: true,
            prioritize_low_action_count_shapes: false,
            ..Self::FULL
        }
    }

    fn triangle_cleaver_8_5_rw() -> ProblemDefinition {
        let h = 1.65239;
        let x1 = -0.86762;
        let x2 = 0.52374;
        let a = pt(0.0, h);
        let b = pt(x1, 0.0);
        let c = pt(x2, 0.0);
        let l1 = (x1 * x1 + h * h).sqrt();
        let l2 = (x2 * x2 + h * h).sqrt();
        let k = 0.5 * (l1 - l2) / l1;
        let px1 = pt(k * x1, h - k * h);
        let px2 = pt(0.5 * (x1 + x2), 0.0);

        ProblemDefinition {
            given_elements: vec![
                Element::SegmentAB(SegmentAB { a, b }),
                Element::SegmentAB(SegmentAB { a, b: c }),
                Element::SegmentAB(SegmentAB { a: c, b }),
            ],
            elements_to_find: vec![Element::LineAB(LineAB { a: px1, b: px2 })],
            action_count: 7,
            multimatch: true,
            // prioritize_low_action_count_shapes: false,
            random_walk_at_n_actions: Some(4),
            // find_all_solutions: true,
            ..Self::BASIC
        }
    }

    fn triangle_cleaver_8_5_adv() -> ProblemDefinition {
        let h = 1.65239;
        let x1 = -0.86762;
        let x2 = 0.52374;
        let a = pt(0.0, h);
        let b = pt(x1, 0.0);
        let c = pt(x2, 0.0);
        let l1 = (x1 * x1 + h * h).sqrt();
        let l2 = (x2 * x2 + h * h).sqrt();
        let k = 0.5 * (l1 - l2) / l1;
        let px1 = pt(k * x1, h - k * h);
        let px2 = pt(0.5 * (x1 + x2), 0.0);

        ProblemDefinition {
            given_elements: vec![
                Element::SegmentAB(SegmentAB { a, b }),
                Element::SegmentAB(SegmentAB { a, b: c }),
                Element::SegmentAB(SegmentAB { a: c, b }),
            ],
            elements_to_find: vec![Element::LineAB(LineAB { a: px1, b: px2 })],
            action_count: 3,
            multimatch: true,
            prioritize_low_action_count_shapes: false,
            ..Self::FULL
        }
    }

    fn torricelli_point_8_6_rw() -> ProblemDefinition {
        let o = pt(0.0, 0.0);
        let a = pt(0.0, 1.22348);
        let k1 = 1.328783;
        let k2 = 1.787345;
        let b = pt(-k1 * 0.75f64.sqrt(), -0.5 * k1);
        let c = pt(k2 * 0.75f64.sqrt(), -0.5 * k2);

        ProblemDefinition {
            given_elements: vec![
                Element::SegmentAB(SegmentAB { a, b }),
                Element::SegmentAB(SegmentAB { a, b: c }),
                Element::SegmentAB(SegmentAB { a: c, b }),
            ],
            elements_to_find: vec![Element::Point(o)],
            action_count: 5,
            multimatch: true,
            // prioritize_low_action_count_shapes: false,
            random_walk_at_n_actions: Some(4),
            // find_all_solutions: true,
            ..Self::BASIC
        }
    }

    fn torricelli_point_8_6_adv() -> ProblemDefinition {
        let o = pt(0.0, 0.0);
        let a = pt(0.0, 1.22348);
        let k1 = 1.328783;
        let k2 = 1.787345;
        let b = pt(-k1 * 0.75f64.sqrt(), -0.5 * k1);
        let c = pt(k2 * 0.75f64.sqrt(), -0.5 * k2);

        ProblemDefinition {
            given_elements: vec![
                Element::SegmentAB(SegmentAB { a, b }),
                Element::SegmentAB(SegmentAB { a, b: c }),
                Element::SegmentAB(SegmentAB { a: c, b }),
            ],
            elements_to_find: vec![Element::Point(o)],
            action_count: 4,
            multimatch: true,
            ..Self::FULL
        }
    }

    fn minimum_perimeter_2_9_7() -> ProblemDefinition {
        // Heronian triangle (21, 20, 13), area = 126, h = 12.6
        // a-c: (0.32, 0.0) t + (0.0, 1.26) (1 - t)
        // b-b': (-1.68, 0.0) + t [(-0.32, 1.26)] = (-1.68, 0.0) + t' (1.26, 0.32);
        // Solution: t = 105/169, t' = 252/169, b': (168/845, 2016/4225)
        // a-b: (-1.68, 0.0) t + (0.0, 1.26) (1 - t)
        // c-c': (0.32, 0.0) + t [(1.68, 1.26)] = (0.32, 0.0) + t'(1.26, -1.68)  -> t = 5/21, t' = -4/7, (-0.4, 0.96)
        // Generalized: (l, 0) t + (0, h) (1 - t) == (-r, 0) + t' (h, l))
        let r = 1.6573;
        let l = 0.3156;
        let h = 1.2572;
        let a = pt(0.0, h);
        let b = pt(-r, 0.0);
        let c = pt(l, 0.0);
        let xa = pt(0.0, 0.0);
        let xb = pt(
            l * (h * h - l * r) / (h * h + l * l),
            l * h * (l + r) / (h * h + l * l),
        );
        let xc = pt(
            -r * (h * h - l * r) / (h * h + r * r),
            r * h * (l + r) / (h * h + r * r),
        );

        ProblemDefinition {
            given_elements: vec![
                Element::SegmentAB(SegmentAB { a, b }),
                Element::SegmentAB(SegmentAB { a, b: c }),
                Element::SegmentAB(SegmentAB { a: c, b }),
            ],
            elements_to_find: vec![
                // Element::LineAB(LineAB { a: xa, b: xb }),
                Element::LineAB(LineAB { a: xb, b: xc }),
                Element::LineAB(LineAB { a: xa, b: xc }),
            ],
            action_count: 7,
            multimatch: true,
            // prioritize_low_action_count_shapes: false,
            random_walk_at_n_actions: Some(4),
            track_supports_in_rw: true,
            ..Self::BASIC
        }
    }

    fn harmonic_mean_of_segments_9_8() -> ProblemDefinition {
        let r = 2.36726;
        let p1 = pt(-1.0, 0.0);
        let p2 = pt(r, 0.0);
        let p = pt(0.0, 0.0);
        let v = pt(0.72387, 0.932483);
        let l = FInt::new(2.0 * 1.0 * r / (1.0 + r));
        let d = v.distance_to(&p);
        let px = Point(l * v.0 / d, l * v.1 / d);

        ProblemDefinition {
            given_elements: vec![
                Element::SegmentAB(SegmentAB { a: p1, b: p2 }),
                Element::Point(p),
                Element::Point(p1),
                Element::Point(p2),
                Element::RayAV(RayAV { a: p, v }),
                Element::Point(v),
            ],
            elements_to_find: vec![Element::Point(px)],
            action_count: 4,
            multimatch: true,
            prioritize_low_action_count_shapes: false,
            ..Self::BASIC
        }
    }

    fn triangle_midsegment_9_10() -> ProblemDefinition {
        let h = 1.65239;
        let x1 = -0.86762;
        let x2 = 0.52374;
        let a = pt(0.0, h);
        let b = pt(x1, 0.0);
        let c = pt(x2, 0.0);
        let px1 = pt(0.5 * x1, 0.5 * h);
        let px2 = pt(0.5 * x2, 0.5 * h);

        ProblemDefinition {
            given_elements: vec![
                Element::SegmentAB(SegmentAB { a, b }),
                Element::SegmentAB(SegmentAB { a, b: c }),
                Element::SegmentAB(SegmentAB { a: c, b }),
            ],
            elements_to_find: vec![Element::LineAB(LineAB { a: px1, b: px2 })],
            action_count: 5,
            multimatch: true,
            // prioritize_low_action_count_shapes: false,
            random_walk_at_n_actions: Some(3),
            track_supports_in_rw: true,
            ..Self::BASIC
        }
    }

    fn chord_trisection_10_8() -> ProblemDefinition {
        let c = pt(0.0, 0.0);
        let p = pt(1.0, 0.0);
        let d2 = FInt::new(3.0 * 0.3456).sqr();
        let v = Point((FInt::new(9.0) - d2).sqrt(), (d2 - FInt::new(1.0)).sqrt());
        ProblemDefinition {
            given_elements: vec![
                Element::CircleCP(CircleCP { c, p }),
                Element::CircleCR(CircleCR {
                    c,
                    r: FInt::new(0.3456),
                }),
                Element::Point(c),
                Element::Point(p),
            ],
            elements_to_find: vec![Element::LineAV(LineAV { a: p, v })],
            action_count: 4,
            ..Self::BASIC
        }
    }

    /*

    Unsolved problems:

    Basic:

    Equilateral triangle in circle (4.4), 6 actions - "all actions explored", solvable in 7 actions (with an additional point)
    Circumscribed Square (5.8), 11 actions - too large
    Inscribed circle (7.7), 8 actions - too large
    Circle tangent to 3 lines (7.8), 6 actions - tried random walk
    Excircle (7.11), 8 actions
    Regular Octagon (8.4), 13 actions
    Triangle cleaver (8.5), 7 actions

    Advanced:

    Parallelogram by 3 midpoints (6.11), 7 actions - solvable in 8 actions
    Angle of 75 (7.3), 3 actions - "all actions explored", solvable in 4 actions


    */

    pub fn get_problem() -> ProblemDefinition {
        Self::minimum_perimeter_2_9_7()
        // Self::circle_tangent_to_three_lines_7_8_rw_alt()
        // Self::parallelogram_by_three_midpoints_6_11_full_partial()
        // Self::midpoint_problem_1_3()

        // Self::chord_trisection_10_8()
        // Self::circumscribed_square_5_8_rw()
        // Self::line_equidistant_from_two_lines_5_7_rw()

        // Too large:
        // Self::circumscribed_square_5_8()
        // Self::circle_tangent_to_square_side_5_10()
        // Self::circle_tangent_to_three_lines_7_8()
        // Self::regular_octagon_8_4()
        // Self::triangle_midsegment_9_10()

        // Finds a solution with too many actions
        // Self::parallelogram_by_three_midpoints_6_11()
        // Self::angle_of_75_7_3_full()

        // Solved with one more action (required solution probably doesn't exist)
        // Self::equilateral_triangle_in_circle_problem_4_4()

        // Solved problems
        // Self::midpoint_problem_1_3()
        // Self::inscribed_square_1_7()
        // Self::tangent_to_circle_at_point_2_8()
        // Self::angle_of_60_4_2()
        // Self::circumscribed_equilateral_triangle_4_3()
        // Self::line_equidistant_from_two_points_5_3()
        // Self::shift_angle_5_6()
        // Self::line_equidistant_from_two_lines_5_7()
        // Self::point_reflection_6_1() - didn't mark the point
        // Self::copy_segment_6_3()
        // Self::chord_trisection_10_8()
        // Self::tangent_to_circle_at_point_2_8_with_perp()
        // Self::square_by_opposite_midpoints_4_9_adv()
        // Self::equilateral_triangle_in_circle_problem_4_4_adv()
        // Self::symmetry_of_four_lines_6_10()
        // Self::symmetry_of_four_lines_6_10_adv()
        // Self::annulus_7_2()
        // Self::annulus_7_2_adv()
        // Self::herons_problem_7_5() - added an unneeded action
        // Self::segment_by_midpoint_7_9_adv()
        // Self::angle_isosceles_7_10()
        // Self::angle_54_trisection_8_2_mm()
        // Self::interior_angles_8_3()
        // Self::regular_octagon_8_4_adv()
        // Self::triangle_cleaver_8_5_adv()
        // Self::torricelli_point_8_6_adv() (required setting priority for found_point to 500)
        // Self::harmonic_mean_of_segments_9_8()

        // Solved with random walk
        // Self::circle_tangent_to_square_side_5_10_rw_mod()
        // Self::torricelli_point_8_6_rw()
    }
}
