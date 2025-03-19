use crate::{
    element::{CircleCP, CircleCR, Element, LineAB, LineAV, RayAV},
    fint::FInt,
    shape::Point,
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
pub enum ActionType {
    TwoPointActionType(TwoPointActionType),
    PointAndLineActionType(PointAndLineActionType),
}
impl ActionType {
    const LINE: Self = ActionType::TwoPointActionType(TwoPointActionType::Line);
    const CIRCLE12: Self = ActionType::TwoPointActionType(TwoPointActionType::Circle12);
    const CIRCLE21: Self = ActionType::TwoPointActionType(TwoPointActionType::Circle21);
    const MID_PERP: Self = ActionType::TwoPointActionType(TwoPointActionType::MidPerp);
    const PERP: Self = ActionType::PointAndLineActionType(PointAndLineActionType::Perp);
    const PAR: Self = ActionType::PointAndLineActionType(PointAndLineActionType::Par);

    pub fn point_count(self) -> u32 {
        match self {
            Self::TwoPointActionType(_) => 2,
            Self::PointAndLineActionType(_) => 1,
        }
    }

    pub fn line_count(self) -> u32 {
        match self {
            Self::TwoPointActionType(_) => 0,
            Self::PointAndLineActionType(_) => 1,
        }
    }
}

pub struct ProblemDefinition {
    pub given_elements: Vec<Element>,
    pub elements_to_find: Vec<Element>,
    pub action_count: u32,
    pub action_types: Vec<ActionType>,
}
#[allow(dead_code)]
impl ProblemDefinition {
    // Easy problem, solved
    fn midpoint_problem_1_3() -> ProblemDefinition {
        let p1 = pt(-1.0, 0.0);
        let p2 = pt(1.0, 0.0);
        let px = pt(0.0, 0.0);
        ProblemDefinition {
            given_elements: vec![Element::Point(p1), Element::Point(p2)],
            elements_to_find: vec![Element::Point(px)],
            action_count: 4,
            action_types: vec![ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
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
            action_types: vec![
                ActionType::LINE,
                ActionType::CIRCLE12,
                ActionType::CIRCLE21,
                ActionType::MID_PERP,
            ],
        }
    }

    // Apparently can't be solved in 7 actions
    fn inscribed_square_problem_1_7() -> ProblemDefinition {
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
            action_types: vec![ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
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
            action_types: vec![ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
        }
    }

    fn tangent_to_circle_at_point_2_8() -> ProblemDefinition {
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
            action_count: 4, // Looks like it can't be solved in 3 actions
            action_types: vec![ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
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
            action_types: vec![
                ActionType::LINE,
                ActionType::CIRCLE12,
                ActionType::CIRCLE21,
                ActionType::PERP,
            ],
        }
    }

    fn equilateral_triangle_in_circle_problem_4_4() -> ProblemDefinition {
        let c = pt(0.0, 0.0);
        let p = pt(1.0, 0.0);
        let p2 = pt(0.1, 0.99f64.sqrt());
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
            action_count: 7, // Unclear whether it can be solved in 6 actions
            action_types: vec![ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
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
            action_types: vec![
                ActionType::LINE,
                ActionType::CIRCLE12,
                ActionType::CIRCLE21,
                ActionType::MID_PERP,
                ActionType::PERP,
            ],
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
            action_types: vec![ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
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
            action_types: vec![
                ActionType::LINE,
                ActionType::CIRCLE12,
                ActionType::CIRCLE21,
                ActionType::MID_PERP,
                ActionType::PERP,
            ],
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
            action_types: vec![ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
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
            action_types: vec![
                ActionType::LINE,
                ActionType::CIRCLE12,
                ActionType::CIRCLE21,
                ActionType::MID_PERP,
                ActionType::PERP,
            ],
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
            action_types: vec![ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
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
            action_types: vec![ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
        }
    }

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
            action_types: vec![ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
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
                Element::Point(px1),
                Element::Point(px2),
                Element::Point(px3),
                Element::Point(px4),
            ],
            action_count: 11,
            action_types: vec![ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
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
            action_types: vec![ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
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
            action_types: vec![ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
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
            action_types: vec![ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
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
            action_types: vec![ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
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
            action_types: vec![ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
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
            action_types: vec![
                ActionType::LINE,
                ActionType::CIRCLE12,
                ActionType::CIRCLE21,
                ActionType::MID_PERP,
                ActionType::PERP,
            ],
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
            action_count: 7, // Actually 10
            action_types: vec![ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
        }
    }

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
            action_types: vec![
                ActionType::LINE,
                ActionType::CIRCLE12,
                ActionType::CIRCLE21,
                ActionType::MID_PERP,
                ActionType::PERP,
            ],
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
            action_types: vec![ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
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
            action_types: vec![
                ActionType::LINE,
                ActionType::CIRCLE12,
                ActionType::CIRCLE21,
                ActionType::MID_PERP,
                ActionType::PERP,
            ],
        }
    }

    fn herons_problem_7_5() -> ProblemDefinition {
        let c = pt(0.12345, 0.0);
        let v = pt(1.0, 0.0);
        let x = pt(0.0, 0.0);
        let p1 = pt(0.6, 0.8);
        let p2 = pt(-0.606, 0.808);
        ProblemDefinition {
            given_elements: vec![
                Element::LineAV(LineAV { a: c, v }),
                Element::Point(p1),
                Element::Point(p2),
            ],
            elements_to_find: vec![
                Element::LineAB(LineAB { a: x, b: p1 }),
                Element::LineAB(LineAB { a: x, b: p2 }),
            ],
            action_count: 5, // Looks like it can't be done in 4 actions
            action_types: vec![ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
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
            action_types: vec![ActionType::LINE, ActionType::CIRCLE12, ActionType::CIRCLE21],
        }
    }

    pub fn get_problem() -> ProblemDefinition {
        Self::equilateral_triangle_in_circle_problem_4_4()

        // Self::midpoint_problem_1_3_with_midperp()

        // Too large:
        // Self::circumscribed_square_5_8()
        // Self::circle_tangent_to_square_side_5_10()

        // Unclear:
        // Self::square_in_square_5_9()

        // Finds a solution with too many actions
        // Self::parallelogram_by_three_midpoints_6_11()
        // Self::parallelogram_by_three_midpoints_6_11_adv()

        // Solved with one more action (required solution probably doesn't exist)
        // Self::tangent_to_circle_at_point_2_8()
        // Self::equilateral_triangle_in_circle_problem_4_4()
        // Self::herons_problem_7_5()

        // Solved problems
        // Self::midpoint_problem_1_3()
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
    }
}
