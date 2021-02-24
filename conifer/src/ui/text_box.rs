use crate::{column, container, row, stack, LayoutBuilder, StackItem};
use cape::node::{interact, rectangle, styled_text, Interaction, KeyCode, Node, Paint, ToNode};
use cape::state::{use_state, Accessor};
use cape::{rgb, size2, ui, Sides2};

pub struct TextBoxBuilder {
    style: TextBoxStyle,
    value: String,
    icon: Node,
    disabled: bool,
    on_change: Option<Box<dyn Fn(String)>>,
}

impl Default for TextBoxBuilder {
    fn default() -> Self {
        TextBoxBuilder {
            style: TextBoxStyle::default_dark(),
            value: String::new(),
            icon: Node::Null,
            disabled: false,
            on_change: None,
        }
    }
}

impl ToNode for TextBoxBuilder {
    #[ui]
    fn to_node(self) -> Node {
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
                        .child(row().child(self.icon.clone()).child(styled_text(
                            &self.value,
                            "sans-serif",
                            None,
                            Some(self.style.text.clone()),
                        )))
                        .margin(self.style.margin),
                    StackItem::center(),
                ),
            move |event| match event {
                Interaction::ReceiveCharacter { character } => {
                    if !character.is_control() && !character.is_ascii_control() {
                        if let Some(on_change) = &self.on_change {
                            on_change(format!("{}{}", self.value.clone(), character));
                        }
                    }
                }
                Interaction::KeyDown { key_code, .. } => match key_code {
                    KeyCode::Back => {
                        if !self.value.is_empty() {
                            if let Some(on_change) = &self.on_change {
                                on_change(self.value[0..self.value.len() - 1].to_string());
                            }
                        }
                    }
                    KeyCode::Left => {}
                    KeyCode::Right => {}
                    _ => {}
                },
                _ => {}
            },
            false,
        )
    }
}

impl TextBoxBuilder {
    pub fn style(mut self, style: TextBoxStyle) -> Self {
        self.style = style;
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn icon(mut self, icon: impl ToNode) -> Self {
        self.icon = icon.to_node();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, f: impl Fn(String) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn state(self, state: impl Accessor<String>) -> Self {
        self.value(state.get()).on_change(move |val| state.set(val))
    }
}

pub struct TextBoxStyle {
    pub border_width: f32,
    pub corner_radius: [f32; 4],
    pub margin: Sides2,
    pub height: f32,
    pub text: Paint,

    pub background_normal: Option<Paint>,
    pub background_hover: Option<Paint>,
    pub background_focus: Option<Paint>,

    pub border_normal: Option<Paint>,
    pub border_hover: Option<Paint>,
    pub border_focus: Option<Paint>,
}

impl TextBoxStyle {
    pub fn default_dark() -> Self {
        TextBoxStyle {
            border_width: 0.,
            corner_radius: [5.; 4],
            margin: Sides2::new(5., 10., 5., 10.),
            height: 30.,
            text: Paint::Solid(rgb(230, 230, 230)),

            background_normal: Paint::Solid(rgb(26, 26, 26)).into(),
            background_hover: Paint::Solid(rgb(30, 30, 30)).into(),
            background_focus: Paint::Solid(rgb(22, 22, 22)).into(),

            border_normal: None,
            border_hover: None,
            border_focus: None,
        }
    }

    pub fn default_dark_invalid() -> Self {
        TextBoxStyle {
            border_width: 2.,

            border_normal: Paint::Solid(rgb(255, 50, 50)).into(),
            border_hover: Paint::Solid(rgb(255, 50, 50)).into(),
            border_focus: Paint::Solid(rgb(255, 50, 50)).into(),

            ..Self::default_dark()
        }
    }

    pub fn default_dark_disabled() -> Self {
        TextBoxStyle {
            text: Paint::Solid(rgb(100, 100, 100)),

            ..Self::default_dark()
        }
    }
}

pub fn text_box() -> TextBoxBuilder {
    TextBoxBuilder::default()
}

pub struct FloatBoxBuilder {
    style: TextBoxStyle,
    value: f64,
    valid: bool,
    disabled: bool,
    min: f64,
    max: f64,
    on_change: Option<Box<dyn Fn(f64)>>,
}

impl Default for FloatBoxBuilder {
    fn default() -> Self {
        FloatBoxBuilder {
            style: TextBoxStyle::default_dark(),
            value: 0.,
            valid: true,
            disabled: false,
            min: std::f64::MIN,
            max: std::f64::MAX,
            on_change: None,
        }
    }
}

impl ToNode for FloatBoxBuilder {
    #[ui]
    fn to_node(mut self) -> Node {
        let on_change = self.on_change.take();
        let min = self.min;
        let max = self.max;
        let value = self.value;

        let text = use_state(String::new);

        text.with(|text| match text.parse::<f64>().ok() {
            Some(num) => {
                if num > max {
                    *text = max.to_string();
                } else if num < min {
                    *text = min.to_string();
                }

                if (num - value).abs() > std::f64::EPSILON {
                    *text = value.to_string()
                }
            }
            None => *text = value.to_string(),
        });

        column()
            .child(
                text_box()
                    .style(self.style)
                    .value(text.get())
                    .on_change(move |val| {
                        if val.contains(|c: char| c != '.' && !c.is_numeric())
                            || val.chars().filter(|&x| x == '.').count() > 1
                        {
                            return;
                        }

                        text.set(val);

                        let num = text.with(|x| x.parse::<f64>().unwrap().clamp(min, max));

                        if (num - value).abs() > std::f64::EPSILON {
                            if let Some(on_change) = &on_change {
                                on_change(num);
                            }
                        }
                    }),
            )
            .to_node()
    }
}

impl FloatBoxBuilder {
    pub fn style(mut self, style: TextBoxStyle) -> Self {
        self.style = style;
        self
    }

    pub fn value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    pub fn valid(mut self, valid: bool) -> Self {
        self.valid = valid;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn min(mut self, min: f64) -> Self {
        self.min = min;
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }

    pub fn on_change(mut self, f: impl Fn(f64) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn state(self, state: impl Accessor<f64>) -> Self {
        self.value(state.get()).on_change(move |val| state.set(val))
    }
}

pub fn float_box() -> FloatBoxBuilder {
    FloatBoxBuilder::default()
}
