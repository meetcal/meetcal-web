use std::collections::HashSet;

use super::{
    EmptyTableRow, SelectOptions, TableSkeleton,
    analytics::percentage,
    filter_options,
    models::{Athlete, LiftingResult, Meet, MeetQuery, normalize},
};
use crate::{
    components::{footer::Footer, header::Header},
    utils::api::{get_api_response, get_api_response_with_query},
};
use leptos::prelude::*;

#[component]
pub fn WsoDashboard() -> impl IntoView {
    let (meet, set_meet) = signal(String::new());
    let (wso, set_wso) = signal(String::new());
    let upcoming = LocalResource::new(|| async { get_api_response::<Meet>("/meets").await });
    let completed =
        LocalResource::new(|| async { get_api_response::<Meet>("/meets/completed").await });
    let wsos = LocalResource::new(|| async { get_api_response::<String>("/data/wso").await });
    let athletes = LocalResource::new(move || load::<Athlete>(meet.get(), "/meets/athletes"));
    let results = LocalResource::new(move || load::<LiftingResult>(meet.get(), "/lifting-results"));
    view! { <Header /><section class="data-page"><p class="data-eyebrow">"Competition data"</p><h1>"WSO meet dashboard"</h1><p class="data-intro">"See participation, make rates, lifted volume, and athlete totals for one WSO at a meet."</p>
        {move || upcoming.with(|upcoming| completed.with(|completed| match (upcoming, completed) { (Some(Ok(upcoming)), Some(Ok(completed))) => { let mut meets = upcoming.iter().chain(completed.iter()).map(|row| row.name.clone()).collect::<Vec<_>>(); meets.sort(); meets.dedup(); view! { <div class="data-filters"><label>"Meet"<select class="data-filter data-filter-wide" on:change=move |event| { set_meet.set(event_target_value(&event)); set_wso.set(String::new()); }><option value="">"Choose a meet"</option><SelectOptions values=meets selected=Some(meet.get()) /></select></label>
            {move || wsos.with(|response| match response { Some(Ok(rows)) if !meet.get().is_empty() => { let options = filter_options(rows.iter().map(String::as_str)); view! { <label>"WSO"<select class="data-filter data-filter-wide" on:change=move |event| set_wso.set(event_target_value(&event))><option value="">"Choose a WSO"</option><SelectOptions values=options selected=Some(wso.get()) /></select></label> }.into_any() }, Some(Err(error)) => view! { <p class="data-status error">{format!("Could not load WSOs: {error}")}</p> }.into_any(), _ => ().into_any() })}</div> }.into_any() }, (Some(Err(error)), _) | (_, Some(Err(error))) => view! { <p class="data-status error">{format!("Could not load meets: {error}")}</p> }.into_any(), _ => view! { <p class="data-status">"Loading meets…"</p> }.into_any() }))}
        {move || if wso.get().is_empty() { view! { <p class="data-status">"Choose a meet and WSO to view its dashboard."</p> }.into_any() } else { athletes.with(|athlete_response| results.with(|result_response| match (athlete_response, result_response) { (Some(Ok(athletes)), Some(Ok(results))) => dashboard(athletes, results, &wso.get()), (Some(Err(error)), _) | (_, Some(Err(error))) => view! { <p class="data-status error">{format!("Could not load WSO dashboard: {error}")}</p> }.into_any(), _ => view! { <TableSkeleton columns=8 /> }.into_any() })) }}
    </section><Footer /> }
}

async fn load<T: serde::de::DeserializeOwned>(meet: String, path: &str) -> Result<Vec<T>, String> {
    if meet.is_empty() {
        Ok(Vec::new())
    } else {
        get_api_response_with_query(path, &MeetQuery { meet })
            .await
            .map_err(|error| error.to_string())
    }
}
fn metric(label: &'static str, value: String) -> impl IntoView {
    view! { <div class="data-metric"><span>{label}</span><strong>{value}</strong></div> }
}

fn dashboard(athletes: &[Athlete], results: &[LiftingResult], selected_wso: &str) -> AnyView {
    let members = athletes
        .iter()
        .filter(|athlete| {
            athlete
                .wso
                .as_deref()
                .is_some_and(|wso| normalize(wso) == normalize(selected_wso))
        })
        .collect::<Vec<_>>();
    let names = members
        .iter()
        .map(|row| normalize(&row.name))
        .collect::<HashSet<_>>();
    let member_results = results
        .iter()
        .filter(|row| names.contains(&normalize(&row.name)))
        .collect::<Vec<_>>();
    let mut attempts = 0usize;
    let mut makes = 0usize;
    let mut snatch_attempts = 0usize;
    let mut snatch_makes = 0usize;
    let mut cj_attempts = 0usize;
    let mut cj_makes = 0usize;
    let mut volume = 0.0;
    for row in &member_results {
        for value in [row.snatch1, row.snatch2, row.snatch3] {
            record(value, &mut snatch_attempts, &mut snatch_makes, &mut volume);
        }
        for value in [row.cj1, row.cj2, row.cj3] {
            record(value, &mut cj_attempts, &mut cj_makes, &mut volume);
        }
    }
    attempts += snatch_attempts + cj_attempts;
    makes += snatch_makes + cj_makes;
    let rows = members.iter().map(|athlete| { let result = member_results.iter().filter(|result| normalize(&result.name) == normalize(&athlete.name)).max_by(|left, right| left.total.total_cmp(&right.total)); view! { <tr><td>{athlete.name.clone()}</td><td>{athlete.gender.clone()}</td><td>{athlete.weight_class.clone()}</td><td>{athlete.club.clone()}</td><td>{athlete.entry_total}</td><td>{result.map(|row| row.snatch_best).unwrap_or_default()}</td><td>{result.map(|row| row.cj_best).unwrap_or_default()}</td><td>{result.map(|row| row.total).unwrap_or_default()}</td></tr> } }).collect_view();
    view! { <div class="data-metric-grid">{metric("Meet athletes", athletes.len().to_string())}{metric("WSO athletes", members.len().to_string())}{metric("Snatch makes", format!("{:.1}%", percentage(snatch_makes, snatch_attempts)))}{metric("C&J makes", format!("{:.1}%", percentage(cj_makes, cj_attempts)))}{metric("Combined makes", format!("{:.1}%", percentage(makes, attempts)))}{metric("Weight lifted", format!("{volume}kg"))}</div>
    <div class="data-table-wrap"><table class="data-table"><thead><tr><th>"Athlete"</th><th>"Gender"</th><th>"Class"</th><th>"Club"</th><th>"Entry total"</th><th>"Snatch"</th><th>"C&J"</th><th>"Total"</th></tr></thead><tbody>{members.is_empty().then(|| view! { <EmptyTableRow columns=8 message="No athletes from this WSO were found at the meet." /> })}{rows}</tbody></table></div> }.into_any()
}
fn record(value: f64, attempts: &mut usize, makes: &mut usize, volume: &mut f64) {
    if value == 0.0 || !value.is_finite() {
        return;
    }
    *attempts += 1;
    if value > 0.0 {
        *makes += 1;
        *volume += value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn misses_count_but_unrecorded_attempts_do_not() {
        let (mut attempts, mut makes, mut volume) = (0, 0, 0.0);
        record(-100.0, &mut attempts, &mut makes, &mut volume);
        record(0.0, &mut attempts, &mut makes, &mut volume);
        record(105.0, &mut attempts, &mut makes, &mut volume);
        assert_eq!((attempts, makes, volume), (2, 1, 105.0));
    }
}
