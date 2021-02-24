pub mod deco;
mod ui;

#[cfg(feature = "kits")]
pub mod kits;

pub use ui::*;

pub(crate) mod util {
    pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + t * (b - a)
    }

    // inverse lerp
    pub fn ilerp(a: f32, b: f32, v: f32) -> f32 {
        (v - a) / (b - a)
    }
}
