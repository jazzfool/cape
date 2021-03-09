use crate::{id::Id, size2, Color, Error, Image, Point2, Rect, Size2};
use fxhash::FxHashMap;
use ordered_float::OrderedFloat;
use skulpin::skia_safe as sk;
use std::{rc::Rc, sync::Arc};

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
    Blur {
        radius: f32,
        tint: Color,
    },
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
pub enum Node {
    Null,
    Interact {
        child: Box<Node>,
        callback: Rc<dyn Fn(&Interaction)>,
        id: Id,
        passthrough: bool,
    },
    Capture {
        child: Box<Node>,
        callback: Rc<dyn Fn(&ResolvedNode)>,
    },
    Layout {
        layout: Rc<dyn Layout>,
        children: Vec<Node>,
    },
    Text {
        text: String,
        font: String,
        size: Option<f32>,
        fill: Option<Paint>,
    },
    Rectangle {
        size: Size2,
        corner_radius: [f32; 4],
        background: Option<Paint>,
        border: f32,
        border_fill: Option<Paint>,
    },
    Draw {
        size: Size2,
        draw_fn: Rc<dyn Fn(Rect, &mut sk::Canvas)>,
    },
    Resolved(ResolvedNode),
}

impl Node {
    /// Returns the `ResolvedNode` version of this node tree.
    pub fn resolve(&self, resources: &mut Resources) -> Result<Option<ResolvedNode>, Error> {
        match self {
            Node::Null => Ok(None),
            Node::Interact {
                child,
                callback,
                id,
                passthrough,
            } => {
                let child = child.resolve(resources)?.ok_or(Error::EmptyNode)?;
                Ok(Some(ResolvedNode::Interact {
                    rect: Rect::new(Default::default(), child.size()),
                    child: Box::new(child),
                    callback: Rc::clone(callback),
                    id: *id,
                    passthrough: *passthrough,
                }))
            }
            Node::Capture { child, callback } => {
                let child = child.resolve(resources)?.ok_or(Error::EmptyNode)?;
                Ok(Some(ResolvedNode::Capture {
                    rect: Rect::new(Default::default(), child.size()),
                    child: Box::new(child),
                    callback: Rc::clone(callback),
                }))
            }
            Node::Layout { layout, children } => {
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
                }))
            }
            Node::Text {
                text,
                font,
                size,
                fill,
            } => {
                let size = size.unwrap_or_else(|| resources.fallback_text_size);
                let font_data = Rc::clone(&resources.fonts[font]);

                let fnt = resources
                    .font_cache
                    .entry((font.clone(), OrderedFloat(size)))
                    .or_insert_with(|| Rc::new(sk::Font::new(&font_data.sk, size)))
                    .clone();

                let (blob, bounds) = if !text.is_empty() {
                    let (blob, bounds) = resources
                        .shaper_cache
                        .entry((text.clone(), font.clone(), OrderedFloat(size)))
                        .or_insert_with(|| {
                            let mut text_blob_builder_run_handler =
                                sk::shaper::TextBlobBuilderRunHandler::new(
                                    &text,
                                    sk::Point::default(),
                                );

                            let shaper = sk::Shaper::new(None);

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
                        })
                        .clone();

                    (Some(blob), bounds)
                } else {
                    (None, size2(0., 0.))
                };

                Ok(Some(ResolvedNode::Text {
                    text: text.clone(),
                    font: font.clone(),
                    font_data,
                    sk_font: fnt,
                    blob,
                    size,
                    fill: fill
                        .clone()
                        .unwrap_or_else(|| resources.fallback_text_fill.clone()),
                    rect: Rect::new(Default::default(), bounds),
                }))
            }
            Node::Rectangle {
                size,
                corner_radius,
                background,
                border,
                border_fill,
            } => Ok(Some(ResolvedNode::Rectangle {
                rect: Rect::new(Default::default(), *size),
                corner_radii: *corner_radius,
                background: background.clone(),
                border: *border,
                border_fill: border_fill.clone(),
            })),
            Node::Draw { size, draw_fn } => Ok(Some(ResolvedNode::Draw {
                rect: Rect::new(Default::default(), *size),
                draw_fn: Rc::clone(draw_fn),
            })),
            Node::Resolved(resolved) => Ok(Some(resolved.clone())),
        }
    }

    pub fn children(&self) -> Vec<&Node> {
        match self {
            Node::Interact { child, .. } => vec![child.as_ref()],
            Node::Layout { children, .. } => children.iter().collect(),
            _ => vec![],
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Node::Null
    }
}

pub trait IntoNode {
    fn into_node(self) -> Node;
}

impl<S: Into<String>> IntoNode for S {
    fn into_node(self) -> Node {
        text(self)
    }
}

