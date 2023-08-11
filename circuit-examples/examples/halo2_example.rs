use rcc_halo2::composer::{H2Composer as C, H2Wire as W, component_of, circuit_main};
use rcc::{Composer, Wire};
use rcc::traits::AlgComposer;

const N: usize = 10;
const M: usize = 10;

#[component_of(e)]
// `mul_seq` is repeated `N` times in this circuit
// Encapsulates a new context to speed up compilation of witness gen code
// Try removing this and test compilation speed
fn mul_seq(e: &mut C, a: W, b: W) -> W
{
    let mut v = vec![a * b];
    e.smart_map(0..M, |_, &i| {
        v.push(v[i] * v[i]);
    });
    v[M]
}

#[component_of(e)]
fn gen(e: &mut C, val: W) -> Vec<(W, W)>
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

    let c: Vec<W> = ab.iter().map(|(ai, bi)| {
        mul_seq(e, *ai, *bi)
    }).collect();

    let sum = e.sum(c);

    val.declare_public("val");
    sum.declare_public("sum");
}

