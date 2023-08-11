use rcc_mockbuilder::mock_builder::{MockWire as W, *};

const N: usize = 1000;
const M: usize = 1000;

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
    smart_map(0..N as u32, |_, &i| {
        (val + i, val - i)
    })
}

#[main_component]
fn my_circuit() {
    let val = input_wire("val");

    let ab = gen(val);

    let c = smart_map(ab.iter(), |_, &(ai, bi)| {
        mul_seq(*ai, *bi)
    });

    let sum = sum(c);

    sum.declare_public("sum");
}

