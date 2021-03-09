use cape::{
    backend::skulpin::{Error, Window, WindowInfo},
    node::{
        iff, interact, rectangle, styled_text, FontProperties, Interaction, IntoNode, KeyCode,
        Paint, Resources,
    },
    point2, rgb, rgba, size2,
    state::{on_lifecycle, use_event, use_state, use_static, Accessor, EventListener, Lifecycle},
    ui,
};

use conifer::{
    button, column, combo_box, container, deco::Decorated, row, stack, Align, Apply, ColumnItem,
    LayoutBuilder, RowItem, StackItem,
};

use conifer::kits::air;

#[ui]
fn counter() -> impl IntoNode {
    let open = use_state(|| false);
    let listener = use_state(EventListener::null);

    on_lifecycle(move |e, _| match e {
        Lifecycle::Create => listener.set(use_event().connect(move |e: &Interaction| {
            if let Interaction::KeyDown {
                key_code: KeyCode::P,
                modifiers,
            } = e
            {
                if modifiers.ctrl() {
                    open.set(!open.get());
                }
            }
        })),
        Lifecycle::Destroy => use_event::<Interaction>().disconnect(listener.get()),
    });

    stack()
        .child_item(
            column()
                .child_item(
                    stack()
                        .child_item(
                            rectangle(
                                size2(0., 50.),
                                [0.; 4],
                                Paint::LinearGradient {
                                    begin: point2(0., 0.),
                                    end: point2(0., 1.),
                                    stops: vec![(0., rgb(56, 56, 56)), (1., rgb(38, 38, 38))],
                                },
                                0.,
                                None,
                            ),
                            StackItem::fill(),
                        )
                        .child_item(
                            button()
                                .child("Compile")
                                .on_click(move |_: &_| open.set(!open.get()))
                                .apply(air::button),
                            StackItem::center(),
                        ),
                    ColumnItem {
                        align: Align::Fill,
                        ..Default::default()
                    },
                )
                .child_item(
                    rectangle(
                        size2(0., 1.),
                        [0.; 4],
                        Paint::Solid(rgb(74, 74, 74)),
                        0.,
                        None,
                    ),
                    ColumnItem {
                        align: Align::Fill,
                        ..Default::default()
                    },
                )
                .child_item(
                    row()
                        .child_item(
                            rectangle(
                                size2(200., 0.),
                                [0.; 4],
                                Paint::Solid(rgb(56, 56, 56)),
                                0.,
                                None,
                            ),
                            RowItem {
                                align: Align::Fill,
                                ..Default::default()
                            },
                        )
                        .child_item(
                            rectangle(
                                size2(1., 0.),
                                [0.; 4],
                                Paint::Solid(rgb(74, 74, 74)),
                                0.,
                                None,
                            ),
                            RowItem {
                                align: Align::Fill,
                                ..Default::default()
                            },
                        )
                        .child_item(
                            rectangle(
                                size2(0., 0.),
                                [0.; 4],
                                Paint::Solid(rgb(38, 38, 38)),
                                0.,
                                None,
                            ),
                            RowItem {
                                align: Align::Fill,
                                fill: true,
                                ..Default::default()
                            },
                        ),
                    ColumnItem {
                        align: Align::Fill,
                        fill: true,
                        ..Default::default()
                    },
                ),
            StackItem::fill(),
        )
        .child_item(
            iff(open.get(), || {
                interact(
                    stack()
                        .child_item(
                            rectangle(
                                size2(0., 0.),
                                [0.; 4],
                                Paint::Solid(rgba(0, 0, 0, 150)),
                                0.,
                                None,
                            ),
                            StackItem::fill(),
                        )
                        .child_item(
                            stack()
                                .child_item(
                                    rectangle(
                                        size2(0., 0.),
                                        [5.; 4],
                                        Paint::Blur {
                                            radius: 20.,
                                            tint: rgba(0, 0, 0, 50),
                                        },
                                        2.,
                                        Paint::Solid(rgba(255, 255, 255, 40)),
                                    ),
                                    StackItem::fill(),
                                )
                                .child(
                                    container()
                                        .margin(cape::Sides2::new(10., 10., 10., 15.))
                                        .child(styled_text(
                                            "Search...",
                                            "sans-serif",
                                            24.,
                                            Paint::Solid(rgba(255, 255, 255, 100)),
                                        )),
                                ),
                            StackItem {
                                wh_offset: Some(size2(400., 400.)),
                                xy: point2(0.5, 0.),
                                xy_anchor: point2(0.5, 0.),
                                xy_offset: point2(0., 75.),
                                ..StackItem::center()
                            },
                        ),
                    |_| {},
                    false,
                )
            }),
            StackItem::fill(),
        )
}

#[ui]
fn test() -> impl IntoNode {
    let count = use_state(|| 0);

    column()
        .child(
            button()
                .child(format!("+100 ({})", count.get()))
                .on_click(move |_: &_| count.set(count.get() + 100)),
        )
        .child(
            button()
                .child("-100")
                .on_click(move |_: &_| count.set(count.get() - 100)),
        )
        .children((0..count.get()).map(|i| format!("[{}]", i)).collect())
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
