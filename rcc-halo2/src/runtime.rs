use polyexen::plaf::{ColumnWitness, Witness};

pub use ark_ff::{BigInteger, BigInt, Field, PrimeField};
pub use ark_bn254::Fr as F;

pub fn rcc_output_to_plaf_witness(mut wires: Vec<Vec<F>>) -> Witness {
    // Removes the public column
    wires.pop();
    Witness {
        num_rows: wires[0].len(),
        columns: vec![ColumnWitness::new(String::from("witness"), 0)],
        witness: wires.iter().map(|c| {
            c.iter().map(|f| {
                Some((*f).into())
            }).collect()
        }).collect()
    }
}
