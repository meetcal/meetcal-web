use super::{
    TableSkeleton,
    analytics::{WrappedStats, wrapped_stats},
    models::LiftingResult,
};
use crate::{
    components::{footer::Footer, header::Header},
    utils::api::get_api_response_with_query,
};
use js_sys::Date;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
struct SearchQuery {
    query: String,
    start_date: String,
    end_date: String,
}
#[derive(Clone, Deserialize)]
struct SearchResponse {
    matched_name: Option<String>,
    suggestions: Vec<String>,
    results: Vec<LiftingResult>,
}
#[derive(Clone)]
struct WrappedRequest {
    first: String,
    second: String,
    year: i32,
}
struct WrappedResponse {
    first_name: String,
    first: WrappedStats,
    second: Option<(String, WrappedStats)>,
}

#[component]
pub fn Wrapped() -> impl IntoView {
    let current_year = Date::new_0().get_full_year() as i32;
    let (first, set_first) = signal(String::new());
    let (second, set_second) = signal(String::new());
    let (year, set_year) = signal(current_year.to_string());
    let (request, set_request) = signal(None::<WrappedRequest>);
    let wrapped = LocalResource::new(move || {
        let request = request.get();
        async move {
            match request {
                Some(request) => load_wrapped(request).await,
                None => Ok(None),
            }
        }
    });
    view! { <Header /><section class="data-page"><p class="data-eyebrow">"Competition data"</p><h1>"Wrapped & comparisons"</h1><p class="data-intro">"Summarize an athlete’s year, or add a second athlete for a side-by-side comparison."</p>
        <form class="data-query-form" on:submit=move |event| { event.prevent_default(); let first = first.get().trim().to_owned(); if first.is_empty() { return; } let selected_year = year.get().parse().unwrap_or(current_year); set_request.set(Some(WrappedRequest { first, second: second.get().trim().to_owned(), year: selected_year })); }>
            <label class="data-query-grow">"Athlete"<input class="data-filter" required=true placeholder="Athlete name" on:input=move |event| set_first.set(event_target_value(&event)) /></label>
            <label class="data-query-grow">"Compare with (optional)"<input class="data-filter" placeholder="Second athlete" on:input=move |event| set_second.set(event_target_value(&event)) /></label>
            <label>"Year"<input class="data-filter data-year-input" inputmode="numeric" maxlength="4" prop:value=current_year on:input=move |event| set_year.set(event_target_value(&event)) /></label>
            <button class="data-search-button" type="submit">"Build wrapped"</button>
        </form>
        {move || wrapped.with(|response| match response { None if request.get().is_some() => view! { <TableSkeleton columns=3 /> }.into_any(), Some(Err(error)) => view! { <p class="data-status error">{format!("Could not build wrapped: {error}")}</p> }.into_any(), Some(Ok(Some(response))) => report(response, request.get().map(|request| request.year).unwrap_or(current_year)), _ => view! { <p class="data-status">"Enter an athlete to build their yearly recap."</p> }.into_any() })}
    </section><Footer /> }
}

async fn athlete(name: &str, year: i32) -> Result<(String, WrappedStats), String> {
    let query = SearchQuery {
        query: name.to_owned(),
        start_date: format!("{year:04}-01-01"),
        end_date: format!("{:04}-01-01", year + 1),
    };
    let response: SearchResponse = get_api_response_with_query("/search", &query)
        .await
        .map_err(|error| error.to_string())?;
    if response.results.is_empty() {
        let suffix = if response.suggestions.is_empty() {
            String::new()
        } else {
            format!(" Try: {}.", response.suggestions.join(", "))
        };
        return Err(format!("No {year} results found for {name}.{suffix}"));
    }
    Ok((
        response.matched_name.unwrap_or_else(|| name.to_owned()),
        wrapped_stats(&response.results),
    ))
}
async fn load_wrapped(request: WrappedRequest) -> Result<Option<WrappedResponse>, String> {
    let (first_name, first) = athlete(&request.first, request.year).await?;
    let second = if request.second.is_empty() {
        None
    } else {
        Some(athlete(&request.second, request.year).await?)
    };
    Ok(Some(WrappedResponse {
        first_name,
        first,
        second,
    }))
}

fn metric(label: &'static str, value: String) -> impl IntoView {
    view! { <div class="data-metric"><span>{label}</span><strong>{value}</strong></div> }
}
fn report(response: &WrappedResponse, year: i32) -> AnyView {
    let stats = &response.first;
    let comparison = response.second.as_ref().map(|(name, second)| { let pairs = [("Meets", stats.total_meets.to_string(), second.total_meets.to_string()), ("Make rate", format!("{:.1}%", stats.make_percentage), format!("{:.1}%", second.make_percentage)), ("Best snatch", format!("{}kg", stats.best_snatch), format!("{}kg", second.best_snatch)), ("Best C&J", format!("{}kg", stats.best_cj), format!("{}kg", second.best_cj)), ("Best total", format!("{}kg", stats.best_total), format!("{}kg", second.best_total)), ("Average total", format!("{:.1}kg", stats.average_total), format!("{:.1}kg", second.average_total)), ("Weight lifted", format!("{}kg", stats.total_weight_lifted), format!("{}kg", second.total_weight_lifted))]; let rows = pairs.into_iter().map(|(label, first, second)| view! { <tr><th>{label}</th><td>{first}</td><td>{second}</td></tr> }).collect_view(); view! { <h2 class="data-section-title">"Comparison"</h2><div class="data-table-wrap"><table class="data-table comparison-table"><thead><tr><th>"Metric"</th><th>{response.first_name.clone()}</th><th>{name.clone()}</th></tr></thead><tbody>{rows}</tbody></table></div> } });
    view! { <h2 class="data-section-title">{format!("{year} wrapped — {}", response.first_name)}</h2><div class="data-metric-grid">{metric("Meets", stats.total_meets.to_string())}{metric("Make rate", format!("{:.1}%", stats.make_percentage))}{metric("Best snatch", format!("{}kg", stats.best_snatch))}{metric("Best C&J", format!("{}kg", stats.best_cj))}{metric("Best total", format!("{}kg", stats.best_total))}{metric("Average total", format!("{:.1}kg", stats.average_total))}{metric("Weight lifted", format!("{}kg", stats.total_weight_lifted))}{metric("First-to-last", format!("{:+}kg", stats.improvement))}{metric("Longest make streak", stats.longest_streak.to_string())}{metric("Favorite attempt", stats.favorite_attempt.map(|value| value.to_string()).unwrap_or_else(|| "—".to_owned()))}{metric("Top meet", stats.top_meet.clone().unwrap_or_else(|| "—".to_owned()))}</div>{comparison} }.into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn search_query_uses_exclusive_next_year() {
        let query = serde_urlencoded::to_string(SearchQuery {
            query: "Test Athlete".to_owned(),
            start_date: "2026-01-01".to_owned(),
            end_date: "2027-01-01".to_owned(),
        })
        .unwrap();
        assert!(query.contains("end_date=2027-01-01"));
    }
}
