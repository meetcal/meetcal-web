use std::collections::HashSet;

use super::{
    EmptyTableRow, TableSkeleton, format_us_date, format_us_time,
    models::{Athlete, LiftingResult, Meet, MeetQuery, ScheduleRow, attempt},
    yes_no,
};
use crate::{
    components::{footer::Footer, header::Header},
    utils::api::{get_api_response, get_api_response_with_query},
};
use leptos::prelude::*;

fn session(value: Option<f64>) -> String {
    value
        .map(|value| {
            if value.fract() == 0.0 {
                format!("{value:.0}")
            } else {
                value.to_string()
            }
        })
        .unwrap_or_else(|| "—".to_owned())
}

#[component]
pub fn MeetCenter() -> impl IntoView {
    let (meet, set_meet) = signal(String::new());
    let (meet_search, set_meet_search) = signal(String::new());
    let upcoming = LocalResource::new(|| async { get_api_response::<Meet>("/meets").await });
    let completed =
        LocalResource::new(|| async { get_api_response::<Meet>("/meets/completed").await });
    let schedule =
        LocalResource::new(move || load_selected::<ScheduleRow>(meet.get(), "/meets/schedule"));
    let athletes = LocalResource::new(move || {
        load_selected::<Athlete>(meet.get(), "/meets/athletes-sessions")
    });
    let results =
        LocalResource::new(move || load_selected::<LiftingResult>(meet.get(), "/lifting-results"));

    view! {
        <Header />
        <section class="data-page">
            <p class="data-eyebrow">"Competition data"</p>
            <h1>"Meets"</h1>
            <p class="data-intro">"Search all meets to see venue details, session schedules, start lists, and published results."</p>
            {move || upcoming.with(|upcoming| completed.with(|completed| match (upcoming, completed) {
                (Some(Ok(upcoming)), Some(Ok(completed))) => {
                    let mut meets = upcoming.iter().chain(completed.iter()).cloned().collect::<Vec<_>>();
                    meets.sort_by(|left, right| right.start_date.cmp(&left.start_date).then_with(|| left.name.cmp(&right.name)));
                    let mut seen = HashSet::new();
                    meets.retain(|row| seen.insert(row.name.clone()));
                    let query = meet_search.get();
                    let normalized_query = query.trim().to_lowercase();
                    let suggestions = (normalized_query.chars().count() >= 3 && meet.get().is_empty()).then(|| {
                        meets
                            .iter()
                            .filter(|row| row.name.to_lowercase().contains(&normalized_query))
                            .take(8)
                            .map(|row| row.name.clone())
                            .collect::<Vec<_>>()
                    });
                    let selected = meets.iter().find(|row| row.name == meet.get()).cloned();
                    view! {
                        <div class="meet-search">
                            <label for="meet-search-input">"Meet"</label>
                            <input id="meet-search-input" class="data-filter data-filter-wide" type="search" autocomplete="off" placeholder="Type at least 3 characters" prop:value=move || meet_search.get() on:input=move |event| { let value = event_target_value(&event); set_meet_search.set(value.clone()); if meet.get() != value { set_meet.set(String::new()); } } />
                            {suggestions.map(|matches| view! {
                                <div class="meet-suggestions" role="listbox" aria-label="Meet suggestions">
                                    {matches.is_empty().then(|| view! { <p>"No matching meets"</p> })}
                                    {matches.into_iter().map(|name| { let selected_name = name.clone(); view! { <button type="button" role="option" on:click=move |_| { set_meet_search.set(selected_name.clone()); set_meet.set(selected_name.clone()); }>{name}</button> } }).collect_view()}
                                </div>
                            })}
                        </div>
                        {selected.map(|row| view! {
                            <article class="meet-summary">
                                <div><span class="data-badge">{row.status}</span><h2>{row.name}</h2>
                                <p>{format!("{} – {} · {}", format_us_date(&row.start_date), format_us_date(&row.end_date), row.time_zone)}</p>
                                <p>{format!("{}, {} · {}, {} {}", row.venue_name, row.venue_street, row.venue_city, row.venue_state, row.venue_zip)}</p></div>
                                <div class="meet-links">
                                    {row.venue_map_apple_url.map(|url| view! { <a href=url target="_blank" rel="noopener noreferrer">"Open venue map"</a> })}
                                    {row.venue_map_pdf_url.map(|url| view! { <a href=url target="_blank" rel="noopener noreferrer">"Venue PDF"</a> })}
                                </div>
                            </article>
                        })}
                    }.into_any()
                }
                (Some(Err(error)), _) | (_, Some(Err(error))) => view! { <p class="data-status error">{format!("Could not load meets: {error}")}</p> }.into_any(),
                _ => view! { <p class="data-status">"Loading meets…"</p> }.into_any(),
            }))}

            {move || if meet.get().is_empty() { view! { <p class="data-status">"Search for and select a meet to load its competition data."</p> }.into_any() } else { view! {
                <h2 class="data-section-title">"Schedule"</h2>
                {move || schedule.with(|response| table_schedule(response.as_ref()))}
                <h2 class="data-section-title">"Start list"</h2>
                {move || athletes.with(|response| table_athletes(response.as_ref()))}
                <h2 class="data-section-title">"Full results"</h2>
                {move || results.with(|response| table_results(response.as_ref()))}
            }.into_any() }}
        </section>
        <Footer />
    }
}

