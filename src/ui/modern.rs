use iced::{
    Color, Element, Length, Pixels,
    widget::{Container, column, container, image, row, space, text},
};

use crate::{
    Message, State,
    api::departures::Product,
    assets::LineIcon,
    state::{DisplayDeparture, Station},
    ui::first_stop,
};

#[allow(clippy::too_many_lines)]
pub fn modern_view(state: &State) -> Element<'_, Message> {
    let black = Color::BLACK;
    let font_size = state.config.modern_ui.font_size;

    let (stop, rows): (Station, Vec<&DisplayDeparture>) = match first_stop(state) {
        Some((s, rows)) => (s.clone(), rows.iter().collect()),
        None => (Station::default(), Vec::new()),
    };

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let departure_font_size = (font_size as f32 / 0.72).round() as u32;

    let destination_text = |txt: String| {
        text(txt)
            .font(state.modern_ui.font)
            .size(font_size)
            .color(black)
    };

    let departure_font = iced::Font {
        family: iced::font::Family::Name("Fira Sans"),
        weight: iced::font::Weight::Bold,
        ..iced::Font::default()
    };

    let line_text = |txt: String| text(txt).font(departure_font).size(font_size).color(black);

    let departure_cell = |d: &DisplayDeparture| -> Element<'_, Message> {
        text(if d.cancelled {
            format!("{} fällt aus", d.mode.as_german_str())
        } else {
            format!("{}′", d.when)
        })
        .font(departure_font)
        .size(departure_font_size)
        .color(black)
        .into()
    };

    let icon_height = Pixels::from(font_size * 2);

    let train_icon_height = Pixels::from(font_size * 3 / 2);

    let line_icon = |d: &DisplayDeparture| -> Option<&LineIcon> {
        let icons = &state.modern_ui.line_icons;
        match d.product {
            Product::Suburban | Product::Subway | Product::Regional => icons.get(&d.line),
            Product::Bus => icons.get("bus"),
            Product::Tram => icons.get("tram"),
            _ => None,
        }
    };

    let line_cell = |d: &DisplayDeparture| -> Element<'_, Message> {
        match d.product {
            Product::Suburban | Product::Subway | Product::Regional => match line_icon(d) {
                Some(icon) => image::viewer(icon.handle.clone())
                    .height(train_icon_height)
                    .into(),
                None => line_text(d.line.clone())
                    .align_x(iced::Alignment::Start)
                    .into(),
            },
            Product::Bus | Product::Tram => match line_icon(d) {
                Some(icon) => row![
                    Element::from(image::viewer(icon.handle.clone()).height(icon_height)),
                    Element::from(line_text(format!(" {}", d.line))),
                ]
                .spacing(2)
                .align_y(iced::Alignment::Center)
                .into(),
                None => line_text(d.line.clone())
                    .align_x(iced::Alignment::Start)
                    .into(),
            },
            _ => line_text(d.line.clone())
                .align_x(iced::Alignment::Start)
                .into(),
        }
    };

    let line_cell_width = |d: &DisplayDeparture| -> u32 {
        #[allow(clippy::cast_possible_truncation)]
        let mut line_chars_count = d.line.chars().count() as u32;
        if matches!(d.product, Product::Bus | Product::Tram) {
            line_chars_count += 1;
        }

        let text_width = line_chars_count * font_size * 3 / 5;
        let icon_width =
            |target_height: u32, width: u32, height: u32| (target_height * width) / height.max(1);
        match (d.product, line_icon(d)) {
            (Product::Suburban | Product::Subway | Product::Regional, Some(icon)) => {
                icon_width(font_size * 3 / 2, icon.width, icon.height)
            }
            (Product::Bus | Product::Tram, Some(icon)) => {
                icon_width(font_size * 2, icon.width, icon.height) + 2 + text_width
            }
            _ => text_width,
        }
    };

    let service_icons: Vec<image::Handle> = stop
        .products
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
            .filter_map(|(_, key)| icons.get(*key).map(|icon| icon.handle.clone()))
            .collect()
        })
        .unwrap_or_default();

    let stop_name_cell: Element<'_, Message> = Container::new(
        text(stop.name)
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

    let visible_rows: Vec<&DisplayDeparture> = rows
        .iter()
        .copied()
        .take(state.config.modern_ui.max_display_rows)
        .collect();

    let separator_color = iced::theme::Theme::Light
        .extended_palette()
        .background
        .strong
        .color;

    let line_col_width = visible_rows
        .iter()
        .copied()
        .map(line_cell_width)
        .max()
        .unwrap_or(0);

    let departure_rows: Vec<Element<'_, Message>> = visible_rows
        .iter()
        .map(|d| {
            let line = Container::new(line_cell(d))
                .width(Pixels::from(line_col_width))
                .height(Length::Fill)
                .align_x(iced::Alignment::Start)
                .align_y(iced::Alignment::Center);
            let destination =
                Container::new(destination_text(d.direction.clone())).width(Length::Fill);
            let departure = Container::new(departure_cell(d)).align_x(iced::Alignment::End);

            row![
                line,
                space().width(Pixels::from(font_size)),
                destination,
                departure,
            ]
            .align_y(iced::Alignment::Center)
            .padding(iced::padding::vertical(Pixels::from(10)))
            .into()
        })
        .collect();

    let separator = || {
        Container::new(space().width(Length::Fill))
            .width(Length::Fill)
            .height(Pixels::from(2))
            .padding(0)
            .style(move |_| container::background(separator_color))
            .into()
    };

    let mut departure_children: Vec<Element<'_, Message>> =
        Vec::with_capacity(2 * departure_rows.len());
    for row in departure_rows {
        departure_children.push(separator());
        departure_children.push(row);
    }

    let departures: Element<'_, Message> = Container::new(column(departure_children))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(5)
        .style(move |_| container::background(Color::WHITE))
        .into();

    let inner = column![header, departures]
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
