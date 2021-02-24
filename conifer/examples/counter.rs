use cape::{
    backend::skulpin::{Error, Window, WindowInfo},
    node::{iff, FontProperties, Resources, ToNode},
    state::{on_lifecycle, use_event, use_state, use_static, Accessor, Lifecycle},
    ui, DEFAULT_DARK_BACKGROUND,
};

use conifer::{button, column, container, deco::Decorated, stack, LayoutBuilder, StackItem};

use conifer::kits::air;

#[ui]
fn counter() -> impl ToNode {
    container().margin(cape::Sides2::new_all_same(20.)).child(
        column()
            .spacing(10.0)
            .child("start")
            .child(
                button()
                    .child("Button 1")
                    .on_click(|_| println!("Hello"))
                    .decorated(air::button),
            )
            .child("end"),
    )
}

#[ui]
fn window(_info: &WindowInfo, resources: &mut Resources) -> Window {
    if !resources.has_font("sans-serif") {
        resources
            .load_font(
                "sans-serif",
                &[String::from("Inter")],
                &FontProperties::default(),
            )
            .unwrap();
    }

    Window {
        body: counter().to_node(),
        background: *DEFAULT_DARK_BACKGROUND,
    }
}

fn main() -> Result<(), Error> {
    use_static(conifer::kits::Palette::new).with(|pal| {
        pal.insert(
            String::from("button-normal"),
            cape::node::Paint::Solid(cape::rgba(40, 40, 40, 255)),
        );

        pal.insert(
            String::from("button-border"),
            cape::node::Paint::Solid(cape::rgba(100, 100, 100, 255)),
        );
    });

    cape::backend::skulpin::run(window)
}
