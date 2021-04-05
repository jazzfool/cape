use crate::{deco, Callback, Stack};
use cape::node::IntoNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ComboBoxCentreState {
    Normal,
    Hovered,
    Opened,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ComboBoxItemState {
    Normal,
    Hovered,
}

#[derive(Default)]
pub struct ComboBox {
    pub values: Vec<String>,
    pub selected: usize,
    pub on_change: Callback<(String, usize)>,
    pub centre_decorator: deco::DecoratorHook<ComboBoxCentreState>,
    pub popup_decorator: deco::DecoratorHook<()>,
    pub item_decorator: deco::DecoratorHook<ComboBoxItemState>,
}

impl IntoNode for ComboBox {
    #[cape::ui]
    fn into_node(self) -> cape::node::Node {
        todo!()
    }
}

impl ComboBox {
    pub fn values(mut self, values: impl IntoIterator<Item = String>) -> Self {
        self.values = values.into_iter().collect();
        self
    }

    pub fn selected(mut self, selected: usize) -> Self {
        self.selected = selected;
        self
    }

    pub fn on_change(mut self, on_change: impl Into<Callback<(String, usize)>>) -> Self {
        self.on_change = on_change.into();
        self
    }

    pub fn centre_decorator(
        mut self,
        decorator: impl Into<deco::DecoratorHook<ComboBoxCentreState>>,
    ) -> Self {
        self.centre_decorator = decorator.into();
        self
    }

    pub fn popup_decorator(mut self, decorator: impl Into<deco::DecoratorHook<()>>) -> Self {
        self.popup_decorator = decorator.into();
        self
    }

    pub fn item_decorator(
        mut self,
        decorator: impl Into<deco::DecoratorHook<ComboBoxItemState>>,
    ) -> Self {
        self.item_decorator = decorator.into();
        self
    }
}

pub fn combo_box() -> ComboBox {
    ComboBox::default()
}
