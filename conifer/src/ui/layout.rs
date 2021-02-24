use cape::node::{Layout, Node, ToNode};
use cape::{point2, size2, ui, Point2, Rect, Sides2, Size2};
use std::rc::Rc;

struct RowLayout {
    items: Vec<RowItem>,
    margin: Sides2,
    spacing: f32,
}

impl Layout for RowLayout {
    fn size(&self, sizes: &[Size2]) -> Size2 {
        let mut total = size2(
            self.spacing * (sizes.len() - 1) as f32 + self.margin.horizontal(),
            0.,
        );
        for (i, &size) in sizes.iter().enumerate() {
            let item = &self.items[i];
            total.width += size.width + item.margin.horizontal();
            let height = size.height + item.margin.vertical();
            if height > total.height {
                total.height = height;
            }
        }
        total
    }

    fn position(&self, rect: Rect, sizes: &[Size2]) -> Vec<Rect> {
        let mut x = rect.origin.x + self.margin.left;
        let mut out = Vec::new();
        for (i, &size) in sizes.iter().enumerate() {
            let mut size = size;
            let item = &self.items[i];
            x += item.margin.left;

            let y = match item.align {
                Align::Begin => rect.origin.y,
                Align::Middle => rect.origin.y + (rect.size.height / 2.) - (size.height / 2.),
                Align::End => rect.origin.y + rect.size.height - size.height,
                Align::Fill => {
                    size.height = rect.size.height;
                    rect.origin.y
                }
            };

            out.push(Rect::new(point2(x, y), size));
            x += size.width + item.margin.right + self.spacing;
        }
        out
    }
}

struct ColumnLayout {
    items: Vec<ColumnItem>,
    margin: Sides2,
    spacing: f32,
}

impl Layout for ColumnLayout {
    fn size(&self, sizes: &[Size2]) -> Size2 {
        let mut total = size2(
            0.,
            self.spacing * (sizes.len() - 1) as f32 + self.margin.vertical(),
        );
        for (i, &size) in sizes.iter().enumerate() {
            let item = &self.items[i];
            total.height += size.height + item.margin.vertical();
            let width = size.width + item.margin.horizontal();
            if width > total.width {
                total.width = width;
            }
        }
        total
    }

    fn position(&self, rect: Rect, sizes: &[Size2]) -> Vec<Rect> {
        let mut y = rect.origin.y + self.margin.top;
        let mut out = Vec::new();
        for (i, &size) in sizes.iter().enumerate() {
            let mut size = size;
            let item = &self.items[i];
            y += item.margin.top;

            let x = match item.align {
                Align::Begin => rect.origin.x,
                Align::Middle => rect.origin.x + (rect.size.width / 2.) - (size.width / 2.),
                Align::End => rect.origin.x + rect.size.width - size.width,
                Align::Fill => {
                    size.width = rect.size.width;
                    rect.origin.x
                }
            };

            out.push(Rect::new(point2(x, y), size));
            y += size.height + item.margin.bottom + self.spacing;
        }
        out
    }
}

struct StackLayout {
    items: Vec<StackItem>,
    margin: Sides2,
    width: Option<f32>,
    height: Option<f32>,
}

impl Layout for StackLayout {
    fn size(&self, sizes: &[Size2]) -> Size2 {
        let mut size: Size2 = size2(0., 0.);

        for sz in sizes {
            size.width = size.width.max(sz.width);
            size.height = size.height.max(sz.height);
        }

        if let Some(width) = self.width {
            size.width = width;
        }

        if let Some(height) = self.height {
            size.height = height;
        }

        size2(
            size.width + self.margin.horizontal(),
            size.height + self.margin.vertical(),
        )
    }

    fn position(&self, rect: Rect, sizes: &[Size2]) -> Vec<Rect> {
        let mut out = Vec::new();

        for i in 0..sizes.len() {
            let mut size = size2(0., 0.);
            let item = &self.items[i];

            if let Some(w) = item.width {
                size.width = w * rect.size.width;
            }
            if let Some(h) = item.height {
                size.height = h * rect.size.height;
            }

            if let Some(wh) = item.wh_offset {
                size += wh;
            }

            let mut xy = point2(item.xy.x * rect.size.width, item.xy.y * rect.size.height);
            xy += item.xy_offset.to_vector() + rect.origin.to_vector();
            xy -= point2(
                item.xy_anchor.x * size.width,
                item.xy_anchor.y * size.height,
            )
            .to_vector();

            out.push(Rect::new(xy, size));
        }

        out
    }
}

