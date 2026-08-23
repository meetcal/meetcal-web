use super::{
    EmptyTableRow, SelectOptions, TableSkeleton, compare_weight_classes, matches_filter,
    weight_class_options,
};
use crate::{
    components::{footer::Footer, header::Header},
    utils::api::get_api_response_with_query,
};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
struct AdaptiveQuery {
    exclude_federation: &'static str,
    gender: String,
}

#[derive(Debug, Deserialize)]
struct AdaptiveRecord {
    weight_class: String,
    snatch: f64,
    cj: f64,
    total: f64,
}

#[component]
pub fn AdaptiveRecords() -> impl IntoView {
    let (gender, set_gender) = signal("Men".to_owned());
    let (weight_class, set_weight_class) = signal(String::new());
    let (sort, set_sort) = signal("weight_asc".to_owned());
    let records = LocalResource::new(move || {
        let query = AdaptiveQuery {
            exclude_federation: "BWL",
            gender: gender.get(),
        };
        async move {
            get_api_response_with_query::<Vec<AdaptiveRecord>, _>("/data/adaptive", &query).await
        }
    });

    view! {
        <Header />
        <section class="data-page">
            <p class="data-eyebrow">"Competition data"</p>
            <h1>"Adaptive records"</h1>
            <p class="data-intro">"Top USAW and USAMW adaptive performances by gender and weight class."</p>

            {move || records.with(|response| match response {
                None => view! { <TableSkeleton columns=4 /> }.into_any(),
                Some(Err(error)) => view! {
                    <p class="data-status error">{format!("Could not load adaptive records: {error}")}</p>
                }.into_any(),
                Some(Ok(records)) => {
                    let weights = weight_class_options(records.iter().map(|row| row.weight_class.as_str()));
                    let selected_weight = weight_class.get();
                    let mut filtered = records
                        .iter()
                        .filter(|row| matches_filter(&row.weight_class, &selected_weight))
                        .collect::<Vec<_>>();
                    match sort.get().as_str() {
                        "snatch_desc" => filtered.sort_by(|left, right| right.snatch.total_cmp(&left.snatch)),
                        "cj_desc" => filtered.sort_by(|left, right| right.cj.total_cmp(&left.cj)),
                        "total_desc" => filtered.sort_by(|left, right| right.total.total_cmp(&left.total)),
                        _ => filtered.sort_by(|left, right| compare_weight_classes(&left.weight_class, &right.weight_class)),
                    }
                    let is_empty = filtered.is_empty();
                    let rows = filtered.into_iter().map(|row| view! {
                        <tr><td>{row.weight_class.clone()}</td><td>{row.snatch}</td><td>{row.cj}</td><td>{row.total}</td></tr>
                    }).collect_view();

                    view! {
                        <div class="data-filters">
                            <label>"Gender"<select class="data-filter" on:change=move |event| { set_gender.set(event_target_value(&event)); set_weight_class.set(String::new()); }><option value="Men">"Men"</option><option value="Women">"Women"</option></select></label>
                            <label>"Weight class"<select class="data-filter" on:change=move |event| set_weight_class.set(event_target_value(&event))><option value="">"All classes"</option><SelectOptions values=weights selected=Some(selected_weight) /></select></label>
                            <label class="data-sort">"Sort"<select class="data-filter" on:change=move |event| set_sort.set(event_target_value(&event))><option value="weight_asc">"Weight class"</option><option value="total_desc">"Total: high to low"</option><option value="snatch_desc">"Snatch: high to low"</option><option value="cj_desc">"Clean & jerk: high to low"</option></select></label>
                        </div>
                        <div class="data-table-wrap"><table class="data-table">
                            <thead><tr><th>"Weight class"</th><th>"Snatch"</th><th>"Clean & jerk"</th><th>"Total"</th></tr></thead>
                            <tbody>{is_empty.then(|| view! { <EmptyTableRow columns=4 message="No adaptive records match these filters." /> })}{rows}</tbody>
                        </table></div>
                    }.into_any()
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
    fn adaptive_query_matches_the_backend_contract() {
        let query = serde_urlencoded::to_string(AdaptiveQuery {
            exclude_federation: "BWL",
            gender: "Women".to_owned(),
        })
        .unwrap();

        assert_eq!(query, "exclude_federation=BWL&gender=Women");
    }

    #[test]
    fn adaptive_record_deserializes_from_the_api_shape() {
        let record: AdaptiveRecord = serde_json::from_str(
            r#"{"weight_class":"71kg","snatch":84.0,"cj":106.0,"total":190.0}"#,
        )
        .unwrap();

        assert_eq!(record.weight_class, "71kg");
        assert_eq!(record.total, 190.0);
    }
}
