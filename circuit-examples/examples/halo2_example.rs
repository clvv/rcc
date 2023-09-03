use rcc_halo2::builder::{H2Wire as W, *};

const N: usize = 10;
const M: usize = 10;

#[component]
fn mul_seq(a: W, b: W) -> W {
    (0..M).fold(a * b, |p, _| p * p)
}

#[component]
fn gen(val: W) -> Vec<(W, W)> {
    (0..N as u32).map(|i| (val + i, val - i)).collect()
}

#[main_component]
fn my_circuit() {
    let val = input_wire("val");

    let ab = gen(val);
    let c: Vec<W> = ab.iter().map(|(ai, bi)| mul_seq(*ai, *bi)).collect();
    let sum = sum(c);

    sum.declare_public("sum");
}
