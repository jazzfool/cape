use cape::{
    backend::skulpin::{Error, Window, WindowInfo},
    cx::{Cx, ExecHandle},
    node::{FontProperties, IntoNode, Resources},
    rgba, ui, Sides2,
};
use conifer::{dark::button, Apply, Button, Callback, Column, Container, LayoutBuilder};

// This example demonstrates how to run an async function without blocking the UI.
// TL;DR: Get your `Future`, put it through `cx.exec()` to get a `JoinHandle` back.
//      In your UI function, poll that `JoinHandle` with `cx.poll()`.
//
// By calling `cx.exec()`, the `Future` is simply wrapped in a another `Future` which awakens the event loop when finished.

async fn download_number() -> u32 {
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    42
}

#[derive(Clone, Copy, PartialEq)]
enum Number {
    None,
    Downloading,
    Done(u32),
}

impl From<Number> for String {
    fn from(num: Number) -> String {
        match num {
            Number::None => String::from("[no data]"),
            Number::Downloading => String::from("Downloading..."),
            Number::Done(num) => format!("The number is {}", num),
        }
    }
}

#[ui]
fn downloader(cx: &mut Cx) -> impl IntoNode {
    let fut = cx.state(|| -> ExecHandle<u32> { None });
    let num = cx.state(|| Number::None);

    if let Some(n) = cx.poll(fut) {
        *cx.at(num) = Number::Done(n);
        *cx.at(fut) = None;
    }

    let disabled = *cx.at(num) != Number::None;

    Container::new().margin(Sides2::new_all_same(10.)).child(
        Column::new().spacing(5.).child(*cx.at(num)).child(
            Button::new(cx)
                .child("Download the number")
                .on_click(Callback::new(move |cx, _| {
                    if *cx.at(num) == Number::None {
                        *cx.at(num) = Number::Downloading;
                        *cx.at(fut) = Some(cx.exec(download_number()));
                    }
                }))
                .disabled(disabled)
                .apply(button),
        ),
    )
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
        body: downloader(cx).into_node(),
        background: rgba(30, 30, 30, 255),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    cape::backend::skulpin::run(window)
}
