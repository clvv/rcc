use proc_macro2::TokenStream;

pub mod runtime_composer;
pub mod mock_composer;

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

/// Composer trait
pub trait Composer {
    type Wire;
    type BaseComposer: Composer;

    fn base_composer(&mut self) -> Option<&mut Self::BaseComposer> {
        None
    }

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
}
