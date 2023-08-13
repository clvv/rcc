#![allow(unused_must_use)]
#![allow(unused_imports)]

use num_bigint::BigUint;
use polyexen::expr::{Column, ColumnKind, ColumnQuery, Expr, PlonkVar};
use polyexen::plaf::backends::halo2::PlafH2Circuit;
use polyexen::plaf::{ColumnFixed, ColumnWitness, Columns, CopyC, Info, Plaf, Poly, Witness};

pub use ark_bn254::Fr as F;
pub use ark_ff::{BigInt, BigInteger, Field, PrimeField};
use halo2_proofs::{dev::MockProver, halo2curves::bn256::Fr};

fn get_minimum_k(plaf: &Plaf) -> u32 {
    let rows = std::cmp::max(9, plaf.info.num_rows);
    usize::BITS - rows.leading_zeros()
}

fn convert_field_element(f: F) -> Fr {
    use ff::PrimeField;

    let u: BigUint = f.into();
    let mut repr = u.to_bytes_le();
    for _ in repr.len()..32 {
        repr.push(0);
    }
    Fr::from_repr(repr.try_into().unwrap()).unwrap()
}

pub fn mock_prove(plaf: Plaf, witness: Witness, instance_f: Vec<Vec<F>>) {
    let k = get_minimum_k(&plaf);

    println!(
        "Public Instance: {:?}",
        instance_f
            .iter()
            .map(|v| {
                v.iter()
                    .map(|&f| {
                        let u: BigUint = f.into();
                        u
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    );

    let circuit = PlafH2Circuit { plaf, wit: witness };

    let instance: Vec<Vec<Fr>> = instance_f
        .iter()
        .map(|v| v.iter().map(|&f| convert_field_element(f)).collect())
        .collect();

    let mock_prover = MockProver::<Fr>::run(k, &circuit, instance).unwrap();

    if mock_prover.verify() == Ok(()) {
        println!("Mock prover succeeded!");
    } else {
        println!("Circuit was not satisfied.");
        // println!("{:?}", circuit);
        mock_prover.assert_satisfied();
    }
}

#[test]
fn test_mock_prove() {
    fn u(n: u32) -> Option<BigUint> {
        Some(BigUint::from(n))
    }

    fn build_test_plaf() -> Plaf {
        let w = ColumnWitness::new(String::from("w"), 0);
        let s = ColumnFixed::new(String::from("s"));
        let columns = Columns {
            witness: vec![w],
            fixed: vec![s],
            public: vec![],
        };

        let info = Info {
            // TODO: Remove hardcoded p
            p: BigUint::parse_bytes(
                b"21888242871839275222246405745257275088548364400416034343698204186575808495617",
                10,
            )
            .unwrap(),
            num_rows: 4,
            challenges: vec![],
        };

        let fixed = |index, rotation| {
            Expr::Var(PlonkVar::Query(ColumnQuery {
                column: Column {
                    kind: ColumnKind::Fixed,
                    index,
                },
                rotation,
            }))
        };

        let witness = |index, rotation| {
            Expr::Var(PlonkVar::Query(ColumnQuery {
                column: Column {
                    kind: ColumnKind::Witness,
                    index,
                },
                rotation,
            }))
        };

        let _public = |index, rotation| {
            Expr::Var(PlonkVar::Query(ColumnQuery {
                column: Column {
                    kind: ColumnKind::Public,
                    index,
                },
                rotation,
            }))
        };

        let exp = (witness(0, 0) + witness(0, 1) * witness(0, 2) - witness(0, 3)) * fixed(0, 0);
        let poly = Poly {
            name: "main".into(),
            exp,
        };

        let copy = CopyC {
            columns: (
                Column {
                    kind: ColumnKind::Witness,
                    index: 0,
                },
                Column {
                    kind: ColumnKind::Witness,
                    index: 0,
                },
            ),
            offsets: vec![(0, 2)],
        };

        Plaf {
            info,
            columns,
            polys: vec![poly],
            metadata: Default::default(),
            lookups: vec![],
            shuffles: vec![],
            copys: vec![copy],
            fixed: vec![vec![u(1), u(0), u(0), u(0)]],
        }
    }

    use polyexen::plaf::PlafDisplayBaseTOML;

    let plaf = build_test_plaf();

    println!("{}", PlafDisplayBaseTOML(&plaf));

    mock_prove(
        plaf,
        Witness {
            num_rows: 4,
            columns: vec![],
            witness: vec![vec![u(1), u(0), u(1), u(1)]],
        },
        vec![],
    );
}
