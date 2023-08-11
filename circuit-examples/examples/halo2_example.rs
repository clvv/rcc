use rcc_halo2::builder::{H2Wire as W, *};

const N: usize = 10;
const M: usize = 10;

#[component]
// `mul_seq` is repeated `N` times in this circuit
// Encapsulates a new context to speed up compilation of witness gen code
// Try removing this and test compilation speed
fn mul_seq(a: W, b: W) -> W {
    let mut v = vec![a * b];
    smart_map(0..M, |_, &i| {
        v.push(v[i] * v[i]);
    });
    v[M]
}

#[component]
fn gen(val: W) -> Vec<(W, W)> {
    smart_map(0..N as u32, |_, &i| (val + i, val - i))
}

#[main_component]
fn my_circuit() {
    let val = input_wire("val");

    let ab = gen(val);

    let c: Vec<W> = ab.iter().map(|(ai, bi)| mul_seq(*ai, *bi)).collect();

    let sum = sum(c);

    val.declare_public("val");
    sum.declare_public("sum");
}
