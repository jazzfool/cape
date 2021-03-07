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

pub trait Props<T: cape::ui::Merge + cape::ui::Expand>: Default {
    fn build(self) -> T;

    fn build_with(self, f: impl FnOnce(Self) -> T) -> T {
        f(self)
    }
}

pub trait FnProps<T: cape::ui::Merge + cape::ui::Expand, P: Props<T>> {
    fn props(self) -> P;
}

impl<T: cape::ui::Merge + cape::ui::Expand, P: Props<T>, F: Fn(P) -> T> FnProps<T, P> for F {
    fn props(self) -> P {
        Default::default()
    }
}
