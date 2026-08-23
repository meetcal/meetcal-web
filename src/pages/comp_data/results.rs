use super::{
    TableSkeleton, athlete_autocomplete::AthleteAutocomplete, format_us_date, models::attempt,
    yes_no,
};
use crate::{
    components::{footer::Footer, header::Header},
    utils::api::get_api_response_with_query,
};
use js_sys::Date;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
struct SearchQuery {
    query: String,
    start_date: String,
    end_date: String,
}

fn search_date_range() -> (String, String) {
    let start = Date::new_0();
    start.set_full_year(start.get_full_year() - 20);

    let end = Date::new_0();
    end.set_date(end.get_date() + 1);

    (format_date(&start), format_date(&end))
}

fn format_date(date: &Date) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date(),
    )
}

#[derive(Deserialize)]
struct SearchResponse {
    matched_name: Option<String>,
    suggestions: Vec<String>,
    results: Vec<LiftingResult>,
}

#[derive(Deserialize)]
struct LiftingResult {
    meet: String,
    date: String,
    age: String,
    body_weight: f64,
    snatch1: f64,
    snatch2: f64,
    snatch3: f64,
    snatch_best: f64,
    cj1: f64,
    cj2: f64,
    cj3: f64,
    cj_best: f64,
    total: f64,
    adaptive: bool,
}

#[component]
pub fn Results() -> impl IntoView {
    let initial_name = use_query_map()
        .with_untracked(|params| params.get("athlete"))
        .unwrap_or_default();
    let initial_request = (!initial_name.is_empty()).then(|| {
        let (start_date, end_date) = search_date_range();
        SearchQuery {
            query: initial_name.clone(),
            start_date,
            end_date,
        }
    });
    let (name, set_name) = signal(initial_name);
    let (request, set_request) = signal(initial_request);
    let (sort, set_sort) = signal("date_desc".to_string());

    let results = LocalResource::new(move || {
        let request = request.get();
        async move {
            match request {
                Some(query) => get_api_response_with_query::<SearchResponse, _>("/search", &query)
                    .await
                    .map_err(|error| error.to_string()),
                None => Ok(SearchResponse {
                    matched_name: None,
                    suggestions: Vec::new(),
                    results: Vec::new(),
                }),
            }
        }
    });

    view! {
        <Header />
        <section class="data-page">
            <p class="data-eyebrow">"Competition data"</p>
            <h1>"Results"</h1>
            <p class="data-intro">"Search an athlete’s competition history."</p>

            <form class="result-search" on:submit=move |event| {
                event.prevent_default();

                let athlete = name.get().trim().to_owned();

                if !athlete.is_empty() {
                    let (start_date, end_date) = search_date_range();
                    set_request.set(Some(SearchQuery {
                        query: athlete,
                        start_date,
                        end_date,
                    }));
                }
            }>
                <AthleteAutocomplete value=name set_value=set_name input_id="results-athlete" />
                <button class="data-search-button" type="submit">"Search"</button>
            </form>

            {move || results.with(|response| match response {
                None => view! { <TableSkeleton columns=14 /> }.into_any(),
                Some(Err(error)) => view! {
                    <p class="data-status error">{format!("Could not load results: {error}")}</p>
                }
                .into_any(),
                Some(Ok(_)) if request.get().is_none() => view! {
                    <p class="data-status">"Enter an athlete name to search."</p>
                }
                .into_any(),
                Some(Ok(response)) => {
                    let suggestions = (!response.suggestions.is_empty()).then(|| {
                        view! {
                            <p class="data-status">"Try: " {response.suggestions.join(", ")}</p>
                        }
                    });
                    let heading = response.matched_name.as_ref().map(|matched_name| {
                        view! {
                            <p class="data-status">
                                "Results for " <strong>{matched_name.clone()}</strong>
                            </p>
                        }
                    });

                    let mut sorted_results = response.results.iter().collect::<Vec<_>>();
                    match sort.get().as_str() {
                        "date_asc" => sorted_results.sort_by(|left, right| left.date.cmp(&right.date)),
                        "total_desc" => sorted_results
                            .sort_by(|left, right| right.total.total_cmp(&left.total)),
                        "total_asc" => sorted_results
                            .sort_by(|left, right| left.total.total_cmp(&right.total)),
                        _ => sorted_results.sort_by(|left, right| right.date.cmp(&left.date)),
                    }

                    let rows = sorted_results
                        .into_iter()
                        .map(|row| {
                            view! {
                                <tr>
                                    <td>{format_us_date(&row.date)}</td>
                                    <td>{row.meet.clone()}</td>
                                    <td>{row.age.clone()}</td>
                                    <td>{row.body_weight}</td>
                                    <td>{attempt(row.snatch1)}</td>
                                    <td>{attempt(row.snatch2)}</td>
                                    <td>{attempt(row.snatch3)}</td>
                                    <td>{row.snatch_best}</td>
                                    <td>{attempt(row.cj1)}</td>
                                    <td>{attempt(row.cj2)}</td>
                                    <td>{attempt(row.cj3)}</td>
                                    <td>{row.cj_best}</td>
                                    <td>{row.total}</td>
                                    <td>{yes_no(row.adaptive)}</td>
                                </tr>
                            }
                        })
                        .collect_view();

                    view! {
                        {heading}
                        {suggestions}
                        <div class="data-filters">
                            <label class="data-sort">
                                "Sort"
                                <select class="data-filter" on:change=move |event| set_sort.set(event_target_value(&event))>
                                    <option value="date_desc">"Date: newest first"</option>
                                    <option value="date_asc">"Date: oldest first"</option>
                                    <option value="total_desc">"Total: high to low"</option>
                                    <option value="total_asc">"Total: low to high"</option>
                                </select>
                            </label>
                        </div>
                        <div class="data-table-wrap">
                            <table class="data-table">
                                <thead>
                                    <tr>
                                        <th>"Date"</th>
                                        <th>"Meet"</th>
                                        <th>"Division"</th>
                                        <th>"Bodyweight"</th>
                                        <th>"S1"</th>
                                        <th>"S2"</th>
                                        <th>"S3"</th>
                                        <th>"Best snatch"</th>
                                        <th>"C&J 1"</th>
                                        <th>"C&J 2"</th>
                                        <th>"C&J 3"</th>
                                        <th>"Best C&J"</th>
                                        <th>"Total"</th>
                                        <th>"Adaptive"</th>
                                    </tr>
                                </thead>
                                <tbody>{rows}</tbody>
                            </table>
                        </div>
                    }
                    .into_any()
                }
            })}
        </section>
        <Footer />
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_response_deserializes_results_and_suggestions() {
        let response: SearchResponse = serde_json::from_str(
            r#"{"matched_name":"Test Athlete","suggestions":["Test Athlete Jr"],"results":[{"meet":"Nationals","date":"2026-06-20","age":"Senior","body_weight":70.5,"snatch1":95.0,"snatch2":100.0,"snatch3":-103.0,"snatch_best":100.0,"cj1":120.0,"cj2":125.0,"cj3":0.0,"cj_best":125.0,"total":225.0,"adaptive":false}]}"#,
        )
        .unwrap();

        assert_eq!(response.matched_name.as_deref(), Some("Test Athlete"));
        assert_eq!(response.suggestions, ["Test Athlete Jr"]);
        assert_eq!(response.results[0].total, 225.0);
        assert_eq!(response.results[0].snatch3, -103.0);
    }
}
