use computation::Computation;
use computation::DrawState;
use computation::PrintState;
use element::CircleCP;
use element::Element;
use element::LineAB;
use element::LineAV;
use fint::FInt;
use hashset2::HashMap2;
use hashset2::WithTwoHashes;
use problems::ProblemDefinition;
use shape::Point;
use shape::ShapeTrait;
// use rayon::prelude::*;

mod computation;
mod element;
mod fint;
mod hashset2;
mod problems;
mod shape;

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

struct Main();
impl Main {
    fn compute() {
        let problem = ProblemDefinition::get_problem();
        // let problem = Self::midpoint_problem_1_3();
        let mut computation = Computation::new(&problem);

        computation.initialize_queue();
        computation.print_state();
        println!("Running...");
        computation.solve();
        println!("Finished");
    }
}

// To compile with debug symbols: RUSTFLAGS=-g cargo build --release
// set RUSTFLAGS=-g&& cargo build --release

fn test1() {
    fn pt(x: f64, y: f64) -> Point {
        Point(FInt::new(x), FInt::new(y))
    }

    let cx = pt(0.0, 0.0);
    let p = pt(0.0, -1.0);
    let v = pt(1.0, 0.0);
    let v2 = pt(0.8, 0.6);
    let px1 = pt(-0.6, 0.8);
    let px2 = pt(0.6, -0.8);
    let line = Element::LineAV(LineAV { a: p, v }).get_shape().unwrap();
    let line1 = Element::LineAV(LineAV { a: px1, v: v2 })
        .get_shape()
        .unwrap();
    let line2 = Element::LineAV(LineAV { a: px2, v: v2 })
        .get_shape()
        .unwrap();
    let pt1 = line1.find_intersection_points(&line)[0].unwrap();
    let pt2 = line2.find_intersection_points(&line)[0].unwrap();
    let circle = Element::CircleCP(CircleCP { c: pt1, p: pt2 })
        .get_shape()
        .unwrap();
    let pt3 = circle.find_intersection_points(&line1)[1].unwrap();
    let circle2 = Element::CircleCP(CircleCP { c: pt2, p: pt3 })
        .get_shape()
        .unwrap();
    let line3 = Element::LineAB(LineAB { a: pt2, b: pt3 })
        .get_shape()
        .unwrap();
    println!("{} {} {}", circle, circle2, line3);

    let pt4 = circle2.find_intersection_points(&line1)[0].unwrap();
    let pt5 = circle.find_intersection_points(&line)[1].unwrap();
    let line4 = Element::LineAB(LineAB { a: pt4, b: pt5 })
        .get_shape()
        .unwrap();
    let pt6 = line4.find_intersection_points(&line3)[0].unwrap();
    println!("{} {}", line4, pt6);
    // 831, 462 - 506, 714 = 325, -252 // 1628, 696
    // tan a1 = 252 / 325
    // tan a2 = 18 / 1122
    // tan (a1 - a2) =
    let x = 252.0 / 325.0;
    let y = 18.0 / 1122.0;
    println!("{}", (x - y) / (1.0 + x * y));
}

fn main() {
    test1();
    Computation::draw_shapes_from_file("shapes1.txt".to_string(), "shapes1.svg".to_string(), 5.0);
    Main::compute();
}
