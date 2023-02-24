use quote::{format_ident, quote};
use rcs::{BaseComposer, Wire, Fp};
use rust_format::{Formatter, RustFmt};
use std::fs;

const N: usize = 50;
const M: usize = 50;

fn mul_seq2(e: &mut BaseComposer, a: Wire, b: Wire) -> Wire {
    e.register_func("mul_seq".into(), quote! {
        let mul_seq = |i, j, k, new: [usize; #M]| {
            mul_to(i, j, new[0]);
            for l in 0..(#M - 1) {
                mul_to(new[l], new[l], new[l+1]);
            }
            mul_to(new[(#M - 1)], new[(#M - 1)], k);
        };
    });

    let new = e.new_wires(M);
    let indices = new.iter().map(|w| w.index);
    let c = e.new_wire();
    let i = a.index;
    let j = b.index;
    let k = c.index;

    let new_id = format_ident!("new_{}", new.get(0).unwrap().index);

    e.runtime(quote! {
        // #c = #a * #b;
        let #new_id = [ #( #indices ),* ];
        mul_seq(#i, #j, #k, #new_id);
    });

    c
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
        mul_seq2(e, *ai, *bi)
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
    fs::write("./src/bin/circuit2_runtime.rs", data).expect("Unable to write file");
}
