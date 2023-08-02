use rcc_halo2::composer::{H2Composer, H2Wire, new_context_of};
use rcc::Composer;
use rcc::arithmetic_logic::{AlgWire, BoolWire};

pub fn my_circuit(e: &mut H2Composer) {
    let val = e.new_wire();
    e.register_input(val);

    let b = val * 2;

    e.declare_public(val);
    e.declare_public(b);
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
