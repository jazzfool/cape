use crate::StackItem;
use cape::{
    node::{interact, rectangle, styled_text, Interaction, KeyCode, Node, Paint, ToNode},
    rgba, size2,
    state::{use_state, Accessor},
    ui, Sides2,
};

pub struct TextBox {
    pub selection_paint: Paint,
    pub selection_corner_radii: [f32; 4],
    pub font: String,
    pub font_size: Option<f32>,
    pub font_fill: Option<Paint>,
    pub padding: Sides2,
    pub width: f32,
    pub value: String,
    pub disabled: bool,
    pub on_change: Option<Box<dyn Fn(String)>>,
}

impl Default for TextBox {
    fn default() -> Self {
        TextBox {
            selection_paint: Paint::Solid(rgba(0, 0, 255, 150)),
            selection_corner_radii: [5.0; 4],
            font: String::from("sans-serif"),
            font_size: None,
            font_fill: None,
            padding: Sides2::default(),
            width: 150.,
            value: String::new(),
            disabled: false,
            on_change: None,
        }
    }
}

impl ToNode for TextBox {
    #[ui]
    fn to_node(self) -> Node {
        let on_change = self.on_change;
        let value = self.value;

        interact(
            stack().width(self.width).child_item(
                container().margin(self.padding).child(styled_text(
                    &value,
                    self.font,
                    self.font_size,
                    self.font_fill,
                )),
                StackItem::fill(),
            ),
            move |event| match event {
                Interaction::ReceiveCharacter { character } => {
                    if !character.is_control() && !character.is_ascii_control() {
                        if let Some(on_change) = &on_change {
                            on_change(format!("{}{}", value.clone(), character));
                        }
                    }
                }
                Interaction::KeyDown { key_code, .. } => match key_code {
                    KeyCode::Back => {
                        if !value.is_empty() {
                            if let Some(on_change) = &on_change {
                                on_change(value[0..value.len() - 1].to_string());
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

impl TextBox {
    pub fn selection_paint(mut self, paint: impl Into<Paint>) -> Self {
        self.selection_paint = paint.into();
        self
    }

    pub fn selection_corner_radii(mut self, radii: [f32; 4]) -> Self {
        self.selection_corner_radii = radii;
        self
    }

    pub fn font(mut self, font: impl ToString) -> Self {
        self.font = font.to_string();
        self
    }

    pub fn font_size(mut self, size: impl Into<Option<f32>>) -> Self {
        self.font_size = size.into();
        self
    }

    pub fn font_fill(mut self, fill: impl Into<Option<Paint>>) -> Self {
        self.font_fill = fill.into();
        self
    }

    pub fn padding(mut self, padding: Sides2) -> Self {
        self.padding = padding;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
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

pub fn text_box() -> TextBox {
    TextBox::default()
}

pub struct FloatBox {
    pub selection_paint: Paint,
    pub selection_corner_radii: [f32; 4],
    pub font: String,
    pub font_size: Option<f32>,
    pub font_fill: Option<Paint>,
    pub padding: Sides2,
    pub width: f32,
    pub value: f64,
    pub valid: bool,
    pub disabled: bool,
    pub min: f64,
    pub max: f64,
    pub on_change: Option<Box<dyn Fn(f64)>>,
}

impl Default for FloatBox {
    fn default() -> Self {
        FloatBox {
            selection_paint: Paint::Solid(rgba(0, 0, 255, 150)),
            selection_corner_radii: [5.0; 4],
            font: String::from("sans-serif"),
            font_size: None,
            font_fill: None,
            padding: Sides2::default(),
            width: 150.,
            value: 0.,
            valid: true,
            disabled: false,
            min: std::f64::MIN,
            max: std::f64::MAX,
            on_change: None,
        }
    }
}

impl ToNode for FloatBox {
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
                    .selection_paint(self.selection_paint)
                    .selection_corner_radii(self.selection_corner_radii)
                    .font(self.font)
                    .font_size(self.font_size)
                    .font_fill(self.font_fill)
                    .padding(self.padding)
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

impl FloatBox {
    pub fn selection_paint(mut self, paint: impl Into<Paint>) -> Self {
        self.selection_paint = paint.into();
        self
    }

    pub fn selection_corner_radii(mut self, radii: [f32; 4]) -> Self {
        self.selection_corner_radii = radii;
        self
    }

    pub fn font(mut self, font: impl ToString) -> Self {
        self.font = font.to_string();
        self
    }

    pub fn font_size(mut self, size: impl Into<Option<f32>>) -> Self {
        self.font_size = size.into();
        self
    }

    pub fn font_fill(mut self, fill: impl Into<Option<Paint>>) -> Self {
        self.font_fill = fill.into();
        self
    }

    pub fn padding(mut self, padding: Sides2) -> Self {
        self.padding = padding;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
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

pub fn float_box() -> FloatBox {
    FloatBox::default()
}
