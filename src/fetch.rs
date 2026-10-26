//! Fetch orchestration: pulls departures for every configured stop and
//! encapsulates the arriving-trip dedup rule.

use std::{collections::HashSet, error::Error, sync::Arc};

use chrono::{Local, TimeZone};
use rapidhash::{HashMapExt, RapidHashMap};

use crate::{
    api::client::BvgClient,
    api::departures::DeparturesResponse,
    config::Config,
    state::{Departures, DisplayDeparture, Station, strip_berlin},
};

const RETENTION_TIME: i64 = 900; // 15 minutes

/// Tracks which arriving trips were already displayed, so that with
/// `show_arriving_trips_only_once` they are shown exactly once.
#[derive(Debug, Default, Clone)]
pub struct ArrivingTripLog {
    shown: RapidHashMap<String, i64>, // trip_id -> timestamp when shown
}

impl ArrivingTripLog {
    /// Remove trips already shown, then record the remaining arriving trips
    /// and prune stale entries.
    pub fn record_and_filter(&mut self, departures: &mut Departures) {
        for departures in departures.stops.values_mut() {
            departures.retain(|d| !self.shown.contains_key(&d.trip_id));
        }

        let now = chrono::Utc::now().timestamp();
        self.shown.extend(
            departures
                .stops
                .values()
                .flatten()
                .filter(|d| d.arriving)
                .map(|d| (d.trip_id.clone(), now)),
        );

        self.shown
            .retain(|_, created| *created > now - RETENTION_TIME);
    }
}

/// Fetch departures for every configured stop, sequentially. A failed stop
/// aborts the whole batch with the underlying error.
pub async fn fetch_departures(
    api: BvgClient,
    config: Arc<Config>,
) -> Result<Departures, Box<dyn Error + Send>> {
    let mut departures = RapidHashMap::with_capacity(config.bvg_stops.len());
    let mut last_update = None;

    // The api returns multiple sub stations for some big stations so we
    // filter them out here.
    let configured_stop_ids: HashSet<&str> = config.bvg_stops.iter().map(|s| s.as_str()).collect();

    for stop_id in &config.bvg_stops {
        let mut response = api
            .departures(
                stop_id,
                config.max_api_results,
                config.max_departure_time_minutes,
            )
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;

        if config.filter_substations {
            response
                .departures
                .retain(|d| configured_stop_ids.contains(d.stop.id.as_str()));
        }

        if response.departures.is_empty() {
            continue;
        }

        let stop = station_from_response(&response, &config.bvg_stops);

        last_update = Local
            .timestamp_opt(response.realtime_data_updated_at, 0)
            .single();

        println!(
            "fetched departures for stop: {}; last updated: {:?}",
            stop.name, last_update
        );

        let entry = departures.entry(stop).or_insert_with(Vec::new);
        for departure in response.departures {
            let display_departure = DisplayDeparture::from_api(&departure, chrono::Utc::now());
            println!("  {}", display_departure);
            entry.push(display_departure);
        }
    }

    Ok(Departures {
        stops: departures,
        last_update,
    })
}

/// Build the `Station` to display from a departures response.
///
/// Uses the name of a departure whose stop id is one of the configured
/// `bvg_stops`, falling back to the first departure's name if none match.
/// The products are merged (logical OR) across all departures in the
/// response, so every product available at any sub-station is reported.
///
/// `response.departures` must be non-empty.
fn station_from_response(response: &DeparturesResponse, bvg_stops: &[String]) -> Station {
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

    Station {
        name: strip_berlin(&name),
        products,
    }
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

        let stop = station_from_response(&response, &bvg_stops);

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

        let stop = station_from_response(&response, &bvg_stops);

        assert_eq!(stop.name, "S Halensee");
        assert!(stop.products.is_some());
    }
}
