use crate::{id::Id, size2, Color, Error, Image, Point2, Rect, Size2};
use ordered_float::OrderedFloat;
use std::{
    any::Any,
    collections::{BTreeMap, HashMap},
    hash::{Hash, Hasher},
    rc::Rc,
    sync::Arc,
};

use crate as cape;

#[derive(Debug, Clone, PartialEq)]
pub enum Paint {
    Solid(Color),
    LinearGradient {
        stops: Vec<(f32, Color)>,
        begin: Point2,
        end: Point2,
    },
    RadialGradient {
        stops: Vec<(f32, Color)>,
        center: Point2,
        radius: f32,
    },
    Image(Rc<Image>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ZOrder {
    Bottom,
    Above,
    Below,
    Top,
}

impl Default for ZOrder {
    fn default() -> Self {
        ZOrder::Above
    }
}

#[derive(Clone)]
pub enum EitherNode {
    Unresolved(Node),
    Resolved(ResolvedNode),
}

impl EitherNode {
    pub fn resolve(&self, resources: &mut Resources) -> Result<ResolvedNode, Error> {
        match self {
            EitherNode::Unresolved(node) => node.resolve(resources),
            EitherNode::Resolved(node) => Ok(node.clone()),
        }
    }
}

#[derive(Clone)]
pub enum Node {
    Null,
    Interact {
        child: Box<EitherNode>,
        callback: Rc<dyn Fn(&Interaction)>,
        id: Id,
        passthrough: bool,
        z_order: ZOrder,
    },
    Capture {
        child: Box<EitherNode>,
        callback: Rc<dyn Fn(&ResolvedNode)>,
        z_order: ZOrder,
    },
    Layout {
        layout: Rc<dyn Layout>,
        children: Vec<EitherNode>,
        z_order: ZOrder,
    },
    Text {
        text: String,
        font: String,
        size: Option<f32>,
        fill: Option<Paint>,
        z_order: ZOrder,
    },
    Rectangle {
        size: Size2,
        corner_radius: [f32; 4],
        background: Option<Paint>,
        border: f32,
        border_fill: Option<Paint>,
        z_order: ZOrder,
    },
    Draw {
        size: Size2,
        draw_fn: Rc<dyn Fn(Rect, &mut skia_safe::Canvas)>,
        z_order: ZOrder,
    },
}

impl Node {
    /// Returns the `ResolvedNode` version of this node tree.
    pub fn resolve(&self, resources: &mut Resources) -> Result<ResolvedNode, Error> {
        match self {
            Node::Null => Ok(ResolvedNode::Null),
            Node::Interact {
                child,
                callback,
                id,
                passthrough,
                z_order,
            } => {
                let child = child.resolve(resources)?;
                Ok(ResolvedNode::Interact {
                    rect: Rect::new(Default::default(), child.size()),
                    child: Box::new(child),
                    callback: Rc::clone(callback),
                    id: *id,
                    passthrough: *passthrough,
                    z_order: *z_order,
                })
            }
            Node::Capture {
                child,
                callback,
                z_order,
            } => {
                let child = child.resolve(resources)?;
                Ok(ResolvedNode::Capture {
                    rect: Rect::new(Default::default(), child.size()),
                    child: Box::new(child),
                    callback: Rc::clone(callback),
                    z_order: *z_order,
                })
            }
            Node::Layout {
                layout,
                children,
                z_order,
            } => {
                let children = children
                    .iter()
                    .map(|child| child.resolve(resources))
                    .collect::<Result<Vec<_>, _>>()?;

                let size = layout.size(
                    &children
                        .iter()
                        .map(|child| child.size())
                        .collect::<Vec<_>>(),
                );

                Ok(ResolvedNode::Layout {
                    layout: Rc::clone(layout),
                    children,
                    rect: Rect::new(Default::default(), size),
                    z_order: *z_order,
                })
            }
            Node::Text {
                text,
                font,
                size,
                fill,
                z_order,
            } => {
                let size = size.unwrap_or_else(|| resources.fallback_text_size);
                let font_data = Rc::clone(&resources.fonts[font]);
                let mut fnt = skia_safe::Font::new(&font_data.sk, size);

                fnt.set_hinting(skia_safe::FontHinting::Normal);
                fnt.set_subpixel(true);

                let (blob, bounds) = if !text.is_empty() {
                    let mut h = std::collections::hash_map::DefaultHasher::new();

                    text.hash(&mut h);
                    font.hash(&mut h);
                    OrderedFloat(size).hash(&mut h);

                    let (blob, size) =
                        resources.blob_cache.entry(h.finish()).or_insert_with(|| {
                            let mut text_blob_builder_run_handler =
                                skia_safe::shaper::TextBlobBuilderRunHandler::new(
                                    &text,
                                    skia_safe::Point::default(),
                                );

                            let shaper = skia_safe::Shaper::new(None);

                            shaper.shape(
                                &text,
                                &fnt,
                                true,
                                std::f32::MAX,
                                &mut text_blob_builder_run_handler,
                            );

                            let blob = text_blob_builder_run_handler.make_blob().unwrap();
                            let bounds = fnt.measure_str(&text, None).1;
                            let bounds = size2(bounds.width(), fnt.spacing());
                            (blob, bounds)
                        });

                    (Some(blob.clone()), *size)
                } else {
                    (None, size2(0., 0.))
                };

                Ok(ResolvedNode::Text {
                    text: text.clone(),
                    font: font.clone(),
                    font_data,
                    sk_font: Rc::new(fnt),
                    blob,
                    size,
                    fill: fill
                        .clone()
                        .unwrap_or_else(|| resources.fallback_text_fill.clone()),
                    bounds,
                    bottom_left: Default::default(),
                    z_order: *z_order,
                })
            }
            Node::Rectangle {
                size,
                corner_radius,
                background,
                border,
                border_fill,
                z_order,
            } => Ok(ResolvedNode::Rectangle {
                rect: Rect::new(Default::default(), *size),
                corner_radii: *corner_radius,
                background: background.clone(),
                border: *border,
                border_fill: border_fill.clone(),
                z_order: *z_order,
            }),
            Node::Draw {
                size,
                draw_fn,
                z_order,
            } => Ok(ResolvedNode::Draw {
                rect: Rect::new(Default::default(), *size),
                draw_fn: Rc::clone(draw_fn),
                z_order: *z_order,
            }),
        }
    }

    pub fn children(&self) -> Vec<&EitherNode> {
        match self {
            Node::Interact { child, .. } => vec![child.as_ref()],
            Node::Layout { children, .. } => children.iter().collect(),
            _ => vec![],
        }
    }

    pub fn text_layout(
        &self,
        resources: &Resources,
    ) -> Option<(Option<skia_safe::TextBlob>, Size2)> {
        if let Node::Text {
            text, font, size, ..
        } = self
        {
            skia_safe::icu::init();

            let size = size.unwrap_or_else(|| resources.fallback_text_size);
            let font_data = Rc::clone(&resources.fonts[font]);
            let fnt = skia_safe::Font::new(&font_data.sk, size);

            Some(if !text.is_empty() {
                let mut text_blob_builder_run_handler =
                    skia_safe::shaper::TextBlobBuilderRunHandler::new(
                        &text,
                        skia_safe::Point::default(),
                    );

                let shaper = skia_safe::Shaper::new(None);

                shaper.shape(
                    &text,
                    &fnt,
                    true,
                    std::f32::MAX,
                    &mut text_blob_builder_run_handler,
                );

                let blob = text_blob_builder_run_handler.make_blob().unwrap();
                let bounds = fnt.measure_str(&text, None).1;
                let bounds = size2(bounds.width(), fnt.spacing());
                (Some(blob), bounds)
            } else {
                (None, size2(0., 0.))
            })
        } else {
            None
        }
    }

    pub fn z_order(&self) -> ZOrder {
        match self {
            Node::Interact { z_order, .. }
            | Node::Layout { z_order, .. }
            | Node::Text { z_order, .. }
            | Node::Rectangle { z_order, .. }
            | Node::Draw { z_order, .. } => *z_order,
            _ => Default::default(),
        }
    }

    pub fn z_order_mut(&mut self) -> Option<&mut ZOrder> {
        match self {
            Node::Interact { z_order, .. }
            | Node::Layout { z_order, .. }
            | Node::Text { z_order, .. }
            | Node::Rectangle { z_order, .. }
            | Node::Draw { z_order, .. } => Some(z_order),
            _ => None,
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Node::Null
    }
}

pub trait ToNode {
    fn to_node(self) -> Node;
}

impl<S: Into<String>> ToNode for S {
    fn to_node(self) -> Node {
        text(self)
    }
}

impl ToNode for Node {
    fn to_node(self) -> Node {
        self
    }
}

pub fn iff<N: ToNode>(cond: bool, f: impl FnOnce() -> N) -> Node {
    if cond {
        f().to_node()
    } else {
        null()
    }
}

pub fn null() -> Node {
    Node::Null
}

#[track_caller]
pub fn interact(
    child: impl ToNode,
    callback: impl Fn(&Interaction) + 'static,
    passthrough: bool,
) -> Node {
    Node::Interact {
        child: Box::new(EitherNode::Unresolved(child.to_node())),
        callback: Rc::new(callback),
        id: Id::current(),
        passthrough,
        z_order: Default::default(),
    }
}

#[track_caller]
pub fn capture(child: impl ToNode, callback: impl Fn(&ResolvedNode) + 'static) -> Node {
    Node::Capture {
        child: Box::new(EitherNode::Unresolved(child.to_node())),
        callback: Rc::new(callback),
        z_order: Default::default(),
    }
}

pub fn text(text: impl Into<String>) -> Node {
    Node::Text {
        text: text.into(),
        font: String::from("sans-serif"),
        size: None,
        fill: None,
        z_order: Default::default(),
    }
}

pub fn styled_text(
    text: impl ToString,
    font: impl ToString,
    size: impl Into<Option<f32>>,
    fill: impl Into<Option<Paint>>,
) -> Node {
    Node::Text {
        text: text.to_string(),
        font: font.to_string(),
        size: size.into(),
        fill: fill.into(),
        z_order: Default::default(),
    }
}

pub fn rectangle(
    size: Size2,
    corner_radius: [f32; 4],
    background: impl Into<Option<Paint>>,
    border: f32,
    border_fill: impl Into<Option<Paint>>,
) -> Node {
    Node::Rectangle {
        size,
        corner_radius,
        background: background.into(),
        border,
        border_fill: border_fill.into(),
        z_order: Default::default(),
    }
}

pub fn draw(size: Size2, draw_fn: impl Fn(Rect, &mut skia_safe::Canvas) + 'static) -> Node {
    Node::Draw {
        size,
        draw_fn: Rc::new(draw_fn),
        z_order: Default::default(),
    }
}

pub fn z_order(node: impl ToNode, z_order: ZOrder) -> Node {
    let mut node = node.to_node();
    if let Some(z) = node.z_order_mut() {
        *z = z_order;
    }
    node
}

pub enum MouseButton {
    Left,
    Middle,
    Right,
}

pub type KeyCode = winit::event::VirtualKeyCode;

pub enum Interaction {
    MouseDown {
        button: MouseButton,
        pos: Point2,
        modifiers: winit::event::ModifiersState,
    },
    MouseUp {
        button: MouseButton,
        pos: Point2,
        modifiers: winit::event::ModifiersState,
    },
    GainFocus,
    LoseFocus,
    ReceiveCharacter {
        character: char,
    },
    KeyDown {
        key_code: KeyCode,
        modifiers: winit::event::ModifiersState,
    },
    KeyUp {
        key_code: KeyCode,
        modifiers: winit::event::ModifiersState,
    },
}

impl Interaction {
    pub fn is_mouse_down(&self) -> bool {
        matches!(self, Interaction::MouseDown { .. })
    }
}

pub trait Layout {
    fn size(&self, sizes: &[Size2]) -> Size2;
    fn position(&self, rect: Rect, sizes: &[Size2]) -> Vec<Rect>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapedGlyph {
    pub id: u16,
    pub offset: Point2,
    pub advance: Point2,
}

/// Mirror of `Node` where references to resources have been resolved and node sizes/position are available and ready for layout.
#[derive(Clone)]
pub enum ResolvedNode {
    Null,
    Interact {
        child: Box<ResolvedNode>,
        callback: Rc<dyn Fn(&Interaction)>,
        rect: Rect,
        id: Id,
        passthrough: bool,
        z_order: ZOrder,
    },
    Capture {
        child: Box<ResolvedNode>,
        callback: Rc<dyn Fn(&ResolvedNode)>,
        rect: Rect,
        z_order: ZOrder,
    },
    Layout {
        layout: Rc<dyn Layout>,
        children: Vec<ResolvedNode>,
        rect: Rect,
        z_order: ZOrder,
    },
    Text {
        text: String,
        font: String,
        font_data: Rc<Font>,
        sk_font: Rc<skia_safe::Font>,
        blob: Option<skia_safe::TextBlob>,
        size: f32,
        fill: Paint,
        bounds: Size2,
        bottom_left: Point2,
        z_order: ZOrder,
    },
    Rectangle {
        rect: Rect,
        corner_radii: [f32; 4],
        background: Option<Paint>,
        border: f32,
        border_fill: Option<Paint>,
        z_order: ZOrder,
    },
    Draw {
        rect: Rect,
        draw_fn: Rc<dyn Fn(Rect, &mut skia_safe::Canvas)>,
        z_order: ZOrder,
    },
}

impl ResolvedNode {
    /// Performs top-down layout of this resolved node tree.
    ///
    /// Layout is applied in-place (hence the `&mut self`).
    pub fn perform_layout(&mut self) {
        match self {
            ResolvedNode::Interact { child, rect, .. } => {
                child.set_rect(*rect);
                child.perform_layout();
            }
            ResolvedNode::Layout {
                layout,
                children,
                rect,
                ..
            } => {
                let rects = layout.position(
                    *rect,
                    &children
                        .iter()
                        .map(|child| child.size())
                        .collect::<Vec<_>>(),
                );
                for (rect, child) in rects.into_iter().zip(children.iter_mut()) {
                    child.set_rect(rect);
                    child.perform_layout();
                }
            }
            _ => {}
        }
    }

    /// Returns the top-left position of this node.
    pub fn position(&self) -> Point2 {
        match self {
            ResolvedNode::Interact { rect, .. }
            | ResolvedNode::Capture { rect, .. }
            | ResolvedNode::Layout { rect, .. }
            | ResolvedNode::Rectangle { rect, .. }
            | ResolvedNode::Draw { rect, .. } => rect.origin,
            ResolvedNode::Text {
                bounds,
                bottom_left,
                ..
            } => *bottom_left - size2(0., bounds.height).to_vector(),
            _ => panic!("null resolved node"),
        }
    }

    /// Returns the size/bounds of this node.
    pub fn size(&self) -> Size2 {
        match self {
            ResolvedNode::Interact { rect, .. }
            | ResolvedNode::Capture { rect, .. }
            | ResolvedNode::Layout { rect, .. }
            | ResolvedNode::Rectangle { rect, .. }
            | ResolvedNode::Draw { rect, .. } => rect.size,
            ResolvedNode::Text { bounds, .. } => *bounds,
            _ => panic!("null resolved node"),
        }
    }

    /// Shorthand for constructing a `Rect` from `position()` and `size()`.
    pub fn rect(&self) -> Rect {
        Rect::new(self.position(), self.size())
    }

    /// Sets the rectangle of this node.
    ///
    /// The size of the rectangle has no effect is this is a `Text` node.
    pub fn set_rect(&mut self, r: Rect) {
        match self {
            ResolvedNode::Interact { rect, .. }
            | ResolvedNode::Capture { rect, .. }
            | ResolvedNode::Layout { rect, .. }
            | ResolvedNode::Rectangle { rect, .. }
            | ResolvedNode::Draw { rect, .. } => *rect = r,
            ResolvedNode::Text { bottom_left, .. } => *bottom_left = r.origin,
            _ => panic!("null resolved node"),
        }
    }

    pub fn children(&self) -> Vec<&ResolvedNode> {
        match self {
            ResolvedNode::Interact { child, .. } | ResolvedNode::Capture { child, .. } => {
                vec![child.as_ref()]
            }
            ResolvedNode::Layout { children, .. } => children.iter().collect(),
            _ => vec![],
        }
    }

    pub fn is_interact(&self) -> bool {
        matches!(self, ResolvedNode::Interact { .. })
    }

    pub fn z_order(&self) -> ZOrder {
        match self {
            ResolvedNode::Interact { z_order, .. }
            | ResolvedNode::Capture { z_order, .. }
            | ResolvedNode::Layout { z_order, .. }
            | ResolvedNode::Text { z_order, .. }
            | ResolvedNode::Rectangle { z_order, .. }
            | ResolvedNode::Draw { z_order, .. } => *z_order,
            _ => panic!("null resolved node"),
        }
    }

    pub fn z_order_mut(&mut self) -> Option<&mut ZOrder> {
        match self {
            ResolvedNode::Interact { z_order, .. }
            | ResolvedNode::Capture { z_order, .. }
            | ResolvedNode::Layout { z_order, .. }
            | ResolvedNode::Text { z_order, .. }
            | ResolvedNode::Rectangle { z_order, .. }
            | ResolvedNode::Draw { z_order, .. } => Some(z_order),
            _ => panic!("null resolved node"),
        }
    }

    pub fn invoke_captures(&self) {
        if let ResolvedNode::Capture { callback, .. } = self {
            callback(self);
        }

        for child in self.children() {
            child.invoke_captures();
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, ResolvedNode::Null)
    }

    /// Flattens the tree into a list, sorted by z-index.
    ///
    /// All the children of `Interact`, `Capture`, and `Layout` nodes will be replaced with `Null`, meaning that invoking any positioning/sizing/z-indexing methods on these nodes will `panic!`.
    /// Therefore, you should only call `flatten` on a tree that has already been layed out and operated on.
    pub fn flatten(&self) -> Vec<ResolvedNode> {
        let mut bottom = Vec::new();
        let mut top = Vec::new();
        let center = self.flatten_impl(&mut bottom, &mut top);
        [bottom, center, top].concat()
    }

    fn flatten_impl(
        &self,
        bottom: &mut Vec<ResolvedNode>,
        top: &mut Vec<ResolvedNode>,
    ) -> Vec<ResolvedNode> {
        let children = self.children();

        let mut v = Vec::new();
        v.reserve(children.len() + 1);

        v.push(self.flat_clone());

        for child in children {
            let mut branch = child.flatten_impl(bottom, top);
            match child.z_order() {
                ZOrder::Bottom => bottom.append(&mut branch),
                ZOrder::Above => v.append(&mut branch),
                ZOrder::Below => v = [branch, v].concat(),
                ZOrder::Top => top.append(&mut branch),
            }
        }

        match self.z_order() {
            ZOrder::Bottom => {
                bottom.append(&mut v);
                vec![]
            }
            ZOrder::Above | ZOrder::Below => v,
            ZOrder::Top => {
                top.append(&mut v);
                vec![]
            }
        }
    }

    fn flat_clone(&self) -> Self {
        match self {
            ResolvedNode::Interact {
                callback,
                rect,
                id,
                passthrough,
                z_order,
                ..
            } => ResolvedNode::Interact {
                child: Box::new(ResolvedNode::Null),
                callback: callback.clone(),
                rect: *rect,
                id: *id,
                passthrough: *passthrough,
                z_order: *z_order,
            },
            ResolvedNode::Capture {
                callback,
                rect,
                z_order,
                ..
            } => ResolvedNode::Capture {
                child: Box::new(ResolvedNode::Null),
                callback: callback.clone(),
                rect: *rect,
                z_order: *z_order,
            },
            ResolvedNode::Layout {
                layout,
                rect,
                z_order,
                ..
            } => ResolvedNode::Layout {
                layout: Rc::clone(layout),
                children: Vec::new(),
                rect: *rect,
                z_order: *z_order,
            },
            _ => self.clone(),
        }
    }
}

pub use font_kit::properties::Properties as FontProperties;

pub struct Font {
    pub font: font_kit::font::Font,
    pub sk: skia_safe::Typeface,
}

impl Font {
    pub fn new(font: font_kit::font::Font) -> Result<Self, Error> {
        Ok(Font {
            sk: skia_safe::Typeface::from_data(
                skia_safe::Data::new_copy(font.copy_font_data().unwrap().as_slice()),
                None,
            )
            .ok_or(Error::SkiaFont)?,
            font,
        })
    }
}

/// Stores resources that will be used throughout the UI (e.g. fonts).
pub struct Resources {
    pub fonts: HashMap<String, Rc<Font>>,
    pub blob_cache: HashMap<u64, (skia_safe::TextBlob, Size2)>,
    pub fallback_text_size: f32,
    pub fallback_text_fill: Paint,
}

impl Resources {
    /// Adds a `font_kit` font stored at `name`.
    pub fn add_font(
        &mut self,
        name: impl Into<String>,
        font: font_kit::font::Font,
    ) -> Result<(), Error> {
        self.fonts.insert(name.into(), Rc::new(Font::new(font)?));
        Ok(())
    }

    /// Loads a font from the best matched family name (`families`) and stores it at `name`.
    ///
    /// # Note
    /// - `name` has nothing to do with which font is selected.
    /// - `families` is a list of fallbacks. The first one that is matched will be selected.
    pub fn load_font(
        &mut self,
        name: impl Into<String>,
        families: &[String],
        properties: &FontProperties,
    ) -> Result<(), Error> {
        self.fonts.insert(
            name.into(),
            Rc::new(Font::new(font_kit::font::Font::from_handle(
                &font_kit::source::SystemSource::new().select_best_match(
                    &families
                        .iter()
                        .map(|family| font_kit::family_name::FamilyName::Title(family.clone()))
                        .collect::<Vec<_>>(),
                    properties,
                )?,
            )?)?),
        );
        Ok(())
    }

    /// Loads a font directly from bytes and stores it at `name`.
    ///
    /// If there is more than one font in the data, the font to load can be specified with `index`.
    pub fn load_font_data(
        &mut self,
        name: impl Into<String>,
        bytes: Arc<Vec<u8>>,
        index: impl Into<Option<u32>>,
    ) -> Result<(), Error> {
        let index = index.into().unwrap_or(0);
        self.fonts.insert(
            name.into(),
            Rc::new(Font::new(font_kit::font::Font::from_bytes(bytes, index)?)?),
        );
        Ok(())
    }

    /// Returns a reference to the font stored at `name`, if any.
    pub fn font(&self, name: &str) -> Option<&Font> {
        self.fonts.get(name).map(|x| x.as_ref())
    }

    pub fn has_font(&self, name: &str) -> bool {
        self.fonts.contains_key(name)
    }
}
