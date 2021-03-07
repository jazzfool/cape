use cape::{
    backend::skulpin::{Error, Window, WindowInfo},
    node::{FontProperties, Paint, Resources},
    point2, rgb, size2,
    state::{use_state, use_static, Accessor},
    ui::{Expand, Merge, NodeLayout, Rectangle, Text},
    Sides2,
};

use conifer::{
    button, combo_box, kits::air, Column, ColumnLayout, ComboBox, Container, ContainerLayout,
    FnProps, Props, Row, RowLayout, Stack, StackItem, StackLayout,
};

type Counter = air::ComboBox<Text>;

#[cape::ui]
fn counter() -> Counter {
    let index = use_state(|| 1);

    combo_box
        .props()
        .items(vec![
            Text::from("Item A"),
            Text::from("Item B"),
            Text::from("Item C"),
        ])
        .state(index)
        .build_with(air::combo_box)
}

#[cape::ui]
fn window(_info: &WindowInfo, resources: &mut Resources) -> Window {
    if !resources.has_font("sans-serif") {
        resources
            .load_font(
                "sans-serif",
                &[String::from("Inter")],
                &FontProperties::default(),
            )
            .unwrap();

        resources
            .load_font(
                "sans-serif-bold",
                &[String::from("Inter")],
                &FontProperties {
                    weight: cape::font_kit::properties::Weight::BLACK,
                    ..Default::default()
                },
            )
            .unwrap();
    }

    let prev = use_state(|| None);

    let next = counter();

    prev.with(|prev: &mut Option<Counter>| {
        if let Some(prev) = prev {
            prev.merge(next);
        } else {
            *prev = Some(next);
        }
    });

    Window {
        body: prev
            .with(|prev| prev.expand(resources))
            .unwrap()
            .remove(0)
            .resolve(resources)
            .unwrap(),
        background: air::BACKGROUND_COLOR,
    }
}

fn main() -> Result<(), Error> {
    use_static(conifer::kits::Palette::new).with(|pal| air::set_palette(pal));

    cape::backend::skulpin::run(window)
}
