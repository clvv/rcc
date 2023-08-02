#![allow(unused_must_use)]
#![allow(unused_imports)]

use num_bigint::BigUint;
use polyexen::expr::{Column, ColumnKind, ColumnQuery, Expr, PlonkVar};
use polyexen::plaf::backends::halo2::PlafH2Circuit;
use polyexen::plaf::{
    ColumnFixed, ColumnWitness, Columns, Info, Plaf, Poly, CopyC, Witness
};

use halo2_proofs::{dev::MockProver, halo2curves::bn256::Fr};

fn u(n: u32) -> Option<BigUint> {
    Some(BigUint::from(n))
}

pub fn mock_prove(
    plaf: Plaf,
    _witness: Vec<Fr>
) {

    let wit = Witness {
        num_rows: 4,
        columns: vec![],
        witness: vec![vec![
            u(1),
            u(0),
            u(1),
            u(1),
        ]],
    };

    let circuit = PlafH2Circuit {
        plaf,
        wit
    };

    let inputs = vec![];

    let mock_prover = MockProver::<Fr>::run(4, &circuit, inputs).unwrap();
    mock_prover.assert_satisfied();
}

#[test]
fn test_mock_prove() {

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
            p: BigUint::parse_bytes(b"21888242871839275222246405745257275088548364400416034343698204186575808495617", 10).unwrap(),
            num_rows: 4,
            challenges: vec![],
        };

        let fixed = |index, rotation| Expr::Var(PlonkVar::Query(ColumnQuery {
            column: Column {
                        kind: ColumnKind::Fixed,
                        index,
                    },
            rotation
        }));

        let witness = |index, rotation| Expr::Var(PlonkVar::Query(ColumnQuery {
            column: Column {
                        kind: ColumnKind::Witness,
                        index,
                    },
            rotation
        }));

        let _public = |index, rotation| Expr::Var(PlonkVar::Query(ColumnQuery {
            column: Column {
                        kind: ColumnKind::Public,
                        index,
                    },
            rotation
        }));

        let exp = (witness(0, 0) + witness(0, 1) * witness(0, 2) - witness(0, 3)) * fixed(0, 0);
        let poly = Poly {
            name: "main".into(),
            exp
        };

        let copy = CopyC {
            columns: (Column {
                        kind: ColumnKind::Witness,
                        index: 0,
                    }, Column {
                        kind: ColumnKind::Witness,
                        index: 0,
                    }),
            offsets: vec![(0, 2)]
        };

        Plaf {
            info,
            columns,
            polys: vec![poly],
            metadata: Default::default(),
            lookups: vec![],
            shuffles: vec![],
            copys: vec![copy],
            fixed: vec![vec![
                u(1),
                u(0),
                u(0),
                u(0),
            ]],
        }
    }

    use polyexen::plaf::PlafDisplayBaseTOML;

    let plaf = build_test_plaf();

    println!("{}", PlafDisplayBaseTOML(&plaf));

    mock_prove(plaf, vec![]);
}
