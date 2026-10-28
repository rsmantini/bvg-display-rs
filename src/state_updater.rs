//! Thin message-handling glue between the iced app and the rest of the crate.

use std::time::Duration;

use iced::{Subscription, Task, exit, keyboard};
use tokio::time::sleep;

use crate::{
    fetch::fetch_departures,
    state::{Departures, State},
};

#[derive(Debug, Clone)]
pub enum Message {
    FetchDepartures,
    DeparturesFetched(Departures),
    Quit,
    RetroUiBlinkTick,
    ToggleUi,
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::FetchDepartures => Task::perform(
            fetch_departures(state.bvg_client.clone(), state.config.clone()),
            |result| match result {
                Ok(departures) => Message::DeparturesFetched(departures),
                Err(e) => {
                    eprintln!("Error fetching departures: {e:?}");
                    Message::FetchDepartures
                }
            },
        ),
        Message::DeparturesFetched(departures) => {
            state.ingest(departures);
            Task::perform(sleep(Duration::from_secs(10)), |()| {
                Message::FetchDepartures
            })
        }
        Message::Quit => exit(),
        Message::ToggleUi => {
            state.selected_ui = state.selected_ui.toggle();
            Task::none()
        }
        Message::RetroUiBlinkTick => {
            state.retro_ui.blink_tick += 1;
            Task::none()
        }
    }
}

pub fn escape_subscription(_state: &State) -> Subscription<Message> {
    keyboard::listen().filter_map(|event| {
        matches!(
            event,
            keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }
        )
        .then_some(Message::Quit)
    })
}
