use super::models::{AthleteSearchQuery, AthleteSearchResponse};
use crate::utils::api::get_api_response_with_query;
use leptos::prelude::*;

#[component]
pub(crate) fn AthleteAutocomplete(
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    input_id: &'static str,
    #[prop(default = "")] wrapper_class: &'static str,
) -> impl IntoView {
    let (suggestion_query, set_suggestion_query) = signal(String::new());
    let suggestions = LocalResource::new(move || {
        let query = suggestion_query.get();
        async move {
            if query.chars().count() < 3 {
                Ok(Vec::new())
            } else {
                get_api_response_with_query::<AthleteSearchResponse, _>(
                    "/search",
                    &AthleteSearchQuery::suggestions(query),
                )
                .await
                .map(|response| response.suggestions)
                .map_err(|error| error.to_string())
            }
        }
    });

    view! {
        <div class=format!("athlete-autocomplete {wrapper_class}")>
            <label for=input_id>"Athlete"</label>
            <input
                id=input_id
                class="data-filter"
                type="search"
                required=true
                autocomplete="off"
                aria-autocomplete="list"
                aria-controls=format!("{input_id}-suggestions")
                placeholder="Athlete name"
                prop:value=move || value.get()
                on:input=move |event| {
                    let next = event_target_value(&event);
                    set_value.set(next.clone());
                    set_suggestion_query.set(if next.trim().chars().count() >= 3 {
                        next.trim().to_owned()
                    } else {
                        String::new()
                    });
                }
            />
            {move || {
                let query = suggestion_query.get();
                if query.chars().count() < 3 {
                    return ().into_any();
                }
                suggestions.with(|response| match response {
                    Some(Ok(matches)) => view! {
                        <div id=format!("{input_id}-suggestions") class="athlete-suggestions" role="listbox" aria-label="Athlete suggestions">
                            {matches.is_empty().then(|| view! { <p>"No matching athletes"</p> })}
                            {matches.iter().cloned().map(|name| {
                                let selected_name = name.clone();
                                view! {
                                    <button type="button" role="option" on:click=move |_| {
                                        set_value.set(selected_name.clone());
                                        set_suggestion_query.set(String::new());
                                    }>{name}</button>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any(),
                    Some(Err(_)) => ().into_any(),
                    None => view! {
                        <div id=format!("{input_id}-suggestions") class="athlete-suggestions" role="status"><p>"Searching…"</p></div>
                    }.into_any(),
                })
            }}
        </div>
    }
}
