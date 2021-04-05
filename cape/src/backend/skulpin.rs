use crate::{
    backend::skia::render_tree,
    cx,
    id::Id,
    node::{Interaction, MouseButton, Node, Paint, ResolvedNode, Resources},
    Color, Point2, Rect, Size2,
};
use skulpin::winit;
use std::{cmp::Ordering, rc::Rc};
use thiserror::Error;
use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::ControlFlow,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("os error within winit: {0}")]
    WinitOs(#[from] winit::error::OsError),
    #[error("failed to create a skulpin renderer: {0}")]
    CreateRenderer(#[from] skulpin::CreateRendererError),
}

pub struct Window {
    pub body: Node,
    pub background: Color,
}

pub struct WindowInfo {
    pub size: Size2,
}

#[derive(Clone)]
struct InteractNode {
    callback: Rc<dyn Fn(&mut cx::Cx, &Interaction)>,
    id: Id,
}

pub fn run(
    mut f: impl FnMut(&WindowInfo, &mut cx::Cx, &mut Resources) -> Window + 'static,
) -> Result<(), Error> {
    skulpin::skia_safe::icu::init();

    let event_loop = winit::event_loop::EventLoop::new();

    let logical_size = winit::dpi::LogicalSize::new(900., 600.);

    let winit_window = winit::window::WindowBuilder::new()
        .with_title("Cape")
        .with_inner_size(logical_size)
        .build(&event_loop)?;

    let window = skulpin::WinitWindow::new(&winit_window);

    let mut renderer = skulpin::RendererBuilder::new()
        .use_vulkan_debug_layer(false)
        .coordinate_system(skulpin::CoordinateSystem::Logical)
        .prefer_mailbox_present_mode()
        .build(&window)?;

    let mut resources = Resources {
        fonts: Default::default(),
        fallback_text_size: 12.,
        fallback_text_fill: Paint::Solid(Color::new(1., 1., 1., 1.)),
        shaper_cache: Default::default(),
        font_cache: Default::default(),
    };

    let mut cx = cx::Cx::new();

    let mut scale_factor = winit_window.scale_factor();

    let size = winit_window.inner_size().to_logical(scale_factor);
    let mut size = Size2::new(size.width, size.height);

    let mut modifiers = winit::event::ModifiersState::default();
    let mut mouse_pos = Point2::default();
    let mut curr_node = ResolvedNode::Null;

    let mut hovered_node: Option<InteractNode> = None;
    let mut pressed_node: Option<InteractNode> = None;
    let mut focused_node: Option<InteractNode> = None;

    event_loop.run(move |event, _window_target, control_flow| {
        let window = skulpin::WinitWindow::new(&winit_window);

        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event:
                    WindowEvent::ScaleFactorChanged {
                        scale_factor: sf,
                        ref new_inner_size,
                    },
                ..
            } => {
                scale_factor = sf;
                let logical = new_inner_size.to_logical(scale_factor);
                size.width = logical.width;
                size.height = logical.height;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(physical_size),
                ..
            } => {
                let logical = physical_size.to_logical(scale_factor);
                size.width = logical.width;
                size.height = logical.height;
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let logical = position.to_logical(scale_factor);
                mouse_pos.x = logical.x;
                mouse_pos.y = logical.y;

                let prev_hovered = hovered_node.clone();

                if let Some(ResolvedNode::Interact { callback, id, .. }) =
                    node_at_point_tree(mouse_pos, &curr_node, None)
                {
                    hovered_node = Some(InteractNode {
                        callback: Rc::clone(callback),
                        id: *id,
                    });
                } else {
                    hovered_node = None;
                }

                if !compare_interact(&prev_hovered, &hovered_node) {
                    try_callback(
                        &prev_hovered,
                        &mut cx,
                        &Interaction::CursorExit { pos: mouse_pos },
                    );
                    try_callback(
                        &hovered_node,
                        &mut cx,
                        &Interaction::CursorEnter { pos: mouse_pos },
                    );
                }

                try_callback(
                    &hovered_node,
                    &mut cx,
                    &Interaction::CursorMove { pos: mouse_pos },
                );
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { button, state, .. },
                ..
            } => {
                let button = match button {
                    winit::event::MouseButton::Left => MouseButton::Left,
                    winit::event::MouseButton::Middle => MouseButton::Middle,
                    winit::event::MouseButton::Right => MouseButton::Right,
                    _ => return,
                };

                let event = match state {
                    ElementState::Pressed => Interaction::MouseDown {
                        button,
                        modifiers,
                        pos: mouse_pos,
                    },
                    ElementState::Released => Interaction::MouseUp {
                        button,
                        modifiers,
                        pos: mouse_pos,
                    },
                };

                let prev_focus = focused_node.clone();

                match state {
                    ElementState::Pressed if pressed_node.is_none() => {
                        if let Some(node) = &hovered_node {
                            pressed_node = Some(node.clone());
                            focused_node = Some(node.clone());
                            (*node.callback)(&mut cx, &event);
                        }
                    }
                    ElementState::Pressed => {
                        if let Some(node) = &pressed_node {
                            (*node.callback)(&mut cx, &event);
                        }
                    }
                    ElementState::Released => {
                        if let Some(node) = pressed_node.clone() {
                            // FIXME(jazzfool): only make pressed_none = None if *all* mouse buttons have been released
                            pressed_node = None;
                            (*node.callback)(&mut cx, &event);
                        }
                    }
                }

                if hovered_node.is_none() {
                    focused_node = None;
                }

                if !compare_interact(&prev_focus, &focused_node) {
                    try_callback(&prev_focus, &mut cx, &Interaction::LoseFocus);
                    try_callback(&focused_node, &mut cx, &Interaction::GainFocus);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if let Some(node) = &focused_node {
                    if let Some(keycode) = input.virtual_keycode {
                        (*node.callback)(
                            &mut cx,
                            &match input.state {
                                ElementState::Pressed => Interaction::KeyDown {
                                    key_code: keycode,
                                    modifiers,
                                },
                                ElementState::Released => Interaction::KeyUp {
                                    key_code: keycode,
                                    modifiers,
                                },
                            },
                        );
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::ReceivedCharacter(character),
                ..
            } => {
                if let Some(node) = &focused_node {
                    (*node.callback)(&mut cx, &Interaction::ReceiveCharacter { character });
                }
            }
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(mods),
                ..
            } => {
                modifiers = mods;
            }
            Event::MainEventsCleared => {
                winit_window.request_redraw();
            }
            Event::RedrawRequested(_window_id) => {
                renderer
                    .draw(&window, |canvas, _coordinate_system_helper| {
                        let w = f(&WindowInfo { size }, &mut cx, &mut resources);

                        curr_node = diff_resolve(
                            &mut resources,
                            w.body,
                            std::mem::replace(&mut curr_node, ResolvedNode::Null),
                            &Rect::new(Point2::new(0., 0.), size),
                            &mut hovered_node,
                            &mut pressed_node,
                            &mut focused_node,
                        );

                        canvas.clear(skulpin::skia_safe::Color::from_argb(
                            (w.background.alpha * 255.) as _,
                            (w.background.red * 255.) as _,
                            (w.background.green * 255.) as _,
                            (w.background.blue * 255.) as _,
                        ));

                        curr_node.perform_layout();
                        render_tree(
                            &mut cx,
                            canvas,
                            &curr_node,
                            &Rect::new(Point2::new(0., 0.), size),
                        )
                        .unwrap();
                    })
                    .expect("failed to render using vulkan");
            }
            _ => {}
        }
    });
}

fn compare_interact(a: &Option<InteractNode>, b: &Option<InteractNode>) -> bool {
    match (a, b) {
        (Some(_), None) | (None, Some(_)) => false,
        (None, None) => true,
        (Some(a), Some(b)) => a.id == b.id,
    }
}

fn try_callback(node: &Option<InteractNode>, cx: &mut cx::Cx, event: &Interaction) {
    if let Some(node) = node {
        (*node.callback)(cx, event);
    }
}

fn try_set_callback(node: &mut Option<InteractNode>, cb: &Rc<dyn Fn(&mut cx::Cx, &Interaction)>) {
    if let Some(node) = node {
        node.callback = cb.clone();
    }
}

fn node_at_point_tree<'a>(
    point: Point2,
    node: &'a ResolvedNode,
    mut last_interact: Option<&'a ResolvedNode>,
) -> Option<&'a ResolvedNode> {
    if node.rect().contains(point) {
        if node.is_interact() {
            last_interact = Some(node);
        }

        for child in node.children() {
            if let node @ Some(_) = node_at_point_tree(point, child, last_interact) {
                return node;
            }
        }

        if node.is_interact() {
            Some(node)
        } else {
            last_interact
        }
    } else {
        last_interact
    }
}

