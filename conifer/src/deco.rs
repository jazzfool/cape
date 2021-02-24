use cape::node::{Node, ToNode};

use crate::LayoutBuilder;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DecoratorCondition {
    hovered: Option<bool>,
    pressed: Option<bool>,
}

impl DecoratorCondition {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn hovered(mut self, hovered: impl Into<Option<bool>>) -> Self {
        self.hovered = hovered.into();
        self
    }

    pub fn pressed(mut self, pressed: impl Into<Option<bool>>) -> Self {
        self.pressed = pressed.into();
        self
    }

    pub fn check(&self, hovered: bool, pressed: bool) -> bool {
        self.hovered.map(|x| x == hovered).unwrap_or(true)
            && self.pressed.map(|x| x == pressed).unwrap_or(true)
    }
}

pub enum Order {
    Back,
    Front,
}

pub trait Decorator {
    fn order(&self) -> Order;
    fn layout(&self) -> crate::StackItem;
    fn node(self) -> Node;
}

pub struct DecoratedNode {
    stack: Vec<(DecoratorCondition, Node, crate::StackItem)>,
    core: usize,
    padding: cape::Sides2,
}

impl DecoratedNode {
    pub fn new(node: Node) -> Self {
        DecoratedNode {
            stack: vec![(Default::default(), node, crate::StackItem::fill())],
            padding: Default::default(),
            core: 0,
        }
    }

    #[inline]
    pub fn decorator(self, deco: impl Decorator) -> Self {
        self.decorator_if(Default::default(), deco)
    }

    pub fn decorator_if(mut self, cond: DecoratorCondition, deco: impl Decorator) -> Self {
        let item = deco.layout();

        match deco.order() {
            Order::Back => {
                self.core += 1;
                self.stack
                    .insert(0, (Default::default(), deco.node(), item))
            }
            Order::Front => self.stack.push((cond, deco.node(), item)),
        }

        self
    }

    #[inline]
    pub fn apply(self, deco_fn: impl FnOnce(DecoratedNode) -> DecoratedNode) -> Self {
        deco_fn(self)
    }

    pub fn padding(mut self, padding: cape::Sides2) -> Self {
        self.padding = padding;
        self
    }
}

impl cape::node::ToNode for DecoratedNode {
    fn to_node(self) -> Node {
        let core = self.core;
        let padding = self.padding;

        crate::stack()
            .children_items(
                self.stack
                    .into_iter()
                    .enumerate()
                    .map(|(i, (cond, node, item))| {
                        if i == core {
                            (
                                crate::container().margin(padding).child(node).to_node(),
                                item,
                            )
                        } else {
                            (node, item)
                        }
                    })
                    .collect(),
            )
            .to_node()
    }
}

pub trait Decorated: Sized {
    fn decorate(self) -> DecoratedNode;

    #[inline]
    fn decorated(self, deco_fn: impl FnOnce(Self) -> DecoratedNode) -> DecoratedNode {
        deco_fn(self)
    }
}

impl<N: cape::node::ToNode> Decorated for N {
    fn decorate(self) -> DecoratedNode {
        DecoratedNode::new(self.to_node())
    }
}
