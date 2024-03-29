use crate::{Callback, LayoutBuilder, Stack, StackItem};
use cape::{
    cx::{Cx, Handle, State},
    node::{interact, rectangle, IntoNode, Node, Paint},
    point2, rgb, size2, Size2,
};

pub struct Slider<'a> {
    cx: &'a mut Cx,
    value: f32,
    range: std::ops::Range<f32>,
    style: SliderStyle,
    disabled: bool,
    width: Option<f32>,
    on_change: Callback<f32>,
}

impl<'a> IntoNode for Slider<'a> {
    #[cape::ui]
    fn into_node(self) -> Node {
        Stack::new()
            .width(self.width)
            .height(self.style.slider_size.height)
            .child_item(
                rectangle(
                    size2(0., self.style.track_height),
                    self.style.track_corner_radius,
                    self.style.track_background,
                    self.style.track_border_width,
                    self.style.track_border,
                ),
                StackItem {
                    width: Some(1.),
                    xy: point2(0.5, 0.5),
                    xy_anchor: point2(0.5, 0.5),
                    ..Default::default()
                },
            )
            .child_item(
                interact(
                    rectangle(
                        size2(0., 0.),
                        self.style.slider_corner_radius,
                        self.style.slider_background,
                        self.style.slider_border_width,
                        self.style.slider_border,
                    ),
                    move |_, _| println!("slider"),
                    false,
                ),
                StackItem {
                    xy: point2(
                        crate::util::ilerp(self.range.start, self.range.end, self.value),
                        0.5,
                    ),
                    xy_anchor: point2(0.5, 0.5),
                    wh_offset: self.style.slider_size.into(),
                    ..Default::default()
                },
            )
            .into_node()
    }
}

impl<'a> Slider<'a> {
    pub fn new(cx: &'a mut Cx) -> Self {
        Slider {
            cx,
            value: 0.,
            range: 0.0..1.0,
            style: SliderStyle::default_dark(),
            disabled: false,
            width: None,
            on_change: Default::default(),
        }
    }

    pub fn value(mut self, value: impl FnOnce(&mut Cx) -> f32) -> Self {
        self.value = value(self.cx);
        self
    }

    pub fn range(mut self, range: std::ops::Range<f32>) -> Self {
        self.range = range;
        self
    }

    pub fn style(mut self, style: SliderStyle) -> Self {
        self.style = style;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn width(mut self, width: impl Into<Option<f32>>) -> Self {
        self.width = width.into();
        self
    }

    pub fn on_change(mut self, on_change: impl Into<Callback<f32>>) -> Self {
        self.on_change = on_change.into();
        self
    }

    pub fn state(self, state: Handle<f32, State>) -> Self {
        self.value(|cx| *cx.at(state))
            .on_change(move |cx: &mut Cx, val: &f32| *cx.at(state) = *val)
    }
}

pub struct SliderStyle {
    pub track_corner_radius: [f32; 4],
    pub track_background: Option<Paint>,
    pub track_border_width: f32,
    pub track_border: Option<Paint>,
    pub track_height: f32,

    pub slider_corner_radius: [f32; 4],
    pub slider_background: Option<Paint>,
    pub slider_border_width: f32,
    pub slider_border: Option<Paint>,
    pub slider_size: Size2,
}

impl SliderStyle {
    pub fn default_dark() -> Self {
        SliderStyle {
            track_corner_radius: [3.; 4],
            track_background: Paint::Solid(rgb(26, 26, 26)).into(),
            track_border_width: 0.,
            track_border: None,
            track_height: 10.,

            slider_corner_radius: [3.; 4],
            slider_background: Paint::Solid(rgb(25, 78, 197)).into(),
            slider_border_width: 0.,
            slider_border: None,
            slider_size: size2(10., 20.),
        }
    }
}
