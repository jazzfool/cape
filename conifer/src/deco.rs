use cape::node::{IntoNode, Node};

use crate::LayoutBuilder;

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
    stack: Vec<(Node, crate::StackItem)>,
    core: usize,
    padding: cape::Sides2,
}

impl DecoratedNode {
    pub fn new(node: Node) -> Self {
        DecoratedNode {
            stack: vec![(node, crate::StackItem::fill())],
            padding: Default::default(),
            core: 0,
        }
    }

    pub fn decorator(mut self, deco: impl Decorator) -> Self {
        let item = deco.layout();

        match deco.order() {
            Order::Back => {
                self.core += 1;
                self.stack.insert(0, (deco.node(), item))
            }
            Order::Front => self.stack.push((deco.node(), item)),
        }

        self
    }

    pub fn apply<State>(self, mut hook: DecoratorHook<State>, state: &State) -> Self {
        hook.apply(self, state)
    }

    pub fn padding(mut self, padding: cape::Sides2) -> Self {
        self.padding = padding;
        self
    }
}

impl cape::node::IntoNode for DecoratedNode {
    fn into_node(self) -> Node {
        let core = self.core;
        let padding = self.padding;

        crate::stack()
            .children_items(
                self.stack
                    .into_iter()
                    .enumerate()
                    .map(|(i, (node, item))| {
                        if i == core {
                            (
                                crate::container().margin(padding).child(node).into_node(),
                                item,
                            )
                        } else {
                            (node, item)
                        }
                    })
                    .collect(),
            )
            .into_node()
    }
}

pub trait Decorated: Sized {
    fn decorate(self) -> DecoratedNode;

    #[inline]
    fn decorated(self, deco_fn: impl FnOnce(Self) -> DecoratedNode) -> DecoratedNode {
        deco_fn(self)
    }
}

impl<N: cape::node::IntoNode> Decorated for N {
    fn decorate(self) -> DecoratedNode {
        DecoratedNode::new(self.into_node())
    }
}

pub enum DecoratorHook<State> {
    None,
    Func(Box<dyn FnMut(&State, DecoratedNode) -> DecoratedNode>),
}

impl<State, F: FnMut(&State, DecoratedNode) -> DecoratedNode + 'static> From<F>
    for DecoratorHook<State>
{
    fn from(f: F) -> Self {
        DecoratorHook::Func(Box::new(f))
    }
}

impl<State> Default for DecoratorHook<State> {
    fn default() -> Self {
        DecoratorHook::None
    }
}

impl<State> DecoratorHook<State> {
    pub fn apply(&mut self, node: DecoratedNode, state: &State) -> DecoratedNode {
        if let DecoratorHook::Func(f) = self {
            f(state, node)
        } else {
            node
        }
    }
}
