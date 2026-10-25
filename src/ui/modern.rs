use iced::{
    Color, Element, Length, Pixels,
    widget::{Container, container, table, text},
};

use crate::{Message, State, state::DisplayDeparture};

/// Minimal, single-screen UI: white background, black text.
/// Only the departures table is shown — line (left aligned),
/// destination, and time as `{minutes}′ₘ`.
pub fn modern_view(state: &State) -> Element<'_, Message> {
    let black = Color::BLACK;
    let font_size = state.config.modern_ui.font_size;

    // Flatten every stop's departures into a single set of table rows.
    let rows: Vec<&DisplayDeparture> = state
        .departures
        .departures
        .values()
        .flat_map(|v| v.iter())
        .collect();

    let cell_text = |txt: String| {
        text(txt).font(state.modern_ui.font).size(font_size).color(black)
    };

    let columns: [iced::widget::table::Column<'_, '_, &DisplayDeparture, Message>; 3] = [
        table::column(text(""), |d: &DisplayDeparture| {
            cell_text(d.line.clone()).align_x(iced::Alignment::Start)
        })
        .width(Length::Shrink),
        table::column(text(""), |d: &DisplayDeparture| cell_text(d.direction.clone()))
            .width(Length::Fill),
        table::column(text(""), |d: &DisplayDeparture| {
            cell_text(if d.cancelled {
                String::new()
            } else {
                format!("{}′ₘ", d.when)
            })
        })
        .width(Length::Shrink)
        .align_x(iced::Alignment::End),
    ];

    Container::new(
        table(
            columns,
            rows.iter()
                .copied()
                .take(state.config.modern_ui.max_display_rows),
        )
        .separator_x(Pixels::ZERO)
        .separator_y(Pixels::from(2)),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(5)
    .style(move |_| container::background(Color::WHITE))
    .into()
}
