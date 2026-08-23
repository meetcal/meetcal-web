use super::{EmptyTableRow, SelectOptions, TableSkeleton, filter_options, matches_filter};
use crate::{
    components::{footer::Footer, header::Header},
    utils::api::get_api_response,
};
use leptos::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Ranking {
    meet: String,
    ranking: f64,
    name: String,
    weight_class: String,
    total: f64,
    percent_a: f64,
    gender: String,
    age_category: String,
}

#[component]
pub fn Rankings() -> impl IntoView {
    let (meet, set_meet) = signal(String::new());
    let (gender, set_gender) = signal(String::new());
    let (age, set_age) = signal(String::new());
    let (weight_class, set_weight_class) = signal(String::new());
    let (sort, set_sort) = signal("rank_asc".to_string());
    let rankings =
        LocalResource::new(|| async { get_api_response::<Ranking>("/data/intl-rankings").await });

    view! {
        <Header />
        <section class="data-page">
            <p class="data-eyebrow">"Competition data"</p>
            <h1>"International rankings"</h1>
            <p class="data-intro">
                "Search current international rankings by athlete, meet, division, or weight class."
            </p>

            {move || rankings.with(|response| match response {
                None => view! { <TableSkeleton columns=8 /> }.into_any(),
                Some(Err(error)) => view! {
                    <p class="data-status error">{format!("Could not load rankings: {error}")}</p>
                }
                .into_any(),
                Some(Ok(rankings)) => {
                    let meets = filter_options(rankings.iter().map(|ranking| ranking.meet.as_str()));
                    let genders = filter_options(rankings.iter().map(|ranking| ranking.gender.as_str()));
                    let ages = filter_options(rankings.iter().map(|ranking| ranking.age_category.as_str()));
                    let weights = filter_options(rankings.iter().map(|ranking| ranking.weight_class.as_str()));
                    let selected_meet = meet.get();
                    let selected_gender = gender.get();
                    let selected_age = age.get();
                    let selected_weight = weight_class.get();
                    let mut filtered_rankings = rankings
                        .iter()
                        .filter(|ranking| {
                            (selected_meet.is_empty() || ranking.meet == selected_meet)
                                && matches_filter(&ranking.gender, &selected_gender)
                                && matches_filter(&ranking.age_category, &selected_age)
                                && matches_filter(&ranking.weight_class, &selected_weight)
                        })
                        .collect::<Vec<_>>();

                    match sort.get().as_str() {
                        "rank_desc" => filtered_rankings
                            .sort_by(|left, right| right.ranking.total_cmp(&left.ranking)),
                        "percent_a_desc" => filtered_rankings
                            .sort_by(|left, right| right.percent_a.total_cmp(&left.percent_a)),
                        "percent_a_asc" => filtered_rankings
                            .sort_by(|left, right| left.percent_a.total_cmp(&right.percent_a)),
                        "total_desc" => filtered_rankings
                            .sort_by(|left, right| right.total.total_cmp(&left.total)),
                        "total_asc" => filtered_rankings
                            .sort_by(|left, right| left.total.total_cmp(&right.total)),
                        _ => filtered_rankings
                            .sort_by(|left, right| left.ranking.total_cmp(&right.ranking)),
                    }
                    let is_empty = filtered_rankings.is_empty();
                    let rows = filtered_rankings
                        .into_iter()
                        .map(|ranking| view! {
                            <tr>
                                <td>{ranking.ranking}</td>
                                <td>{ranking.name.clone()}</td>
                                <td>{ranking.meet.clone()}</td>
                                <td>{ranking.gender.clone()}</td>
                                <td>{ranking.age_category.clone()}</td>
                                <td>{ranking.weight_class.clone()}</td>
                                <td>{ranking.total}</td>
                                <td>{ranking.percent_a}</td>
                            </tr>
                        })
                        .collect_view();

                    view! {
                        <div class="data-filters">
                            <label>
                                "Meet"
                                <select class="data-filter" on:change=move |event| set_meet.set(event_target_value(&event))>
                                    <option value="">"All meets"</option>
                                    <SelectOptions values=meets selected=Some(selected_meet.clone()) />
                                </select>
                            </label>
                            <label>
                                "Gender"
                                <select class="data-filter" on:change=move |event| set_gender.set(event_target_value(&event))>
                                    <option value="">"All genders"</option>
                                    <SelectOptions values=genders selected=Some(selected_gender.clone()) />
                                </select>
                            </label>
                            <label>
                                "Age"
                                <select class="data-filter" on:change=move |event| set_age.set(event_target_value(&event))>
                                    <option value="">"All ages"</option>
                                    <SelectOptions values=ages selected=Some(selected_age.clone()) />
                                </select>
                            </label>
                            <label>
                                "Weight class"
                                <select class="data-filter" on:change=move |event| set_weight_class.set(event_target_value(&event))>
                                    <option value="">"All classes"</option>
                                    <SelectOptions values=weights selected=Some(selected_weight.clone()) />
                                </select>
                            </label>
                            <label class="data-sort">
                                "Sort"
                                <select class="data-filter" on:change=move |event| set_sort.set(event_target_value(&event))>
                                    <option value="rank_asc">"Rank: low to high"</option>
                                    <option value="rank_desc">"Rank: high to low"</option>
                                    <option value="percent_a_desc">"Percent A: high to low"</option>
                                    <option value="percent_a_asc">"Percent A: low to high"</option>
                                    <option value="total_desc">"Total: high to low"</option>
                                    <option value="total_asc">"Total: low to high"</option>
                                </select>
                            </label>
                        </div>
                        <div class="data-table-wrap">
                            <table class="data-table">
                                <thead>
                                    <tr>
                                        <th>"Rank"</th>
                                        <th>"Athlete"</th>
                                        <th>"Meet"</th>
                                        <th>"Gender"</th>
                                        <th>"Age"</th>
                                        <th>"Weight class"</th>
                                        <th>"Total"</th>
                                        <th>"Percent A"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {is_empty.then(|| view! { <EmptyTableRow columns=8 message="No rankings match these filters." /> })}
                                    {rows}
                                </tbody>
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
    fn international_ranking_deserializes_from_the_api_shape() {
        let ranking: Ranking = serde_json::from_str(
            r#"{"meet":"World Championships","ranking":2.0,"name":"Test Athlete","weight_class":"88kg","total":350.0,"percent_a":97.4,"gender":"Men","age_category":"Senior"}"#,
        )
        .unwrap();

        assert_eq!(ranking.ranking, 2.0);
        assert_eq!(ranking.percent_a, 97.4);
    }
}
