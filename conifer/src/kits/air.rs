use crate::{
    deco::{self, Decorated},
    FnProps, Props,
};
use cape::{
    frgb,
    node::{draw, Node, Paint},
    point2, rgb,
    ui::{self, NodeLayout},
    ToSkia,
};

pub const BACKGROUND_COLOR: cape::Color = frgb(22. / 255., 22. / 255., 22. / 255.);

pub fn set_palette(pal: &mut super::Palette) {
    pal.insert(String::from("button-border"), Paint::Solid(rgb(90, 90, 90)));
    pal.insert(String::from("button-normal"), Paint::Solid(rgb(39, 39, 39)));
    pal.insert(
        String::from("button-hovered"),
        Paint::Solid(rgb(45, 45, 45)),
    );
    pal.insert(
        String::from("combo-box-background"),
        Paint::Solid(rgb(39, 39, 39)),
    );
    pal.insert(
        String::from("combo-box-border"),
        Paint::Solid(rgb(90, 90, 90)),
    );
    pal.insert(
        String::from("combo-box-popup-background"),
        Paint::Solid(rgb(10, 10, 10)),
    );
    pal.insert(
        String::from("combo-box-popup-border"),
        Paint::Solid(rgb(90, 90, 90)),
    );
}

pub type Background = ui::Rectangle;

pub fn background(paint: super::Paint, radii: [f32; 4]) -> Background {
    Background {
        background: paint.resolve(),
        border: 0.,
        border_fill: None,
        corner_radius: radii,
        size: Default::default(),
        z_order: Default::default(),
    }
}

pub type Border = ui::Rectangle;

pub fn border(paint: super::Paint, radii: [f32; 4], width: f32) -> Border {
    ui::Rectangle {
        background: None,
        border: width,
        border_fill: paint.resolve(),
        corner_radius: radii,
        size: Default::default(),
        z_order: Default::default(),
    }
}

pub type Button<T> = Decorated<(Background, Border), crate::Button<T>, ()>;

#[cape::ui]
pub fn button<T: Default + ui::Merge + ui::Expand>(props: crate::ButtonProps<T>) -> Button<T> {
    deco::decorated(deco::DecoratedProps {
        below: |state| {
            (
                background(
                    super::Paint::Palette(String::from(if state.hovered {
                        "button-hovered"
                    } else {
                        "button-normal"
                    })),
                    [5.; 4],
                ),
                border(
                    super::Paint::Palette(String::from("button-border")),
                    [5.; 4],
                    1.,
                ),
            )
        },
        child: props.padding(cape::Sides2::new(5., 15., 5., 15.)).build(),
        above: |_| {},
        margin: Default::default(),
        z_order: Default::default(),
    })
}

pub type ComboBox<Item> = crate::ComboBox<ui::Rectangle, ui::Rectangle, Item>;

#[cape::ui]
pub fn combo_box<Item: Default + Clone + ui::Merge + ui::Expand>(
    props: crate::ComboBoxProps<ui::Rectangle, ui::Rectangle, Item>,
) -> ComboBox<Item> {
    props
        .centre_padding(cape::Sides2::new(5., 15., 5., 15.))
        .centre_item(crate::StackItem::center())
        .centre(ui::Rectangle {
            background: super::Paint::Palette(String::from("combo-box-background")).resolve(),
            border_fill: super::Paint::Palette(String::from("combo-box-border")).resolve(),
            border: 1.,
            corner_radius: [5.; 4],
            ..Default::default()
        })
        .popup(ui::Rectangle {
            background: super::Paint::Palette(String::from("combo-box-popup-background")).resolve(),
            border_fill: super::Paint::Palette(String::from("combo-box-popup-border")).resolve(),
            border: 1.,
            corner_radius: [5.; 4],
            ..Default::default()
        })
        .build()
}
