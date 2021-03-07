use crate::{
    id::Id,
    node::{self, EitherNode, Interaction, Node, Paint, ResolvedNode, Resources, ZOrder},
    Error, Rect, Size2,
};
use paste::paste;
use std::{cmp::Ordering, iter::FromIterator, rc::Rc};

pub trait Merge: Sized {
    fn merge(&mut self, other: Self) {
        *self = other;
    }
}

impl Merge for () {}

macro_rules! impl_tuple_merge {
    ($($t:ident)+) => {
        #[allow(non_snake_case)]
        impl<$($t: Merge),*> Merge for ($($t,)*) {
            paste!{
                fn merge(&mut self, other: Self) {
                    let ($([<$t x>],)+) = other;
                    let ($($t,)+) = self;
                    $($t.merge([<$t x>]);)+
                }
            }
        }
    };
}

pub trait Expand {
    fn expand(&mut self, resources: &mut Resources) -> Result<Vec<EitherNode>, Error>;
}

impl Expand for () {
    fn expand(&mut self, _resources: &mut Resources) -> Result<Vec<EitherNode>, Error> {
        Ok(vec![])
    }
}

macro_rules! impl_tuple_expand {
    ($($t:ident)+) => {
        #[allow(non_snake_case)]
        impl<$($t: Expand),*> Expand for ($($t,)*) {
            paste! {
                fn expand(&mut self, resources: &mut Resources) -> Result<Vec<EitherNode>, Error> {
                    let ($($t,)+) = self;

                    let mut v = Vec::new();
                    $(v.append(&mut $t.expand(resources)?);)*
                    Ok(v)
                }
            }
        }
    };
}

pub trait LayoutChildren<Item: Clone + Sized + Merge>: Merge + Expand {
    type Out;

    fn merge(a: &mut Self::Out, b: Self::Out);
    fn expand(
        a: &mut Self::Out,
        resources: &mut Resources,
    ) -> Result<Vec<(EitherNode, Item)>, Error>;
}

pub trait IntoPair<A, B: Default> {
    fn into_pair(self) -> (A, B);
}

impl<A, B: Default> IntoPair<A, B> for (A, B) {
    fn into_pair(self) -> (A, B) {
        self
    }
}

impl<A, B: Default> IntoPair<A, B> for A {
    fn into_pair(self) -> (A, B) {
        (self, Default::default())
    }
}

#[doc(hidden)]
pub struct ChildrenWrapper<T>(T);

macro_rules! impl_layout_items_transformer {
    ($($t:ident)+) => {
        paste!{
            #[allow(non_snake_case)]
            impl<Item: Clone + Sized + Merge, $($t: Merge + Expand),*> LayoutChildren<Item> for ($($t,)*) {
                type Out = ($(($t, Item),)*);

                fn merge(a: &mut Self::Out, b: Self::Out) {
                    let ($([<$t x>],)+) = b;
                    let ($($t,)+) = a;

                    $(
                        $t.0.merge([<$t x>].0);
                        $t.1.merge([<$t x>].1);
                    )*
                }

                fn expand(a: &mut Self::Out, resources: &mut Resources) -> Result<Vec<(EitherNode, Item)>, Error> {
                    let ($($t,)+) = a;

                    Ok([
                        $(
                            $t.0.expand(resources)?.into_iter().map(|x| (x, $t.1.clone())).collect::<Vec<_>>()
                        ),*
                        ].concat())
                    }

            }

            #[allow(non_snake_case)]
            impl<Item: Default + Clone + Sized + Merge, $($t: Merge + Expand),*, $([<$t x>]: IntoPair<$t, Item>),*> From<ChildrenWrapper<($([<$t x>],)*)>> for ($(($t, Item),)*) {
                fn from(x: ChildrenWrapper<($([<$t x>],)*)>) -> Self {
                    let ($($t,)+) = x.0;

                    (
                        $($t.into_pair(),)+
                    )
                }
            }
        }
    };
}

macro_rules! impl_tuple {
    ($m:tt) => {
        $m! {A}
        $m! {A B}
        $m! {A B C}
        $m! {A B C D}
        $m! {A B C D E}
        $m! {A B C D E F}
        $m! {A B C D E F G}
        $m! {A B C D E F G H}
        $m! {A B C D E F G H I}
        $m! {A B C D E F G H I J}
        $m! {A B C D E F G H I J K}
        $m! {A B C D E F G H I J K L}
        $m! {A B C D E F G H I J K L M}
        $m! {A B C D E F G H I J K L M N}
        $m! {A B C D E F G H I J K L M N O}
        $m! {A B C D E F G H I J K L M N O P}
    };
}

impl_tuple! {impl_tuple_merge}
impl_tuple! {impl_tuple_expand}
impl_tuple! {impl_layout_items_transformer}

impl<T: Merge> Merge for Option<T> {
    fn merge(&mut self, other: Self) {
        match (self.as_mut(), other) {
            (Some(a), Some(b)) => a.merge(b),
            (None, Some(b)) => *self = Some(b),
            (Some(_), None) => *self = None,
            _ => {}
        }
    }
}

