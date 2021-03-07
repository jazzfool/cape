use crate::*;
use cape::{
    node::{interact, Interaction, MouseButton, Node, ToNode, ZOrder},
    ui::{self, NodeLayout},
};

pub type Button<T> = ui::Interact<ContainerLayout<(T,)>>;

pub struct ButtonProps<T: ui::Merge + ui::Expand> {
    pub child: T,
    pub padding: cape::Sides2,
    pub on_click: Box<dyn Fn(&Interaction)>,
    pub disabled: bool,
    pub z_order: ZOrder,
}

impl<T: Default + ui::Merge + ui::Expand> Default for ButtonProps<T> {
    fn default() -> Self {
        ButtonProps {
            child: Default::default(),
            padding: Default::default(),
            on_click: Box::new(|_| {}),
            disabled: false,
            z_order: Default::default(),
        }
    }
}

impl<T: Default + ui::Merge + ui::Expand> Props<Button<T>> for ButtonProps<T> {
    #[cape::ui]
    fn build(self) -> Button<T> {
        let on_click = self.on_click;

        ui::Interact::new(
            Container::default().children((self.child,)),
            move |e| match e {
                Interaction::MouseDown {
                    button: MouseButton::Left,
                    ..
                } => {
                    on_click(e);
                }
                _ => {}
            },
            self.z_order,
            false,
        )
    }
}

impl<T: ui::Merge + ui::Expand> ButtonProps<T> {
    pub fn child(mut self, child: impl Into<T>) -> Self {
        self.child = child.into();
        self
    }

    pub fn padding(mut self, padding: cape::Sides2) -> Self {
        self.padding = padding;
        self
    }

    pub fn on_click(mut self, f: impl Fn(&Interaction) + 'static) -> Self {
        self.on_click = Box::new(f);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

pub fn button<T: Default + ui::Merge + ui::Expand>(props: ButtonProps<T>) -> Button<T> {
    props.build()
}