fn diff_resolve(
    resources: &mut Resources,
    new: Node,
    old: ResolvedNode,
    cull: &crate::Rect,
    hovered: &mut Option<InteractNode>,
    pressed: &mut Option<InteractNode>,
    focused: &mut Option<InteractNode>,
) -> ResolvedNode {
    match (new, old) {
        (Node::Null, ResolvedNode::Null) => Ok(Some(ResolvedNode::Null)),
        (
            Node::Interact {
                child: new_child,
                callback,
                id,
                passthrough,
            },
            ResolvedNode::Interact { child, .. },
        ) => {
            if compare_interact(
                hovered,
                &Some(InteractNode {
                    callback: callback.clone(),
                    id,
                }),
            ) {
                try_set_callback(hovered, &callback);
            }

            if compare_interact(
                pressed,
                &Some(InteractNode {
                    callback: callback.clone(),
                    id,
                }),
            ) {
                try_set_callback(pressed, &callback);
            }

            if compare_interact(
                focused,
                &Some(InteractNode {
                    callback: callback.clone(),
                    id,
                }),
            ) {
                try_set_callback(focused, &callback);
            }

            let child = Box::new(diff_resolve(
                resources, *new_child, *child, cull, hovered, pressed, focused,
            ));
            Ok(Some(ResolvedNode::Interact {
                rect: crate::Rect::new(Default::default(), child.size()),
                child,
                callback,
                id,
                passthrough,
            }))
        }
        (
            Node::Text {
                text: new_text,
                font: new_font,
                size: new_size,
                fill,
            },
            ResolvedNode::Text {
                text,
                font,
                size,
                font_data,
                sk_font,
                blob,
                rect,
                ..
            },
        ) => {
            let new_size = new_size.unwrap_or(resources.fallback_text_size);

            if new_text != text || new_font != font || (new_size - size).abs() > std::f32::EPSILON {
                Node::Text {
                    text: new_text,
                    font: new_font,
                    size: Some(new_size),
                    fill,
                }
                .resolve(resources)
            } else {
                Ok(Some(ResolvedNode::Text {
                    text: new_text,
                    font: new_font,
                    font_data,
                    sk_font,
                    blob,
                    size: new_size,
                    fill: fill.unwrap_or_else(|| resources.fallback_text_fill.clone()),
                    rect,
                }))
            }
        }
        (
            Node::Layout {
                layout,
                children: new_children,
            },
            ResolvedNode::Layout {
                mut children, rect, ..
            },
        ) => {
            if !cull.intersects(&rect) {
                return ResolvedNode::Null;
            }

            match children.len().cmp(&new_children.len()) {
                Ordering::Less => children.append(&mut vec![
                    ResolvedNode::Null;
                    new_children.len() - children.len()
                ]),
                Ordering::Greater => children.truncate(new_children.len()),
                _ => {}
            }

            let children = new_children
                .into_iter()
                .zip(children.into_iter())
                .map(|(new_child, old_child)| {
                    diff_resolve(
                        resources, new_child, old_child, cull, hovered, pressed, focused,
                    )
                })
                .collect::<Vec<_>>();

            let size = layout.size(
                &children
                    .iter()
                    .map(|child| child.size())
                    .collect::<Vec<_>>(),
            );

            Ok(Some(ResolvedNode::Layout {
                layout,
                children,
                rect: crate::Rect::new(Default::default(), size),
            }))
        }
        (new, _) => new.resolve(resources),
    }
    .unwrap()
    .unwrap()
}
