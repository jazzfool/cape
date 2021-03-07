use cape::{
    node,
    state::{use_state, Accessor},
    ui::{self, NodeLayout},
};

pub struct DecoratorState {
    pub hovered: bool,
    pub pressed: bool,
}

pub struct DecoratedProps<Below, T, Above, FnBelow, FnAbove>
where
    FnBelow: Fn(&DecoratorState) -> Below,
    FnAbove: Fn(&DecoratorState) -> Above,
{
    pub below: FnBelow,
    pub child: T,
    pub above: FnAbove,
    pub margin: cape::Sides2,
    pub z_order: node::ZOrder,
}

impl<Below, T, Above, FnBelow, FnAbove> DecoratedProps<Below, T, Above, FnBelow, FnAbove>
where
    Below: ui::Merge + ui::Expand,
    T: ui::Merge + ui::Expand,
    Above: ui::Merge + ui::Expand,
    FnBelow: Fn(&DecoratorState) -> Below,
    FnAbove: Fn(&DecoratorState) -> Above,
{
    pub fn below(mut self, below: FnBelow) -> Self {
        self.below = below;
        self
    }

    pub fn child(mut self, child: T) -> Self {
        self.child = child;
        self
    }

    pub fn above(mut self, above: FnAbove) -> Self {
        self.above = above;
        self
    }

    pub fn margin(mut self, margin: cape::Sides2) -> Self {
        self.margin = margin;
        self
    }

    pub fn z_order(mut self, z_order: node::ZOrder) -> Self {
        self.z_order = z_order;
        self
    }
}

pub type Decorated<Below, T, Above> =
    ui::Interact<crate::ContainerLayout<(crate::StackLayout<(Below, T, Above)>,)>>;

#[cape::ui]
pub fn decorated<Below, T, Above, FnBelow, FnAbove>(
    props: DecoratedProps<Below, T, Above, FnBelow, FnAbove>,
) -> Decorated<Below, T, Above>
where
    Below: ui::Merge + ui::Expand,
    T: ui::Merge + ui::Expand,
    Above: ui::Merge + ui::Expand,
    FnBelow: Fn(&DecoratorState) -> Below,
    FnAbove: Fn(&DecoratorState) -> Above,
{
    let hovered = use_state(|| false);
    let pressed = use_state(|| false);

    let state = DecoratorState {
        hovered: hovered.get(),
        pressed: pressed.get(),
    };

    Decorated::new(
        crate::Container {
            margin: props.margin,
        }
        .children((crate::Stack::default().children((
            ((props.below)(&state), crate::StackItem::fill()),
            (props.child, crate::StackItem::fill()),
            ((props.above)(&state), crate::StackItem::fill()),
        )),)),
        |_| {},
        Default::default(),
        true,
    )
}
