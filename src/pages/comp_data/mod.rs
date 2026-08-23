use leptos::prelude::*;
use std::cmp::Ordering;

pub mod adaptive_records;
pub(crate) mod analytics;
pub(crate) mod athlete_autocomplete;
pub mod club_dashboard;
pub mod data_home;
pub mod meet_center;
pub(crate) mod models;
pub mod national_rankings;
pub mod qual_totals;
pub mod rankings;
pub mod records;
pub mod results;
pub mod standards;
pub mod wrapped;
pub mod wso_dashboard;
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

pub(crate) fn yes_no(value: bool) -> &'static str {
    if value { "Yes" } else { "No" }
}

pub(crate) fn format_us_date(value: &str) -> String {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let parts = value.split('-').collect::<Vec<_>>();
    let parsed = parts
        .as_slice()
        .try_into()
        .ok()
        .and_then(|[year, month, day]: [&str; 3]| {
            Some((
                year.parse::<u32>().ok()?,
                month.parse::<usize>().ok()?,
                day.parse::<u32>().ok()?,
            ))
        });
    match parsed {
        Some((year, month @ 1..=12, day @ 1..=31)) => {
            format!("{} {day}, {year}", MONTHS[month - 1])
        }
        _ => value.to_owned(),
    }
}

pub(crate) fn format_us_time(value: &str) -> String {
    let parts = value.split(':').collect::<Vec<_>>();
    let Some((hour, minute)) = parts
        .first()
        .and_then(|hour| hour.parse::<u32>().ok())
        .zip(parts.get(1).and_then(|minute| minute.parse::<u32>().ok()))
    else {
        return value.to_owned();
    };
    if hour > 23 || minute > 59 {
        return value.to_owned();
    }
    let suffix = if hour < 12 { "AM" } else { "PM" };
    let display_hour = match hour % 12 {
        0 => 12,
        hour => hour,
    };
    format!("{display_hour}:{minute:02} {suffix}")
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

    #[test]
    fn dates_and_times_use_us_display_formats() {
        assert_eq!(format_us_date("2026-06-20"), "June 20, 2026");
        assert_eq!(format_us_time("08:00:00"), "8:00 AM");
        assert_eq!(format_us_time("12:30"), "12:30 PM");
    }
}
