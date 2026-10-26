//! View dispatch for the iced app.

pub mod modern;
pub mod retro;

use crate::{
    config::Ui,
    state::{DisplayDeparture, State, Station},
    state_updater::Message,
};
use iced::Element;

/// Pick the view based on the currently selected UI and make a click
/// anywhere on the screen toggle between the UIs.
pub fn view(state: &State) -> Element<'_, Message> {
    let content = if state.selected_ui == Ui::Modern {
        modern::modern_view(state)
    } else {
        retro::retro_view(state)
    };

    iced::widget::MouseArea::new(content)
        .on_press(Message::ToggleUi)
        .into()
}

pub(crate) fn first_stop(state: &State) -> Option<(&Station, &[DisplayDeparture])> {
    state
        .departures
        .stops
        .iter()
        .next()
        .map(|(k, v)| (k, v.as_slice()))
}
