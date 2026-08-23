use super::{
    EmptyTableRow, SelectOptions, TableSkeleton,
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
            <p class="data-intro">"Open a meet to see its venue, session schedule, start list, and complete results."</p>
            {move || upcoming.with(|upcoming_response| completed.with(|completed_response| match (upcoming_response, completed_response) {
                (Some(Ok(upcoming)), Some(Ok(completed))) => {
                    let mut all = upcoming.iter().chain(completed.iter()).collect::<Vec<_>>();
                    all.sort_by(|left, right| right.start_date.cmp(&left.start_date));
                    let names = all.iter().map(|row| row.name.clone()).collect::<Vec<_>>();
                    let selected = all.into_iter().find(|row| row.name == meet.get()).cloned();
                    view! {
                        <div class="data-filters"><label>"Meet"<select class="data-filter data-filter-wide" on:change=move |event| set_meet.set(event_target_value(&event))>
                            <option value="">"Choose a meet"</option><SelectOptions values=names selected=Some(meet.get()) />
                        </select></label></div>
                        {selected.map(|row| view! {
                            <article class="meet-summary">
                                <div><span class="data-badge">{row.status}</span><h2>{row.name}</h2>
                                <p>{format!("{} – {} · {}", row.start_date, row.end_date, row.time_zone)}</p>
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

            {move || if meet.get().is_empty() { view! { <p class="data-status">"Choose a meet to load its competition data."</p> }.into_any() } else { view! {
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
            let body = rows.iter().map(|row| view! { <tr><td>{row.date.clone()}</td><td>{session(Some(row.session_id))}</td><td>{row.platform.clone()}</td><td>{row.weight_class.clone()}</td><td>{row.weigh_in_time.clone()}</td><td>{row.start_time.clone()}</td></tr> }).collect_view();
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
        Some(Ok(rows)) => { let mut rows = rows.iter().collect::<Vec<_>>(); rows.sort_by(|left, right| right.total.total_cmp(&left.total)); let body = rows.iter().map(|row| view! { <tr><td>{row.name.clone()}</td><td>{row.age.clone()}</td><td>{row.body_weight}</td><td>{attempt(row.snatch1)}</td><td>{attempt(row.snatch2)}</td><td>{attempt(row.snatch3)}</td><td>{row.snatch_best}</td><td>{attempt(row.cj1)}</td><td>{attempt(row.cj2)}</td><td>{attempt(row.cj3)}</td><td>{row.cj_best}</td><td>{row.total}</td><td>{yes_no(row.adaptive)}</td></tr> }).collect_view(); view! { <div class="data-table-wrap"><table class="data-table"><thead><tr><th>"Athlete"</th><th>"Division"</th><th>"Bodyweight"</th><th>"S1"</th><th>"S2"</th><th>"S3"</th><th>"Best snatch"</th><th>"C&J 1"</th><th>"C&J 2"</th><th>"C&J 3"</th><th>"Best C&J"</th><th>"Total"</th><th>"Adaptive"</th></tr></thead><tbody>{rows.is_empty().then(|| view! { <EmptyTableRow columns=13 message="No results have been published for this meet." /> })}{body}</tbody></table></div> }.into_any() }
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
