pub mod deco;
mod ui;

#[cfg(feature = "kits")]
pub mod kits;

pub use ui::*;

pub enum Callback<T> {
    None,
    Func(std::rc::Rc<dyn Fn(&T)>),
}

impl<T> Default for Callback<T> {
    fn default() -> Self {
        Callback::None
    }
}

impl<T, F: Fn(&T) + 'static> From<F> for Callback<T> {
    fn from(f: F) -> Self {
        Callback::Func(std::rc::Rc::new(f))
    }
}

impl<T> Callback<T> {
    pub fn take(&mut self) -> Self {
        std::mem::replace(self, Callback::None)
    }

    pub fn call(&self, arg: &T) {
        if let Callback::Func(f) = self {
            f(arg);
        }
    }
}

pub trait Apply: Sized {
    fn apply(self, f: impl FnOnce(Self) -> Self) -> Self {
        f(self)
    }
}

impl<T> Apply for T {}

pub(crate) mod util {
    pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + t * (b - a)
    }

    // inverse lerp
    pub fn ilerp(a: f32, b: f32, v: f32) -> f32 {
        (v - a) / (b - a)
    }
}
