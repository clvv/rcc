use rcc_mockcomposer::mock_composer::{MockComposer, MockWire, new_context_of};
use rcc::Composer;

const N: usize = 100;
const M: usize = 100;

#[new_context_of(e)]
// `mul_seq` is repeated `N` times in this circuit
// Encapsulates a new context to speed up compilation of witness gen code
// Try removing this and test compilation speed
fn mul_seq(e: &mut MockComposer, a: MockWire, b: MockWire) -> MockWire {
    let mut v = vec![a * b];
    e.smart_map(0..M, |_, &i| {
        v.push(v[i] * v[i]);
    });
    v[M]
}

#[new_context_of(e)]
fn gen(e: &mut MockComposer, val: MockWire) -> Vec<(MockWire, MockWire)> where
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

pub fn my_circuit(e: &mut MockComposer) {
    let val = e.new_wire();
    e.register_input(val);

    let ab = gen(e, val);

    let c: Vec<MockWire> = ab.iter().map(|(ai, bi)| {
        mul_seq(e, *ai, *bi)
    }).collect();

    let sum = e.sum(c);

    e.log(sum);
}

fn main() {
    use rust_format::{Formatter, RustFmt};

    let composer = &mut MockComposer::new();

    // Compile the circuit
    my_circuit(composer);

    // Compose the rust witness gen code
    let witness_gen_code = composer.compose_rust_witness_gen();

    let lib = format!("{}", witness_gen_code);

    // Write it to `examples/circuit_runtime.rs`
    let data = RustFmt::default().format_str(lib).unwrap();

    std::fs::write("examples/circuit_runtime_lib.rs", data).expect("Unable to write file");
}
