use crate::{stack, LayoutBuilder, StackItem};
use cape::node::{rectangle, IntoNode, Paint};
use cape::{rgb, size2};

pub struct ProgressBarStyle {
    pub border_width: f32,
    pub corner_radius: [f32; 4],
    pub height: f32,

    pub background: Option<Paint>,
    pub fill: Option<Paint>,
    pub border: Option<Paint>,
}

impl ProgressBarStyle {
    pub fn default_dark() -> Self {
        ProgressBarStyle {
            border_width: 0.,
            corner_radius: [3.; 4],
            height: 15.,

            background: Paint::Solid(rgb(26, 26, 26)).into(),
            fill: Paint::Solid(rgb(25, 78, 197)).into(),
            border: None,
        }
    }
}

pub fn progress_bar(
    value: f32,
    width: f32,
    style: impl Into<Option<ProgressBarStyle>>,
) -> impl IntoNode {
    let style = style.into().unwrap_or_else(ProgressBarStyle::default_dark);

    stack()
        .width(width)
        .height(style.height)
        .child_item(
            rectangle(
                size2(0., 0.),
                style.corner_radius,
                style.background,
                style.border_width,
                style.border,
            ),
            StackItem::fill(),
        )
        .child_item(
            rectangle(size2(0., 0.), style.corner_radius, style.fill, 0., None),
            StackItem {
                width: Some(value),
                height: Some(1.0),
                ..Default::default()
            },
        )
}
