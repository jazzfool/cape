use crate::node::Resources;
use fxhash::FxHashMap;
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use crate as cape;
use crate::id::Id;

type StoreId = Id;
type StateValue = RefCell<Box<dyn Any>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventListener(usize);

impl Default for EventListener {
    fn default() -> Self {
        EventListener::null()
    }
}

impl EventListener {
    pub fn null() -> Self {
        EventListener(std::usize::MAX)
    }
}

thread_local! {
    static STATE_STORE: RefCell<FxHashMap<(TypeId, StoreId), StateValue>> = RefCell::new(Default::default());
    static CACHE_STORE: RefCell<FxHashMap<(TypeId, StoreId), RefCell<Cached>>> = RefCell::new(Default::default());
    static STATIC_STORE: RefCell<FxHashMap<TypeId, StateValue>> = RefCell::new(Default::default());
    static ON_RENDER_STORE: RefCell<FxHashMap<StoreId, Box<dyn FnMut(&Resources)>>> = RefCell::new(Default::default());
    static ON_LIFECYCLE_STORE: RefCell<FxHashMap<StoreId, (bool, bool, Box<dyn FnMut(Lifecycle, &Resources)>)>> = RefCell::new(Default::default());
    static EVENT_STORE: RefCell<FxHashMap<TypeId, Box<dyn Any>>> = RefCell::new(Default::default());
}

struct Cached {
    value: Box<dyn Any>,
    arg: Box<dyn Any>,
}

impl Cached {
    fn update<Arg: PartialEq + ToOwned + 'static, T: 'static>(
        &mut self,
        arg: &Arg,
        init: impl FnOnce(&Arg) -> T,
    ) {
        if self.arg.downcast_ref::<Arg>().unwrap() != arg {
            self.value = Box::new(init(arg));
            self.arg = Box::new(arg.to_owned());
        }
    }
}

pub trait Accessor<T: 'static>: 'static {
    fn set(&self, val: T);
    fn get(&self) -> T
    where
        T: Clone;
    fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R;
}

pub struct StateAccessor<T: 'static> {
    type_id: TypeId,
    id: StoreId,
    phantom: std::marker::PhantomData<T>,
}

impl<T: 'static> std::fmt::Debug for StateAccessor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateAccessor")
            .field("type_id", &self.type_id)
            .field("id", &self.id)
            .finish()
    }
}

impl<T: 'static> Eq for StateAccessor<T> {}

impl<T: 'static> PartialEq for StateAccessor<T> {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id && self.id == other.id
    }
}

impl<T: 'static> Hash for StateAccessor<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_id.hash(state);
        self.id.hash(state);
    }
}

impl<T: 'static> Clone for StateAccessor<T> {
    fn clone(&self) -> Self {
        StateAccessor {
            type_id: self.type_id,
            id: self.id,
            phantom: Default::default(),
        }
    }
}

impl<T: 'static> Copy for StateAccessor<T> {}

impl<T: 'static> Accessor<T> for StateAccessor<T> {
    fn set(&self, val: T) {
        STATE_STORE.with(|store| {
            store
                .borrow_mut()
                .insert((self.type_id, self.id), RefCell::new(Box::new(val)))
        });
    }

    fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        STATE_STORE.with(|store| {
            f(store
                .borrow()
                .get(&(self.type_id, self.id))
                .unwrap()
                .borrow_mut()
                .downcast_mut::<T>()
                .unwrap())
        })
    }

    fn get(&self) -> T
    where
        T: Clone,
    {
        STATE_STORE.with(|store| {
            store.borrow()[&(self.type_id, self.id)]
                .borrow()
                .downcast_ref::<T>()
                .unwrap()
                .clone()
        })
    }
}

pub struct StaticAccessor<T: 'static> {
    type_id: TypeId,
    phantom: std::marker::PhantomData<T>,
}

impl<T: 'static> std::fmt::Debug for StaticAccessor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticAccessor")
            .field("type_id", &self.type_id)
            .finish()
    }
}

impl<T: 'static> Eq for StaticAccessor<T> {}

impl<T: 'static> PartialEq for StaticAccessor<T> {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

impl<T: 'static> Hash for StaticAccessor<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_id.hash(state);
    }
}

impl<T: 'static> Clone for StaticAccessor<T> {
    fn clone(&self) -> Self {
        StaticAccessor {
            type_id: self.type_id,
            phantom: Default::default(),
        }
    }
}

