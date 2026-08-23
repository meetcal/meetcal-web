use super::{EmptyTableRow, SelectOptions, TableSkeleton, filter_options, yes_no};
use crate::{
    components::{footer::Footer, header::Header},
    utils::api::{get_api_response, get_api_response_with_query},
};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
struct ClubQuery {
    club: String,
}
#[derive(Clone, Serialize)]
struct StatsQuery {
    club: String,
    meet: String,
}
#[derive(Clone, Deserialize)]
struct ClubAthlete {
    meet: String,
}
#[derive(Clone, Deserialize)]
struct MeetStats {
    total_athletes: i64,
    gold_medals: i64,
    silver_medals: i64,
    bronze_medals: i64,
    total_prs: i64,
    perfect_6_for_6: i64,
    total_weight_lifted: f64,
    snatch_make_rate: i64,
    cj_make_rate: i64,
    combined_make_rate: i64,
    athlete_results: Vec<AthleteResult>,
}
#[derive(Clone, Deserialize)]
struct AthleteResult {
    name: String,
    weight_class: String,
    snatch_best: f64,
    cj_best: f64,
    total: f64,
    body_weight: f64,
    snatch_medal: Option<String>,
    cj_medal: Option<String>,
    total_medal: Option<String>,
    is_pr: bool,
    perfect_lifts: bool,
}

#[component]
pub fn ClubDashboard() -> impl IntoView {
    let (club, set_club) = signal(String::new());
    let (meet, set_meet) = signal(String::new());
    let clubs = LocalResource::new(|| async { get_api_response::<String>("/clubs").await });
    let athletes = LocalResource::new(move || {
        let club = club.get();
        async move {
            if club.is_empty() {
                Ok(Vec::new())
            } else {
                get_api_response_with_query::<Vec<ClubAthlete>, _>(
                    "/clubs/athletes",
                    &ClubQuery { club },
                )
                .await
            }
        }
    });
    let stats = LocalResource::new(move || {
        let query = StatsQuery {
            club: club.get(),
            meet: meet.get(),
        };
        async move {
            if query.club.is_empty() || query.meet.is_empty() {
                Ok(None)
            } else {
                get_api_response_with_query::<MeetStats, _>("/clubs/meet-stats", &query)
                    .await
                    .map(Some)
            }
        }
    });
    view! {
        <Header />
        <section class="data-page">
            <p class="data-eyebrow">"Competition data"</p>
            <h1>"Club meet dashboard"</h1>
            <p class="data-intro">"Review a club’s athletes, medals, PRs, make rates, and meet totals."</p>
            {move || clubs.with(|response| match response {
                None => view! { <p class="data-status">"Loading clubs…"</p> }.into_any(),
                Some(Err(error)) => view! { <p class="data-status error">{format!("Could not load clubs: {error}")}</p> }.into_any(),
                Some(Ok(clubs)) => view! {
                    <div class="data-filters">
                        <label>"Club"<select class="data-filter data-filter-wide" on:change=move |event| { set_club.set(event_target_value(&event)); set_meet.set(String::new()); }>
                            <option value="">"Choose a club"</option><SelectOptions values=clubs.clone() selected=Some(club.get()) />
                        </select></label>
                        {move || athletes.with(|response| match response {
                            Some(Ok(rows)) if !club.get().is_empty() => {
                                let meets = filter_options(rows.iter().map(|row| row.meet.as_str()));
                                view! { <label>"Meet"<select class="data-filter data-filter-wide" on:change=move |event| set_meet.set(event_target_value(&event))>
                                    <option value="">"Choose a completed meet"</option><SelectOptions values=meets selected=Some(meet.get()) />
                                </select></label> }.into_any()
                            }
                            Some(Err(error)) => view! { <p class="data-status error">{format!("Could not load club meets: {error}")}</p> }.into_any(),
                            _ => ().into_any(),
                        })}
                    </div>
                }.into_any(),
            })}
            {move || stats.with(|response| match response {
                None if !meet.get().is_empty() => view! { <TableSkeleton columns=10 /> }.into_any(),
                Some(Err(error)) => view! { <p class="data-status error">{format!("Could not load club dashboard: {error}")}</p> }.into_any(),
                Some(Ok(Some(stats))) => dashboard(stats),
                _ => view! { <p class="data-status">"Choose a club and completed meet to view its dashboard."</p> }.into_any(),
            })}
        </section>
        <Footer />
    }
}

fn metric(label: &'static str, value: String) -> impl IntoView {
    view! { <div class="data-metric"><span>{label}</span><strong>{value}</strong></div> }
}
fn medal(value: &Option<String>) -> String {
    value.clone().unwrap_or_else(|| "—".to_owned())
}
fn dashboard(stats: &MeetStats) -> AnyView {
    let rows = stats.athlete_results.iter().map(|row| view! { <tr><td>{row.name.clone()}</td><td>{row.weight_class.clone()}</td><td>{row.body_weight}</td><td>{row.snatch_best}</td><td>{medal(&row.snatch_medal)}</td><td>{row.cj_best}</td><td>{medal(&row.cj_medal)}</td><td>{row.total}</td><td>{medal(&row.total_medal)}</td><td>{yes_no(row.is_pr)}</td><td>{yes_no(row.perfect_lifts)}</td></tr> }).collect_view();
    view! { <div class="data-metric-grid">{metric("Athletes", stats.total_athletes.to_string())}{metric("Gold medals", stats.gold_medals.to_string())}{metric("Silver medals", stats.silver_medals.to_string())}{metric("Bronze medals", stats.bronze_medals.to_string())}{metric("Total PRs", stats.total_prs.to_string())}{metric("6 for 6", stats.perfect_6_for_6.to_string())}{metric("Weight lifted", format!("{}kg", stats.total_weight_lifted))}{metric("Snatch makes", format!("{}%", stats.snatch_make_rate))}{metric("C&J makes", format!("{}%", stats.cj_make_rate))}{metric("Combined makes", format!("{}%", stats.combined_make_rate))}</div>
    <div class="data-table-wrap"><table class="data-table"><thead><tr><th>"Athlete"</th><th>"Class"</th><th>"Bodyweight"</th><th>"Snatch"</th><th>"Snatch medal"</th><th>"C&J"</th><th>"C&J medal"</th><th>"Total"</th><th>"Total medal"</th><th>"PR"</th><th>"6 for 6"</th></tr></thead><tbody>{stats.athlete_results.is_empty().then(|| view! { <EmptyTableRow columns=11 message="No club results were found for this meet." /> })}{rows}</tbody></table></div> }.into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stats_payload_deserializes() {
        let stats: MeetStats = serde_json::from_str(r#"{"total_athletes":1,"gold_medals":1,"silver_medals":0,"bronze_medals":0,"total_prs":1,"perfect_6_for_6":1,"total_weight_lifted":225.0,"snatch_make_rate":100,"cj_make_rate":100,"combined_make_rate":100,"athlete_results":[]}"#).unwrap();
        assert_eq!(stats.total_athletes, 1);
    }
}
