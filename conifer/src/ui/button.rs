use crate::{
    container,
    deco::{self, Decorated},
    Callback, LayoutBuilder,
};
use cape::node::{interact, Interaction, IntoNode, Node};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
    Disabled,
}

#[derive(Default)]
pub struct Button {
    pub child: Node,
    pub padding: cape::Sides2,
    pub on_click: Callback<Interaction>,
    pub disabled: bool,
    pub decorator: deco::DecoratorHook<ButtonState>,
}

impl IntoNode for Button {
    #[cape::ui]
    fn into_node(self) -> Node {
        let on_click = self.on_click;

        interact(
            container().child(self.child).margin(self.padding),
            move |event| {
                if event.is_mouse_down() {
                    on_click.call(event);
                }
            },
            false,
        )
        .decorate()
        .apply(self.decorator, &ButtonState::Normal)
        .into_node()
    }
}

impl Button {
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

    pub fn decorator(mut self, decorator: impl Into<deco::DecoratorHook<ButtonState>>) -> Self {
        self.decorator = decorator.into();
        self
    }
}

pub fn button() -> Button {
    Button::default()
}
