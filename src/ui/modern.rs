use iced::{
    Color, Element, Length, Pixels,
    widget::{Container, column, container, image, row, space, table, text},
};

use crate::{Message, State, api::departures::Mode, state::DisplayDeparture};

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

    // The destination uses the base font size; the departure time is
    // larger (destination : departure time = 0.72 : 1).
    let departure_font_size = (font_size as f32 / 0.72).round() as u32;

    // The destination is rendered in the configured font at the base size.
    let destination_text = |txt: String| {
        text(txt)
            .font(state.modern_ui.font)
            .size(font_size)
            .color(black)
    };

    // The departure time and the line name are rendered in Fira Sans Bold.
    // The TTF's family name is "Fira Sans" (bold is a weight, not a
    // family), so the font is matched by family name + weight.
    let departure_font = iced::Font {
        family: iced::font::Family::Name("Fira Sans"),
        weight: iced::font::Weight::Bold,
        ..iced::Font::default()
    };

    // The line name uses the departure time font at the base size.
    let line_text = |txt: String| text(txt).font(departure_font).size(font_size).color(black);

    // The departure time is rendered as `{minutes}′` in the full size.
    let departure_cell = |d: &DisplayDeparture| -> Element<'_, Message> {
        text(if d.cancelled {
            String::new()
        } else {
            format!("{}′", d.when)
        })
        .font(departure_font)
        .size(departure_font_size)
        .color(black)
        .into()
    };

    // Icons scale with the font size (2x the font size in height), keeping
    // their aspect ratio: the width shrinks to match.
    let icon_height = Pixels::from(font_size * 2);

    // Determine the line icon from the departure's mode:
    // - S-Bahn / U-Bahn (`Mode::Train`): the icon matching the line name
    //   (`S41`, `U7`, ...), replacing the line text when present.
    // - Bus / Tram: the generic `bus` / `tram` icon, shown together with
    //   the line name.
    let line_icon = |d: &DisplayDeparture| -> Option<image::Handle> {
        let icons = &state.modern_ui.line_icons;
        match d.mode {
            Mode::Train => icons.get(&d.line).cloned(),
            Mode::Bus => icons.get("bus").cloned(),
            Mode::Tram => icons.get("tram").cloned(),
            Mode::Unknown => None,
        }
    };

    let line_cell = |d: &DisplayDeparture| -> Element<'_, Message> {
        match d.mode {
            // S-Bahn / U-Bahn: icon instead of the line text when available.
            Mode::Train => match line_icon(d) {
                Some(icon) => image::viewer(icon).height(icon_height).into(),
                None => line_text(d.line.clone())
                    .align_x(iced::Alignment::Start)
                    .into(),
            },
            // Bus / Tram: icon followed by the line name.
            Mode::Bus | Mode::Tram => match line_icon(d) {
                Some(icon) => row![
                    Element::from(image::viewer(icon).height(icon_height)),
                    Element::from(line_text(d.line.clone())),
                ]
                .spacing(2)
                .align_y(iced::Alignment::Center)
                .into(),
                None => line_text(d.line.clone())
                    .align_x(iced::Alignment::Start)
                    .into(),
            },
            // Unknown mode: plain line text.
            Mode::Unknown => line_text(d.line.clone())
                .align_x(iced::Alignment::Start)
                .into(),
        }
    };

    let stop_name = state
        .departures
        .departures
        .keys()
        .map(|s| s.name.clone())
        .collect::<Vec<_>>()
        .join(", ");

    let products = state
        .departures
        .departures
        .keys()
        .next()
        .and_then(|s| s.products);

    let service_icons: Vec<image::Handle> = products
        .map(|p| {
            let icons = &state.modern_ui.line_icons;
            [
                (p.bus == Some(true), "bus"),
                (p.tram == Some(true), "tram"),
                (p.suburban == Some(true), "sbahn"),
                (p.subway == Some(true), "ubahn"),
                (p.regional == Some(true), "bahn"),
            ]
            .iter()
            .filter(|(enabled, _)| *enabled)
            .filter_map(|(_, key)| icons.get(*key).cloned())
            .collect()
        })
        .unwrap_or_default();

    let stop_name_cell: Element<'_, Message> = Container::new(
        text(stop_name)
            .font(departure_font)
            .size(font_size)
            .color(black),
    )
    .padding(iced::padding::left(15))
    .into();

    let bvg_logo_cell: Element<'_, Message> =
        Container::new(image::viewer(state.bvg_logo.clone()).height(icon_height))
            .padding(iced::padding::right(10))
            .into();

    let mut header_children: Vec<Element<'_, Message>> = vec![stop_name_cell];
    for icon in service_icons {
        header_children.push(Element::from(image::viewer(icon).height(icon_height)));
    }

    header_children.push(Element::from(space().width(Length::Fill)));
    header_children.push(bvg_logo_cell);

    let header: Element<'_, Message> = row(header_children)
        .spacing(font_size / 2)
        .align_y(iced::Alignment::End)
        .width(Length::Fill)
        .into();

    let columns: [iced::widget::table::Column<'_, '_, &DisplayDeparture, Message>; 4] = [
        table::column(text(""), line_cell).width(Length::Shrink),
        // Empty spacer column providing the gap between the line name
        // and the destination (half the icon height).
        table::column(text(""), |_| text("")).width(Pixels::from(font_size)),
        // Vertically centered so the destination text aligns with the
        // icon / line name row (the icons are taller than the text).
        table::column(text(""), |d: &DisplayDeparture| {
            destination_text(d.direction.clone())
        })
        .width(Length::Fill)
        .align_y(iced::Alignment::Center),
        table::column(text(""), departure_cell)
            .width(Length::Shrink)
            .align_x(iced::Alignment::End)
            .align_y(iced::Alignment::Center),
    ];

    let table: Element<'_, Message> = Container::new(
        table(
            columns,
            rows.iter()
                .copied()
                .take(state.config.modern_ui.max_display_rows),
        )
        .separator_x(Pixels::ZERO)
        .separator_y(Pixels::from(2))
        .padding_y(Pixels::from(10)),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(5)
    .style(move |_| container::background(Color::WHITE))
    .into();

    let inner = column![header, table]
        .spacing(5)
        .width(Length::Fill)
        .height(Length::Fill);

    Container::new(inner)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(5)
        .style(move |_| container::background(Color::WHITE))
        .into()
}
