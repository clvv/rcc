use rcc::mock_composer::{MockComposer, MockWire, new_context_of};
use rcc::Composer;

const N: usize = 10;
const M: usize = 10;

#[new_context_of(e)]
// `mul_seq` is repeated `N` times in this circuit
// Encapsulates a new context to speed up compilation of witness gen code
// Try removing this and test compilation speed
fn mul_seq(e: &mut MockComposer, a: MockWire, b: MockWire) -> MockWire {
    let mut v = vec![a * b];
    for i in 0..M {
        v.push(v[i] * v[i]);
    }
    v[M]
}

#[new_context_of(e)]
fn gen(e: &mut MockComposer, val: MockWire) -> Vec<(MockWire, MockWire)> where
{
    (0..N as u32).map(|i| {
        (
            val + i,
            val - i,
        )
    }).collect()
}

pub fn my_circuit(e: &mut MockComposer) {
    let val = e.new_wire();
    e.arg_read(val, 1);

    let ab = gen(e, val);

    let c: Vec<MockWire> = ab.iter().map(|(ai, bi)| {
        mul_seq(e, *ai, *bi)
    }).collect();

    let sum = e.sum(c);

    e.log(sum);
}

fn main() {
    use quote::quote;
    use rust_format::{Formatter, RustFmt};

    let composer = &mut MockComposer::new();

    // Compile the circuit
    my_circuit(composer);

    // Compose the rust witness gen code
    let witness_gen_code = composer.compose_rust_witness_gen();

    // Wrap it in a bare file that simply runs the witness gen code
    let raw = format!("{}", quote! {
        // use rcc::{BigInt, PrimeField, F};
        fn main() {
            let compute = #witness_gen_code;
            compute();
        }
    });

    // Write it to `examples/circuit_runtime.rs`
    let data = RustFmt::default().format_str(raw).unwrap();

    std::fs::write("examples/circuit_runtime.rs", data).expect("Unable to write file");
}
