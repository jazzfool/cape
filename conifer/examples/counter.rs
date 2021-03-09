use std::rc::Rc;

use cape::{
    backend::skulpin::{Error, Window, WindowInfo},
    node::{
        iff, interact, rectangle, styled_text, FontProperties, Interaction, IntoNode, KeyCode,
        Node, Paint, ResolvedNode, Resources,
    },
    point2, rgb, rgba, size2,
    state::{
        on_lifecycle, use_cache, use_event, use_state, use_static, Accessor, EventListener,
        Lifecycle,
    },
    ui,
};

use conifer::{
    button, column, combo_box, container, deco::Decorated, row, stack, Align, Apply, ColumnItem,
    LayoutBuilder, RowItem, StackItem,
};

use conifer::kits::air;

#[ui]
fn test() -> impl IntoNode {
    let hovered = use_state(|| false);

    column()
        .child(interact(
            if hovered.get() { "Hey!" } else { "Sup" },
            move |e| match e {
                Interaction::MouseDown { .. } => hovered.set(true),
                Interaction::MouseUp { .. } => hovered.set(false),
                _ => {}
            },
            false,
        ))
        .children((0..10000).map(|i| format!("[{}]", i)).collect())
}

#[ui]
fn window(info: &WindowInfo, resources: &mut Resources) -> Window {
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
        body: stack()
            .width(info.size.width)
            .height(info.size.height)
            .child_item(test(), StackItem::fill())
            .into_node(),
        background: air::BACKGROUND_COLOR,
    }
}

fn main() -> Result<(), Error> {
    use_static(conifer::kits::Palette::default).with(|pal| {
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
