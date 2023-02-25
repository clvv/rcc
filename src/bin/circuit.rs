// use quote::{format_ident, quote};
use rcs::{BaseComposer, Wire, Fp};
use rust_format::{Formatter, RustFmt};
use std::fs;

const N: usize = 500;
const M: usize = 500;

fn mul_seq(e: &mut BaseComposer, a: Wire, b: Wire) -> Wire {
    let _e = e.new_context("mul_seq".into());

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
    let _e = e.new_context("gen".into());

    let a = (0..N).map(|i| {
        e.add_const(val, Fp::from(i as u32))
    }).collect();

    let b = (0..N).map(|i| {
        e.sub_const(val, Fp::from(i as u32))
    }).collect();

    (a, b)
}

pub fn my_circuit(e: &mut BaseComposer) {
    let _e = e.new_context("my_circuit".into());

    let val = e.new_wire();
    e.compose_read(val, 1);
    let (a, b) = gen(e, val);
    let c: Vec<Wire> = a.iter().zip(b.iter()).map(|(ai, bi)| {
        mul_seq(e, *ai, *bi)
    }).collect();
    let sum = e.sum(c);
    e.compose_log(sum);
}

fn main() {
    let composer = &mut BaseComposer::new();
    my_circuit(composer);
    let circuit = composer.compose_final_circuit();
    let raw = format!("{circuit}");
    let data = RustFmt::default().format_str(raw).unwrap();
    fs::write("./src/bin/circuit_runtime.rs", data).expect("Unable to write file");
}
