// use rcc_halo2::builder::{H2Wire as W, *};
use rcc_mockbuilder::mock_builder::{MockWire as W, *};
use rcc::traits::{UInt32, NaiveUInt32};
use rcc_lib::sha256::*;

type U32 = NaiveUInt32<Boolean<W>>;

#[main_component]
fn my_circuit() {
    let _val = input_wire("val");

    let a = 'a' as u32;
    let four_bytes_of_a = U32::from_const(a ^ a << 8 ^ a << 16 ^ a << 24);

    let hash: [U32; 8] = sha256(vec![four_bytes_of_a; 1]);

    for (i, u) in hash.iter().enumerate() {
        u.to_dense().declare_public(format!("{i}").as_str());
    }
}