struct ContainerLayout {
    margin: Sides2,
}

impl Layout for ContainerLayout {
    fn size(&self, sizes: &[Size2]) -> Size2 {
        let size = if !sizes.is_empty() {
            assert!(sizes.len() == 1);
            sizes[0]
        } else {
            size2(0., 0.)
        };
        size2(
            size.width + self.margin.horizontal(),
            size.height + self.margin.vertical(),
        )
    }

    fn position(&self, rect: Rect, sizes: &[Size2]) -> Vec<Rect> {
        if !sizes.is_empty() {
            assert!(sizes.len() == 1);
            vec![rect.inner_rect(self.margin)]
        } else {
            vec![]
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Align {
    Begin,
    Middle,
    End,
    Fill,
}

impl Default for Align {
    fn default() -> Self {
        Align::Begin
    }
}

pub trait LayoutBuilder: Sized {
    type Item: Default;

    fn max_children() -> usize {
        std::usize::MAX
    }

    fn get_children(&mut self) -> &mut Vec<Node>;
    fn get_items(&mut self) -> &mut Vec<Self::Item>;

    #[track_caller]
    fn child(mut self, node: impl ToNode) -> Self {
        assert!(
            self.get_children().len() < Self::max_children(),
            "max layout children reached"
        );

        self.get_children().push(node.to_node());
        self.get_items().push(Default::default());
        self
    }

    #[track_caller]
    fn children(mut self, nodes: Vec<impl ToNode>) -> Self {
        assert!(
            self.get_children().len() + nodes.len() <= Self::max_children(),
            "adding these children will exceed max layout children"
        );

        self.get_items()
            .extend((0..nodes.len()).map(|_| Default::default()));
        self.get_children()
            .extend(nodes.into_iter().map(|x| x.to_node()));
        self
    }

    #[track_caller]
    fn child_item(mut self, node: impl ToNode, item: impl Into<Self::Item>) -> Self {
        assert!(
            self.get_children().len() < Self::max_children(),
            "max layout children reached"
        );

        self.get_children().push(node.to_node());
        self.get_items().push(item.into());
        self
    }

    #[track_caller]
    fn children_items(mut self, nodes: Vec<(impl ToNode, impl Into<Self::Item>)>) -> Self {
        assert!(
            self.get_children().len() + nodes.len() <= Self::max_children(),
            "adding these children will exceed max layout children"
        );

        let (nodes, items): (Vec<_>, Vec<_>) = nodes.into_iter().unzip();
        self.get_items().extend(items.into_iter().map(|x| x.into()));
        self.get_children()
            .extend(nodes.into_iter().map(|x| x.to_node()));
        self
    }
}

pub struct RowBuilder {
    children: Vec<Node>,
    items: Vec<RowItem>,

    margin: Sides2,
    spacing: f32,
}

impl RowBuilder {
    pub fn margin(mut self, margin: Sides2) -> Self {
        self.margin = margin;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
}

impl LayoutBuilder for RowBuilder {
    type Item = RowItem;

    fn get_children(&mut self) -> &mut Vec<Node> {
        &mut self.children
    }

    fn get_items(&mut self) -> &mut Vec<Self::Item> {
        &mut self.items
    }
}

impl ToNode for RowBuilder {
    #[ui]
    fn to_node(self) -> Node {
        Node::Layout {
            layout: Rc::new(RowLayout {
                items: self.items,
                margin: self.margin,
                spacing: self.spacing,
            }),
            children: self.children,
            z_index: Default::default(),
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
pub struct RowItem {
    pub align: Align,
    pub margin: Sides2,
}

pub fn row() -> RowBuilder {
    RowBuilder {
        children: Vec::new(),
        items: Vec::new(),
        margin: Sides2::zero(),
        spacing: 0.,
    }
}

pub struct ColumnBuilder {
    children: Vec<Node>,
    items: Vec<ColumnItem>,

    margin: Sides2,
    spacing: f32,
}

impl ColumnBuilder {
    pub fn margin(mut self, margin: Sides2) -> Self {
        self.margin = margin;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
}

impl LayoutBuilder for ColumnBuilder {
    type Item = ColumnItem;

    fn get_children(&mut self) -> &mut Vec<Node> {
        &mut self.children
    }

    fn get_items(&mut self) -> &mut Vec<Self::Item> {
        &mut self.items
    }
}

impl ToNode for ColumnBuilder {
    #[ui]
    fn to_node(self) -> Node {
        Node::Layout {
            layout: Rc::new(ColumnLayout {
                items: self.items,
                margin: self.margin,
                spacing: self.spacing,
            }),
            children: self.children,
            z_index: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct ColumnItem {
    pub align: Align,
    pub margin: Sides2,
}

pub fn column() -> ColumnBuilder {
    ColumnBuilder {
        children: Vec::new(),
        items: Vec::new(),

        margin: Sides2::zero(),
        spacing: 0.,
    }
}

pub struct StackBuilder {
    children: Vec<Node>,
    items: Vec<StackItem>,

    margin: Sides2,
    width: Option<f32>,
    height: Option<f32>,
}

impl StackBuilder {
    pub fn margin(mut self, margin: Sides2) -> Self {
        self.margin = margin;
        self
    }

    pub fn width(mut self, width: impl Into<Option<f32>>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Option<f32>>) -> Self {
        self.height = height.into();
        self
    }
}

impl LayoutBuilder for StackBuilder {
    type Item = StackItem;

    fn get_children(&mut self) -> &mut Vec<Node> {
        &mut self.children
    }

    fn get_items(&mut self) -> &mut Vec<Self::Item> {
        &mut self.items
    }
}

impl ToNode for StackBuilder {
    #[ui]
    fn to_node(self) -> Node {
        Node::Layout {
            layout: Rc::new(StackLayout {
                items: self.items,
                margin: self.margin,
                width: self.width,
                height: self.height,
            }),
            children: self.children,
            z_index: Default::default(),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct StackItem {
    pub xy: Point2,
    pub xy_offset: Point2,
    pub xy_anchor: Point2,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub wh_offset: Option<Size2>,
}

impl StackItem {
    pub fn center() -> Self {
        StackItem {
            xy: point2(0.5, 0.5),
            xy_anchor: point2(0.5, 0.5),
            ..Default::default()
        }
    }

    pub fn top_left() -> Self {
        Default::default()
    }

    pub fn top_center() -> Self {
        StackItem {
            xy: point2(0.5, 0.),
            xy_anchor: point2(0.5, 0.),
            ..Default::default()
        }
    }

    pub fn top_right() -> Self {
        StackItem {
            xy: point2(1., 0.),
            xy_anchor: point2(1., 0.),
            ..Default::default()
        }
    }

    pub fn right_center() -> Self {
        StackItem {
            xy: point2(1., 0.5),
            xy_anchor: point2(1., 0.5),
            ..Default::default()
        }
    }

    pub fn bottom_right() -> Self {
        StackItem {
            xy: point2(1., 1.),
            xy_anchor: point2(1., 1.),
            ..Default::default()
        }
    }

    pub fn bottom_center() -> Self {
        StackItem {
            xy: point2(0.5, 1.),
            xy_anchor: point2(0.5, 1.),
            ..Default::default()
        }
    }

    pub fn bottom_left() -> Self {
        StackItem {
            xy: point2(0., 1.),
            xy_anchor: point2(0., 1.),
            ..Default::default()
        }
    }

    pub fn left_center() -> Self {
        StackItem {
            xy: point2(0., 0.5),
            xy_anchor: point2(0., 0.5),
            ..Default::default()
        }
    }

    pub fn fill() -> Self {
        StackItem {
            width: Some(1.),
            height: Some(1.),
            ..Default::default()
        }
    }
}

pub fn stack() -> StackBuilder {
    StackBuilder {
        children: Vec::new(),
        items: Vec::new(),

        margin: Sides2::zero(),
        width: None,
        height: None,
    }
}

pub struct ContainerBuilder {
    children: Vec<Node>,
    items: Vec<()>,

    margin: Sides2,
}

impl ContainerBuilder {
    pub fn margin(mut self, margin: Sides2) -> Self {
        self.margin = margin;
        self
    }
}

impl LayoutBuilder for ContainerBuilder {
    type Item = ();

    fn max_children() -> usize {
        1
    }

    fn get_children(&mut self) -> &mut Vec<Node> {
        &mut self.children
    }

    fn get_items(&mut self) -> &mut Vec<Self::Item> {
        &mut self.items
    }
}

impl ToNode for ContainerBuilder {
    #[ui]
    fn to_node(self) -> Node {
        Node::Layout {
            layout: Rc::new(ContainerLayout {
                margin: self.margin,
            }),
            children: self.children,
            z_index: Default::default(),
        }
    }
}

pub fn container() -> ContainerBuilder {
    ContainerBuilder {
        children: Vec::new(),
        items: Vec::new(),

        margin: Sides2::zero(),
    }
}
