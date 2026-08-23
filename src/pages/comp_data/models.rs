use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Meet {
    pub end_date: String,
    pub name: String,
    pub start_date: String,
    pub time_zone: String,
    pub venue_city: String,
    pub venue_name: String,
    pub venue_state: String,
    pub venue_street: String,
    pub venue_zip: String,
    pub status: String,
    pub venue_map_pdf_url: Option<String>,
    pub venue_map_apple_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ScheduleRow {
    pub date: String,
    pub platform: String,
    pub session_id: f64,
    pub start_time: String,
    pub weigh_in_time: String,
    pub weight_class: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Athlete {
    pub name: String,
    pub age: f64,
    pub club: String,
    pub wso: Option<String>,
    pub gender: String,
    pub weight_class: String,
    pub entry_total: f64,
    pub adaptive: bool,
    pub session_number: Option<f64>,
    pub session_platform: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct LiftingResult {
    pub meet: String,
    pub date: String,
    pub name: String,
    pub age: String,
    pub body_weight: f64,
    pub snatch1: f64,
    pub snatch2: f64,
    pub snatch3: f64,
    pub snatch_best: f64,
    pub cj1: f64,
    pub cj2: f64,
    pub cj3: f64,
    pub cj_best: f64,
    pub total: f64,
    pub adaptive: bool,
}

#[derive(Clone, Serialize)]
pub(crate) struct MeetQuery {
    pub meet: String,
}

pub(crate) fn normalize(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub(crate) fn attempt(value: f64) -> String {
    if value == 0.0 {
        "—".to_owned()
    } else if value < 0.0 {
        format!("{}×", -value)
    } else {
        value.to_string()
    }
}
