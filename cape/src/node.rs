use crate::id::Id;
use crate::{size2, Color, Error, Image, Point2, Rect, Size2};
use std::any::Any;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::sync::Arc;

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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ZIndex(pub i32);

#[derive(Clone)]
pub enum Node {
    Null,
    Interact {
        child: Box<Node>,
        callback: Rc<dyn Fn(&Interaction)>,
        id: Id,
        passthrough: bool,
        z_index: ZIndex,
    },
    Capture {
        child: Box<Node>,
        callback: Rc<dyn Fn(&ResolvedNode)>,
        z_index: ZIndex,
    },
    Layout {
        layout: Rc<dyn Layout>,
        children: Vec<Node>,
        z_index: ZIndex,
    },
    Text {
        text: String,
        font: String,
        size: Option<f32>,
        fill: Option<Paint>,
        z_index: ZIndex,
    },
    Rectangle {
        size: Size2,
        corner_radius: [f32; 4],
        background: Option<Paint>,
        border: f32,
        border_fill: Option<Paint>,
        z_index: ZIndex,
    },
    Draw {
        size: Size2,
        draw_fn: Rc<dyn Fn(Rect, &mut skia_safe::Canvas)>,
        z_index: ZIndex,
    },
}

impl Node {
    /// Returns the `ResolvedNode` version of this node tree.
    pub fn resolve(&self, resources: &Resources) -> Result<Option<ResolvedNode>, Error> {
        match self {
            Node::Null => Ok(None),
            Node::Interact {
                child,
                callback,
                id,
                passthrough,
                z_index,
            } => {
                let child = child.resolve(resources)?.ok_or(Error::EmptyNode)?;
                Ok(Some(ResolvedNode::Interact {
                    rect: Rect::new(Default::default(), child.size()),
                    child: Box::new(child),
                    callback: Rc::clone(callback),
                    id: *id,
                    passthrough: *passthrough,
                    z_index: *z_index,
                }))
            }
            Node::Capture {
                child,
                callback,
                z_index,
            } => {
                let child = child.resolve(resources)?.ok_or(Error::EmptyNode)?;
                Ok(Some(ResolvedNode::Capture {
                    rect: Rect::new(Default::default(), child.size()),
                    child: Box::new(child),
                    callback: Rc::clone(callback),
                    z_index: *z_index,
                }))
            }
            Node::Layout {
                layout,
                children,
                z_index,
            } => {
                let children = children
                    .iter()
                    .map(|child| child.resolve(resources))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .filter_map(|x| x)
                    .collect::<Vec<_>>();
                let size = layout.size(
                    &children
                        .iter()
                        .map(|child| child.size())
                        .collect::<Vec<_>>(),
                );
                Ok(Some(ResolvedNode::Layout {
                    layout: Rc::clone(layout),
                    children,
                    rect: Rect::new(Default::default(), size),
                    z_index: *z_index,
                }))
            }
            Node::Text {
                text,
                font,
                size,
                fill,
                z_index,
            } => {
                skia_safe::icu::init();

                let size = size.unwrap_or_else(|| resources.fallback_text_size);
                let font_data = Rc::clone(&resources.fonts[font]);
                let fnt = skia_safe::Font::new(&font_data.sk, size);

                let (blob, bounds) = if !text.is_empty() {
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
                };

                Ok(Some(ResolvedNode::Text {
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
                    z_index: *z_index,
                }))
            }
            Node::Rectangle {
                size,
                corner_radius,
                background,
                border,
                border_fill,
                z_index,
            } => Ok(Some(ResolvedNode::Rectangle {
                rect: Rect::new(Default::default(), *size),
                corner_radii: *corner_radius,
                background: background.clone(),
                border: *border,
                border_fill: border_fill.clone(),
                z_index: *z_index,
            })),
            Node::Draw {
                size,
                draw_fn,
                z_index,
            } => Ok(Some(ResolvedNode::Draw {
                rect: Rect::new(Default::default(), *size),
                draw_fn: Rc::clone(draw_fn),
                z_index: *z_index,
            })),
        }
    }

