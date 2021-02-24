# Cape

### *(yet another) Desktop UI library in Rust*

*\*inspired by React, Moxie, Crochet, Flutter, and Iced*

At its core, Cape provides two main things:

1. Functional state storage; state retrieved via `use_state`, persistent between tree rebuilds.
2. The node primitive; a minimal type to build a DOM from, providing layout, drawing, and interaction.

## Conifer

The supporting toolkit built on Cape is called Conifer.

Conifer is a modular, theme-able UI toolkit, essentially replacing the need for a separate styling interface:

```rust
fn my_button_theme(node: conifer::ButtonBuilder) -> deco::DecoratedNode {
    // Decorators inflict different visual styles onto a base node.
    // You can also make your own decorators.
    // Decorators are actually just nodes stacked onto the base node.
    // Because of this, decorators can do fully custom drawing directly into a Skia canvas and can store state and have behavior, so you can get wild.
    node.padding(cape::Sides2::new(10., 10., 10., 10.))
        .decorate()
        .decorator_if(conifer::ButtonState::NORMAL, conifer::deco::background(conifer::Paint::Palette("button-normal")))
		.decorator_if(conifer::ButtonState::HOVERED, conifer::deco::background(conifer::Paint::Palette("button-hovered")))
		.decorator_if(conifer::ButtonState::PRESSED, conifer::deco::background(conifer::Paint::Palette("button-pressed")))
		.decorator(conifer::deco::air::shadow(true))
		.decorator(conifer::deco::air::border(5.0, conifer::Paint::Palette("border")))
}

#[ui]
fn test() -> impl ToNode {
    column()
        .child(
            button()
                .child("Button 1")
                .on_click(|_| println!("Hello, world!"))
                .decorated(my_button_theme) // Apply a list of decorators to this button
        )
        .child(
            button()
                .child("Button 2")
                .on_click(|_| println!("Hey!"))
                .decorated(conifer::deco::air::button) // Apply a pre-made set of decorators to achieve a certain theme (air theme).
        )
}
```

## **Examples**

These are the first 4 tasks from 7GUIs.

*Counter*

```rust
#[cape::ui]
fn counter() -> impl ToNode {
    let count = use_state(|| 0);

    column()
        .child(
            button()
                .child("Click me!")
                .on_click(move |_| count.set(count.get() + 1))
        )
        .child(format!("Count: {}", count.get()))
}
```

*Temperature Converter*

```rust
#[cape::ui]
fn temperature_converter() -> impl ToNode {
    let celsius = use_state(|| 0.);

    row()
        .spacing(5.)
        .child(float_box().state(celsius))
        .child("Celsius")
        .child("=")
        .child(
            float_box()
                .value(celsius.get() * (9./5.) + 32.)
                .on_change(move |val| {
                    celsius.set((val - 32.) * (5./9.));
                })
        )
        .child("Fahrenheit")
}
```

*Flight Booker*

```rust
enum Flight {
    OneWay,
    Return,
}

/* impl Display for Flight */
/* impl FromStr for Flight */

#[cape::ui]
fn flight_booker() -> impl ToNode {
    let flight = use_state(|| Flight::OneWay);
    let start_date = use_state(|| String::from("1.1.2020"));
    let end_date = use_state(|| String::from("1.2.2020"));

    column()
        .spacing(5.)
        .child(
            combo_box()
                .values(&[Flight::OneWay.to_string(), Flight::Return.to_string()])
                .selected(flight.get() as _)
                .on_change(move |val, _idx| {
                    flight.set(Flight::from_str(val));
                })
        )
        .children(vec![
            text_box().state(start_date),
            text_box()
                .state(end_date)
                .disabled(flight.get() == Flight::OneWay),
        ])
        .child(button().child("Book").on_click(move |_| {
            let flight = flight.to_string();
            let end_date = if flight.get() == Flight::OneWay {
                "".into()
            } else {
                format!("to {}", end_date.get());
            };

            println!("Booked {} from {} {}", flight, start_date.get(), end_date);
        }))
}
```

*Timer*

```rust
#[cape::ui]
fn timer() -> impl ToNode {
    let duration = use_state(|| 10.);
    let elapsed = use_state(|| 0.);
    let start = use_state(|| std::time::Instant::now());

    on_render(move || {
        elapsed.with(|elapsed| {
            *elapsed = (std::time::Instant::now() - start).as_secs_f32().min(duration.get());
        });
    });

    column()
        .spacing(5.)
        .child("Elapsed:")
        .child(progress_bar().amount(elapsed.get() / duration.get()))
        .child(format!("{}s", elapsed.get()))
        .child("Duration:")
        .child(slider().scale(0.0..20.0).value(duration.get()).on_change(move |val| {
            if (duration.get() - elapsed.get()).abs() < std::f32::EPSILON {
                elapsed.set(0.);
            }

            duration.set(val);
        }))
        .child(button().child("reset").on_click(move |_| {
            elapsed.set(0.);
        }))
}
```

