use crate::api::departures::DeparturesResponse;
use reqwest::Client;

pub async fn get_departures(
    client: &Client,
    stop_id: &str,
    results: u32,
) -> Result<DeparturesResponse, reqwest::Error> {
    let url = format!("https://v6.bvg.transport.rest/stops/{}/departures", stop_id);
    let response = client
        .get(&url)
        .query(&[("results", results.to_string())])
        .send()
        .await?
        .json::<DeparturesResponse>()
        .await?;
    Ok(response)
}

pub fn get_departures_raw(
    client: &reqwest::blocking::Client,
    stop_id: &str,
    results: u32,
) -> Result<serde_json::Value, reqwest::Error> {
    let url = format!("https://v6.bvg.transport.rest/stops/{}/departures", stop_id);
    let response = client
        .get(&url)
        .query(&[("results", results.to_string())])
        .send()?
        .json::<serde_json::Value>()?;
    Ok(response)
}
