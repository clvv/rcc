use rcc_halo2::composer::{H2Composer, H2Wire, new_context_of};
use rcc::Composer;
use rcc::traits::{AlgWire, AlgComposer};

const N: usize = 2;
const M: usize = 2;

#[new_context_of(e)]
// `mul_seq` is repeated `N` times in this circuit
// Encapsulates a new context to speed up compilation of witness gen code
// Try removing this and test compilation speed
fn mul_seq<C, W>(e: &mut C, a: W, b: W) -> W
where W: AlgWire, C: Composer<Wire = W>
{
    let mut v = vec![a * b];
    e.smart_map(0..M, |_, &i| {
        v.push(v[i] * v[i]);
    });
    v[M]
}

#[new_context_of(e)]
fn gen<C, W>(e: &mut C, val: W) -> Vec<(W, W)>
where W: AlgWire, C: Composer<Wire = W>
{
    let mut v = vec![];
    e.smart_map(0..N as u32, |_, &i| {
        v.push((
            val + i,
            val - i,
        ));
    });
    v
}

#[new_context_of(e)]
pub fn my_circuit(e: &mut H2Composer) {
    let val = e.new_wire();
    e.register_input(val);

    let ab = gen(e, val);

    let c: Vec<H2Wire> = ab.iter().map(|(ai, bi)| {
        mul_seq(e, *ai, *bi)
    }).collect();

    let sum = e.sum(c);

    e.declare_public(val);
    e.declare_public(sum);
}

fn main() {
    let composer = &mut H2Composer::new();

    // Compile the circuit
    my_circuit(composer);

    composer.write_config(std::path::PathBuf::from("examples/halo2_example.rs"));
}
