// use rcc_halo2::builder::{H2Wire as W, *};
use rcc_mockbuilder::mock_builder::{MockWire as W, *};
use rcc::traits::{UInt32, NaiveUInt32, BoolWire};
use rcc_lib::sha256::*;

type U32 = NaiveUInt32<Boolean<W>>;

fn read_u32() -> U32 {
    let v: Vec<_> = (0..32).map(|i| {
        let w = input_wire(format!("{}-th bit", i).as_str());
        builder().assert_bool(w)
    }).collect();
    U32 { repr: v.try_into().unwrap_or_else(|_| [Boolean::<W>::from_const(0); 32]) }
}

/// This circuit takes input a 4-byte string, interprets it as a U32 and hashes it via SHA256
#[main_component]
fn my_circuit() {
    let a = read_u32();

    let hash: [U32; 8] = sha256(vec![a; 1]);

    for (i, u) in hash.iter().enumerate() {
        u.to_dense().declare_public(format!("{i}").as_str());
    }
}
