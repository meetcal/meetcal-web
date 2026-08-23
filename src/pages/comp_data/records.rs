use super::{EmptyTableRow, SelectOptions, TableSkeleton, filter_options, matches_filter};
use crate::{
    components::{footer::Footer, header::Header},
    utils::api::get_api_response,
};
use leptos::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Record {
    age_category: String,
    gender: String,
    weight_class: String,
    record_type: String,
    snatch_record: f64,
    cj_record: f64,
    total_record: f64,
}

#[component]
pub fn Records() -> impl IntoView {
    let (record_type, set_record_type) = signal(String::new());
    let (gender, set_gender) = signal(String::new());
    let (age, set_age) = signal(String::new());
    let (weight_class, set_weight_class) = signal(String::new());
    let (sort, set_sort) = signal("total_desc".to_string());
    let records =
        LocalResource::new(|| async { get_api_response::<Record>("/data/records").await });

    view! {
        <Header />
        <section class="data-page">
            <p class="data-eyebrow">"Competition data"</p>
            <h1>"Records"</h1>
            <p class="data-intro">
                "Filter record performances by organization, division, and weight class."
            </p>

            {move || records.with(|response| match response {
                None => view! { <TableSkeleton columns=7 /> }.into_any(),
                Some(Err(error)) => view! {
                    <p class="data-status error">{format!("Could not load records: {error}")}</p>
                }
                .into_any(),
                Some(Ok(records)) => {
                    let types = filter_options(
                        records
                            .iter()
                            .filter(|record| record.record_type != "BWL")
                            .map(|record| record.record_type.as_str()),
                    );
                    let genders = filter_options(records.iter().map(|record| record.gender.as_str()));
                    let ages = filter_options(records.iter().map(|record| record.age_category.as_str()));
                    let weights = filter_options(records.iter().map(|record| record.weight_class.as_str()));
                    let selected_type = record_type.get();
                    let selected_gender = gender.get();
                    let selected_age = age.get();
                    let selected_weight = weight_class.get();
                    let mut filtered_records = records
                        .iter()
                        .filter(|record| {
                            (selected_type.is_empty() || record.record_type == selected_type)
                                && matches_filter(&record.gender, &selected_gender)
                                && matches_filter(&record.age_category, &selected_age)
                                && matches_filter(&record.weight_class, &selected_weight)
                                && record.record_type != "BWL"
                        })
                        .collect::<Vec<_>>();

                    match sort.get().as_str() {
                        "snatch_desc" => filtered_records.sort_by(|left, right| {
                            right.snatch_record.total_cmp(&left.snatch_record)
                        }),
                        "snatch_asc" => filtered_records.sort_by(|left, right| {
                            left.snatch_record.total_cmp(&right.snatch_record)
                        }),
                        "cj_desc" => filtered_records.sort_by(|left, right| {
                            right.cj_record.total_cmp(&left.cj_record)
                        }),
                        "cj_asc" => filtered_records.sort_by(|left, right| {
                            left.cj_record.total_cmp(&right.cj_record)
                        }),
                        "total_asc" => filtered_records.sort_by(|left, right| {
                            left.total_record.total_cmp(&right.total_record)
                        }),
                        _ => filtered_records.sort_by(|left, right| {
                            right.total_record.total_cmp(&left.total_record)
                        }),
                    }
                    let is_empty = filtered_records.is_empty();
                    let rows = filtered_records
                        .into_iter()
                        .map(|record| view! {
                            <tr>
                                <td>{record.record_type.clone()}</td>
                                <td>{record.gender.clone()}</td>
                                <td>{record.age_category.clone()}</td>
                                <td>{record.weight_class.clone()}</td>
                                <td>{record.snatch_record}</td>
                                <td>{record.cj_record}</td>
                                <td>{record.total_record}</td>
                            </tr>
                        })
                        .collect_view();

                    view! {
                        <div class="data-filters">
                            <label>
                                "Organization"
                                <select class="data-filter" on:change=move |event| set_record_type.set(event_target_value(&event))>
                                    <option value="">"All organizations"</option>
                                    <SelectOptions values=types selected=Some(selected_type.clone()) />
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
                                    <option value="total_desc">"Total: high to low"</option>
                                    <option value="total_asc">"Total: low to high"</option>
                                    <option value="snatch_desc">"Snatch: high to low"</option>
                                    <option value="snatch_asc">"Snatch: low to high"</option>
                                    <option value="cj_desc">"Clean & jerk: high to low"</option>
                                    <option value="cj_asc">"Clean & jerk: low to high"</option>
                                </select>
                            </label>
                        </div>
                        <div class="data-table-wrap">
                            <table class="data-table">
                                <thead>
                                    <tr>
                                        <th>"Type"</th>
                                        <th>"Gender"</th>
                                        <th>"Age"</th>
                                        <th>"Weight class"</th>
                                        <th>"Snatch"</th>
                                        <th>"Clean & jerk"</th>
                                        <th>"Total"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {is_empty.then(|| view! { <EmptyTableRow columns=7 message="No records match these filters." /> })}
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
    fn record_deserializes_from_the_api_shape() {
        let record: Record = serde_json::from_str(
            r#"{"age_category":"Senior","gender":"Women","weight_class":"77kg","record_type":"USAW","snatch_record":130.0,"cj_record":160.0,"total_record":287.0}"#,
        )
        .unwrap();

        assert_eq!(record.record_type, "USAW");
        assert_eq!(record.total_record, 287.0);
    }
}
