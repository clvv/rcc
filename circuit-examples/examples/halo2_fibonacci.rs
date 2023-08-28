use rcc_halo2::builder::{H2Wire as W, *};

#[main_component]
fn my_circuit() {
    let which_fib_number = 14;
    assert!(which_fib_number > 3);

    let first_fib_number = input_wire("first_fib_number");
    let second_fib_number = input_wire("second_fib_number");
    let output_fib_number = input_wire("output_fib_number");

    let mut fib_numbers = Vec::new();

    fib_numbers.push(first_fib_number);
    fib_numbers.push(second_fib_number);

    // Fibonacci constraints: fib_numbers[i] = fib_numbers[i-1] + fib_numbers[i-2]
    //
    for _ in 0..(which_fib_number - 2) {
        let next_fib_number: W =
            fib_numbers[fib_numbers.len() - 1] + fib_numbers[fib_numbers.len() - 2];
        fib_numbers.push(next_fib_number);
    }

    first_fib_number.declare_public("first_fib_number");
    second_fib_number.declare_public("second_fib_number");

    let last_fib_number = *fib_numbers.last().unwrap();
    last_fib_number.declare_public("last_fib_number");

    // Constrains the last fibonacci number to equal the third public input.
    //
    assert!(last_fib_number == output_fib_number);
}
