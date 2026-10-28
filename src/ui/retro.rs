use std::time::Duration;

use iced::{
    Color, Element, Font, Length, Pixels, Subscription, font, time,
    widget::{Container, column, container, image, row, space, table, text},
};

use crate::{
    Message, State,
    state::{DisplayDeparture, Station},
    ui::first_stop,
};

#[allow(clippy::too_many_lines)]
pub fn retro_view(state: &State) -> Element<'_, Message> {
    let black = Color::BLACK;

    // Shared column widths so the header row and the table stay aligned.
    // `Ziel` fills the remaining space; the others size to their content.
    let col_line = Length::Shrink;
    let col_destination = Length::Fill;
    let col_departure = Length::Shrink;

    let (stop, rows): (Station, Vec<&DisplayDeparture>) = match first_stop(state) {
        Some((s, rows)) => (s.clone(), rows.iter().collect()),
        None => (Station::default(), Vec::new()),
    };

    // Box 1: white background, black text — the column titles.
    // Each title is wrapped in a container with padding that mirrors the
    // table's cell padding so the header lines up with the columns.
    let header_style = |_: &iced::Theme| container::background(Color::WHITE);

    let font_size = state.config.retro_ui.font_size;
    let bold_black_text = |txt: String| {
        text(txt)
            .font(Font {
                weight: font::Weight::Bold,
                ..Font::DEFAULT
            })
            .color(black)
            .size(font_size)
    };

    let header = row![
        Element::from(
            Container::new(bold_black_text("Linie".to_string()))
                .width(col_line)
                .padding(iced::padding::all(10))
                .style(header_style),
        ),
        Element::from(
            Container::new(bold_black_text("Ziel".to_string()))
                .width(col_destination)
                .padding(iced::padding::all(10))
                .style(header_style),
        ),
        Element::from(
            Container::new(bold_black_text("Abfahrt".to_string()))
                .width(col_departure)
                .padding(iced::padding::all(10))
                .style(header_style),
        ),
    ]
    .spacing(1) // matches the table's separator_x
    .width(Length::Fill);

    let yellow = Color::from_rgb(1.0, 0.67, 0.02);
    let blink_tick = state.retro_ui.blink_tick;

    let departures_text = |txt: String, blink, font_size| {
        let txt = text(txt).font(state.retro_ui.font).size(font_size);
        if blink && blink_tick.is_multiple_of(2) {
            txt.color(black)
        } else {
            txt.color(yellow)
        }
    };

    // Box 2: black background, yellow text — the departure table (no header).
    let columns: [iced::widget::table::Column<'_, '_, &DisplayDeparture, Message>; 3] = [
        table::column(text(""), |d: &DisplayDeparture| {
            departures_text(d.line.clone(), false, font_size).width(col_line)
        }),
        table::column(text(""), |d: &DisplayDeparture| {
            departures_text(d.direction.clone(), d.arriving, font_size)
        })
        .width(col_destination),
        table::column(text(""), |d: &DisplayDeparture| {
            let mut font_size = font_size;
            let txt = if d.cancelled {
                if state.config.retro_ui.reduce_canceled_text {
                    font_size /= 2;
                }
                format!("{} fällt aus", d.mode.as_german_str())
            } else if d.arriving {
                "jetzt".to_string()
            } else {
                format!("{} min", d.when)
            };
            departures_text(txt, d.arriving, font_size)
        })
        .width(col_departure)
        .align_x(iced::Alignment::End),
    ];

    let table: Element<'_, Message> = Container::new(
        table(
            columns,
            rows.iter()
                .copied()
                .take(state.config.retro_ui.max_display_rows),
        )
        .separator(Pixels::ZERO),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(5)
    .style(move |_| container::background(black))
    .clip(true)
    .into();

    let last_update = state
        .departures
        .last_update
        .map(|t| format!("{}", t.format("%H:%M:%S")))
        .unwrap_or_default();

    let logo = image::viewer(state.bvg_logo.clone()).height(Length::Fill);

    let footer: Element<'_, Message> = Container::new(
        row![
            Container::new(bold_black_text(stop.name)).padding(iced::padding::all(15)),
            space().width(Length::Fill),
            Container::new(text(last_update).color(black)).padding(iced::padding::all(15)),
            logo,
        ]
        .spacing(10),
    )
    .width(Length::Fill)
    .height(Length::Shrink)
    .style(|_| container::background(Color::WHITE))
    .into();

    // Stack the three boxes, each occupying the full width.
    let inner = column![
        Element::from(
            Container::new(header)
                .width(Length::Fill)
                .padding(5)
                .style(|_| container::background(Color::WHITE)),
        ),
        table,
        footer,
    ]
    .spacing(5)
    .width(Length::Fill)
    .height(Length::Fill);

    Container::new(inner)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .style(|_| container::background(Color::BLACK))
        .into()
}

pub fn blink_tick_subscription(state: &State) -> Subscription<Message> {
    time::every(Duration::from_millis(
        state.config.retro_ui.blink_speed_millis,
    ))
    .map(|_| Message::RetroUiBlinkTick)
}
