#[macro_export]
/// Implements a common set of global functions for given Composer and Wire
macro_rules! impl_global_builder {
    ($builder:ident, $wire:ident) => {
        use std::sync::OnceLock;
        static mut GLOBAL_COMPOSER: OnceLock<$builder> = OnceLock::new();

        /// Returns a mutable reference to the global composer
        pub fn builder() -> &'static mut $builder {
            unsafe {
                GLOBAL_COMPOSER.get_mut().unwrap()
            }
        }

        /// This is called automatically in `circuit_main`
        pub fn set_builder() {
            let c = $builder::new();
            unsafe {
                GLOBAL_COMPOSER.set(c);
            }
        }

        /// Allocate and returns a new wire
        pub fn new_wire() -> $wire {
            builder().new_wire()
        }

        /// Allocate and return a vector of `n` new wires
        pub fn new_wires(n: usize) -> Vec<$wire> {
            builder().new_wires(n)
        }

        /// Allocate and return a named new input wire
        pub fn input_wire(name: &str) -> $wire {
            builder().input_wire(name)
        }

        /// Allocate and return a new named vector of input wires
        pub fn input_wires(name: &str, n: usize) -> Vec<$wire> {
            builder().input_wires(name, n)
        }

        /// Decalre a wire to be public with `name`
        pub fn declare_public(w: $wire, name: &str) {
            builder().declare_public(w, name)
        }

        /// Sums a vector of wires
        pub fn sum(v: Vec<$wire>) -> $wire {
            builder().sum(v)
        }

        /// A map that generates optimized witness generation code. Using this over standard
        /// iterators helps with compilation time for the generated witness generator
        pub fn smart_map<T, U>(iter: impl Iterator<Item = T>, f: impl FnMut(&mut $builder, &T) -> U) -> Vec<U> {
            builder().smart_map(iter, f)
        }
    }
}
