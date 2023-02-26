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
        let add_const_1 = |wires_add_const: &[usize], in_add_const_0| {
            (*wire(wires_add_const[1usize])) =
                (*wire(in_add_const_0)) + (*wire(wires_add_const[0usize]));
        };
        let sub_const_3 = |wires_sub_const: &[usize], in_sub_const_0, in_sub_const_1| {
            (*wire(wires_sub_const[0usize])) = (*wire(in_sub_const_0)) - (*wire(in_sub_const_1));
        };
        let gen_1 = |wires_gen: &[usize], in_gen_0| {
            add_const_1(wires_gen[0usize..2usize].try_into().unwrap(), in_gen_0);
            sub_const_3(
                wires_gen[2usize..3usize].try_into().unwrap(),
                in_gen_0,
                wires_gen[0usize],
            );
            add_const_1(wires_gen[3usize..5usize].try_into().unwrap(), in_gen_0);
            sub_const_3(
                wires_gen[5usize..6usize].try_into().unwrap(),
                in_gen_0,
                wires_gen[3usize],
            );
            add_const_1(wires_gen[6usize..8usize].try_into().unwrap(), in_gen_0);
            sub_const_3(
                wires_gen[8usize..9usize].try_into().unwrap(),
                in_gen_0,
                wires_gen[6usize],
            );
            add_const_1(wires_gen[9usize..11usize].try_into().unwrap(), in_gen_0);
            sub_const_3(
                wires_gen[11usize..12usize].try_into().unwrap(),
                in_gen_0,
                wires_gen[9usize],
            );
            add_const_1(wires_gen[12usize..14usize].try_into().unwrap(), in_gen_0);
            sub_const_3(
                wires_gen[14usize..15usize].try_into().unwrap(),
                in_gen_0,
                wires_gen[12usize],
            );
            add_const_1(wires_gen[15usize..17usize].try_into().unwrap(), in_gen_0);
            sub_const_3(
                wires_gen[17usize..18usize].try_into().unwrap(),
                in_gen_0,
                wires_gen[15usize],
            );
            add_const_1(wires_gen[18usize..20usize].try_into().unwrap(), in_gen_0);
            sub_const_3(
                wires_gen[20usize..21usize].try_into().unwrap(),
                in_gen_0,
                wires_gen[18usize],
            );
            add_const_1(wires_gen[21usize..23usize].try_into().unwrap(), in_gen_0);
            sub_const_3(
                wires_gen[23usize..24usize].try_into().unwrap(),
                in_gen_0,
                wires_gen[21usize],
            );
            add_const_1(wires_gen[24usize..26usize].try_into().unwrap(), in_gen_0);
            sub_const_3(
                wires_gen[26usize..27usize].try_into().unwrap(),
                in_gen_0,
                wires_gen[24usize],
            );
            add_const_1(wires_gen[27usize..29usize].try_into().unwrap(), in_gen_0);
            sub_const_3(
                wires_gen[29usize..30usize].try_into().unwrap(),
                in_gen_0,
                wires_gen[27usize],
            );
        };
        let mul_31 = |wires_mul: &[usize], in_mul_0, in_mul_1| {
            (*wire(wires_mul[0usize])) = (*wire(in_mul_0)) * (*wire(in_mul_1));
        };
        let mul_32 = |wires_mul: &[usize], in_mul_0| {
            (*wire(wires_mul[0usize])) = (*wire(in_mul_0)) * (*wire(in_mul_0));
        };
        let mul_seq_31 = |wires_mul_seq: &[usize], in_mul_seq_0, in_mul_seq_1| {
            mul_31(
                wires_mul_seq[0usize..1usize].try_into().unwrap(),
                in_mul_seq_0,
                in_mul_seq_1,
            );
            mul_32(
                wires_mul_seq[1usize..2usize].try_into().unwrap(),
                wires_mul_seq[0usize],
            );
            mul_32(
                wires_mul_seq[2usize..3usize].try_into().unwrap(),
                wires_mul_seq[1usize],
            );
            mul_32(
                wires_mul_seq[3usize..4usize].try_into().unwrap(),
                wires_mul_seq[2usize],
            );
            mul_32(
                wires_mul_seq[4usize..5usize].try_into().unwrap(),
                wires_mul_seq[3usize],
            );
            mul_32(
                wires_mul_seq[5usize..6usize].try_into().unwrap(),
                wires_mul_seq[4usize],
            );
            mul_32(
                wires_mul_seq[6usize..7usize].try_into().unwrap(),
                wires_mul_seq[5usize],
            );
            mul_32(
                wires_mul_seq[7usize..8usize].try_into().unwrap(),
                wires_mul_seq[6usize],
            );
            mul_32(
                wires_mul_seq[8usize..9usize].try_into().unwrap(),
                wires_mul_seq[7usize],
            );
            mul_32(
                wires_mul_seq[9usize..10usize].try_into().unwrap(),
                wires_mul_seq[8usize],
            );
            mul_32(
                wires_mul_seq[10usize..11usize].try_into().unwrap(),
                wires_mul_seq[9usize],
            );
        };
        let add_141 = |wires_add: &[usize], in_add_0, in_add_1| {
            (*wire(wires_add[0usize])) = (*wire(in_add_0)) + (*wire(in_add_1));
        };
        let sum_141 = |wires_sum: &[usize],
                       in_sum_0,
                       in_sum_1,
                       in_sum_2,
                       in_sum_3,
                       in_sum_4,
                       in_sum_5,
                       in_sum_6,
                       in_sum_7,
                       in_sum_8,
                       in_sum_9| {
            add_141(
                wires_sum[0usize..1usize].try_into().unwrap(),
                in_sum_0,
                in_sum_1,
            );
            add_141(
                wires_sum[1usize..2usize].try_into().unwrap(),
                wires_sum[0usize],
                in_sum_2,
            );
            add_141(
                wires_sum[2usize..3usize].try_into().unwrap(),
                wires_sum[1usize],
                in_sum_3,
            );
            add_141(
                wires_sum[3usize..4usize].try_into().unwrap(),
                wires_sum[2usize],
                in_sum_4,
            );
            add_141(
                wires_sum[4usize..5usize].try_into().unwrap(),
                wires_sum[3usize],
                in_sum_5,
            );
            add_141(
                wires_sum[5usize..6usize].try_into().unwrap(),
                wires_sum[4usize],
                in_sum_6,
            );
            add_141(
                wires_sum[6usize..7usize].try_into().unwrap(),
                wires_sum[5usize],
                in_sum_7,
            );
            add_141(
                wires_sum[7usize..8usize].try_into().unwrap(),
                wires_sum[6usize],
                in_sum_8,
            );
            add_141(
                wires_sum[8usize..9usize].try_into().unwrap(),
                wires_sum[7usize],
                in_sum_9,
            );
        };
        let wires_: Vec<usize> = (0..150usize).collect();
        (*wire(wires_[0usize])) = F::from(args.get(1usize).unwrap().parse::<i32>().unwrap());
        gen_1(wires_[1usize..31usize].try_into().unwrap(), wires_[0usize]);
        mul_seq_31(
            wires_[31usize..42usize].try_into().unwrap(),
            wires_[2usize],
            wires_[3usize],
        );
        mul_seq_31(
            wires_[42usize..53usize].try_into().unwrap(),
            wires_[5usize],
            wires_[6usize],
        );
        mul_seq_31(
            wires_[53usize..64usize].try_into().unwrap(),
            wires_[8usize],
            wires_[9usize],
        );
        mul_seq_31(
            wires_[64usize..75usize].try_into().unwrap(),
            wires_[11usize],
            wires_[12usize],
        );
        mul_seq_31(
            wires_[75usize..86usize].try_into().unwrap(),
            wires_[14usize],
            wires_[15usize],
        );
        mul_seq_31(
            wires_[86usize..97usize].try_into().unwrap(),
            wires_[17usize],
            wires_[18usize],
        );
        mul_seq_31(
            wires_[97usize..108usize].try_into().unwrap(),
            wires_[20usize],
            wires_[21usize],
        );
        mul_seq_31(
            wires_[108usize..119usize].try_into().unwrap(),
            wires_[23usize],
            wires_[24usize],
        );
        mul_seq_31(
            wires_[119usize..130usize].try_into().unwrap(),
            wires_[26usize],
            wires_[27usize],
        );
        mul_seq_31(
            wires_[130usize..141usize].try_into().unwrap(),
            wires_[29usize],
            wires_[30usize],
        );
        sum_141(
            wires_[141usize..150usize].try_into().unwrap(),
            wires_[41usize],
            wires_[52usize],
            wires_[63usize],
            wires_[74usize],
            wires_[85usize],
            wires_[96usize],
            wires_[107usize],
            wires_[118usize],
            wires_[129usize],
            wires_[140usize],
        );
        println!("{}", (*wire(wires_[149usize])).into_bigint());
        wires
    };
    compute();
}
