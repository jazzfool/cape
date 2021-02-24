use crate::{column, container, stack, LayoutBuilder, StackItem};
use cape::node::{
    iff, interact, rectangle, z_index, Interaction, MouseButton, Node, Paint, ToNode, ZIndex,
};
use cape::state::{on_render, use_cache, use_state, Accessor};
use cape::{point2, rgb, size2, Sides2};
use std::rc::Rc;

pub struct ComboBoxBuilder {
    style: ComboBoxStyle,
    values: Vec<String>,
    selected: usize,
    on_change: Option<Rc<dyn Fn(String, usize)>>,
}

impl Default for ComboBoxBuilder {
    fn default() -> Self {
        ComboBoxBuilder {
            style: ComboBoxStyle::default_dark(),
            values: Vec::new(),
            selected: 0,
            on_change: None,
        }
    }
}

impl ToNode for ComboBoxBuilder {
    #[cape::ui]
    fn to_node(self) -> Node {
        assert!(!self.values.is_empty(), "combo box cannot be empty");
        assert!(
            self.selected < self.values.len(),
            "selected index must be valid"
        );

        fn item(
            idx: usize,
            value: String,
            on_change: Option<Rc<dyn Fn(String, usize)>>,
            style: &ComboBoxStyle,
            opened: impl Accessor<bool>,
            selected: usize,
        ) -> impl ToNode {
            interact(
                stack()
                    .height(style.height)
                    .child_item(
                        rectangle(
                            size2(0., 0.),
                            style.corner_radius,
                            if idx == selected {
                                style.item_selected.clone()
                            } else {
                                style.item_normal.clone()
                            },
                            0.,
                            None,
                        ),
                        StackItem::fill(),
                    )
                    .child_item(
                        value.clone(),
                        StackItem {
                            xy: point2(0., 0.5),
                            xy_anchor: point2(0., 0.5),
                            xy_offset: point2(style.margin.left, 0.),
                            ..Default::default()
                        },
                    ),
                move |event| {
                    if let Interaction::MouseDown {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        if let Some(on_change) = &on_change {
                            opened.set(false);
                            on_change(value.clone(), idx);
                        }
                    }
                },
                false,
            )
        }

        let opened = use_state(|| false);
        let width = use_state(|| -1.0f32);
        let values = use_cache(&self.values, |values| {
            width.set(-1.);
            values
                .iter()
                .map(|value| value.to_node())
                .collect::<Vec<_>>()
        });

        on_render(move |resources| {
            if width.get() < 0. {
                values.with(|values| {
                    width.set(values.iter().fold(0., |width, value| {
                        width.max(value.text_layout(resources).unwrap().1.width)
                    }));
                });
            }
        });

        stack()
            .width(width.get() + self.style.margin.horizontal())
            .height(self.style.height)
            // The combo-box itself
            .child_item(
                interact(
                    stack()
                        .height(self.style.height)
                        .child_item(
                            rectangle(
                                size2(0., 0.),
                                self.style.corner_radius,
                                self.style.background_normal.clone(),
                                self.style.border_width,
                                self.style.border_normal.clone(),
                            ),
                            StackItem::fill(),
                        )
                        .child_item(
                            container()
                                .margin(self.style.margin)
                                .child(&self.values[self.selected]),
                            StackItem::left_center(),
                        ),
                    move |event| match event {
                        Interaction::MouseDown { .. } => opened.set(!opened.get()),
                        Interaction::LoseFocus => opened.set(false),
                        _ => {}
                    },
                    false,
                ),
                StackItem::fill(),
            )
            // The popup list
            .child_item(
                iff(opened.get(), || {
                    z_index(
                        stack()
                            .child_item(
                                rectangle(
                                    size2(0., 0.),
                                    self.style.corner_radius,
                                    None,
                                    self.style.popup_border_width,
                                    self.style.popup_border.clone(),
                                ),
                                StackItem::fill(),
                            )
                            .child_item(
                                column().children_items(
                                    self.values
                                        .iter()
                                        .enumerate()
                                        .map(|(i, value)| {
                                            (
                                                item(
                                                    i,
                                                    value.clone(),
                                                    self.on_change.clone(),
                                                    &self.style,
                                                    opened,
                                                    self.selected,
                                                ),
                                                crate::ui::ColumnItem {
                                                    align: crate::ui::Align::Fill,
                                                    ..Default::default()
                                                },
                                            )
                                        })
                                        .collect(),
                                ),
                                StackItem::fill(),
                            ),
                        ZIndex(std::i32::MAX),
                    )
                }),
                StackItem {
                    xy_offset: point2(0., self.selected as f32 * -self.style.height),
                    width: Some(1.),
                    ..Default::default()
                },
            )
            .to_node()
    }
}

impl ComboBoxBuilder {
    pub fn style(mut self, style: ComboBoxStyle) -> Self {
        self.style = style;
        self
    }

    pub fn values(mut self, values: &[String]) -> Self {
        self.values = values.to_vec();
        self
    }

    pub fn selected(mut self, selected: usize) -> Self {
        self.selected = selected;
        self
    }

    pub fn on_change(mut self, on_change: impl Fn(String, usize) + 'static) -> Self {
        self.on_change = Some(Rc::new(on_change));
        self
    }

    pub fn state(self, state: impl Accessor<usize>) -> Self {
        self.selected(state.get())
            .on_change(move |_, i| state.set(i))
    }
}

pub struct ComboBoxStyle {
    pub border_width: f32,
    pub corner_radius: [f32; 4],
    pub margin: Sides2,
    pub height: f32,

    pub background_normal: Option<Paint>,
    pub background_hover: Option<Paint>,
    pub background_click: Option<Paint>,

    pub border_normal: Option<Paint>,
    pub border_hover: Option<Paint>,
    pub border_click: Option<Paint>,

    pub item_normal: Option<Paint>,
    pub item_hover: Option<Paint>,
    pub item_selected: Option<Paint>,

    pub popup_border_width: f32,
    pub popup_border: Option<Paint>,
}

impl ComboBoxStyle {
    pub fn default_dark() -> Self {
        ComboBoxStyle {
            border_width: 0.,
            corner_radius: [3.0; 4],
            margin: Sides2::new(5., 10., 5., 10.),
            height: 25.,

            background_normal: Paint::Solid(rgb(26, 26, 26)).into(),
            background_hover: Paint::Solid(rgb(30, 30, 30)).into(),
            background_click: Paint::Solid(rgb(22, 22, 22)).into(),

            border_normal: None,
            border_hover: None,
            border_click: None,

            item_normal: Paint::Solid(rgb(26, 26, 26)).into(),
            item_hover: Paint::Solid(rgb(30, 30, 30)).into(),
            item_selected: Paint::Solid(rgb(25, 78, 197)).into(),

            popup_border_width: 2.,
            popup_border: Paint::Solid(rgb(70, 70, 70)).into(),
        }
    }
}

pub fn combo_box() -> ComboBoxBuilder {
    ComboBoxBuilder::default()
}
