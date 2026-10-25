//! Types describing the BVG departures API response.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

/// Root response of the departures endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeparturesResponse {
    pub departures: Vec<Departure>,
    pub realtime_data_updated_at: i64,
}

/// A single upcoming departure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Departure {
    pub trip_id: String,
    pub stop: Stop,
    pub when: Option<DateTime<Utc>>,
    pub planned_when: DateTime<Utc>,
    pub delay: Option<i32>,
    #[serde(default)]
    pub cancelled: bool,
    pub platform: Option<String>,
    pub planned_platform: Option<String>,
    pub prognosis_type: Option<String>,
    pub direction: String,
    pub provenance: Option<String>,
    pub line: Line,
    pub remarks: Vec<Remark>,
    pub origin: Option<Stop>,
    pub destination: Option<Stop>,
    pub current_trip_position: Option<Location>,
    /// Present only for some departures (e.g. `"low"`, `"medium"`).
    pub occupancy: Option<String>,
}

/// A stop (station) where the departure departs from or arrives at.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stop {
    #[serde(rename = "type")]
    pub kind: String,
    pub id: Option<String>,
    pub name: String,
    pub location: Option<Location>,
    pub products: Option<Products>,
    pub ids: Option<StopIds>,
}

/// Geographic coordinates of a place.
///
/// `id` is present for stop locations but omitted for `currentTripPosition`,
/// hence it is optional.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    #[serde(rename = "type")]
    pub kind: String,
    pub id: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f64>,
}

/// Flags describing which product categories are available at a stop.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Products {
    pub suburban: Option<bool>,
    pub subway: Option<bool>,
    pub tram: Option<bool>,
    pub bus: Option<bool>,
    pub ferry: Option<bool>,
    pub express: Option<bool>,
    pub regional: Option<bool>,
}

/// Alternative identifiers for a stop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopIds {
    pub ifopt: Option<String>,
}

/// The line / vehicle running the trip.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    #[serde(rename = "type")]
    pub kind: String,
    pub id: Option<String>,
    pub fahrt_nr: Option<String>,
    pub name: String,
    pub public: Option<bool>,
    pub admin_code: Option<String>,
    pub product_name: Option<String>,
    #[serde(deserialize_with = "deserialize_mode")]
    pub mode: Mode,
    pub product: Option<String>,
    pub operator: Option<Operator>,
}

/// The operator running the line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operator {
    #[serde(rename = "type")]
    pub kind: String,
    pub id: Option<String>,
    pub name: Option<String>,
}

/// An informational remark attached to a departure (e.g. hints).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remark {
    #[serde(rename = "type")]
    pub kind: String,
    pub code: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Mode {
    Unknown,
    Bus,
    Train,
    Tram,
}

impl From<&str> for Mode {
    fn from(s: &str) -> Self {
        let s = s.to_lowercase();
        match s.as_str() {
            "bus" => Mode::Bus,
            "train" => Mode::Train,
            "tram" => Mode::Tram,
            _ => Mode::Unknown,
        }
    }
}

impl Mode {
    pub fn as_german_str(&self) -> &'static str {
        match self {
            Mode::Unknown => "",
            Mode::Bus => "Bus",
            Mode::Train => "Zug",
            Mode::Tram => "Tram",
        }
    }
}

