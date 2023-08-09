use rcc_mockcomposer::mock_composer::{MockComposer, MockWire, component_of};
use rcc::{Wire, traits::AlgComposer, Composer};

const N: usize = 1000;
const M: usize = 1000;

#[component_of(e)]
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

#[component_of(e)]
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

#[component_of(e)]
pub fn my_circuit(e: &mut MockComposer) {
    let val = e.input_wire("val");

    let ab = gen(e, val);

    let mut c: Vec<MockWire> = vec![];
    e.smart_map(ab.iter(), |e, &(ai, bi)| {
        c.push(mul_seq(e, *ai, *bi))
    });
    // let c: Vec<MockWire> = ab.iter().map(|(ai, bi)| {
    //     mul_seq(e, *ai, *bi)
    // }).collect();

    let sum = e.sum(c);

    sum.declare_public("sum");
}

fn main() {
    use rust_format::{Formatter, RustFmt};

    let composer = &mut MockComposer::new();

    // Compile the circuit
    my_circuit(composer);

    // Compose the rust witness gen code
    let witness_gen_code = composer.compose_rust_witness_gen();
    let lib = format!("{}", witness_gen_code);
    let data = RustFmt::default().format_str(lib).unwrap();
    std::fs::write("examples/mock_example_runtime_lib.rs", data).expect("Unable to write file");
}
