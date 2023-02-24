use quote::{format_ident, quote};
use rcs::{BaseComposer, Wire, Fp};
use rust_format::{Formatter, RustFmt};
use std::fs;

const N: usize = 50;
const M: usize = 50;

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

fn gen(e: &mut BaseComposer, val: Wire) -> (Vec<Wire>, Vec<Wire>) {
    let a = (0..N).map(|i| {
        e.add_const(val, Fp::from(i as u32))
    }).collect();

    let b = (0..N).map(|i| {
        e.sub_const(val, Fp::from(i as u32))
    }).collect();
    (a, b)
}

pub fn my_circuit(e: &mut BaseComposer) {
    let val = e.new_wire();
    e.compose_read(val, 1);
    let (a, b) = gen(e, val);
    let c: Vec<Wire> = a.iter().zip(b.iter()).map(|(ai, bi)| {
        mul_seq(e, *ai, *bi)
    }).collect();
    let sum = e.sum(c);
    e.compose_log(sum)
}

fn main() {
    let composer = &mut BaseComposer::default();
    my_circuit(composer);
    let circuit = composer.compose_final_circuit();
    let raw = format!("{circuit}");
    let data = RustFmt::default().format_str(raw).unwrap();
    fs::write("./src/bin/circuit_runtime.rs", data).expect("Unable to write file");
}
