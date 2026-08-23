use leptos::prelude::*;
use std::cmp::Ordering;

pub mod adaptive_records;
pub mod data_home;
pub mod national_rankings;
pub mod qual_totals;
pub mod rankings;
pub mod records;
pub mod results;
pub mod standards;
pub mod wso_records;

pub(crate) fn filter_options<'a>(values: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut options = values
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    options.sort();
    options.dedup();
    options
}

pub(crate) fn weight_class_options<'a>(values: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut options = values
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    options.sort_by(|left, right| compare_weight_classes(left, right));
    options.dedup();
    options
}

pub(crate) fn compare_weight_classes(left: &str, right: &str) -> Ordering {
    let left_key = weight_class_key(left);
    let right_key = weight_class_key(right);

    match (left_key.0, right_key.0) {
        (Some(left_weight), Some(right_weight)) => left_weight
            .cmp(&right_weight)
            .then_with(|| left_key.1.cmp(&right_key.1))
            .then_with(|| left.cmp(right)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => left.cmp(right),
    }
}

fn weight_class_key(value: &str) -> (Option<u32>, bool) {
    let digits = value
        .chars()
        .skip_while(|character| !character.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect::<String>();

    (digits.parse().ok(), value.contains('+'))
}

pub(crate) fn matches_filter(value: &str, selected: &str) -> bool {
    selected.is_empty() || value == selected
}

#[component]
pub(crate) fn SelectOptions(values: Vec<String>, selected: Option<String>) -> impl IntoView {
    values
        .into_iter()
        .map(|value| {
            let is_selected = selected.as_ref().is_some_and(|selected| selected == &value);

            view! {
                <option value=value.clone() selected=is_selected>
                    {value.clone()}
                </option>
            }
        })
        .collect_view()
}

#[component]
pub(crate) fn TableSkeleton(columns: usize) -> impl IntoView {
    let header_cells = (0..columns)
        .map(|_| view! { <th><span class="data-skeleton"></span></th> })
        .collect_view();
    let rows = (0..8)
        .map(|_| {
            let cells = (0..columns)
                .map(|_| view! { <td><span class="data-skeleton"></span></td> })
                .collect_view();

            view! { <tr>{cells}</tr> }
        })
        .collect_view();

    view! {
        <div class="data-table-wrap" aria-busy="true" aria-label="Loading data">
            <table class="data-table data-table-skeleton">
                <thead><tr>{header_cells}</tr></thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
    }
}

#[component]
pub(crate) fn EmptyTableRow(columns: usize, message: &'static str) -> impl IntoView {
    view! {
        <tr class="data-empty-row">
            <td colspan=columns>{message}</td>
        </tr>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_options_are_trimmed_sorted_unique_and_nonempty() {
        let options = filter_options(["  Women ", "Men", "", "Men", "   ", "Youth"].into_iter());

        assert_eq!(options, ["Men", "Women", "Youth"]);
    }

    #[test]
    fn weight_classes_sort_by_weight_with_open_classes_last() {
        let options = weight_class_options(
            ["110+kg", "32kg", "110kg", "63+kg", "30kg", "63kg", "32kg"].into_iter(),
        );

        assert_eq!(
            options,
            ["30kg", "32kg", "63kg", "63+kg", "110kg", "110+kg"]
        );
    }

    #[test]
    fn empty_selection_matches_everything() {
        assert!(matches_filter("Senior", ""));
        assert!(matches_filter("Senior", "Senior"));
        assert!(!matches_filter("Junior", "Senior"));
    }
}
