use std::u8;

use anyhow::Result;
use futures::{StreamExt, stream};
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::vehicle::Vehicle;

const NEW_CAR_URL: &str = "https://stolo-data-service.prod.stolo.eu-central-1.aws.bmw.cloud/vehiclesearch/search/fr-fr/stocklocator";
const USED_CAR_URL: &str = "https://stolo-data-service.prod.stolo.eu-central-1.aws.bmw.cloud/vehiclesearch/search/fr-fr/stocklocator_uc";
const MAX_RESULT: u32 = 50;
const CONCURRENT_REQUESTS: usize = 10;

fn build_search_url(new_car: bool, max_result: u32, start_index: Option<u32>) -> Result<Url> {
    let base_url = if new_car { NEW_CAR_URL } else { USED_CAR_URL };

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

    #[serde(rename = "resultsContext")]
    results_context: ResultsContext,
}

#[derive(Serialize)]
struct SearchContext {
    model: SearchModel,
}

#[derive(Serialize)]
struct SearchModel {
    #[serde(rename = "marketingModelRange")]
    marketing_model_range: MarketingModelRange,
}

#[derive(Serialize)]
struct MarketingModelRange {
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

pub async fn search_cars(new_car: bool, count: u32) -> Result<Vec<Vehicle>> {
    let request_body: SearchRequest = SearchRequest {
        search_context: vec![SearchContext {
            model: SearchModel {
                marketing_model_range: MarketingModelRange {
                    value: vec!["iX2_U10E".to_string()],
                },
            },
        }],
        results_context: ResultsContext {
            sort: vec![Sort {
                by: SortBy::Price,
                order: SortOrder::Asc,
            }],
        },
    };

    let total_count = get_total_count(new_car, &request_body).await;

    let max = if total_count > count as u32 {
        count as u32
    } else {
        total_count
    };

    let step = if max > MAX_RESULT { MAX_RESULT } else { max };

    // split into chunks of MAX_RESULT
    let chunks: Vec<u32> = (0..max).step_by(step as usize).collect();

    let responses = stream::iter(chunks)
        .map(|start_index| query_search(new_car, step, start_index, &request_body))
        .buffer_unordered(CONCURRENT_REQUESTS);

    let mut vehicles: Vec<Vehicle> = Vec::new();

    responses
        .for_each(|response| {
            match response {
                Ok(res) => {
                    vehicles.extend(res.hits.into_iter().map(|hit| hit.vehicle));
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
            futures::future::ready(())
        })
        .await;

    Ok(vehicles.to_vec())
}

async fn query_search(
    new_car: bool,
    max_result: u32,
    start_index: u32,
    body: &SearchRequest,
) -> Result<SearchResponse> {
    let response: reqwest::Response = reqwest::Client::new()
        .post(build_search_url(new_car, max_result, Some(start_index))?)
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

async fn get_total_count(new_car: bool, body: &SearchRequest) -> u32 {
    let response = query_search(new_car, MAX_RESULT, 0, body).await;

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
        let url = build_search_url(true, 42, None).expect("Failed to build default URL");
        assert_eq!(
            url.as_str(),
            "https://stolo-data-service.prod.stolo.eu-central-1.aws.bmw.cloud/vehiclesearch/search/fr-fr/stocklocator?brand=BMW&maxResults=42&startIndex=0"
        );
    }

    #[test]
    fn test_build_search_url_for_new_cars() {
        let url = build_search_url(true, 42, None).expect("Failed to build URL for new cars");
        assert!(url.as_str().starts_with(NEW_CAR_URL));
    }

    #[test]
    fn test_build_search_url_for_used_cars() {
        let url = build_search_url(false, 42, None).expect("Failed to build URL for used cars");
        assert!(url.as_str().starts_with(USED_CAR_URL));
    }

    #[test]
    fn test_build_search_url_with_max_results() {
        let url = build_search_url(true, 109, None).expect("Failed to build URL with max_result");
        assert_eq!(
            url.as_str(),
            "https://stolo-data-service.prod.stolo.eu-central-1.aws.bmw.cloud/vehiclesearch/search/fr-fr/stocklocator?brand=BMW&maxResults=50&startIndex=0"
        );
    }
    #[test]
    fn test_build_search_url_with_start_index() {
        let url = build_search_url(true, 42, Some(42000))
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
                model: SearchModel {
                    marketing_model_range: MarketingModelRange {
                        value: vec!["iX2_U10E".to_string()],
                    },
                },
            }],
            results_context: ResultsContext {
                sort: vec![Sort {
                    by: SortBy::Price,
                    order: SortOrder::Asc,
                }],
            },
        };

        let request_json = serde_json::to_string(&request).expect("Failed to serialize request");

        assert_eq!(request_json, expected_json);
    }

    #[test]
    fn terst_search_request_serialize_with_desc_sort() {
        let expected_json = r#"{"searchContext":[{"model":{"marketingModelRange":{"value":["iX2_U10E"]}}}],"resultsContext":{"sort":[{"by":"PRICE","order":"DESC"}]}}"#;
        let request: SearchRequest = SearchRequest {
            search_context: vec![SearchContext {
                model: SearchModel {
                    marketing_model_range: MarketingModelRange {
                        value: vec!["iX2_U10E".to_string()],
                    },
                },
            }],
            results_context: ResultsContext {
                sort: vec![Sort {
                    by: SortBy::Price,
                    order: SortOrder::Desc,
                }],
            },
        };

        let request_json = serde_json::to_string(&request).expect("Failed to serialize request");

        assert_eq!(request_json, expected_json);
    }
}
