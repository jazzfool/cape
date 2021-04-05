use crate::{Callback, Container, LayoutBuilder, Stack, StackItem};
use cape::{
    cx::{Cx, Handle, State},
    node::{interact, Interaction, IntoNode, MouseButton, Node},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
    Disabled,
}

pub struct Button<'a> {
    pub cx: &'a mut Cx,

    pub child: Node,
    pub padding: cape::Sides2,
    pub on_click: Callback<Interaction>,
    pub disabled: bool,
    pub background: Option<Node>,

    hovered: Handle<bool, State>,
    pressed: Handle<bool, State>,
    focused: Handle<bool, State>,
}

impl<'a> IntoNode for Button<'a> {
    fn into_node(self) -> Node {
        let on_click = self.on_click;

        let hovered = self.hovered;
        let pressed = self.pressed;
        let focused = self.focused;

        interact(
            Stack::new()
                .child_item(self.background.unwrap_or(Node::Null), StackItem::fill())
                .child(Container::new().child(self.child).margin(self.padding)),
            move |cx, event| match event {
                Interaction::MouseDown {
                    button: MouseButton::Left,
                    ..
                } => {
                    *cx.at(pressed) = true;
                    on_click.call(cx, event);
                }
                Interaction::MouseUp {
                    button: MouseButton::Left,
                    ..
                } => *cx.at(pressed) = false,
                Interaction::CursorEnter { .. } => *cx.at(hovered) = true,
                Interaction::CursorExit { .. } => *cx.at(hovered) = false,
                Interaction::GainFocus => *cx.at(focused) = true,
                Interaction::LoseFocus => *cx.at(focused) = false,
                _ => {}
            },
            false,
        )
        .into_node()
    }
}

impl<'a> Button<'a> {
    #[cape::ui]
    pub fn new(cx: &'a mut Cx) -> Self {
        let hovered = cx.state(|| false);
        let pressed = cx.state(|| false);
        let focused = cx.state(|| false);

        Button {
            child: Default::default(),
            padding: Default::default(),
            on_click: Default::default(),
            disabled: false,
            background: None,

            hovered,
            pressed,
            focused,

            cx,
        }
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        self.child = child.into_node();
        self
    }

    pub fn padding(mut self, padding: cape::Sides2) -> Self {
        self.padding = padding;
        self
    }

    pub fn on_click(mut self, f: impl Into<Callback<Interaction>>) -> Self {
        self.on_click = f.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn background(mut self, background: impl Into<Option<Node>>) -> Self {
        self.background = background.into();
        self
    }

    pub fn hovered(self, hovered: &mut bool) -> Self {
        *hovered = *self.cx.at(self.hovered);
        self
    }

    pub fn pressed(self, pressed: &mut bool) -> Self {
        *pressed = *self.cx.at(self.pressed);
        self
    }

    pub fn focused(self, focused: &mut bool) -> Self {
        *focused = *self.cx.at(self.focused);
        self
    }
}
