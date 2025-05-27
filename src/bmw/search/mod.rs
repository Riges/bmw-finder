//! BMW API search logic
// Handles vehicle search logic, API requests, and result aggregation.

use std::collections::HashMap;

use anyhow::Result;
use futures::{StreamExt, TryStreamExt, stream};
use reqwest::{Client, Url};
use uuid::Uuid;

use crate::config::{Condition, Configuration};
use crate::vehicle::Vehicle;
pub mod dto;

use self::dto::*;

const NEW_CAR_URL: &str = "https://stolo-data-service.prod.stolo.eu-central-1.aws.bmw.cloud/vehiclesearch/search/fr-fr/stocklocator";
const USED_CAR_URL: &str = "https://stolo-data-service.prod.stolo.eu-central-1.aws.bmw.cloud/vehiclesearch/search/fr-fr/stocklocator_uc";
const MAX_RESULT: u32 = 50;
const CONCURRENT_REQUESTS: usize = 5;

// === Public API ===

/// Search vehicles according to the configuration.
pub async fn search(configuration: &Configuration) -> Result<HashMap<uuid::Uuid, Vehicle>> {
    let client = Client::new();
    let request_body: SearchRequest = SearchRequest {
        search_context: vec![SearchContext {
            model: Some(SearchModel {
                marketing_model_range: FilterWithValues {
                    value: configuration.models().to_vec(),
                },
            }),
            vss_ids: None,
        }],
        results_context: Some(ResultsContext {
            sort: vec![Sort {
                by: SortBy::Price,
                order: SortOrder::Asc,
            }],
        }),
    };

    let total_count = get_total_count(&client, configuration.condition, request_body.clone()).await;
    let calls = determine_calls_needed(configuration, request_body.clone(), total_count);

    let vehicles = stream::iter(&calls)
        .map(|call| {
            query_search(
                &client,
                call.condition,
                call.max_result,
                call.start_index,
                call.body.clone(),
            )
        })
        .buffer_unordered(CONCURRENT_REQUESTS)
        .try_fold(
            Vec::with_capacity(calls.len() * (MAX_RESULT as usize)),
            |mut acc, resp| async move {
                let SearchResponse { hits, .. } = resp;
                acc.extend(hits.into_iter().map(|hit| hit.vehicle));
                Ok(acc)
            },
        )
        .await
        .map_err(|_| anyhow::anyhow!("Error in one of the requests"))?;

    let vehicles_map: HashMap<Uuid, Vehicle> =
        vehicles.into_iter().map(|v| (v.vss_id, v)).collect();
    Ok(vehicles_map)
}

/// Search a vehicle by its VSS ID.
#[allow(dead_code)]
pub async fn search_by_vss_id(
    configuration: &Configuration,
    vss_id: &Uuid,
) -> Result<Option<Vehicle>> {
    let client = Client::new();
    let request_body: SearchRequest = SearchRequest {
        search_context: vec![SearchContext {
            model: None,
            vss_ids: Some(FilterWithValues {
                value: vec![vss_id.to_string()],
            }),
        }],
        results_context: None,
    };

    let response = query_search(&client, configuration.condition, 1, 0, request_body).await;

    match response {
        Ok(res) if res.hits.is_empty() => Ok(None),
        Ok(res) if res.hits.first().is_some() => Ok(Some(res.hits[0].vehicle.clone())),
        Err(e) => {
            return Err(e);
        }
        _ => Err(anyhow::anyhow!("Unexpected response format")),
    }
}

// === Private helpers ===

fn build_search_url(
    condition: Condition,
    max_result: u32,
    start_index: Option<u32>,
) -> Result<Url> {
    let base_url = match condition {
        Condition::New => NEW_CAR_URL,
        Condition::Used => USED_CAR_URL,
    };

    let params = [
        ("brand", "BMW"),
        (
            "maxResults",
            match max_result {
                x if x > MAX_RESULT => &MAX_RESULT.to_string(),
                x => &x.to_string(),
            },
        ),
        (
            "startIndex",
            match start_index {
                Some(x) => &x.to_string(),
                None => "0",
            },
        ),
    ];

    Url::parse_with_params(base_url, &params).map_err(anyhow::Error::from)
}

async fn query_search(
    client: &Client,
    condition: Condition,
    max_result: u32,
    start_index: u32,
    body: SearchRequest,
) -> Result<SearchResponse> {
    let response: reqwest::Response = client
        .post(build_search_url(condition, max_result, Some(start_index))?)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Error: {}", response.status()));
    }

    response
        .json::<SearchResponse>()
        .await
        .map_err(anyhow::Error::from)
}

