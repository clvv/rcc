pub mod runtime_composer;
pub mod mock_composer;

/// Composer trait
pub trait Composer {
    type Wire;
    type ContextMarker;
}

