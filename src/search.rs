use std::{collections::HashMap, u8};

use anyhow::Result;
use futures::{StreamExt, stream};
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::{
    config::{Condition, Configuration},
    vehicle::Vehicle,
};

const NEW_CAR_URL: &str = "https://stolo-data-service.prod.stolo.eu-central-1.aws.bmw.cloud/vehiclesearch/search/fr-fr/stocklocator";
const USED_CAR_URL: &str = "https://stolo-data-service.prod.stolo.eu-central-1.aws.bmw.cloud/vehiclesearch/search/fr-fr/stocklocator_uc";
const MAX_RESULT: u32 = 50;
const CONCURRENT_REQUESTS: usize = 5;

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

#[derive(Serialize)]
struct SearchRequest {
    #[serde(rename = "searchContext")]
    search_context: Vec<SearchContext>,

    #[serde(rename = "resultsContext", skip_serializing_if = "Option::is_none")]
    results_context: Option<ResultsContext>,
}

#[derive(Serialize)]
struct SearchContext {
    #[serde(rename = "model", skip_serializing_if = "Option::is_none")]
    model: Option<SearchModel>,

    #[serde(rename = "vssIds", skip_serializing_if = "Option::is_none")]
    vss_ids: Option<FilterWithValues>,
}

#[derive(Serialize)]
struct SearchModel {
    #[serde(rename = "marketingModelRange")]
    marketing_model_range: FilterWithValues,
}

#[derive(Serialize)]
struct FilterWithValues {
    #[serde(rename = "value")]
    value: Vec<String>,
}

#[derive(Serialize)]
struct ResultsContext {
    sort: Vec<Sort>,
}
#[derive(Serialize)]
struct Sort {
    by: SortBy,
    order: SortOrder,
}

#[derive(Serialize)]
enum SortBy {
    #[serde(rename = "PRICE")]
    Price,
}

#[derive(Serialize)]
#[allow(dead_code)]
enum SortOrder {
    #[serde(rename = "ASC")]
    Asc,
    #[serde(rename = "DESC")]
    Desc,
}

#[derive(Deserialize)]
struct SearchResponse {
    metadata: Metadata,
    hits: Vec<Hit>,
}

#[derive(Deserialize)]
struct Metadata {
    #[serde(rename = "totalCount")]
    total_count: u32,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Hit {
    country: String,
    score: f32,
    vehicle: Vehicle,
}

pub async fn search_cars(configuration: &Configuration) -> Result<HashMap<uuid::Uuid, Vehicle>> {
    let request_body: SearchRequest = SearchRequest {
        search_context: vec![SearchContext {
            model: Some(SearchModel {
                marketing_model_range: FilterWithValues {
                    value: configuration.models.clone(),
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

    let total_count = get_total_count(configuration.condition, &request_body).await;

    let max = match configuration.limit {
        Some(l) if total_count > l => l,
        _ => total_count,
    };

    let step = if max > MAX_RESULT { MAX_RESULT } else { max };

    // split into chunks of MAX_RESULT
    let chunks: Vec<u32> = (0..max).step_by(step as usize).collect();

    let responses = stream::iter(chunks)
        .map(|start_index| query_search(configuration.condition, step, start_index, &request_body))
        .buffer_unordered(CONCURRENT_REQUESTS);

    let mut vehicles = HashMap::new();

    responses
        .for_each(|response| {
            match response {
                Ok(res) => {
                    res.hits.into_iter().for_each(|hit| {
                        vehicles.insert(hit.vehicle.vss_id, hit.vehicle);
                    });
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
            futures::future::ready(())
        })
        .await;

    Ok(vehicles)
}

pub async fn search_cars_by_vss_id(
    configuration: &Configuration,
    vss_id: &str,
) -> Result<Option<Vehicle>> {
    let request_body: SearchRequest = SearchRequest {
        search_context: vec![SearchContext {
            model: None,
            vss_ids: Some(FilterWithValues {
                value: vec![vss_id.to_string()],
            }),
        }],
        results_context: None,
    };

    let response = query_search(configuration.condition, 1, 0, &request_body).await;

    match response {
        Ok(res) if res.hits.is_empty() => Ok(None),
        Ok(res) if res.hits.first().is_some() => Ok(Some(res.hits[0].vehicle.clone())),
        Err(e) => {
            return Err(e);
        }
        _ => Err(anyhow::anyhow!("Unexpected response format")),
    }
}

async fn query_search(
    condition: Condition,
    max_result: u32,
    start_index: u32,
    body: &SearchRequest,
) -> Result<SearchResponse> {
    let response: reqwest::Response = reqwest::Client::new()
        .post(build_search_url(condition, max_result, Some(start_index))?)
        .json::<SearchRequest>(body)
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

async fn get_total_count(condition: Condition, body: &SearchRequest) -> u32 {
    let response = query_search(condition, MAX_RESULT, 0, body).await;

    match response {
        Ok(res) => res.metadata.total_count,
        _ => 0,
    }
}

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
                        value: vec!["iX2_U10E".to_string()],
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
                        value: vec!["iX2_U10E".to_string()],
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
