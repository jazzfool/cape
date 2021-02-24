use crate::CallId;

pub type Id = CallId;

use crate as cape;

#[crate::unique_ui]
pub fn context<K, R>(key: &K, f: impl FnOnce() -> R) -> R
where
    K: Send + std::hash::Hash + Clone + Eq + 'static,
{
    f()
}
