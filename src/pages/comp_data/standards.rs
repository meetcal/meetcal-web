use super::{EmptyTableRow, SelectOptions, TableSkeleton, filter_options, matches_filter};
use crate::{
    components::{footer::Footer, header::Header},
    utils::api::get_api_response,
};
use leptos::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Standard {
    age_category: String,
    gender: String,
    standard_a: f64,
    standard_b: f64,
    weight_class: String,
}

#[component]
pub fn Standards() -> impl IntoView {
    let (gender, set_gender) = signal(String::new());
    let (age, set_age) = signal(String::new());
    let (weight_class, set_weight_class) = signal(String::new());
    let (sort, set_sort) = signal("standard_a_desc".to_string());
    let standards =
        LocalResource::new(|| async { get_api_response::<Standard>("/data/standards").await });

    view! {
        <Header />
        <section class="data-page">
            <p class="data-eyebrow">"Competition data"</p>
            <h1>"Standards"</h1>
            <p class="data-intro">
                "A and B standards by gender, age category, and weight class."
            </p>

            {move || standards.with(|response| match response {
                None => view! { <TableSkeleton columns=5 /> }.into_any(),
                Some(Err(error)) => view! {
                    <p class="data-status error">{format!("Could not load standards: {error}")}</p>
                }
                .into_any(),
                Some(Ok(rows)) => {
                    let genders = filter_options(rows.iter().map(|row| row.gender.as_str()));
                    let ages = filter_options(rows.iter().map(|row| row.age_category.as_str()));
                    let weights = filter_options(rows.iter().map(|row| row.weight_class.as_str()));
                    let selected_gender = gender.get();
                    let selected_age = age.get();
                    let selected_weight = weight_class.get();
                    let mut filtered_standards = rows
                        .iter()
                        .filter(|row| {
                            matches_filter(&row.gender, &selected_gender)
                                && matches_filter(&row.age_category, &selected_age)
                                && matches_filter(&row.weight_class, &selected_weight)
                        })
                        .collect::<Vec<_>>();

                    match sort.get().as_str() {
                        "standard_a_asc" => filtered_standards.sort_by(|left, right| {
                            left.standard_a.total_cmp(&right.standard_a)
                        }),
                        "standard_b_desc" => filtered_standards.sort_by(|left, right| {
                            right.standard_b.total_cmp(&left.standard_b)
                        }),
                        "standard_b_asc" => filtered_standards.sort_by(|left, right| {
                            left.standard_b.total_cmp(&right.standard_b)
                        }),
                        _ => filtered_standards.sort_by(|left, right| {
                            right.standard_a.total_cmp(&left.standard_a)
                        }),
                    }
                    let is_empty = filtered_standards.is_empty();
                    let rows = filtered_standards
                        .into_iter()
                        .map(|row| view! {
                            <tr>
                                <td>{row.gender.clone()}</td>
                                <td>{row.age_category.clone()}</td>
                                <td>{row.weight_class.clone()}</td>
                                <td>{row.standard_a}</td>
                                <td>{row.standard_b}</td>
                            </tr>
                        })
                        .collect_view();

                    view! {
                        <div class="data-filters">
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
                                    <option value="standard_a_desc">"A: high to low"</option>
                                    <option value="standard_a_asc">"A: low to high"</option>
                                    <option value="standard_b_desc">"B: high to low"</option>
                                    <option value="standard_b_asc">"B: low to high"</option>
                                </select>
                            </label>
                        </div>
                        <div class="data-table-wrap">
                            <table class="data-table">
                                <thead>
                                    <tr>
                                        <th>"Gender"</th>
                                        <th>"Age"</th>
                                        <th>"Weight class"</th>
                                        <th>"A"</th>
                                        <th>"B"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {is_empty.then(|| view! { <EmptyTableRow columns=5 message="No standards match these filters." /> })}
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
    fn standard_deserializes_from_the_api_shape() {
        let standard: Standard = serde_json::from_str(
            r#"{"age_category":"Junior","gender":"Men","standard_a":285.0,"standard_b":260.0,"weight_class":"79kg"}"#,
        )
        .unwrap();

        assert_eq!(standard.standard_a, 285.0);
        assert_eq!(standard.standard_b, 260.0);
    }
}