impl<T: Expand> Expand for Option<T> {
    fn expand(&mut self, resources: &mut Resources) -> Result<Vec<EitherNode>, Error> {
        if let Some(x) = self {
            x.expand(resources)
        } else {
            Ok(vec![])
        }
    }
}

pub enum Conditional<A: Merge + Expand, B: Merge + Expand> {
    A(A),
    B(B),
}

impl<A: Merge + Expand, B: Merge + Expand> Conditional<A, B> {
    pub fn new(cond: bool, a: impl FnOnce() -> A, b: impl FnOnce() -> B) -> Self {
        if cond {
            Conditional::A(a())
        } else {
            Conditional::B(b())
        }
    }
}

impl<A: Merge + Expand, B: Merge + Expand> Merge for Conditional<A, B> {
    fn merge(&mut self, other: Self) {
        match (self, other) {
            (Conditional::A(a), Conditional::A(a2)) => a.merge(a2),
            (x @ Conditional::A(_), b @ Conditional::B(_)) => *x = b,
            (Conditional::B(b), Conditional::B(b2)) => b.merge(b2),
            (x @ Conditional::B(_), a @ Conditional::A(_)) => *x = a,
        }
    }
}

impl<A: Merge + Expand, B: Merge + Expand> Expand for Conditional<A, B> {
    fn expand(&mut self, resources: &mut Resources) -> Result<Vec<EitherNode>, Error> {
        match self {
            Conditional::A(a) => a.expand(resources),
            Conditional::B(b) => b.expand(resources),
        }
    }
}

/*
#[derive(Default)]
pub struct DynamicList<T: Merge + Expand> {
    pub values: Vec<T>,
}


impl<T: Merge + Expand> FromIterator<T> for DynamicList<T> {
    fn from_iter<U: IntoIterator<Item = T>>(iter: U) -> Self {
        DynamicList {
            values: iter.into_iter().collect(),
        }
    }
}
*/

impl<T: Merge + Expand> Expand for Vec<T> {
    fn expand(&mut self, resources: &mut Resources) -> Result<Vec<EitherNode>, Error> {
        Ok(self
            .iter_mut()
            .map(|x| x.expand(resources))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .fold(Vec::new(), |mut v, mut x| {
                v.append(&mut x);
                v
            }))
    }
}

impl<T: Merge + Expand> Merge for Vec<T> {
    fn merge(&mut self, mut other: Self) {
        let len = other.len();
        let min = self.len().min(other.len());

        for (i, other) in other.drain(0..min).enumerate() {
            self[i].merge(other);
        }

        match len.cmp(&self.len()) {
            Ordering::Less => self.truncate(len),
            Ordering::Greater => self.append(&mut other),
            _ => {}
        }
    }
}

pub struct Interact<T: Merge + Expand> {
    pub child: T,
    pub callback: Rc<dyn Fn(&Interaction)>,
    pub z_order: ZOrder,
    pub passthrough: bool,
    id: Id,
}

impl<T: Merge + Expand> Interact<T> {
    #[track_caller]
    pub fn new(
        child: T,
        callback: impl Fn(&Interaction) + 'static,
        z_order: ZOrder,
        passthrough: bool,
    ) -> Self {
        Interact {
            child,
            callback: Rc::new(callback),
            z_order,
            passthrough,
            id: Id::current(),
        }
    }
}

impl<T: Merge + Expand> Merge for Interact<T> {
    fn merge(&mut self, other: Self) {
        self.child.merge(other.child);
        self.callback = other.callback;
        self.z_order = other.z_order;
        self.passthrough = other.passthrough;
        self.id = other.id;
    }
}

impl<T: Merge + Expand> Expand for Interact<T> {
    fn expand(&mut self, resources: &mut Resources) -> Result<Vec<EitherNode>, Error> {
        let child = self.child.expand(resources)?.remove(0);
        Ok(vec![EitherNode::Unresolved(Node::Interact {
            child: Box::new(child),
            callback: Rc::clone(&self.callback),
            id: self.id,
            passthrough: self.passthrough,
            z_order: self.z_order,
        })])
    }
}

pub struct Capture<T: Merge + Expand> {
    pub child: T,
    pub callback: Rc<dyn Fn(&ResolvedNode)>,
    pub z_order: ZOrder,
}

impl<T: Merge + Expand> Merge for Capture<T> {
    fn merge(&mut self, other: Self) {
        self.child.merge(other.child);
        self.callback = other.callback;
        self.z_order = other.z_order;
    }
}

impl<T: Merge + Expand> Expand for Capture<T> {
    fn expand(&mut self, resources: &mut Resources) -> Result<Vec<EitherNode>, Error> {
        let child = self.child.expand(resources)?.remove(0);
        Ok(vec![EitherNode::Unresolved(Node::Capture {
            child: Box::new(child),
            callback: Rc::clone(&self.callback),
            z_order: self.z_order,
        })])
    }
}

