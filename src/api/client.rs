use crate::api::departures::DeparturesResponse;

const BASE_URL: &str = "https://v6.bvg.transport.rest/stops";

/// Thin wrapper around the BVG departures API.
#[derive(Clone, Default)]
pub struct BvgClient {
    client: reqwest::Client,
}

impl BvgClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
    /// Fetch the upcoming departures for one stop.
    pub async fn departures(
        &self,
        stop_id: &str,
        results: u32,
        duration: u32,
    ) -> Result<DeparturesResponse, reqwest::Error> {
        let url = format!("{BASE_URL}/{stop_id}/departures");
        let response = self
            .client
            .get(&url)
            .query(&[
                ("results", results.to_string()),
                ("duration", duration.to_string()),
            ])
            .send()
            .await?
            .json::<DeparturesResponse>()
            .await?;
        Ok(response)
    }
}
