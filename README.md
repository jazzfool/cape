# Cape

## Experimental: Reactive, statically-known UIs.

*\*inspired by React, Moxie, Crochet, and SwiftUI*

The main ideas are:
- A UI tree encoded in the type system to make diffing easy (based on SwiftUI)
- A node primitive to enact as the final form of the UI
- React-like hook state management

```rust
type Counter =
    ColumnLayout<(
        Text,
        RowLayout<(
            Button<Text>,
            Button<Text>,
        )>,
    )>;

#[ui]
fn counter() -> Counter {
    let count = use_state(|| 0);

    Column {
        spacing: 5.0,
        ..Column::default()
    }
    .children((
        Text::from(format!("Count: {}", count.get())),
        Row {
            spacing: 5.0,
            ..Row::default()
        }
        .children((
            button
                .props()
                .child("Increment")
                .on_click(move |_| count.set(count.get() + 1))
                .build(),
            button
                .props()
                .child("Decrement")
                .on_click(move |_| count.set(count.get() - 1))
                .build(),
        )),
    ))
}
```

Obviously the entire UI cannot be known at compile time. Some helper types are given for dynamic content: `AnyNode`, `DynamicList<T>`, `Conditional<A, B>`, and `impl`s for `Option<T>`.

`AnyNode` should be avoided in favour of `DynamicList<T>` where possible because `DynamicList<T>` does an element-wise `Merge`.

Types that are the building blocks of the static UI tree are just implementors of `Merge` and `Expand`.

- `Merge` merges the next UI tree into the current one, in place. Tree state can be retained and partially updated efficiently.
- `Expand` expands the static UI tree into a tree of node primitives.

Why go through all this trouble just to represent the UI statically? In short, it makes tree diffing and other optimizations much simpler and faster. Diffing (`Merge`) is as simple as comparing fields. As a bonus, all the non-dynamic parts of the entire UI are strongly-typed so that they can be dealt with at a semantic level instead of as an opaque DOM.
