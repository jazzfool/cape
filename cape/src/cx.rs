use crate::{call, id::Id, node::Resources};
use fxhash::FxHashMap;
use std::{
    any::{Any, TypeId},
    collections::hash_map::Entry,
    marker::PhantomData,
};

pub struct Handle<T, M: PartialEq + Copy>(TypeId, M, PhantomData<T>);

impl<T, M: PartialEq + Copy> Clone for Handle<T, M> {
    fn clone(&self) -> Self {
        Handle(self.0, self.1, Default::default())
    }
}

impl<T, M: PartialEq + Copy> PartialEq for Handle<T, M> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl<T, M: PartialEq + Copy> Copy for Handle<T, M> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct State(Id);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Cache(Id);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StaticState;

pub struct Cx {
    state: FxHashMap<(TypeId, Id), Box<dyn Any>>,
    cache: FxHashMap<(TypeId, Id), Cached>,
    statics: FxHashMap<TypeId, Box<dyn Any>>,
    on_render: Option<FxHashMap<Id, Box<dyn FnMut(&mut Cx, &mut Resources)>>>,
    on_lifecycle: FxHashMap<Id, (bool, bool, Box<dyn FnMut(&mut Cx, Lifecycle, &Resources)>)>,
    events: FxHashMap<TypeId, Box<dyn Any>>,
}

impl Default for Cx {
    fn default() -> Self {
        Cx {
            state: Default::default(),
            cache: Default::default(),
            statics: Default::default(),
            on_render: Some(Default::default()),
            on_lifecycle: Default::default(),
            events: Default::default(),
        }
    }
}

impl Cx {
    pub fn new() -> Self {
        Default::default()
    }

    #[track_caller]
    pub fn state<T: 'static>(&mut self, init: impl FnOnce() -> T) -> Handle<T, State> {
        call(move || {
            let key = (TypeId::of::<T>(), Id::current());
            self.state.entry(key).or_insert_with(|| Box::new(init()));
            Handle(key.0, State(key.1), Default::default())
        })
    }

    pub fn at<T: 'static>(&mut self, handle: Handle<T, State>) -> &mut T {
        self.state
            .get_mut(&(handle.0, handle.1 .0))
            .unwrap()
            .downcast_mut()
            .unwrap()
    }

    #[track_caller]
    pub fn static_state<T: 'static>(&mut self, init: impl FnOnce() -> T) -> Handle<T, StaticState> {
        let key = TypeId::of::<T>();
        self.statics.entry(key).or_insert_with(|| Box::new(init()));
        Handle(key, StaticState, Default::default())
    }

    pub fn static_at<T: 'static>(&mut self, handle: Handle<T, StaticState>) -> &mut T {
        self.statics
            .get_mut(&handle.0)
            .unwrap()
            .downcast_mut()
            .unwrap()
    }

    #[track_caller]
    pub fn cache<T: 'static, U: PartialEq + Clone + 'static>(
        &mut self,
        arg: &U,
        f: impl FnOnce(&U) -> T,
    ) -> Handle<T, Cache> {
        call(move || {
            let key = (TypeId::of::<T>(), Id::current());

            match self.cache.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(Cached {
                        value: Box::new(f(arg)),
                        arg: Box::new(arg.clone()),
                    });
                }
                Entry::Occupied(mut entry) => {
                    let entry_arg = entry.get_mut().arg.downcast_mut::<U>().unwrap();
                    if entry_arg != arg {
                        *entry_arg = arg.clone();
                        entry.get_mut().value = Box::new(f(arg));
                    }
                }
            }

            Handle(key.0, Cache(key.1), Default::default())
        })
    }

    pub fn cache_at<T: 'static>(&self, handle: Handle<T, Cache>) -> &T {
        self.cache
            .get(&(handle.0, handle.1 .0))
            .unwrap()
            .value
            .downcast_ref::<T>()
            .unwrap()
    }

    #[track_caller]
    pub fn lazy<Arg: PartialEq + Clone + 'static, Out: Clone + 'static>(
        &mut self,
        arg: Handle<Arg, State>,
        f: impl FnOnce(&mut Cx) -> Out,
    ) -> Out {
        call(move || {
            let key = (TypeId::of::<Out>(), Id::current());
            match self.cache.entry(key) {
                Entry::Occupied(entry) => {
                    let arg = self
                        .state
                        .get(&(arg.0, arg.1 .0))
                        .unwrap()
                        .downcast_ref::<Arg>()
                        .unwrap();
                    if entry.get().arg.downcast_ref::<Arg>().unwrap() == arg {
                        entry.get().value.downcast_ref::<Out>().cloned().unwrap()
                    } else {
                        let arg = arg.clone();
                        let v = f(self);
                        let entry = self.cache.get_mut(&key).unwrap();
                        entry.value = Box::new(v.clone());
                        entry.arg = Box::new(arg);
                        v
                    }
                }
                Entry::Vacant(_) => {
                    let v = f(self);
                    let arg = self
                        .state
                        .get(&(arg.0, arg.1 .0))
                        .unwrap()
                        .downcast_ref::<Arg>()
                        .unwrap()
                        .to_owned();
                    self.cache.insert(
                        key,
                        Cached {
                            value: Box::new(v.clone()),
                            arg: Box::new(arg),
                        },
                    );
                    v
                }
            }
        })
    }

    #[track_caller]
    pub fn on_render(&mut self, f: impl FnMut(&mut Cx, &mut Resources) + 'static) {
        self.on_render
            .as_mut()
            .unwrap()
            .entry(Id::current())
            .or_insert_with(|| Box::new(f));
    }

    pub fn invoke_on_render(&mut self, resources: &mut Resources) {
        if let Some(mut on_render) = self.on_render.take() {
            for on_render in on_render.values_mut() {
                on_render(self, resources);
            }
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lifecycle {
    Create,
    Destroy,
}
