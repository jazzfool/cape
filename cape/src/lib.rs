use thiserror::Error;

pub mod backend;
pub mod id;
pub mod node;
pub mod state;

pub use cape_macro::{ui, unique_ui};
pub use euclid::{point2, rect, size2};
pub use image::RgbaImage as Image;
pub use palette::rgb::LinSrgba as Color;
pub use skia_safe as skia;
pub use topo::{self, CallId};

pub type Point2 = euclid::Point2D<f32, euclid::UnknownUnit>;
pub type Point3 = euclid::Point3D<f32, euclid::UnknownUnit>;
pub type Size2 = euclid::Size2D<f32, euclid::UnknownUnit>;
pub type Sides2 = euclid::SideOffsets2D<f32, euclid::UnknownUnit>;
pub type Rect = euclid::Rect<f32, euclid::UnknownUnit>;

pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::new(
        r as f32 / 255.,
        g as f32 / 255.,
        b as f32 / 255.,
        a as f32 / 255.,
    )
}

pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    rgba(r, g, b, 255)
}

pub const fn frgba(red: f32, green: f32, blue: f32, alpha: f32) -> Color {
    Color {
        color: palette::rgb::Rgb {
            standard: std::marker::PhantomData,
            red,
            green,
            blue,
        },
        alpha,
    }
}

pub const fn frgb(red: f32, green: f32, blue: f32) -> Color {
    frgba(red, green, blue, 1.)
}

#[track_caller]
pub fn call<F: FnOnce() -> R, R>(f: F) -> R {
    topo::call(f)
}

#[track_caller]
pub fn call_unique<K, F, R>(key: &K, op: F) -> R
where
    F: FnOnce() -> R,
    K: Send + Clone + std::hash::Hash + Eq + 'static,
{
    topo::call_in_slot(key, op)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("error when loading font: {0}")]
    FontLoading(#[from] font_kit::error::FontLoadingError),
    #[error("error when selecting a font: {0}")]
    FontSelect(#[from] font_kit::error::SelectionError),
    #[error("error when loading glyph: {0}")]
    GlyphLoading(#[from] font_kit::error::GlyphLoadingError),
    #[error("error with char codepoints when processing text")]
    CodepointError,
    #[error("no such font \"{0}\" found in resources")]
    InvalidFont(String),
    #[error("error with skia font interface")]
    SkiaFont,
    #[error("invalid child placed in an interact/capture node")]
    EmptyNode,
}

pub trait ToSkia<T> {
    fn to_skia(&self) -> T;
}

impl ToSkia<skia::Rect> for Rect {
    fn to_skia(&self) -> skia::Rect {
        skia::Rect::from_xywh(
            self.origin.x,
            self.origin.y,
            self.size.width,
            self.size.height,
        )
    }
}

impl ToSkia<skia::Point> for Point2 {
    fn to_skia(&self) -> skia::Point {
        skia::Point::new(self.x, self.y)
    }
}

impl ToSkia<skia::Point3> for Point3 {
    fn to_skia(&self) -> skia::Point3 {
        skia::Point3::new(self.x, self.y, self.z)
    }
}

impl ToSkia<skia::Color> for Color {
    fn to_skia(&self) -> skia::Color {
        skia::Color::from_argb(
            (self.alpha.clamp(0., 1.) * 255.) as u8,
            (self.red.clamp(0., 1.) * 255.) as u8,
            (self.green.clamp(0., 1.) * 255.) as u8,
            (self.blue.clamp(0., 1.) * 255.) as u8,
        )
    }
}

pub(crate) mod util {
    pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + t * (b - a)
    }

    // inverse lerp
    pub fn ilerp(a: f32, b: f32, v: f32) -> f32 {
        (v - a) / (b - a)
    }
}