async fn get_total_count(client: &Client, condition: Condition, body: SearchRequest) -> u32 {
    let response = query_search(client, condition, 1, 0, body).await;

    match response {
        Ok(res) => res.metadata.total_count,
        Err(e) => {
            eprintln!("Error fetching total count: {:?}", e);
            return 0;
        }
    }
}

struct CallDefinition {
    condition: Condition,
    start_index: u32,
    max_result: u32,
    body: SearchRequest,
}

fn determine_calls_needed(
    configuration: &Configuration,
    body: SearchRequest,
    total_count: u32,
) -> Vec<CallDefinition> {
    let max = match configuration.limit {
        Some(l) if total_count > l => l,
        _ => total_count,
    };

    if max < 1 {
        return vec![];
    }

    let step = if max > MAX_RESULT { MAX_RESULT } else { max };

    // split into chunks of MAX_RESULT
    (0..max)
        .step_by(step as usize)
        .map(|start_index| CallDefinition {
            condition: configuration.condition,
            start_index,
            max_result: step,
            body: body.clone(),
        })
        .collect()
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_search_url_with_defaults() {
        let url = build_search_url(Condition::New, 42, None).expect("Failed to build default URL");
        assert_eq!(
            url.as_str(),
            "https://stolo-data-service.prod.stolo.eu-central-1.aws.bmw.cloud/vehiclesearch/search/fr-fr/stocklocator?brand=BMW&maxResults=42&startIndex=0"
        );
    }

    #[test]
    fn test_build_search_url_for_new_cars() {
        let url =
            build_search_url(Condition::New, 42, None).expect("Failed to build URL for new cars");
        assert!(url.as_str().starts_with(NEW_CAR_URL));
    }

    #[test]
    fn test_build_search_url_for_used_cars() {
        let url =
            build_search_url(Condition::Used, 42, None).expect("Failed to build URL for used cars");
        assert!(url.as_str().starts_with(USED_CAR_URL));
    }

    #[test]
    fn test_build_search_url_with_max_results() {
        let url = build_search_url(Condition::New, 109, None)
            .expect("Failed to build URL with max_result");
        assert_eq!(
            url.as_str(),
            "https://stolo-data-service.prod.stolo.eu-central-1.aws.bmw.cloud/vehiclesearch/search/fr-fr/stocklocator?brand=BMW&maxResults=50&startIndex=0"
        );
    }
    #[test]
    fn test_build_search_url_with_start_index() {
        let url = build_search_url(Condition::New, 42, Some(42000))
            .expect("Failed to build URL with start index 42000");
        assert_eq!(
            url.as_str(),
            "https://stolo-data-service.prod.stolo.eu-central-1.aws.bmw.cloud/vehiclesearch/search/fr-fr/stocklocator?brand=BMW&maxResults=42&startIndex=42000"
        );
    }

    #[test]
    fn test_search_request_serialize() {
        let expected_json = r#"{"searchContext":[{"model":{"marketingModelRange":{"value":["iX2_U10E"]}}}],"resultsContext":{"sort":[{"by":"PRICE","order":"ASC"}]}}"#;
        let request: SearchRequest = SearchRequest {
            search_context: vec![SearchContext {
                model: Some(SearchModel {
                    marketing_model_range: FilterWithValues {
                        value: vec![String::from("iX2_U10E")],
                    },
                }),
                vss_ids: None,
            }],
            results_context: Some(ResultsContext {
                sort: vec![Sort {
                    by: SortBy::Price,
                    order: SortOrder::Asc,
                }],
            }),
        };

        let request_json = serde_json::to_string(&request).expect("Failed to serialize request");

        assert_eq!(request_json, expected_json);
    }

    #[test]
    fn terst_search_request_serialize_with_desc_sort() {
        let expected_json = r#"{"searchContext":[{"model":{"marketingModelRange":{"value":["iX2_U10E"]}}}],"resultsContext":{"sort":[{"by":"PRICE","order":"DESC"}]}}"#;
        let request: SearchRequest = SearchRequest {
            search_context: vec![SearchContext {
                model: Some(SearchModel {
                    marketing_model_range: FilterWithValues {
                        value: vec![String::from("iX2_U10E")],
                    },
                }),
                vss_ids: None,
            }],
            results_context: Some(ResultsContext {
                sort: vec![Sort {
                    by: SortBy::Price,
                    order: SortOrder::Desc,
                }],
            }),
        };

        let request_json = serde_json::to_string(&request).expect("Failed to serialize request");

        assert_eq!(request_json, expected_json);
    }
}
