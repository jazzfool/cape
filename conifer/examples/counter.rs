use cape::{
    backend::skulpin::{Error, Window, WindowInfo},
    cx::Cx,
    node::{FontProperties, IntoNode, Resources},
    rgba, ui, Sides2,
};

use conifer::{dark::button, Apply, Button, Callback, Column, Container, LayoutBuilder, Row};

#[ui]
fn counter(cx: &mut Cx) -> impl IntoNode {
    let count = cx.state(|| 0);

    Container::new()
        .margin(Sides2::new_all_same(10.))
        .child(
            Column::new()
                .spacing(5.)
                .child(format!("Count: {}", cx.at(count)))
                .child(
                    Row::new()
                        .spacing(5.)
                        .child(
                            Button::new(cx)
                                .child("Increment")
                                .on_click(Callback::new(move |cx, _| *cx.at(count) += 1))
                                .apply(button),
                        )
                        .child(
                            Button::new(cx)
                                .child("Decrement")
                                .on_click(Callback::new(move |cx, _| *cx.at(count) -= 1))
                                .apply(button),
                        ),
                ),
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
