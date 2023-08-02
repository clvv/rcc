use rcc_halo2::composer::{H2Composer, H2Wire, new_context_of};
use rcc::Composer;
use rcc::arithmetic_logic::{AlgWire, AlgComposer};

const N: usize = 10;
const M: usize = 10;

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
    use rust_format::{Formatter, RustFmt};

    let composer = &mut H2Composer::new();

    // Compile the circuit
    my_circuit(composer);

    // Compose the rust witness gen code
    let witness_gen_code = composer.compose_rust_witness_gen();

    composer.print_plaf_toml();

    let lib = format!("{}", witness_gen_code);

    // Write it to `examples/circuit_runtime.rs`
    let data = RustFmt::default().format_str(lib).unwrap();

    std::fs::write("examples/circuit2_runtime_lib.rs", data).expect("Unable to write file");
}
