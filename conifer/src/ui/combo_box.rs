use crate::Callback;
use cape::node::IntoNode;

#[derive(Default)]
pub struct ComboBox {
    pub values: Vec<String>,
    pub selected: usize,
    pub on_change: Callback<(String, usize)>,
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
}