impl<T: 'static> Copy for StaticAccessor<T> {}

impl<T: 'static> Accessor<T> for StaticAccessor<T> {
    fn set(&self, val: T) {
        STATIC_STORE.with(|store| {
            store
                .borrow_mut()
                .insert(self.type_id, RefCell::new(Box::new(val)))
        });
    }

    fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        STATIC_STORE.with(|store| {
            f(store
                .borrow()
                .get(&self.type_id)
                .unwrap()
                .borrow_mut()
                .downcast_mut::<T>()
                .unwrap())
        })
    }

    fn get(&self) -> T
    where
        T: Clone,
    {
        STATIC_STORE.with(|store| {
            store.borrow()[&self.type_id]
                .borrow()
                .downcast_ref::<T>()
                .unwrap()
                .clone()
        })
    }
}

pub struct CacheAccessor<T: 'static> {
    type_id: TypeId,
    id: StoreId,
    phantom: std::marker::PhantomData<T>,
}

impl<T: 'static> std::fmt::Debug for CacheAccessor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheAccessor")
            .field("type_id", &self.type_id)
            .field("id", &self.id)
            .finish()
    }
}

impl<T: 'static> Eq for CacheAccessor<T> {}

impl<T: 'static> PartialEq for CacheAccessor<T> {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id && self.id == other.id
    }
}

impl<T: 'static> Hash for CacheAccessor<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_id.hash(state);
        self.id.hash(state);
    }
}

impl<T: 'static> Clone for CacheAccessor<T> {
    fn clone(&self) -> Self {
        CacheAccessor {
            type_id: self.type_id,
            id: self.id,
            phantom: Default::default(),
        }
    }
}

impl<T: 'static> Copy for CacheAccessor<T> {}

impl<T: 'static> CacheAccessor<T> {
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        CACHE_STORE.with(|store| {
            f(store
                .borrow()
                .get(&(self.type_id, self.id))
                .unwrap()
                .borrow()
                .value
                .downcast_ref::<T>()
                .unwrap())
        })
    }

    pub fn get(&self) -> T
    where
        T: Clone,
    {
        CACHE_STORE.with(|store| {
            store
                .borrow_mut()
                .get(&(self.type_id, self.id))
                .unwrap()
                .borrow()
                .value
                .downcast_ref::<T>()
                .unwrap()
                .clone()
        })
    }
}

pub struct EventAccessor<T: 'static> {
    type_id: TypeId,
    phantom: PhantomData<T>,
}

impl<T: 'static> std::fmt::Debug for EventAccessor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventAccessor")
            .field("type_id", &self.type_id)
            .finish()
    }
}

impl<T: 'static> Eq for EventAccessor<T> {}

impl<T: 'static> PartialEq for EventAccessor<T> {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

impl<T: 'static> Hash for EventAccessor<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_id.hash(state);
    }
}

impl<T: 'static> Clone for EventAccessor<T> {
    fn clone(&self) -> Self {
        EventAccessor {
            type_id: self.type_id,
            phantom: Default::default(),
        }
    }
}

impl<T: 'static> Copy for EventAccessor<T> {}

impl<T: 'static> EventAccessor<T> {
    pub fn connect(&self, listener: impl FnMut(&T) + 'static) -> EventListener {
        EVENT_STORE.with(|store| {
            let mut b = store.borrow_mut();
            let v = b
                .entry(self.type_id)
                .or_insert_with(|| Box::new(Vec::<Option<Box<dyn FnMut(&T)>>>::new()))
                .downcast_mut::<Vec<Option<Box<dyn FnMut(&T)>>>>()
                .unwrap();
            v.push(Some(Box::new(listener)));
            EventListener(v.len() - 1)
        })
    }

    pub fn disconnect(&self, listener: EventListener) {
        EVENT_STORE.with(|store| {
            store
                .borrow_mut()
                .entry(self.type_id)
                .or_insert_with(|| Box::new(Vec::<Option<Box<dyn FnMut(&T)>>>::new()))
                .downcast_mut::<Vec<Option<Box<dyn FnMut(&T)>>>>()
                .unwrap()
                .remove(listener.0);
        });
    }

    pub fn emit(&self, event: &T) {
        EVENT_STORE.with(|store| {
            store
                .borrow_mut()
                .entry(self.type_id)
                .or_insert_with(|| Box::new(Vec::<Option<Box<dyn FnMut(&T)>>>::new()))
                .downcast_mut::<Vec<Option<Box<dyn FnMut(&T)>>>>()
                .unwrap()
                .iter_mut()
                .for_each(|listener| {
                    if let Some(listener) = listener {
                        (*listener)(event);
                    }
                });
        });
    }
}

