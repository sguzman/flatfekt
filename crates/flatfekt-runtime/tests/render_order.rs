use flatfekt_runtime::{
  RenderKind,
  render_sort_key
};

#[test]
fn render_sort_key_is_deterministic() {
  let a = render_sort_key(
    "a",
    0.0,
    RenderKind::Sprite
  );
  let b = render_sort_key(
    "a",
    0.0,
    RenderKind::Sprite
  );
  assert_eq!(a, b);
}

#[test]
fn render_sort_key_sorts_stably() {
  let mut items = vec![
    ("b", 1.0, RenderKind::Text),
    ("a", 1.0, RenderKind::Sprite),
    ("a", 0.0, RenderKind::Sprite),
    ("a", 0.0, RenderKind::Shape),
  ];

  items.sort_by_key(|(id, z, kind)| {
    render_sort_key(id, *z, *kind)
  });

  // Primary: z (via bits), then kind,
  // then id tie-break.
  assert_eq!(items[0].0, "a");
  assert_eq!(
    items[0].2,
    RenderKind::Shape
  );
  assert_eq!(
    items[1].2,
    RenderKind::Sprite
  );
}
