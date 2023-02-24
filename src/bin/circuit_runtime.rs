use rcs::{BigInt, PrimeField, F};
use std::env;
fn main() {
    let args: Vec<String> = env::args().collect();
    let wires: Vec<F> = vec![F::default(); 12usize];
    let wires_: Vec<usize> = (0..12usize).collect();
    let wire = |i: usize| unsafe { &mut *(wires.get_unchecked(i) as *const F as *mut F) };
    let add_const = |wires_add_const: &[usize], v_add_const_0| {
        (*wire(wires_add_const[0usize])) = (*wire(v_add_const_0)) + F::from(BigInt!("0"));
    };
    let sub_const = |wires_sub_const: &[usize], v_sub_const_0| {
        (*wire(wires_sub_const[0usize])) = (*wire(v_sub_const_0)) - F::from(BigInt!("0"));
    };
    let gen = |wires_gen: &[usize], v_gen_0| {
        add_const(wires_gen[0usize..1usize].try_into().unwrap(), v_gen_0);
        add_const(wires_gen[1usize..2usize].try_into().unwrap(), v_gen_0);
        sub_const(wires_gen[2usize..3usize].try_into().unwrap(), v_gen_0);
        sub_const(wires_gen[3usize..4usize].try_into().unwrap(), v_gen_0);
    };
    let mul_seq = |wires_mul_seq: &[usize], v_mul_seq_0, v_mul_seq_0| {
        (*wire(wires_mul_seq[0usize])) = (*wire(v_mul_seq_0)) * (*wire(v_mul_seq_0));
        (*wire(wires_mul_seq[1usize])) =
            (*wire(wires_mul_seq[0usize])) * (*wire(wires_mul_seq[0usize]));
        (*wire(wires_mul_seq[2usize])) =
            (*wire(wires_mul_seq[1usize])) * (*wire(wires_mul_seq[1usize]));
    };
    let my_circuit = |wires_my_circuit: &[usize]| {
        (*wire(wires_my_circuit[0usize])) =
            F::from(args.get(1usize).unwrap().parse::<i32>().unwrap());
        gen(
            wires_my_circuit[1usize..5usize].try_into().unwrap(),
            wires_my_circuit[0usize],
        );
        mul_seq(
            wires_my_circuit[5usize..8usize].try_into().unwrap(),
            wires_my_circuit[0usize],
            wires_my_circuit[0usize],
        );
        mul_seq(
            wires_my_circuit[8usize..11usize].try_into().unwrap(),
            wires_my_circuit[0usize],
            wires_my_circuit[0usize],
        );
        (*wire(wires_my_circuit[11usize])) = (*wire(v_my_circuit_0)) + (*wire(v_my_circuit_0));
        println!("{}", (*wire(wires_my_circuit[11usize])).into_bigint());
    };
    my_circuit(wires_[0usize..12usize].try_into().unwrap());
}