async fn load_selected<T: serde::de::DeserializeOwned>(
    meet: String,
    path: &str,
) -> Result<Vec<T>, String> {
    if meet.is_empty() {
        return Ok(Vec::new());
    }
    get_api_response_with_query(path, &MeetQuery { meet })
        .await
        .map_err(|error| error.to_string())
}

fn table_schedule(response: Option<&Result<Vec<ScheduleRow>, String>>) -> AnyView {
    match response {
        None => view! { <TableSkeleton columns=7 /> }.into_any(),
        Some(Err(error)) => {
            view! { <p class="data-status error">{format!("Could not load schedule: {error}")}</p> }
                .into_any()
        }
        Some(Ok(rows)) => {
            let body = rows.iter().map(|row| view! { <tr><td>{format_us_date(&row.date)}</td><td>{session(Some(row.session_id))}</td><td>{row.platform.clone()}</td><td>{row.weight_class.clone()}</td><td>{format_us_time(&row.weigh_in_time)}</td><td>{format_us_time(&row.start_time)}</td></tr> }).collect_view();
            view! { <div class="data-table-wrap"><table class="data-table"><thead><tr><th>"Date"</th><th>"Session"</th><th>"Platform"</th><th>"Weight class"</th><th>"Weigh-in"</th><th>"Start"</th></tr></thead><tbody>{rows.is_empty().then(|| view! { <EmptyTableRow columns=6 message="No schedule has been published for this meet." /> })}{body}</tbody></table></div> }.into_any()
        }
    }
}

fn table_athletes(response: Option<&Result<Vec<Athlete>, String>>) -> AnyView {
    match response {
        None => view! { <TableSkeleton columns=9 /> }.into_any(),
        Some(Err(error)) => view! { <p class="data-status error">{format!("Could not load start list: {error}")}</p> }.into_any(),
        Some(Ok(rows)) => { let body = rows.iter().map(|row| view! { <tr><td>{row.name.clone()}</td><td>{row.gender.clone()}</td><td>{row.age}</td><td>{row.weight_class.clone()}</td><td>{row.entry_total}</td><td>{row.club.clone()}</td><td>{row.wso.clone().unwrap_or_else(|| "—".to_owned())}</td><td>{session(row.session_number)}</td><td>{row.session_platform.clone().unwrap_or_else(|| "—".to_owned())}</td><td>{yes_no(row.adaptive)}</td></tr> }).collect_view(); view! { <div class="data-table-wrap"><table class="data-table"><thead><tr><th>"Athlete"</th><th>"Gender"</th><th>"Age"</th><th>"Weight class"</th><th>"Entry total"</th><th>"Club"</th><th>"WSO"</th><th>"Session"</th><th>"Platform"</th><th>"Adaptive"</th></tr></thead><tbody>{rows.is_empty().then(|| view! { <EmptyTableRow columns=10 message="No start list has been published for this meet." /> })}{body}</tbody></table></div> }.into_any() }
    }
}

fn table_results(response: Option<&Result<Vec<LiftingResult>, String>>) -> AnyView {
    match response {
        None => view! { <TableSkeleton columns=13 /> }.into_any(),
        Some(Err(error)) => view! { <p class="data-status error">{format!("Could not load meet results: {error}")}</p> }.into_any(),
        Some(Ok(rows)) => { let mut rows = rows.iter().collect::<Vec<_>>(); rows.sort_by(|left, right| right.total.total_cmp(&left.total)); let body = rows.iter().map(|row| { let athlete_url = format!("/results?athlete={}", js_sys::encode_uri_component(&row.name)); view! { <tr><td><a class="data-athlete-link" href=athlete_url>{row.name.clone()}</a></td><td><strong>{row.total}</strong></td><td>{row.age.clone()}</td><td>{row.body_weight}</td><td>{attempt(row.snatch1)}</td><td>{attempt(row.snatch2)}</td><td>{attempt(row.snatch3)}</td><td>{row.snatch_best}</td><td>{attempt(row.cj1)}</td><td>{attempt(row.cj2)}</td><td>{attempt(row.cj3)}</td><td>{row.cj_best}</td><td>{yes_no(row.adaptive)}</td></tr> } }).collect_view(); view! { <div class="data-table-wrap"><table class="data-table"><thead><tr><th>"Athlete"</th><th>"Total"</th><th>"Division"</th><th>"Bodyweight"</th><th>"S1"</th><th>"S2"</th><th>"S3"</th><th>"Best snatch"</th><th>"C&J 1"</th><th>"C&J 2"</th><th>"C&J 3"</th><th>"Best C&J"</th><th>"Adaptive"</th></tr></thead><tbody>{rows.is_empty().then(|| view! { <EmptyTableRow columns=13 message="No results have been published for this meet." /> })}{body}</tbody></table></div> }.into_any() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn absent_sessions_use_dash() {
        assert_eq!(session(None), "—");
        assert_eq!(session(Some(3.0)), "3");
    }
}
