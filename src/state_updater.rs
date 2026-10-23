use std::{error::Error, sync::Arc, time::Duration};

use chrono::{Local, TimeZone};
use iced::{Subscription, Task, exit, keyboard};
use rapidhash::{HashMapExt, RapidHashMap};
use reqwest::Client;
use tokio::time::sleep;

use crate::{
    config::Config,
    state::{Departures, DisplayDeparture, State},
};

// 10 minutes
const RETENTION_TIME: i64 = 900;

#[derive(Debug, Clone)]
pub enum Message {
    FetchDepartures,
    DeparturesFetched(Departures),
    Quit,
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::FetchDepartures => Task::perform(
            fetch_departures(state.http_client.clone(), state.config.clone()),
            |result| match result {
                Ok(departures) => Message::DeparturesFetched(departures),
                Err(e) => {
                    eprintln!("Error fetching departures: {:?}", e);
                    Message::FetchDepartures
                }
            },
        ),
        Message::DeparturesFetched(departures) => {
            update_departures(state, departures);
            Task::perform(sleep(Duration::from_secs(10)), |_| Message::FetchDepartures)
        }
        Message::Quit => exit(),
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

async fn fetch_departures(
    http_client: Client,
    config: Arc<Config>,
) -> Result<Departures, Box<dyn Error + Send>> {
    let mut departures = RapidHashMap::with_capacity(config.bvg_stops.len());
    let mut last_update = None;

    for stop_id in &config.bvg_stops {
        let response =
            crate::api::client::get_departures(&http_client, stop_id, config.max_api_results)
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;

        let Some(stop_name) = response.departures.first().map(|d| d.stop.name.clone()) else {
            continue;
        };

        last_update = Local
            .timestamp_opt(response.realtime_data_updated_at, 0)
            .single();

        println!(
            "fetched departures for stop: {}; last updated: {:?}",
            stop_name, last_update
        );

        let entry = departures.entry(stop_name.clone()).or_insert_with(Vec::new);
        for departure in response.departures {
            let display_departure = DisplayDeparture::from(departure);
            println!("  {}", display_departure);
            entry.push(display_departure);
        }
    }

    Ok(Departures {
        departures,
        last_update,
    })
}

fn update_departures(state: &mut State, mut departures: Departures) {
    // Arriving trips sometimes linger for minutes in the API response
    // show them only once if this config is set
    if state.config.show_arriving_trips_only_once {
        for departures in departures.departures.values_mut() {
            departures.retain(|d| !state.displayed_arriving_trips.contains_key(&d.trip_id));
        }

        let now = chrono::Utc::now().timestamp();
        state.displayed_arriving_trips.extend(
            departures
                .departures
                .values()
                .flatten()
                .filter(|d| d.arriving)
                .map(|d| (d.trip_id.clone(), now)),
        );

        state
            .displayed_arriving_trips
            .retain(|_, created| *created > now - RETENTION_TIME);
    }

    state.departures = departures;
}