pub trait NodeLayout: Sized + Merge {
    type Item: Clone + Sized + Merge;

    fn node_layout(&self, items: Vec<Self::Item>) -> Rc<dyn node::Layout>;

    fn children<
        InnerChildren,
        Children: IntoPair<InnerChildren, ZOrder>,
        Inner: LayoutChildren<Self::Item>,
    >(
        self,
        children: Children,
    ) -> Layout<Self, Inner>
    where
        ChildrenWrapper<InnerChildren>: Into<Inner::Out>,
    {
        let (children, z_order) = children.into_pair();
        Layout {
            layout: self,
            children: ChildrenWrapper(children).into(),
            z_order,
        }
    }
}

#[derive(Default)]
pub struct Layout<T: NodeLayout, Children: LayoutChildren<T::Item>> {
    pub layout: T,
    pub children: Children::Out,
    pub z_order: ZOrder,
}

impl<T: NodeLayout + Merge, Children: LayoutChildren<T::Item>> Merge for Layout<T, Children> {
    fn merge(&mut self, other: Self) {
        self.layout.merge(other.layout);
        <Children as LayoutChildren<T::Item>>::merge(&mut self.children, other.children);
    }
}

impl<T: NodeLayout + Merge, Children: LayoutChildren<T::Item>> Expand for Layout<T, Children> {
    fn expand(&mut self, resources: &mut Resources) -> Result<Vec<EitherNode>, Error> {
        let (children, items) =
            <Children as LayoutChildren<T::Item>>::expand(&mut self.children, resources)?
                .into_iter()
                .unzip();

        Ok(vec![EitherNode::Unresolved(Node::Layout {
            layout: self.layout.node_layout(items),
            children,
            z_order: self.z_order,
        })])
    }
}

#[derive(Clone)]
pub struct Text {
    text: String,
    font: String,
    size: Option<f32>,
    fill: Option<Paint>,
    node: ResolvedNode,
}

impl Default for Text {
    fn default() -> Self {
        Text {
            text: Default::default(),
            font: String::from("sans-serif"),
            size: None,
            fill: None,
            node: ResolvedNode::Null,
        }
    }
}

impl Text {
    pub fn new(text: String) -> Self {
        Text {
            text,
            font: String::from("sans-serif"),
            size: None,
            fill: None,
            node: ResolvedNode::Null,
        }
    }

    pub fn font(mut self, font: impl ToString) -> Self {
        self.font = font.to_string();
        self
    }

    pub fn size(mut self, size: impl Into<Option<f32>>) -> Self {
        self.size = size.into();
        self
    }

    pub fn fill(mut self, fill: impl Into<Option<Paint>>) -> Self {
        self.fill = fill.into();
        self
    }
}

impl<S: ToString> From<S> for Text {
    fn from(s: S) -> Self {
        Text::new(s.to_string())
    }
}

impl Merge for Text {
    fn merge(&mut self, other: Self) {
        if self.text != other.text {
            self.text = other.text;
            self.node = ResolvedNode::Null;
        }
    }
}

impl Expand for Text {
    fn expand(&mut self, resources: &mut Resources) -> Result<Vec<EitherNode>, Error> {
        if let ResolvedNode::Null = self.node {
            self.node = node::styled_text(&self.text, &self.font, self.size, self.fill.clone())
                .resolve(resources)?;
        }

        Ok(vec![EitherNode::Resolved(self.node.clone())])
    }
}

#[derive(Default)]
pub struct Rectangle {
    pub size: Size2,
    pub corner_radius: [f32; 4],
    pub background: Option<Paint>,
    pub border: f32,
    pub border_fill: Option<Paint>,
    pub z_order: ZOrder,
}

impl Merge for Rectangle {
    fn merge(&mut self, other: Self) {
        *self = other;
    }
}

impl Expand for Rectangle {
    fn expand(&mut self, _resources: &mut Resources) -> Result<Vec<EitherNode>, Error> {
        Ok(vec![EitherNode::Unresolved(Node::Rectangle {
            size: self.size,
            corner_radius: self.corner_radius,
            background: self.background.clone(),
            border: self.border,
            border_fill: self.border_fill.clone(),
            z_order: self.z_order,
        })])
    }
}

pub struct Draw {
    pub size: Size2,
    pub draw_fn: Rc<dyn Fn(Rect, &mut skia_safe::Canvas)>,
    pub z_order: ZOrder,
}

impl Merge for Draw {
    fn merge(&mut self, other: Self) {
        *self = other;
    }
}

impl Expand for Draw {
    fn expand(&mut self, _resources: &mut Resources) -> Result<Vec<EitherNode>, Error> {
        Ok(vec![EitherNode::Unresolved(Node::Draw {
            size: self.size,
            draw_fn: Rc::clone(&self.draw_fn),
            z_order: self.z_order,
        })])
    }
}
