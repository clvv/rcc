use rcs::{BigInt, PrimeField, F};
use std::env;
fn main() {
    let args: Vec<String> = env::args().collect();
    let wires: Vec<F> = vec![F::default(); 10usize];
    let wire = |i: usize| unsafe { &mut *(wires.get_unchecked(i) as *const F as *mut F) };
}
