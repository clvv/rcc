fn main() {
    let compute = || {
        use ark_bn254::Fr as F;
        use ark_ff::{BigInt, PrimeField};
        type WireVal = F;
        use std::env;
        let args: Vec<String> = env::args().collect();
        let wires: Vec<WireVal> = vec![WireVal::default(); 150usize];
        let wire =
            |i: usize| unsafe { &mut *(wires.get_unchecked(i) as *const WireVal as *mut WireVal) };
        (*wire(1usize)) = F::from(BigInt!("0"));
        (*wire(4usize)) = F::from(BigInt!("1"));
        (*wire(7usize)) = F::from(BigInt!("2"));
        (*wire(10usize)) = F::from(BigInt!("3"));
        (*wire(13usize)) = F::from(BigInt!("4"));
        (*wire(16usize)) = F::from(BigInt!("5"));
        (*wire(19usize)) = F::from(BigInt!("6"));
        (*wire(22usize)) = F::from(BigInt!("7"));
        (*wire(25usize)) = F::from(BigInt!("8"));
        (*wire(28usize)) = F::from(BigInt!("9"));
        let wires_: Vec<usize> = (0..150usize).collect();
        wires
    };
    compute();
}
