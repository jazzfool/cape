use crate::id::Id;
use crate::node::{Interaction, MouseButton, Node, Paint, ResolvedNode, Resources};
use crate::state::{call_on_lifecycles, call_on_renders};
use crate::{
    backend::skia::{render_list, Cache as SkiaCache},
    state::use_event,
};
use crate::{Color, Point2};
use skulpin::winit;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("os error within winit: {0}")]
    WinitOs(#[from] winit::error::OsError),
    #[error("failed to create a skulpin renderer: {0}")]
    CreateRenderer(#[from] skulpin::CreateRendererError),
}

pub struct Window {
    pub body: ResolvedNode,
    pub background: Color,
}

pub struct WindowInfo {}

pub fn run(
    mut f: impl FnMut(&WindowInfo, &mut Resources) -> Window + 'static,
) -> Result<(), Error> {
    skia_safe::icu::init();

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

    let mut skia_cache = SkiaCache::default();

    let mut resources = Resources {
        fonts: Default::default(),
        blob_cache: Default::default(),
        fallback_text_size: 12.,
        fallback_text_fill: Paint::Solid(Color::new(1., 1., 1., 1.)),
    };

    let mut modifiers = winit::event::ModifiersState::default();
    let mut mouse_pos = Point2::default();
    let mut latest_nodes: Option<Vec<ResolvedNode>> = None;
    let mut focus_node: Option<ResolvedNode> = None;

    event_loop.run(move |event, _window_target, control_flow| {
        let out_evt = use_event();

        let window = skulpin::WinitWindow::new(&winit_window);

        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => *control_flow = winit::event_loop::ControlFlow::Exit,
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::ModifiersChanged(mods),
                ..
            } => {
                modifiers = mods;
            }
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let position = position.to_logical(winit_window.scale_factor());
                mouse_pos = Point2::new(position.x, position.y);
            }
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::MouseInput { button, state, .. },
                ..
            } => {
                let button = match button {
                    winit::event::MouseButton::Left => MouseButton::Left,
                    winit::event::MouseButton::Middle => MouseButton::Middle,
                    winit::event::MouseButton::Right => MouseButton::Right,
                    _ => MouseButton::Left,
                };

                let event = match state {
                    winit::event::ElementState::Pressed => Interaction::MouseDown {
                        button,
                        pos: mouse_pos,
                        modifiers,
                    },
                    winit::event::ElementState::Released => Interaction::MouseUp {
                        button,
                        pos: mouse_pos,
                        modifiers,
                    },
                };

                if let Some(node) = latest_nodes
                    .as_ref()
                    .and_then(|nodes| node_at_point(mouse_pos, &nodes, &event))
                {
                    let old_node = focus_node.take().and_then(|node| {
                        if let ResolvedNode::Interact { callback, id, .. } = node {
                            Some((callback, id))
                        } else {
                            None
                        }
                    });

                    if let ResolvedNode::Interact { callback, id, .. } = node {
                        focus_node = Some(node.clone());

                        if let Some((old_callback, old_id)) = old_node {
                            if old_id != *id {
                                old_callback(&Interaction::LoseFocus);
                                callback(&Interaction::GainFocus);
                            }
                        }

                        (*callback)(&event);
                    }
                } else if let Some(ResolvedNode::Interact { callback, .. }) = focus_node.take() {
                    callback(&Interaction::LoseFocus);
                }
            }
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::ReceivedCharacter(character),
                ..
            } => {
                if let Some(ResolvedNode::Interact { callback, .. }) = &focus_node {
                    callback(&Interaction::ReceiveCharacter { character });
                }
            }
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if let Some(ResolvedNode::Interact { callback, .. }) = &focus_node {
                    let event = match input.state {
                        winit::event::ElementState::Pressed => Interaction::KeyDown {
                            key_code: input.virtual_keycode.unwrap(),
                            modifiers,
                        },
                        winit::event::ElementState::Released => Interaction::KeyUp {
                            key_code: input.virtual_keycode.unwrap(),
                            modifiers,
                        },
                    };

                    callback(&event);
                }
            }
            winit::event::Event::MainEventsCleared => {
                winit_window.request_redraw();
            }
            winit::event::Event::RedrawRequested(_window_id) => {
                let latest_nodes = &mut latest_nodes;
                renderer
                    .draw(&window, |canvas, _coordinate_system_helper| {
                        let mut w = f(&WindowInfo {}, &mut resources);

                        call_on_lifecycles(&resources);

                        w.body.perform_layout();
                        w.body.invoke_captures();

                        *latest_nodes = Some(w.body.flatten());

                        canvas.clear(skulpin::skia_safe::Color::from_argb(
                            (w.background.alpha * 255.) as _,
                            (w.background.red * 255.) as _,
                            (w.background.green * 255.) as _,
                            (w.background.blue * 255.) as _,
                        ));
                        render_list(
                            &mut skia_cache,
                            canvas,
                            &resources,
                            latest_nodes.as_ref().unwrap(),
                        )
                        .expect("failed to render using skia");
                    })
                    .expect("failed to render using vulkan");
                if let Some(ResolvedNode::Interact { id, .. }) = focus_node.take() {
                    focus_node = find_interact(id, latest_nodes.as_ref().unwrap()).cloned();
                }
            }
            _ => {}
        }

        if let Some(event) = event.to_static() {
            out_evt.emit(&event);
        }

        call_on_renders(&resources);
    });
}

fn node_at_point<'a>(
    point: Point2,
    nodes: &'a [ResolvedNode],
    event: &'a Interaction,
) -> Option<&'a ResolvedNode> {
    for node in nodes.iter().rev() {
        if let ResolvedNode::Interact {
            rect,
            passthrough,
            callback,
            ..
        } = &node
        {
            if rect.contains(point) {
                if *passthrough {
                    (*callback)(event);
                } else {
                    return Some(node);
                }
            }
        }
    }

    None
}

fn find_interact(interact_id: Id, nodes: &[ResolvedNode]) -> Option<&ResolvedNode> {
    for node in nodes {
        if let ResolvedNode::Interact { id, .. } = node {
            if *id == interact_id {
                return Some(node);
            }
        }
    }
    None
}
