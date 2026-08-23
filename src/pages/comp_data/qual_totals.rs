use super::{
    EmptyTableRow, SelectOptions, TableSkeleton, filter_options, matches_filter,
    weight_class_options,
};
use crate::{
    components::{footer::Footer, header::Header},
    utils::api::get_api_response,
};
use leptos::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct QualifyingTotal {
    pub qualifying_total: f64,
    pub event_name: String,
    pub gender: String,
    pub age_category: String,
    pub weight_class: String,
}

#[component]
pub fn QualifyingTotals() -> impl IntoView {
    let (gender, set_gender) = signal(String::new());
    let (meet, set_meet) = signal(String::new());
    let (age, set_age) = signal(String::new());
    let (weight_class, set_weight_class) = signal(String::new());
    let (sort, set_sort) = signal("total_asc".to_string());

    let totals = LocalResource::new(move || async move {
        get_api_response::<QualifyingTotal>("/data/qualifying-totals").await
    });

    view! {
        <Header />

        <section class="data-page">
            <p class="data-eyebrow">"Competition data"</p>
            <h1>"Qualifying totals"</h1>
            <p class="data-intro">
                "Filter qualification totals by event, gender, age category, or weight class."
            </p>

            {move || {
                totals.with(|response| match response {
                    None => view! { <TableSkeleton columns=5 /> }.into_any(),
                    Some(Err(error)) => view! {
                        <p class="data-status error">
                            {format!("Could not load qualifying totals: {error}")}
                        </p>
                    }
                    .into_any(),
                    Some(Ok(rows)) => {
                        let meets = filter_options(rows.iter().map(|row| row.event_name.as_str()));
                        let genders = filter_options(rows.iter().map(|row| row.gender.as_str()));
                        let ages = filter_options(rows.iter().map(|row| row.age_category.as_str()));
                        let weights = weight_class_options(rows.iter().map(|row| row.weight_class.as_str()));

                        let selected_meet = meet.get();
                        let selected_gender = gender.get();
                        let selected_age = age.get();
                        let selected_weight = weight_class.get();
                        let mut filtered_totals = rows
                            .iter()
                            .filter(|row| {
                                (selected_meet.is_empty() || row.event_name == selected_meet)
                                    && matches_filter(&row.gender, &selected_gender)
                                    && matches_filter(&row.age_category, &selected_age)
                                    && matches_filter(&row.weight_class, &selected_weight)
                            })
                            .collect::<Vec<_>>();

                        match sort.get().as_str() {
                            "total_desc" => filtered_totals.sort_by(|left, right| {
                                right.qualifying_total.total_cmp(&left.qualifying_total)
                            }),
                            "event_asc" => filtered_totals.sort_by(|left, right| {
                                left.event_name.cmp(&right.event_name)
                            }),
                            "event_desc" => filtered_totals.sort_by(|left, right| {
                                right.event_name.cmp(&left.event_name)
                            }),
                            _ => filtered_totals.sort_by(|left, right| {
                                left.qualifying_total.total_cmp(&right.qualifying_total)
                            }),
                        }
                        let is_empty = filtered_totals.is_empty();
                        let rows = filtered_totals
                            .into_iter()
                            .map(|row| {
                                view! {
                                    <tr>
                                        <td>{row.event_name.clone()}</td>
                                        <td>{row.gender.clone()}</td>
                                        <td>{row.age_category.clone()}</td>
                                        <td>{row.weight_class.clone()}</td>
                                        <td>{row.qualifying_total}</td>
                                    </tr>
                                }
                            })
                            .collect_view();

                        view! {
                            <div class="data-filters">
                                <label>
                                    "Event"
                                    <select class="data-filter" on:change=move |event| set_meet.set(event_target_value(&event))>
                                        <option value="">"All events"</option>
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
                                        <option value="total_asc">"Total: low to high"</option>
                                        <option value="total_desc">"Total: high to low"</option>
                                        <option value="event_asc">"Event: A to Z"</option>
                                        <option value="event_desc">"Event: Z to A"</option>
                                    </select>
                                </label>
                            </div>
                            <DataTable>
                                <thead>
                                    <tr>
                                        <th>"Event"</th>
                                        <th>"Gender"</th>
                                        <th>"Age"</th>
                                        <th>"Weight class"</th>
                                        <th>"Total"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {is_empty.then(|| view! { <EmptyTableRow columns=5 message="No qualifying totals match these filters." /> })}
                                    {rows}
                                </tbody>
                            </DataTable>
                        }
                        .into_any()
                    }
                })
            }}
        </section>

        <Footer />
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualifying_total_deserializes_from_the_api_shape() {
        let total: QualifyingTotal = serde_json::from_str(
            r#"{"qualifying_total":215.5,"event_name":"Nationals","gender":"Women","age_category":"Senior","weight_class":"69kg"}"#,
        )
        .unwrap();

        assert_eq!(total.event_name, "Nationals");
        assert_eq!(total.qualifying_total, 215.5);
    }
}

#[component]
pub fn DataTable(children: Children) -> impl IntoView {
    view! {
        <div class="data-table-wrap">
            <table class="data-table">
                {children()}
            </table>
        </div>
    }
}
