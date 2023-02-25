use rcc::{BaseComposer, Wire, F, component};
use quote::quote;
use rust_format::{Formatter, RustFmt};
use std::fs;

const N: usize = 10;
const M: usize = 10;

#[component(e)]
fn mul_seq(e: &mut BaseComposer, a: Wire, b: Wire) -> Wire {
    let mut v = vec![e.mul(a, b)];
    for i in 0..M {
        v.push(e.mul(
                *v.get(i).unwrap(),
                *v.get(i).unwrap()
        ));
    }
    *v.get(M).unwrap()
}

#[component(e)]
fn gen(e: &mut BaseComposer, val: Wire) -> (Vec<Wire>, Vec<Wire>) {
    let (a, b): (Vec<Wire>, Vec<Wire>) = (0..N).map(|i| {
        (
            e.add_const(val, F::from(i as u32)),
            e.sub_const(val, F::from(i as u32))
        )
    }).unzip();

    (a, b)
}

#[component(e)]
pub fn my_circuit(e: &mut BaseComposer) {
    let val = e.new_wire();
    e.arg_read(val, 1);

    let (a, b) = gen(e, val);
    let c: Vec<Wire> = a.iter().zip(b.iter()).map(|(ai, bi)| {
        mul_seq(e, *ai, *bi)
    }).collect();
    let sum = e.sum(c);
    e.log(sum);
}

fn main() {
    let composer = &mut BaseComposer::new();

    // Compile the circuit
    my_circuit(composer);

    // Compose the rust witness gen code
    let witness_gen_code = composer.compose_rust_witness_gen();

    // Wrap it in a bare file that simply runs the witness gen code
    let raw = format!("{}", quote! {
        // use rcc::{BigInt, PrimeField, F};
        fn main() {
            let compute = #witness_gen_code;
            compute();
        }
    });

    // Write it to `src/bin/circuit_runtime.rs`
    let data = RustFmt::default().format_str(raw).unwrap();
    fs::write("./src/bin/circuit_runtime.rs", data).expect("Unable to write file");
}
