//! DTOs for BMW API search endpoint.
use crate::vehicle::Vehicle;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    #[serde(rename = "searchContext")]
    pub search_context: Vec<SearchContext>,
    #[serde(rename = "resultsContext", skip_serializing_if = "Option::is_none")]
    pub results_context: Option<ResultsContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchContext {
    pub model: Option<SearchModel>,
    #[serde(rename = "vssIds", skip_serializing_if = "Option::is_none")]
    pub vss_ids: Option<FilterWithValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchModel {
    #[serde(rename = "marketingModelRange")]
    pub marketing_model_range: FilterWithValues,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterWithValues {
    pub value: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultsContext {
    pub sort: Vec<Sort>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort {
    pub by: SortBy,
    pub order: SortOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SortBy {
    Price,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResponse {
    pub hits: Vec<Hit>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Hit {
    pub vehicle: Vehicle,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Metadata {
    #[serde(rename = "totalCount")]
    pub total_count: u32,
}