impl IntoNode for Node {
    fn into_node(self) -> Node {
        self
    }
}

pub fn iff<N: IntoNode>(cond: bool, f: impl FnOnce() -> N) -> Node {
    if cond {
        f().into_node()
    } else {
        null()
    }
}

pub fn null() -> Node {
    Node::Null
}

#[track_caller]
pub fn interact(
    child: impl IntoNode,
    callback: impl Fn(&Interaction) + 'static,
    passthrough: bool,
) -> Node {
    Node::Interact {
        child: Box::new(child.into_node()),
        callback: Rc::new(callback),
        id: Id::current(),
        passthrough,
    }
}

pub fn text(text: impl Into<String>) -> Node {
    Node::Text {
        text: text.into(),
        font: String::from("sans-serif"),
        size: None,
        fill: None,
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
    }
}

pub fn draw(size: Size2, draw_fn: impl Fn(Rect, &mut sk::Canvas) + 'static) -> Node {
    Node::Draw {
        size,
        draw_fn: Rc::new(draw_fn),
    }
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
    },
    Capture {
        child: Box<ResolvedNode>,
        callback: Rc<dyn Fn(&ResolvedNode)>,
        rect: Rect,
    },
    Layout {
        layout: Rc<dyn Layout>,
        children: Vec<ResolvedNode>,
        rect: Rect,
    },
    Text {
        text: String,
        font: String,
        font_data: Rc<Font>,
        sk_font: Rc<sk::Font>,
        blob: Option<sk::TextBlob>,
        size: f32,
        fill: Paint,
        rect: Rect,
    },
    Rectangle {
        rect: Rect,
        corner_radii: [f32; 4],
        background: Option<Paint>,
        border: f32,
        border_fill: Option<Paint>,
    },
    Draw {
        rect: Rect,
        draw_fn: Rc<dyn Fn(Rect, &mut sk::Canvas)>,
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
            | ResolvedNode::Text { rect, .. }
            | ResolvedNode::Draw { rect, .. } => rect.origin,
            _ => crate::point2(0., 0.),
        }
    }

    /// Returns the size/bounds of this node.
    pub fn size(&self) -> Size2 {
        match self {
            ResolvedNode::Interact { rect, .. }
            | ResolvedNode::Capture { rect, .. }
            | ResolvedNode::Layout { rect, .. }
            | ResolvedNode::Rectangle { rect, .. }
            | ResolvedNode::Text { rect, .. }
            | ResolvedNode::Draw { rect, .. } => rect.size,
            _ => size2(0., 0.),
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
            ResolvedNode::Text { rect, .. } => rect.origin = r.origin,
            _ => {}
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
    pub fn flatten(&self, cull: &Rect) -> Vec<ResolvedNode> {
        let mut out = vec![];
        self.flatten_impl(&mut out, cull);
        out
    }

    fn flatten_impl(&self, out: &mut Vec<ResolvedNode>, cull: &Rect) {
        if self.rect().intersects(cull) {
            out.push(self.flat_clone());

            for child in self.children() {
                child.flatten_impl(out, cull);
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
                ..
            } => ResolvedNode::Interact {
                child: Box::new(ResolvedNode::Null),
                callback: callback.clone(),
                rect: *rect,
                id: *id,
                passthrough: *passthrough,
            },
            ResolvedNode::Capture { callback, rect, .. } => ResolvedNode::Capture {
                child: Box::new(ResolvedNode::Null),
                callback: callback.clone(),
                rect: *rect,
            },
            ResolvedNode::Layout { layout, rect, .. } => ResolvedNode::Layout {
                layout: Rc::clone(layout),
                children: Vec::new(),
                rect: *rect,
            },
            _ => self.clone(),
        }
    }
}

pub use font_kit::properties::Properties as FontProperties;

pub struct Font {
    pub font: font_kit::font::Font,
    pub sk: sk::Typeface,
}

impl Font {
    pub fn new(font: font_kit::font::Font) -> Result<Self, Error> {
        Ok(Font {
            sk: sk::Typeface::from_data(
                sk::Data::new_copy(font.copy_font_data().unwrap().as_slice()),
                None,
            )
            .ok_or(Error::SkiaFont)?,
            font,
        })
    }
}

/// Stores resources that will be used throughout the UI (e.g. fonts).
pub struct Resources {
    pub fonts: FxHashMap<String, Rc<Font>>,
    pub fallback_text_size: f32,
    pub fallback_text_fill: Paint,
    pub shaper_cache: FxHashMap<(String, String, OrderedFloat<f32>), (sk::TextBlob, Size2)>,
    pub font_cache: FxHashMap<(String, OrderedFloat<f32>), Rc<sk::Font>>,
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
