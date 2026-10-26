use std::{error::Error, sync::Arc, time::Duration};

use chrono::{Local, TimeZone};
use iced::{Subscription, Task, exit, keyboard};
use rapidhash::{HashMapExt, RapidHashMap};
use reqwest::Client;
use tokio::time::sleep;

use crate::{
    api::departures::DeparturesResponse,
    config::Config,
    state::{Departures, DisplayDeparture, State, Stop, strip_berlin},
};

// 10 minutes
const RETENTION_TIME: i64 = 900;

#[derive(Debug, Clone)]
pub enum Message {
    FetchDepartures,
    DeparturesFetched(Departures),
    Quit,
    RetroUiBlinkTick,
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

async fn fetch_departures(
    http_client: Client,
    config: Arc<Config>,
) -> Result<Departures, Box<dyn Error + Send>> {
    let mut departures = RapidHashMap::with_capacity(config.bvg_stops.len());
    let mut last_update = None;

    for stop_id in &config.bvg_stops {
        let mut response = crate::api::client::get_departures(
            &http_client,
            stop_id,
            config.max_api_results,
            config.max_departure_time_minutes,
        )
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;

        // The api returns multiple sub stations for some big stations so we filter them out here.
        if config.filter_substations {
            response
                .departures
                .retain(|d| config.bvg_stops.contains(&d.stop.id));
        }

        if response.departures.is_empty() {
            continue;
        }

        let stop = stop_from_response(&response, &config.bvg_stops);

        last_update = Local
            .timestamp_opt(response.realtime_data_updated_at, 0)
            .single();

        println!(
            "fetched departures for stop: {}; last updated: {:?}",
            stop.name, last_update
        );

        let entry = departures.entry(stop).or_insert_with(Vec::new);
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

/// Build the `Stop` to display from a departures response.
///
/// Uses the name of a departure whose stop id is one of the configured
/// `bvg_stops`, falling back to the first departure's name if none match.
/// The products are merged (logical OR) across all departures in the
/// response, so every product available at any sub-station is reported.
///
/// `response.departures` must be non-empty.
fn stop_from_response(response: &DeparturesResponse, bvg_stops: &[String]) -> Stop {
    let name = response
        .departures
        .iter()
        .find(|d| bvg_stops.contains(&d.stop.id))
        .or_else(|| response.departures.first())
        .map(|d| d.stop.name.clone())
        .unwrap_or_default();

    let products = response
        .departures
        .iter()
        .filter_map(|d| d.stop.products)
        .reduce(|a, b| a.merge(&b));

    Stop {
        name: strip_berlin(&name),
        products,
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::departures::Products;

    const RESPONSE_JSON: &str = r#"{
        "departures": [
            {
                "tripId": "a",
                "stop": {
                    "type": "stop",
                    "id": "900040101",
                    "name": "S Halensee (Berlin)",
                    "products": {
                        "suburban": true, "subway": false, "tram": false,
                        "bus": false, "ferry": false, "express": false, "regional": false
                    }
                },
                "plannedWhen": "2026-10-19T22:19:00+02:00",
                "direction": "U Mehringdamm",
                "line": { "type": "line", "name": "M19", "mode": "bus" },
                "remarks": []
            },
            {
                "tripId": "b",
                "stop": {
                    "type": "stop",
                    "id": "900040102",
                    "name": "U Halensee (Berlin)",
                    "products": {
                        "suburban": false, "subway": true, "tram": false,
                        "bus": true, "ferry": false, "express": false, "regional": false
                    }
                },
                "plannedWhen": "2026-10-19T22:20:00+02:00",
                "direction": "U Mehringdamm",
                "line": { "type": "line", "name": "M19", "mode": "bus" },
                "remarks": []
            }
        ],
        "realtimeDataUpdatedAt": 1792441116
    }"#;

    fn response() -> DeparturesResponse {
        serde_json::from_str(RESPONSE_JSON).expect("response should deserialize")
    }

    #[test]
    fn uses_configured_stop_name_and_merges_products() {
        let response = response();
        let bvg_stops = vec!["900040101".to_string()];

        let stop = stop_from_response(&response, &bvg_stops);

        assert_eq!(stop.name, "S Halensee");
        assert_eq!(
            stop.products,
            Some(Products {
                suburban: Some(true),
                subway: Some(true),
                tram: Some(false),
                bus: Some(true),
                ferry: Some(false),
                express: Some(false),
                regional: Some(false),
            })
        );
    }

    #[test]
    fn falls_back_to_first_name_when_no_stop_id_matches() {
        let response = response();
        let bvg_stops = vec!["9999999".to_string()];

        let stop = stop_from_response(&response, &bvg_stops);

        assert_eq!(stop.name, "S Halensee");
        assert!(stop.products.is_some());
    }
}
