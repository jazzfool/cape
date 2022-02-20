use cape::{
    backend::skulpin::{Error, Window, WindowInfo},
    cx::Cx,
    node::{FontProperties, IntoNode, Resources},
    rgba, ui, Sides2,
};

use conifer::{dark::button, Apply, Button, Callback, Column, Container, LayoutBuilder, Row};

// Straight-forward performance test: How long of a list can Cape handle?

#[ui]
fn counter(cx: &mut Cx) -> impl IntoNode {
    let count = cx.state(|| 0);

    Container::new()
        .margin(Sides2::new_all_same(10.))
        .child(
            Column::new()
                .spacing(1.)
                .children((0..50000).map(|i| format!("Item #{}", i))),
        )
        .into_node()
}

#[ui]
fn window(_info: &WindowInfo, cx: &mut Cx, resources: &mut Resources) -> Window {
    if !resources.has_font("sans-serif") {
        resources
            .load_font(
                "sans-serif",
                &[String::from("Segoe UI")],
                &FontProperties::default(),
            )
            .unwrap();
    }

    Window {
        body: counter(cx).into_node(),
        background: rgba(30, 30, 30, 255),
    }
}

fn main() -> Result<(), Error> {
    cape::backend::skulpin::run(window)
}