    pub fn children(&self) -> Vec<&Node> {
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

    pub fn z_index(&self) -> ZIndex {
        match self {
            Node::Interact { z_index, .. }
            | Node::Layout { z_index, .. }
            | Node::Text { z_index, .. }
            | Node::Rectangle { z_index, .. }
            | Node::Draw { z_index, .. } => *z_index,
            _ => Default::default(),
        }
    }

    pub fn z_index_mut(&mut self) -> Option<&mut ZIndex> {
        match self {
            Node::Interact { z_index, .. }
            | Node::Layout { z_index, .. }
            | Node::Text { z_index, .. }
            | Node::Rectangle { z_index, .. }
            | Node::Draw { z_index, .. } => Some(z_index),
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
        child: Box::new(child.to_node()),
        callback: Rc::new(callback),
        id: Id::current(),
        passthrough,
        z_index: Default::default(),
    }
}

pub fn text(text: impl Into<String>) -> Node {
    Node::Text {
        text: text.into(),
        font: String::from("sans-serif"),
        size: None,
        fill: None,
        z_index: Default::default(),
    }
}

pub fn styled_text(
    text: impl Into<String>,
    font: impl Into<String>,
    size: impl Into<Option<f32>>,
    fill: impl Into<Option<Paint>>,
) -> Node {
    Node::Text {
        text: text.into(),
        font: font.into(),
        size: size.into(),
        fill: fill.into(),
        z_index: Default::default(),
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
        z_index: Default::default(),
    }
}

pub fn draw(size: Size2, draw_fn: impl Fn(Rect, &mut skia_safe::Canvas) + 'static) -> Node {
    Node::Draw {
        size,
        draw_fn: Rc::new(draw_fn),
        z_index: Default::default(),
    }
}

pub fn z_index(node: impl ToNode, z_index: ZIndex) -> Node {
    let mut node = node.to_node();
    if let Some(z) = node.z_index_mut() {
        *z = z_index;
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
///
/// You should *not* construct this yourself.
#[derive(Clone)]
pub enum ResolvedNode {
    Null,
    Interact {
        child: Box<ResolvedNode>,
        callback: Rc<dyn Fn(&Interaction)>,
        rect: Rect,
        id: Id,
        passthrough: bool,
        z_index: ZIndex,
    },
    Capture {
        child: Box<ResolvedNode>,
        callback: Rc<dyn Fn(&ResolvedNode)>,
        rect: Rect,
        z_index: ZIndex,
    },
    Layout {
        layout: Rc<dyn Layout>,
        children: Vec<ResolvedNode>,
        rect: Rect,
        z_index: ZIndex,
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
        z_index: ZIndex,
    },
    Rectangle {
        rect: Rect,
        corner_radii: [f32; 4],
        background: Option<Paint>,
        border: f32,
        border_fill: Option<Paint>,
        z_index: ZIndex,
    },
    Draw {
        rect: Rect,
        draw_fn: Rc<dyn Fn(Rect, &mut skia_safe::Canvas)>,
        z_index: ZIndex,
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

    pub fn z_index(&self) -> ZIndex {
        match self {
            ResolvedNode::Interact { z_index, .. }
            | ResolvedNode::Capture { z_index, .. }
            | ResolvedNode::Layout { z_index, .. }
            | ResolvedNode::Text { z_index, .. }
            | ResolvedNode::Rectangle { z_index, .. }
            | ResolvedNode::Draw { z_index, .. } => *z_index,
            _ => panic!("null resolved node"),
        }
    }

    pub fn z_index_mut(&mut self) -> Option<&mut ZIndex> {
        match self {
            ResolvedNode::Interact { z_index, .. }
            | ResolvedNode::Capture { z_index, .. }
            | ResolvedNode::Layout { z_index, .. }
            | ResolvedNode::Text { z_index, .. }
            | ResolvedNode::Rectangle { z_index, .. }
            | ResolvedNode::Draw { z_index, .. } => Some(z_index),
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
    /// All the children of `Interact` and `Capture` nodes will be replaced with `Null`, meaning that invoking any positioning/sizing/z-indexing methods on these nodes will `panic!`.
    /// Therefore, you should only call `flatten` on a tree that has already been layed out and operated on.
    pub fn flatten(&self) -> impl Iterator<Item = Self> {
        let mut map = BTreeMap::<ZIndex, Vec<ResolvedNode>>::new();

        self.flatten_impl(&mut map, Default::default());

        map.into_iter()
            .fold(Vec::new(), |mut vec, (_z, mut nodes)| {
                vec.append(&mut nodes);
                vec
            })
            .into_iter()
    }

    fn flatten_impl(&self, map: &mut BTreeMap<ZIndex, Vec<ResolvedNode>>, mut base_z: ZIndex) {
        // NOTE(jazzfool): if absolute ZIndex were ever to be implemented: change the '+=' to '=' when it is absolute
        base_z.0 += self.z_index().0;
        map.entry(base_z).or_default().push(self.clone());
        match map.get_mut(&base_z).unwrap().last_mut().unwrap() {
            ResolvedNode::Interact { child, .. } | ResolvedNode::Capture { child, .. } => {
                *child = Box::new(ResolvedNode::Null)
            }
            ResolvedNode::Layout { children, .. } => children.clear(),
            _ => {}
        }

        for child in self.children() {
            child.flatten_impl(map, base_z);
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
