use criterion::{criterion_group, criterion_main, Criterion};

fn resolve_text(res: &mut cape::node::Resources) {
    cape::node::text("sT;vajFDY@B/ax k[ +Tq:10P $/zn_*BRWBl8y4 LEq?n|'o8")
        .resolve(res)
        .unwrap();
}

fn resolve_rect(res: &mut cape::node::Resources) {
    cape::node::rectangle(
        cape::size2(25.75, 50.5),
        [5.; 4],
        cape::node::Paint::Solid(cape::rgb(255, 0, 255)),
        1.75,
        cape::node::Paint::Solid(cape::rgb(0, 255, 0)),
    )
    .resolve(res)
    .unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    cape::skia::shaper::icu::init();

    let mut resources = cape::node::Resources {
        fonts: Default::default(),
        fallback_text_size: 13.,
        fallback_text_fill: cape::node::Paint::Solid(cape::Color::new(1., 1., 1., 1.)),
        shaper_cache: Default::default(),
        font_cache: Default::default(),
    };

    resources
        .load_font_data(
            "sans-serif",
            std::sync::Arc::new(include_bytes!("NotoSans-Regular.ttf").to_vec()),
            None,
        )
        .unwrap();

    c.bench_function("resolve text node", |b| {
        b.iter(|| resolve_text(&mut resources))
    });

    c.bench_function("resolve rect node", |b| {
        b.iter(|| resolve_rect(&mut resources))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
