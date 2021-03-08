use crate::node::{Paint, ResolvedNode, Resources};
use crate::{Color, Point2, Rect, ToSkia};
use skia_safe::{self as sk, Canvas};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("error loading typeface \"{0}\"")]
    LoadingTypeface(String),
    #[error("error rendering text \"{0}\"")]
    Text(String),
    #[error("error converting paint")]
    PaintConversion,
    #[error("error rendering image")]
    Image,
}

#[derive(Default)]
pub struct Cache {}

impl Cache {
    pub fn new() -> Self {
        Default::default()
    }
}

fn rect_offset(rect: Rect, offset: Point2) -> Point2 {
    rect.origin + Point2::new(offset.x * rect.size.width, offset.y * rect.size.height).to_vector()
}

fn convert_paint(p: &Paint, rect: Rect, stroke: Option<f32>) -> Result<sk::Paint, Error> {
    let mut paint = sk::Paint::default();
    paint.set_anti_alias(true);
    if let Some(stroke) = stroke {
        paint.set_style(sk::PaintStyle::Stroke);
        paint.set_stroke_width(stroke);
    } else {
        paint.set_style(sk::PaintStyle::Fill);
    }
    match p {
        Paint::Solid(color) => {
            paint.set_color(color.to_skia());
        }
        Paint::LinearGradient { stops, begin, end } => {
            paint.set_shader(
                sk::gradient_shader::linear(
                    (
                        convert_point(rect_offset(rect, *begin)),
                        convert_point(rect_offset(rect, *end)),
                    ),
                    stops
                        .iter()
                        .map(|stop| stop.1.to_skia())
                        .collect::<Vec<_>>()
                        .as_slice(),
                    stops
                        .iter()
                        .map(|stop| stop.0)
                        .collect::<Vec<_>>()
                        .as_slice(),
                    sk::TileMode::default(),
                    None,
                    None,
                )
                .ok_or(Error::PaintConversion)?,
            );
        }
        Paint::RadialGradient {
            stops,
            center,
            radius,
        } => {
            paint.set_shader(
                sk::gradient_shader::radial(
                    convert_point(rect_offset(rect, *center)),
                    *radius,
                    stops
                        .iter()
                        .map(|stop| stop.1.to_skia())
                        .collect::<Vec<_>>()
                        .as_slice(),
                    stops
                        .iter()
                        .map(|stop| stop.0)
                        .collect::<Vec<_>>()
                        .as_slice(),
                    sk::TileMode::default(),
                    None,
                    None,
                )
                .ok_or(Error::PaintConversion)?,
            );
        }
        Paint::Image(img) => {
            paint.set_shader(
                sk::Image::from_raster_data(
                    &sk::ImageInfo::new(
                        sk::ISize::new(img.width() as _, img.height() as _),
                        sk::ColorType::RGBA8888,
                        sk::AlphaType::Unpremul,
                        None,
                    ),
                    unsafe { sk::Data::new_bytes(img.as_raw()) },
                    4 * img.width() as usize,
                )
                .ok_or(Error::Image)?
                .to_shader(None, None),
            );
        }
        _ => {}
    }
    Ok(paint)
}

fn convert_point(point: Point2) -> sk::Point {
    sk::Point::new(point.x, point.y)
}

pub fn render_list(
    cache: &mut Cache,
    canvas: &mut Canvas,
    resources: &Resources,
    list: &[ResolvedNode],
) -> Result<(), Error> {
    for node in list {
        render_node(cache, canvas, resources, node)?;
    }
    Ok(())
}

pub fn render_tree(
    cache: &mut Cache,
    canvas: &mut Canvas,
    resources: &Resources,
    node: &ResolvedNode,
) -> Result<(), Error> {
    render_node(cache, canvas, resources, node)?;

    for child in node.children() {
        render_tree(cache, canvas, resources, child)?;
    }

    Ok(())
}

pub fn render_node(
    cache: &mut Cache,
    canvas: &mut Canvas,
    resources: &Resources,
    node: &ResolvedNode,
) -> Result<(), Error> {
    match node {
        ResolvedNode::Text {
            fill,
            bottom_left,
            blob,
            ..
        } => {
            if let Paint::Blur { .. } = fill {
                panic!("text does not support blur paint");
            }

            if let Some(blob) = blob {
                canvas.draw_text_blob(
                    blob,
                    convert_point(*bottom_left),
                    &convert_paint(fill, node.rect(), None)?,
                );
            }
        }
        ResolvedNode::Rectangle {
            rect,
            corner_radii,
            background,
            border,
            border_fill,
            ..
        } => {
            if let Some(bg) = background {
                let rrect = sk::RRect::new_rect_radii(
                    rect.to_skia(),
                    &[
                        sk::Vector::new(corner_radii[0], corner_radii[0]),
                        sk::Vector::new(corner_radii[1], corner_radii[1]),
                        sk::Vector::new(corner_radii[2], corner_radii[2]),
                        sk::Vector::new(corner_radii[3], corner_radii[3]),
                    ],
                );

                match bg {
                    Paint::Blur { radius, tint } => {
                        canvas.save();

                        canvas.clip_rrect(rrect, None, true);

                        let blur = sk::image_filters::blur(
                            (*radius, *radius),
                            sk::TileMode::Clamp,
                            None,
                            &rect.to_skia().round(),
                        )
                        .ok_or(Error::PaintConversion)?;

                        canvas.save_layer(&sk::canvas::SaveLayerRec::default().backdrop(&blur));

                        canvas.draw_rect(
                            rect.to_skia(),
                            &convert_paint(&Paint::Solid(*tint), *rect, None)?,
                        );

                        canvas.restore();
                        canvas.restore();
                    }
                    _ => {
                        canvas.draw_rrect(rrect, &convert_paint(bg, *rect, None)?);
                    }
                }
            }

            if let Some(b) = border_fill {
                if let Paint::Blur { .. } = b {
                    panic!("border fill does not support blur paint");
                }

                canvas.draw_rrect(
                    sk::RRect::new_rect_radii(
                        rect.to_skia(),
                        &[
                            sk::Vector::new(corner_radii[0], corner_radii[0]),
                            sk::Vector::new(corner_radii[1], corner_radii[1]),
                            sk::Vector::new(corner_radii[2], corner_radii[2]),
                            sk::Vector::new(corner_radii[3], corner_radii[3]),
                        ],
                    ),
                    &convert_paint(b, *rect, Some(*border))?,
                );
            }
        }
        ResolvedNode::Draw { rect, draw_fn, .. } => draw_fn(*rect, canvas),
        _ => {}
    }

    Ok(())
}
