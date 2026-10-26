# Plan: Replace the `table` widget in the Modern UI with manual rows/columns

Goal: build the departure rows in `src/ui/modern.rs` manually (like the header element),
instead of using `iced::widget::table`, while preserving the exact visual layout.

## What the `table` widget provides today (must be preserved)

1. **Aligned columns** — each `Shrink` column's width is the widest cell in that column,
   so line name, destination, and departure time line up vertically across rows.
2. **Per-column layout**
   - line cell: `Shrink`, left-aligned
   - spacer column: fixed `font_size` px (half the icon height, since icon height = `2 * font_size`)
   - destination: `Fill`, vertically centered
   - departure: `Shrink`, right-aligned — its right edge sits at the container's right edge
     because the `Fill` destination column absorbs the slack.
3. **Row geometry** — 2 px between rows (`separator_y`), `padding_y(10)`, wrapped in a
   `Container` with `Fill x Fill`, `padding(5)` and a white background.

## Key constraint

iced 0.14 exposes **no public API** to measure text widths
(`iced::text::horizontal_width` does not exist) or to query an
`image::Handle`'s dimensions. Since manual rows need explicit pixel widths to keep
the line-name column aligned, the icon width must be derived from the original
aspect ratio. The icons are loaded from PNG files (`src/main.rs`,
`load_line_icons`), so width/height can be read directly from the PNG header
(8-byte signature + IHDR chunk, bytes 16–23) with **zero new dependencies**.

## Steps

### 1. `src/main.rs` — store icon dimensions alongside the handles

- Add a small helper:
  `fn png_size(bytes: &[u8]) -> Option<(u32, u32)>`
  (validate the PNG signature, read the big-endian width/height from IHDR).
- Change `load_line_icons` to return `RapidHashMap<String, (image::Handle, u32, u32)>`
  (handle, width, height), falling back to `(1, 1)` if the header can't be parsed.

### 2. `src/state.rs` — update the state type

- `ModernUiState::line_icons` becomes
  `RapidHashMap<String, (image::Handle, u32, u32)>`
  (handle + original pixel size, used to compute the scaled icon width).
- `State::new`'s parameter type follows.

### 3. `src/ui/modern.rs` — build rows/columns manually

- Drop the `table` import and the `columns`/`table` block.
- Keep the existing `line_cell`, `destination_text`, and `departure_cell` closures.
- Change `line_icon` to return `Option<&(image::Handle, u32, u32)>` so callers
  get the original dimensions; adjust `line_cell` accordingly.
- Compute the **common line-column width** (max over all *visible* rows, i.e. after
  `take(max_display_rows)`), mirroring the table's `Shrink`-column semantics:
  - Train + icon → scaled icon width: `icon_height * orig_w / orig_h`
  - Bus/Tram + icon → scaled icon width + 2 (icon/text spacing) + estimated line-text width
  - Text only → estimated width: `chars * font_size * 3 / 5`
    (Fira Sans Bold digit/letter width ≈ 0.6 em; fallback only for rows without an icon —
    the column is dominated by the real icon widths).
- Build each visible row as:

  ```rust
  row![
      line_cell(d).width(Pixels::from(line_col_width)).align_x(iced::Alignment::Start),
      space().width(Pixels::from(font_size)),                 // the old spacer column
      destination_text(d.direction.clone()).width(Length::Fill),
      departure_cell(d).align_x(iced::Alignment::End),
  ]
  .align_y(iced::Alignment::Center)
  ```

  The departure cell needs **no explicit width**: as the last `Shrink` cell after the
  `Fill` destination, its right edge lands at the row's right edge for every row —
  exactly like the table.

- Wrap the rows in `column(rows).spacing(2).padding_y(10)` inside the same
  `Container` (`.width(Fill).height(Fill).padding(5)`, white background) as before.

## Result / trade-offs

- Visual behavior is preserved: columns stay aligned, the 2 px row spacing, the
  `font_size` gap after the line name, 10 px vertical padding, and 5 px container
  padding all match the table version.
- Right-edge alignment of the departure times falls out naturally from
  `Fill` + right-aligned last cell — no text measurement needed there.
- The only approximation is the estimated width for *text-only* line cells
  (rows without an icon); it only affects those rows, and only by ~±a pixel.
- Verification: `cargo check` after implementation.