fn deserialize_mode<'de, D>(deserializer: D) -> Result<Mode, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    Ok(Mode::from(string.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_departures_response() {
        let json = r#"{
            "departures": [
                {
                    "tripId": "1|38929|0|86|19102026",
                    "stop": {
                        "type": "stop",
                        "id": "900040101",
                        "name": "S Halensee (Berlin)",
                        "location": { "type": "location", "id": "900040101", "latitude": 52.496698, "longitude": 13.290147 },
                        "products": { "suburban": true, "subway": false, "tram": false, "bus": true, "ferry": false, "express": false, "regional": false },
                        "ids": { "ifopt": "de:11000:900040101" }
                    },
                    "when": "2026-10-19T22:19:00+02:00",
                    "plannedWhen": "2026-10-19T22:19:00+02:00",
                    "delay": 0,
                    "platform": null,
                    "plannedPlatform": null,
                    "prognosisType": "prognosed",
                    "direction": "U Mehringdamm",
                    "provenance": null,
                    "line": {
                        "type": "line",
                        "id": "de-vbb-11000000-bus-m19",
                        "fahrtNr": "91086",
                        "name": "M19",
                        "public": true,
                        "adminCode": "BVB---",
                        "productName": "Bus",
                        "mode": "bus",
                        "product": "bus",
                        "operator": { "type": "operator", "id": "berliner-verkehrsbetriebe", "name": "Berliner Verkehrsbetriebe" }
                    },
                    "remarks": [],
                    "origin": null,
                    "destination": {
                        "type": "stop",
                        "id": "900017171",
                        "name": "U Mehringdamm (Berlin) [Pos 7 Endstelle]",
                        "location": { "type": "location", "id": "900017171", "latitude": 52.49277, "longitude": 13.38839 },
                        "products": { "suburban": false, "subway": false, "tram": false, "bus": true, "ferry": false, "express": false, "regional": false },
                        "ids": { "ifopt": "de:11000:900017171" }
                    },
                    "currentTripPosition": { "type": "location", "latitude": 52.496995, "longitude": 13.290578 },
                    "occupancy": "low"
                }
            ],
            "realtimeDataUpdatedAt": 1792441116
        }"#;

        let resp: DeparturesResponse =
            serde_json::from_str(json).expect("payload should deserialize");
        assert_eq!(resp.departures.len(), 1);
        assert_eq!(resp.departures[0].trip_id, "1|38929|0|86|19102026");
        assert_eq!(resp.departures[0].delay, Some(0));
        assert_eq!(resp.departures[0].platform, None);
        assert_eq!(resp.departures[0].occupancy.as_deref(), Some("low"));
        assert_eq!(resp.realtime_data_updated_at, 1792441116);
    }

    #[test]
    fn deserializes_cancelled_departure() {
        // A cancelled trip has `when: null`, `prognosisType: null`,
        // `currentTripPosition` omitted, and a `cancelled: true` flag.
        let json = r#"{
            "departures": [
                {
                    "cancelled": true,
                    "delay": null,
                    "direction": "S Westend (Berlin)",
                    "line": {
                        "adminCode": "DBS---",
                        "fahrtNr": "21065",
                        "id": "de-vbb-11000000-s-bahn-s46-s46-d",
                        "mode": "train",
                        "name": "S46",
                        "operator": { "id": "s-bahn-berlin-gmbh", "name": "S-Bahn Berlin GmbH", "type": "operator" },
                        "product": "suburban",
                        "productName": "S",
                        "public": true,
                        "type": "line"
                    },
                    "origin": null,
                    "plannedPlatform": "1",
                    "plannedWhen": "2026-10-20T00:34:00+02:00",
                    "platform": null,
                    "prognosedPlatform": null,
                    "prognosedWhen": null,
                    "prognosisType": null,
                    "provenance": null,
                    "remarks": [ { "code": "FK", "text": "Bicycle conveyance", "type": "hint" } ],
                    "stop": {
                        "id": "900040101",
                        "ids": { "ifopt": "de:11000:900040101" },
                        "location": { "id": "900040101", "latitude": 52.496698, "longitude": 13.290147, "type": "location" },
                        "name": "S Halensee (Berlin)",
                        "products": { "bus": true, "express": false, "ferry": false, "regional": false, "suburban": true, "subway": false, "tram": false },
                        "type": "stop"
                    },
                    "tripId": "1|1604|6|86|19102026",
                    "when": null
                }
            ],
            "realtimeDataUpdatedAt": 1792449247
        }"#;

        let resp: DeparturesResponse =
            serde_json::from_str(json).expect("cancelled payload should deserialize");
        assert_eq!(resp.departures.len(), 1);
        let d = &resp.departures[0];
        assert!(d.cancelled);
        assert!(d.when.is_none());
        assert!(d.prognosis_type.is_none());
        assert!(d.current_trip_position.is_none());
        assert!(d.platform.is_none());
        assert_eq!(d.planned_platform.as_deref(), Some("1"));
    }
}
