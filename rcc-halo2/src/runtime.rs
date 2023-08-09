use polyexen::plaf::{ColumnWitness, Witness};

pub use ark_ff::{BigInteger, BigInt, Field};
pub use ark_bn254::Fr as F;
pub use halo2_proofs::halo2curves::bn256::Fr;

pub use std::collections::HashMap;

// runtime composer expects WireVal to be defined
pub type WireVal = F;
pub type AllWires = Vec<Vec<F>>;

#[derive(Copy, Clone)]
pub struct WireRef {
    pub column: usize,
    pub row: usize,
}

pub fn rcc_output_to_plaf_witness_and_instance(mut wires: AllWires) -> (Witness, Vec<Vec<F>>) {
    let instance = wires.pop().unwrap();
    (
        Witness {
            num_rows: wires[0].len(),
            columns: vec![ColumnWitness::new(String::from("witness"), 0)],
            witness: wires.iter().map(|c| {
                c.iter().map(|f| {
                    Some((*f).into())
                }).collect()
            }).collect()
        },
        vec![instance]
    )
}