#[crate::ui]
pub fn use_state<T: 'static>(init: impl FnOnce() -> T) -> StateAccessor<T> {
    let acc = StateAccessor {
        type_id: TypeId::of::<T>(),
        id: Id::current(),
        phantom: Default::default(),
    };

    STATE_STORE.with(|store| {
        let mut store = store.borrow_mut();
        if !store.contains_key(&(acc.type_id, acc.id)) {
            store.insert((acc.type_id, acc.id), RefCell::new(Box::new(init())));
        }
    });

    acc
}

#[crate::ui]
pub fn use_cache<Arg: PartialEq + ToOwned + 'static, T: 'static>(
    arg: &Arg,
    init: impl FnOnce(&Arg) -> T,
) -> CacheAccessor<T> {
    let acc = CacheAccessor {
        type_id: TypeId::of::<T>(),
        id: Id::current(),
        phantom: Default::default(),
    };

    CACHE_STORE.with(|store| {
        if !store.borrow().contains_key(&(acc.type_id, acc.id)) {
            store.borrow_mut().insert(
                (acc.type_id, acc.id),
                RefCell::new(Cached {
                    value: Box::new(init(arg)),
                    arg: Box::new(arg.to_owned()),
                }),
            );
        } else {
            store
                .borrow()
                .get(&(acc.type_id, acc.id))
                .unwrap()
                .borrow_mut()
                .update(arg, init);
        }
    });

    acc
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lifecycle {
    Create,
    Destroy,
}

pub fn use_static<T: 'static>(init: impl FnOnce() -> T) -> StaticAccessor<T> {
    let acc = StaticAccessor {
        type_id: TypeId::of::<T>(),
        phantom: Default::default(),
    };

    STATIC_STORE.with(|store| {
        let mut store = store.borrow_mut();
        if !store.contains_key(&acc.type_id) {
            store.insert(acc.type_id, RefCell::new(Box::new(init())));
        }
    });

    acc
}

#[crate::ui]
pub fn on_render(f: impl FnMut(&Resources) + 'static) {
    let id = Id::current();
    ON_RENDER_STORE.with(|store| {
        let mut store = store.borrow_mut();
        if !store.contains_key(&id) || store.get(&id).unwrap().type_id() != f.type_id() {
            store.insert(id, Box::new(f));
        }
    });
}

#[crate::ui]
pub fn on_lifecycle(f: impl FnMut(Lifecycle, &Resources) + 'static) {
    let id = Id::current();
    ON_LIFECYCLE_STORE.with(|store| {
        let mut store = store.borrow_mut();

        if let Some((alive, _, _)) = store.get_mut(&id) {
            *alive = true;
        } else {
            store.insert(id, (true, true, Box::new(f)));
        }
    })
}

pub fn call_on_renders(resources: &Resources) {
    ON_RENDER_STORE.with(|store| {
        for on_render in store.borrow_mut().values_mut() {
            on_render(resources);
        }
    });
}

pub fn call_on_lifecycles(resources: &Resources) {
    let mut destroys = Vec::new();
    ON_LIFECYCLE_STORE.with(|store| {
        let mut store = store.borrow_mut();

        for (id, (alive, new, on_lifecycle)) in store.iter_mut() {
            if !*alive {
                destroys.push(*id);
                on_lifecycle(Lifecycle::Destroy, resources);
            } else {
                if *new {
                    *new = false;
                    on_lifecycle(Lifecycle::Create, resources);
                }

                *alive = false;
            }
        }

        for destroy in &destroys {
            store.remove(destroy);
        }
    });
}

pub fn use_event<T: 'static>() -> EventAccessor<T> {
    let acc = EventAccessor {
        type_id: TypeId::of::<T>(),
        phantom: Default::default(),
    };

    EVENT_STORE.with(|store| {
        store
            .borrow_mut()
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(Vec::<Option<Box<dyn FnMut(&T)>>>::new()));
    });

    acc
}
