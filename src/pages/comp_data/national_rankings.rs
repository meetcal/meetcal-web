use super::{EmptyTableRow, TableSkeleton};
use crate::{
    components::{footer::Footer, header::Header},
    utils::api::get_api_response_with_query,
};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
struct NationalRankingQuery {
    federation: String,
    age_category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    year: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NationalRanking {
    name: String,
    total: f64,
    date: Option<String>,
}

#[component]
pub fn NationalRankings() -> impl IntoView {
    let (federation, set_federation) = signal("USAW".to_owned());
    let (division, set_division) = signal(String::new());
    let (year, set_year) = signal(String::new());
    let (request, set_request) = signal(None::<NationalRankingQuery>);

    let rankings = LocalResource::new(move || {
        let request = request.get();
        async move {
            match request {
                Some(query) => {
                    let path = if query.year.is_some() {
                        "/data/nat-rankings-year"
                    } else {
                        "/data/nat-rankings"
                    };
                    get_api_response_with_query::<Vec<NationalRanking>, _>(path, &query)
                        .await
                        .map_err(|error| error.to_string())
                }
                None => Ok(Vec::new()),
            }
        }
    });

    view! {
        <Header />
        <section class="data-page">
            <p class="data-eyebrow">"Competition data"</p>
            <h1>"National rankings"</h1>
            <p class="data-intro">
                "Find each athlete’s best total for a USAW or USAMW division. Add a year to limit the rankings to that season."
            </p>

            <form class="data-query-form" on:submit=move |event| {
                event.prevent_default();
                let age_category = division.get().trim().to_owned();
                if !age_category.is_empty() {
                    let selected_year = year.get().trim().to_owned();
                    set_request.set(Some(NationalRankingQuery {
                        federation: federation.get(),
                        age_category,
                        year: (!selected_year.is_empty()).then_some(selected_year),
                    }));
                }
            }>
                <label>
                    "Federation"
                    <select class="data-filter" on:change=move |event| set_federation.set(event_target_value(&event))>
                        <option value="USAW">"USAW"</option>
                        <option value="USAMW">"USAMW"</option>
                    </select>
                </label>
                <label class="data-query-grow">
                    "Division"
                    <input
                        class="data-filter"
                        placeholder="Open Men's 60kg"
                        required=true
                        on:input=move |event| set_division.set(event_target_value(&event))
                    />
                </label>
                <label>
                    "Year (optional)"
                    <input
                        class="data-filter data-year-input"
                        inputmode="numeric"
                        maxlength="4"
                        placeholder="All time"
                        on:input=move |event| set_year.set(event_target_value(&event))
                    />
                </label>
                <button class="data-search-button" type="submit">"View rankings"</button>
            </form>
            <p class="data-help">"Division examples: Open Men's 60kg, Junior Women's 48kg, Men's Masters (40-44) 110+kg."</p>

            {move || rankings.with(|response| match response {
                None => view! { <TableSkeleton columns=4 /> }.into_any(),
                Some(Err(error)) => view! {
                    <p class="data-status error">{format!("Could not load national rankings: {error}")}</p>
                }.into_any(),
                Some(Ok(_)) if request.get().is_none() => view! {
                    <p class="data-status">"Enter a division to view its national rankings."</p>
                }.into_any(),
                Some(Ok(rankings)) => {
                    let mut ranked = rankings.iter().collect::<Vec<_>>();
                    ranked.sort_by(|left, right| right.total.total_cmp(&left.total));
                    let rows = ranked.into_iter().enumerate().map(|(index, row)| view! {
                        <tr>
                            <td>{index + 1}</td>
                            <td>{row.name.clone()}</td>
                            <td>{row.total}</td>
                            <td>{row.date.clone().unwrap_or_else(|| "—".to_owned())}</td>
                        </tr>
                    }).collect_view();
                    let empty = rankings.is_empty().then(|| view! {
                        <EmptyTableRow columns=4 message="No rankings matched this federation and division." />
                    });

                    view! {
                        <div class="data-table-wrap">
                            <table class="data-table">
                                <thead><tr><th>"Rank"</th><th>"Athlete"</th><th>"Total"</th><th>"Date"</th></tr></thead>
                                <tbody>{empty}{rows}</tbody>
                            </table>
                        </div>
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
    fn all_time_query_omits_the_year() {
        let query = serde_urlencoded::to_string(NationalRankingQuery {
            federation: "USAW".to_owned(),
            age_category: "Open Men's 60kg".to_owned(),
            year: None,
        })
        .unwrap();

        assert_eq!(query, "federation=USAW&age_category=Open+Men%27s+60kg");
    }

    #[test]
    fn seasonal_query_includes_the_year() {
        let query = serde_urlencoded::to_string(NationalRankingQuery {
            federation: "USAMW".to_owned(),
            age_category: "Women's Masters (40-44) 69kg".to_owned(),
            year: Some("2026".to_owned()),
        })
        .unwrap();

        assert!(query.contains("federation=USAMW"));
        assert!(query.contains("year=2026"));
    }

    #[test]
    fn ranking_accepts_a_missing_date() {
        let ranking: NationalRanking =
            serde_json::from_str(r#"{"name":"Test Athlete","total":245.0,"date":null}"#).unwrap();

        assert_eq!(ranking.name, "Test Athlete");
        assert_eq!(ranking.date, None);
    }
}
