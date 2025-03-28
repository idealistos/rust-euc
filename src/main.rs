use computation::Computation;
use computation::PrintState;

use fint::FInt;
use hashset2::HashMap2;
use hashset2::WithTwoHashes;
use problems::ProblemDefinition;
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

fn main() {
    // Computation::draw_shapes_from_file("shapes2.txt".to_string(), "shapes2.svg".to_string(), 5.0);
    Main::compute();
}
