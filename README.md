# Cape

### *(yet another) Desktop UI library in Rust*

*\*inspired by React, Moxie, and Crochet*

---

- No `RefCell`
- No global state
- Reactive & declarative
- Caching helpers
- Retained DOM-like backing tree (reconciliation)

---

```rust
#[cape::ui]
fn counter(cx: &mut Cx) -> impl IntoNode {
    let count = cx.state(|| 0);

    Column::new()
        .spacing(5.)
        .child(format!("Count: {}", cx.at(count)))
        .child(
            Row::new()
                .spacing(5.)
                .child(
                    Button::new(cx)
                        .child("Increment")
                        .on_click(move |cx, _| *cx.at(count) += 1),
                )
                .child(
                    Button::new(cx)
                        .child("Decrement")
                        .on_click(move |cx, _| *cx.at(count) -= 1),
                ),
        )
}
```

---

### What's missing

- Long list views (direct DOM mutation)
- Z-order (implies a simple layer compositor)
- Correct text input (IME)
- Accessibility (First priority will be Microsoft UIAutomation)

Not quite as sophisticated as Crochet, more based around empirical needs and wants.