## **Under the hood**

Cape implements React hooks in a similar way to Moxie; using `topo`, call sites (from `#[track_caller]`) are used as stable identifiers. It also adds a `Key` in order to resolve ambiguities such as items in collections. This is demonstrated below.

```rust
#[cape::unique_ui]
fn slider(key: &usize) -> Node {
    let amount = use_state(|| 0.0);
    // ...
}

#[cape::ui]
fn sliders() -> Node {
    let next_id = use_state(|| 0);
    let all_ids = use_state(|| Vec::new());

    // ...
    
    column()
        .children(all_ids.with(|all_ids| all_ids.iter().map(|id| context(&0, || slider(&id)))))
}
```

Without `key: &usize`, the above example would not work correctly; should the sliders be reordered or removed, subsequent calls to `slider` would receive the incorrect state.

The `context` wrapper provides a context for the keys. Should the sliders be repeated twice, the second time, a second context can be used to ensure the keys from the first context don't overlap.

This can also be used for things other than just state. For example, caching;

```rust
#[cape::ui]
fn lab_image(key: Key, rgb: &RgbImage) -> Node {
    // Whenever `rgb` changes, the cached value is updated with the closure.
    let image = use_cache(rgb, |rgb| rgb.convert_colorspace());

    image.with(|image| {
        // Immutable access only.
        image.get_pixel(53, 332);
    });

    // ...
}
```

All this data is stored in thread-local tables.

The only rule for `use_state` and `use_cache` is that you can only access the same state/cache once at a time.

You can also create your own hooks by utilizing the `Id` type to uniquely identify widgets and, for example, use as the key in a hash table.

Handles to states and caches can be copied around freely.

```rust
fn display_state(count: StateAccessor<i32>) -> impl ToNode {
    format!("Count: {}", count.get())
}

#[cape::ui]
fn example(key: Key) -> Node {
    let count = use_state(|| 0);

    column()
        .child(display_state(count))
        .child(/* ... */)
}
```

The same can be done with the cache via `CacheAccessor`.

The hooks can also be used for side-effect purposes:
```rust
#[ui]
fn animated() -> Node {
    let animation = use_state(|| Animation::new());

    on_render(move || {
        // Called every render.
        animation.with(|animation| animation.update());
    });

    on_lifecycle(move |e| {
        if let LifecycleEvent::Create = e {
            // Called only once on the first time this is instantiated.
            animation.with(|animation| animation.initialize(0., 10.));
        }
    });

    // ...
}
```

(*Note:* `CacheAccessor::get` / `StateAccessor::get` require that the inner type is `Clone`able, otherwise it can be accessed by reference through `with`)

All rendering is based on Skia. Any windowing and Skia backend can be used, but a Winit + Vulkan backend (via Skulpin) is provided.

Everything is built on the `Node` primitives;
- `Node::Interact`: Receives interaction events (clicking, typing, focus, etc).
- `Node::Capture`: Receives the corresponding `&ResolvedNode::Capture` (i.e. a callback for directly observing the finalized UI tree.)
- `Node::Layout`: Positions and resizes its child `Node`s.
- `Node::Text`: Shaped text (via Harfbuzz and Skia.)
- `Node::Rectangle`: Rounded rectangle with a border.
- `Node::Draw`: Arbitrary drawing with Skia (i.e. a `&mut Canvas` callback.)

For each of these variants, there is a corresponding variant in `ResolvedNode`. A `ResolvedNode` is the same as a node, except it contains the real rectangle of the node and all references to resources have been resolved.

These primitives aim to be sufficient enough to build any widget possible.

The backend can choose how to handle the `Node` tree, but the `skulpin` backend does the following:
1. Resolve the `Node` tree into a `ResolvedNode` tree.
2. Perform layout on the `ResolvedNode` tree.
3. Flatten the `ResolvedNode` tree by z-index.
4. Use the flattened `ResolvedNode` tree to send callbacks and render.

### Planned Optimizations

- Keeping track of mutable state accesses each frame and using that to mask out unchanged `Node`s. This is an alternative to tree diffing.
- Use of memoization (`use_cache`/`use_once`) wherever possible.
- Partial repaint (depends on partial `Node` updating, i.e. the first item in this list.)

## **Status**

Purely experimental and closed development.

Since there are already `N` Rust UI libraries and this one doesn't really stand out, most of the development will be private and this repository will only be periodically updated.

Do not use this for any remotely serious projects.
