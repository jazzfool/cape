pub mod air;

use cape::state::Accessor;

pub type Palette = std::collections::HashMap<String, cape::node::Paint>;

pub enum Paint {
    Palette(String),
    Owned(cape::node::Paint),
}

impl Paint {
    pub fn resolve(self) -> Option<cape::node::Paint> {
        match self {
            Paint::Palette(paint) => {
                cape::state::use_static(Palette::default).with(|pal| pal.get(&paint).cloned())
            }
            Paint::Owned(paint) => Some(paint),
        }
    }
}
