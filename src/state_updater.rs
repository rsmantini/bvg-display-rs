use std::{error::Error, sync::Arc, time::Duration};

use chrono::{Local, TimeZone};
use iced::Task;
use rapidhash::{HashMapExt, RapidHashMap};
use reqwest::Client;
use tokio::time::sleep;

use crate::{
    config::Config,
    state::{Departures, DisplayDeparture, State},
};

#[derive(Debug, Clone)]
pub enum Message {
    FetchDepartures,
    DeparturesFetched(Departures),
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
            state.departures = departures;
            Task::perform(sleep(Duration::from_secs(10)), |_| Message::FetchDepartures)
        }
    }
}

async fn fetch_departures(
    http_client: Client,
    config: Arc<Config>,
) -> Result<Departures, Box<dyn Error + Send>> {
    let mut departures = RapidHashMap::with_capacity(config.bvg_stops.len());
    let mut last_update = None;

    for stop_id in &config.bvg_stops {
        let response =
            crate::api::client::get_departures(&http_client, stop_id, config.num_results)
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;

        let Some(stop_name) = response.departures.first().map(|d| d.stop.name.clone()) else {
            continue;
        };

        let entry = departures.entry(stop_name.clone()).or_insert_with(Vec::new);
        for departure in response.departures {
            let display_departure = DisplayDeparture::from(departure);
            entry.push(display_departure);
        }
        last_update = Local
            .timestamp_opt(response.realtime_data_updated_at, 0)
            .single();
    }

    Ok(Departures {
        departures,
        last_update,
    })
}
