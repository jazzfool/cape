//! Simple dark theme

use crate::{Apply, Button, LayoutBuilder, Stack, StackItem};
use cape::{
    frgb,
    node::{iff, rectangle, IntoNode, Node, Paint},
    rgba, rrr, Color, Sides2,
};

const BLUE_ACCENT: Color = frgb(0.17647, 0.33333, 0.890196);

fn with_opacity(a: f32) -> impl Fn(Color) -> Color {
    move |mut c| {
        c.alpha = a;
        c
    }
}

pub fn button(mut btn: Button) -> Button {
    let mut hovered = false;
    let mut pressed = false;
    let mut focused = false;

    let disabled = btn.disabled;

    if disabled {
        if let Node::Text { fill, .. } = &mut btn.child {
            *fill = Some(Paint::Solid(rgba(255, 255, 255, 120)));
        }
    }

    btn.hovered(&mut hovered)
        .pressed(&mut pressed)
        .focused(&mut focused)
        .padding(Sides2::new(5., 10., 5., 10.))
        .background(
            Stack::new()
                .child_item(
                    iff(focused && !disabled, || {
                        rectangle(
                            Default::default(),
                            [7.; 4],
                            Paint::Solid(BLUE_ACCENT.apply(with_opacity(0.5))),
                            0.,
                            None,
                        )
                    }),
                    StackItem::inflate(Sides2::new_all_same(3.)),
                )
                .child_item(
                    rectangle(
                        Default::default(),
                        [5.; 4],
                        Paint::Solid(rrr(if disabled {
                            70
                        } else if pressed {
                            60
                        } else if hovered {
                            80
                        } else {
                            70
                        })),
                        1.,
                        Paint::Solid(rrr(5)),
                    ),
                    StackItem::fill(),
                )
                .into_node(),
        )
}
