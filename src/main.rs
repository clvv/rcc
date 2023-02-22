use rcs::print_circuit;
use rust_format::{Formatter, RustFmt};

// use quote::quote;
// use rayon::prelude::*;

use std::fs;

// statements:
//   - signal / signal_input / signal_output
//   - invocation
//   - for loop

// environment:
//   - allocated cells
//   - variable to cell mapping
//   - comptime constant values

// compile(component, env) -> compute closure for component
// compute closure: read cells, write cells

fn main() {
    // let n = 50000000 as usize;

    // let cells: Vec<Cell> = vec![Cell::default(); 3 * n];

    // let comp: Vec<_> = (0..n).map(|i| {
    //     let cells = &cells;
    //     move || {
    //         let a = cells.get(3*i + 0).unwrap().value;
    //         let b = cells.get(3*i + 1).unwrap().value;
    //         let c_cell = cells.get(3*i + 2).unwrap();
    //         unsafe {
    //             let mut c = &mut *(c_cell as *const Cell as *mut Cell);
    //             c.value = a * b;
    //         }
    //     }
    // }).collect();

    // comp.iter().for_each(|c| c());

    let circuit = print_circuit();
    let raw = format!("{circuit}");
    let data = RustFmt::default().format_str(raw).unwrap();
    fs::write("./src/bin/circuit.rs", data).expect("Unable to write file");
}
