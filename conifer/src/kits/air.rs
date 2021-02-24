use crate::{
    deco::{self, Decorated},
    ui,
};
use cape::node::{draw, Node, ToNode};
use cape::state::{use_static, Accessor};
use cape::ToSkia;

struct Shadow {}

impl deco::Decorator for Shadow {
    fn order(&self) -> deco::Order {
        deco::Order::Back
    }

    fn layout(&self) -> crate::StackItem {
        crate::StackItem::fill()
    }

    fn node(self) -> Node {
        draw(cape::Size2::new(0., 0.), |mut rect, canvas| {
            //rect /= 2.0;
            //rect /= 1.5;

            let mut path = cape::skia::Path::new();

            path.add_rrect(
                cape::skia::RRect::new_rect_radii(
                    rect.to_skia(),
                    &[cape::skia::Point::new(5., 5.); 4],
                ),
                None,
            );

            let shadow_x = (rect.min_x() + rect.max_x()) / 2.;
            let shadow_y = rect.min_y() - 600.;

            canvas.draw_shadow(
                &path,
                cape::Point3::new(0., 0., 7.).to_skia(),
                cape::Point3::new(shadow_x, shadow_y, 600.).to_skia(),
                800.,
                cape::Color::new(1., 1., 1., 0.2).to_skia(),
                cape::Color::new(0., 0., 0., 0.5).to_skia(),
                None,
            );
        })
    }
}

pub fn shadow() -> impl deco::Decorator {
    Shadow {}
}

struct Background {
    paint: super::Paint,
    radii: [f32; 4],
}

impl deco::Decorator for Background {
    fn order(&self) -> deco::Order {
        deco::Order::Back
    }

    fn layout(&self) -> crate::StackItem {
        crate::StackItem::fill()
    }

    fn node(self) -> Node {
        cape::node::rectangle(
            Default::default(),
            self.radii,
            self.paint
                .resolve()
                .expect("failed to resolve paint from palette"),
            0.,
            None,
        )
    }
}

pub fn background(paint: super::Paint, radii: [f32; 4]) -> impl deco::Decorator {
    Background { paint, radii }
}

struct Border {
    paint: super::Paint,
    radii: [f32; 4],
    width: f32,
}

impl deco::Decorator for Border {
    fn order(&self) -> deco::Order {
        deco::Order::Back
    }

    fn layout(&self) -> crate::StackItem {
        crate::StackItem::fill()
    }

    fn node(self) -> Node {
        cape::node::rectangle(
            cape::Size2::default(),
            self.radii,
            None,
            self.width,
            self.paint
                .resolve()
                .expect("failed to resolve paint from palette"),
        )
    }
}

pub fn border(paint: super::Paint, radii: [f32; 4], width: f32) -> impl deco::Decorator {
    Border {
        paint,
        radii,
        width,
    }
}

pub fn button(node: crate::ButtonBuilder) -> deco::DecoratedNode {
    node.padding(cape::Sides2::new(5., 15., 5., 15.))
        .decorate()
        .decorator(border(
            super::Paint::Palette(String::from("button-border")),
            [5.; 4],
            1.,
        ))
        .decorator(background(
            super::Paint::Palette(String::from("button-normal")),
            [5.; 4],
        ))
        .decorator(shadow())
}
