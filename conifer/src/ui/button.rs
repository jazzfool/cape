use crate::{container, LayoutBuilder};
use cape::node::{interact, Interaction, Node, ToNode};

pub struct ButtonBuilder {
    child: Node,
    padding: cape::Sides2,
    on_click: Vec<Box<dyn Fn(&Interaction)>>,
    disabled: bool,
}

impl Default for ButtonBuilder {
    fn default() -> Self {
        ButtonBuilder {
            child: Default::default(),
            padding: Default::default(),
            on_click: Vec::new(),
            disabled: false,
        }
    }
}

impl ToNode for ButtonBuilder {
    #[cape::ui]
    fn to_node(self) -> Node {
        let on_click = self.on_click;

        interact(
            container().child(self.child).margin(self.padding),
            move |event| {
                if event.is_mouse_down() {
                    for e in &on_click {
                        e(event);
                    }
                }
            },
            false,
        )
    }
}

impl ButtonBuilder {
    pub fn child(mut self, child: impl ToNode) -> Self {
        self.child = child.to_node();
        self
    }

    pub fn padding(mut self, padding: cape::Sides2) -> Self {
        self.padding = padding;
        self
    }

    pub fn on_click(mut self, f: impl Fn(&Interaction) + 'static) -> Self {
        self.on_click.push(Box::new(f));
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

pub fn button() -> ButtonBuilder {
    ButtonBuilder::default()
}
