#![allow(unused_must_use)]

use proc_macro2::TokenStream;

pub mod runtime_composer;
pub mod traits;

pub trait Wire: Sized + Copy + Clone {
    type Composer: Composer<Wire = Self>;
    fn composer(&self) -> &mut <Self as Wire>::Composer;
}

/// Composer trait
pub trait Composer {
    type Wire: Sized + Copy + Clone;
    type BaseComposer: Composer;

    fn base_composer(&mut self) -> Option<&mut Self::BaseComposer> {
        None
    }

    fn new_wire(&mut self) -> Self::Wire;

    fn new_wires(&mut self, num: usize) -> Vec<Self::Wire> {
        (0..num).map(|_| self.new_wire()).collect()
    }

    fn input_wire(&mut self, name: &str) -> Self::Wire;
    fn input_wires(&mut self, name: &str, num: usize) -> Vec<Self::Wire> ;
    fn declare_public(&mut self, w: Self::Wire);

    fn enter_context(&mut self, name: String) {
        if let Some(e) = self.base_composer() {
            e.enter_context(name)
        }
    }

    fn exit_context(&mut self) {
        if let Some(e) = self.base_composer() {
            e.exit_context()
        }
    }

    fn new_context(&mut self, name: String) -> ContextMarker {
        if let Some(e) = self.base_composer() {
            e.new_context(name)
        } else {
            ContextMarker { func: Box::new(|| {}) }
        }
    }

    fn runtime(&mut self, code: TokenStream) {
        if let Some(e) = self.base_composer() {
            e.runtime(code)
        }
    }

    /// Use this method for repeated (>1000) components to speed up compilation times
    /// Map f over N elements with new contexts for every sqrt(N) iterations
    fn smart_map_base_10<T>(&mut self, iter: impl Iterator<Item = T>, mut f: impl FnMut(&mut Self, &T) -> ()) {
        let items: Vec<T> = iter.collect();
        let power = (items.len() as f64).log(10.0).ceil() as u32;
        let step_sizes = (0..power).map(|i| 10usize.pow(power - i));
        items.iter().enumerate().for_each(|(i, item)| {
            let to_process = step_sizes.clone().filter(|size| i % size == 0);
            if i > 0 {
                to_process.clone().for_each(|_| self.exit_context());
            }
            to_process.for_each(|size| {
                self.enter_context(format!("smart_loop_p{size}"));
            });
            f(self, item)
        });
        step_sizes.for_each(|_| self.exit_context());
    }

    /// Use this method for repeated (>1000) components to speed up compilation times
    /// Map f over N elements with new contexts for every sqrt(N) iterations
    fn smart_map<T>(&mut self, iter: impl Iterator<Item = T>, mut f: impl FnMut(&mut Self, &T) -> ()) {
        let items: Vec<T> = iter.collect();
        let step_size = (items.len() as f64).sqrt() as usize;
        items.iter().enumerate().for_each(|(i, item)| {
            if i % step_size == 0 {
                if i > 0 {
                    self.exit_context();
                }
                self.enter_context(String::from("smart_loop"));
            }
            f(self, item)
        });
        self.exit_context();
    }
}

impl Composer for () {
    type Wire = ();
    type BaseComposer = ();

    fn new_wire(&mut self) -> () {
        ()
    }

    fn input_wire(&mut self, _: &str) -> () {  }
    fn input_wires(&mut self, _: &str, _: usize) -> Vec<()> { Vec::new() }
    fn declare_public(&mut self, _: ()) -> () { }
}

/// A hack to keep automatically call a function when a Rust context exits
pub struct ContextMarker {
    func: Box<dyn Fn() -> ()>
}

impl ContextMarker {
    fn new(func: Box<dyn Fn() -> ()>) -> Self {
        Self { func }
    }
}

impl Drop for ContextMarker {
    fn drop(&mut self) {
        (self.func)()
    }
}

