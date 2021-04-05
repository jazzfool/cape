use crate::{column, Callback, Container, LayoutBuilder, Row, Stack, StackItem};
use cape::{
    cx::{Cx, Handle, State},
    node::{interact, rectangle, styled_text, Interaction, IntoNode, KeyCode, Node, Paint},
    rgb, size2, ui, Sides2,
};

pub struct TextBox<'a> {
    cx: &'a mut Cx,
    style: TextBoxStyle,
    value: String,
    icon: Node,
    disabled: bool,
    on_change: Callback<String>,
}

impl<'a> IntoNode for TextBox<'a> {
    #[ui]
    fn into_node(self) -> Node {
        let value = self.value;
        let on_change = self.on_change;

        interact(
            Stack::new()
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
                    Container::new()
                        .child(Row::new().child(self.icon.clone()).child(styled_text(
                            &value,
                            "sans-serif",
                            None,
                            Some(self.style.text.clone()),
                        )))
                        .margin(self.style.margin),
                    StackItem::center(),
                ),
            move |cx, event| match event {
                Interaction::ReceiveCharacter { character } => {
                    if !character.is_control() && !character.is_ascii_control() {
                        on_change.call(cx, &format!("{}{}", value.clone(), character));
                    }
                }
                Interaction::KeyDown { key_code, .. } => match key_code {
                    KeyCode::Back => {
                        if !value.is_empty() {
                            on_change.call(cx, &value[0..value.len() - 1].to_string());
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

impl<'a> TextBox<'a> {
    pub fn new(cx: &'a mut Cx) -> Self {
        TextBox {
            cx,
            style: TextBoxStyle::default_dark(),
            value: String::new(),
            icon: Node::Null,
            disabled: false,
            on_change: Default::default(),
        }
    }

    pub fn style(mut self, style: TextBoxStyle) -> Self {
        self.style = style;
        self
    }

    pub fn value<T: ToString>(mut self, value: impl FnOnce(&mut Cx) -> T) -> Self {
        self.value = value(self.cx).to_string();
        self
    }

    pub fn icon(mut self, icon: impl IntoNode) -> Self {
        self.icon = icon.into_node();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, f: impl Into<Callback<String>>) -> Self {
        self.on_change = f.into();
        self
    }

    pub fn state(self, state: Handle<String, State>) -> Self {
        self.value(|cx| cx.at(state).clone())
            .on_change(move |cx: &mut Cx, val: &String| *cx.at(state) = val.clone())
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

pub struct FloatBox<'a> {
    cx: &'a mut Cx,
    style: TextBoxStyle,
    value: f64,
    valid: bool,
    disabled: bool,
    min: f64,
    max: f64,
    on_change: Callback<f64>,
}

impl<'a> IntoNode for FloatBox<'a> {
    #[ui]
    fn into_node(mut self) -> Node {
        let on_change = self.on_change.take();
        let min = self.min;
        let max = self.max;
        let value = self.value;

        let text = self.cx.state(String::new);

        match self.cx.at(text).parse::<f64>().ok() {
            Some(num) => {
                if num > max {
                    *self.cx.at(text) = max.to_string();
                } else if num < min {
                    *self.cx.at(text) = min.to_string();
                }

                if (num - value).abs() > std::f64::EPSILON {
                    *self.cx.at(text) = value.to_string()
                }
            }
            None => *self.cx.at(text) = value.to_string(),
        }

        column()
            .child(
                TextBox::new(self.cx)
                    .style(self.style)
                    .value(|cx| cx.at(text).clone())
                    .on_change(move |cx: &mut Cx, val: &String| {
                        if val.contains(|c: char| c != '.' && !c.is_numeric())
                            || val.chars().filter(|&x| x == '.').count() > 1
                        {
                            return;
                        }

                        *cx.at(text) = val.clone();

                        let num = cx.at(text).parse::<f64>().unwrap().clamp(min, max);

                        if (num - value).abs() > std::f64::EPSILON {
                            on_change.call(cx, &num);
                        }
                    }),
            )
            .into_node()
    }
}

impl<'a> FloatBox<'a> {
    pub fn new(cx: &'a mut Cx) -> Self {
        FloatBox {
            cx,
            style: TextBoxStyle::default_dark(),
            value: 0.,
            valid: true,
            disabled: false,
            min: std::f64::MIN,
            max: std::f64::MAX,
            on_change: Default::default(),
        }
    }

    pub fn style(mut self, style: TextBoxStyle) -> Self {
        self.style = style;
        self
    }

    pub fn value(mut self, value: impl FnOnce(&mut Cx) -> f64) -> Self {
        self.value = value(self.cx);
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

    pub fn on_change(mut self, on_change: impl Into<Callback<f64>>) -> Self {
        self.on_change = on_change.into();
        self
    }

    pub fn state(self, state: Handle<f64, State>) -> Self {
        self.value(|cx| *cx.at(state))
            .on_change(move |cx: &mut Cx, val: &f64| *cx.at(state) = *val)
    }
}
