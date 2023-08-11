use rcc_mockcomposer::mock_composer::{MockComposer as C, MockWire as W, component_of, circuit_main};
use rcc::{Wire, traits::AlgComposer, Composer};

const N: usize = 1000;
const M: usize = 1000;

#[component_of(e)]
// `mul_seq` is repeated `N` times in this circuit
// Encapsulates a new context to speed up compilation of witness gen code
// Try removing this and test compilation speed
fn mul_seq(e: &mut C, a: W, b: W) -> W {
    let mut v = vec![a * b];
    e.smart_map(0..M, |_, &i| {
        v.push(v[i] * v[i]);
    });
    v[M]
}

#[component_of(e)]
fn gen(e: &mut C, val: W) -> Vec<(W, W)> where
{
    e.smart_map(0..N as u32, |_, &i| {
        (
            val + i,
            val - i,
        )
    })
}

#[circuit_main]
#[component_of(e)]
fn my_circuit(e: &mut C) {
    let val = e.input_wire("val");

    let ab = gen(e, val);

    let c = e.smart_map(ab.iter(), |e, &(ai, bi)| {
        mul_seq(e, *ai, *bi)
    });

    let sum = e.sum(c);

    sum.declare_public("sum");
}

