use chrono::{NaiveDate, Timelike};
use itertools::Itertools;
use maud::{html, Markup};
use std::collections::HashMap;

use crate::model::*;

fn route_type_css(route_type: i32) -> &'static str {
    match route_type {
        0 | 900..=906 => "tram",
        1 | 400..=404 => "subway",
        2 => "generic",
        3 | 700..=716 | 800 => "bus",
        101..=103 => "intercity",
        106 => "regional",
        109 => "suburban",
        _ => "generic"
    }
}

fn route_sorting_key(route: &Route) -> (i32, String) {
    (
        match route.route_type {
            0 | 900..=999 => 3,
            1 | 400..=499 => 1,
            2 | 100..=199 => 2,
            3 | 700..=899 => 4,
            _ => 5
        },
        format!("{:0>10}", route.route_short_name)
    )
}

fn format_route(route: &Route) -> Markup {
    html! {span class={"route" " " (route_type_css(route.route_type))} {(route.route_short_name)}}
}
/*
fn departures_list(stop_times: &Vec<StopTimesExtra>) -> Markup {
    html! {
        table class="departures_table" {
            tr {
                th {"Line"}
                th {"Destination"}
                th {"Depature"}
            }

            @for st in stop_times {
                tr class="link" {
                    td {div class="route generic" {(st.route_short_name)}}
                    td {(st.last_stop_name)}
                    td {(st.departure_time.time().format("%H:%M"))}
                }
            }
        }
    }
}
*/

fn departure_minutes(stop_times: &Vec<&StopTimesExtra>) -> Markup {
    let stop_times_by_hour: HashMap<(NaiveDate, u32), Vec<&StopTimesExtra>> =
        stop_times.into_iter().cloned().into_group_map_by(|st| {
            let d = st.stop_time.departure_time;
            (d.date().naive_local(), d.time().hour())
        });
    let mut ordered_by_hour: Vec<((NaiveDate, u32), Vec<&StopTimesExtra>)> =
        stop_times_by_hour.into_iter().collect();
    ordered_by_hour.sort_by_key(|(day_and_hour, _)| day_and_hour.clone());

    html! {
    ul class="departure_minutes" {
    @for ((_, hour), list) in ordered_by_hour {
        li {b class="trip_hour" {(hour)} {
            span {
                @for st in list {
                    " "
                    span class="trip_minute link" data-trip-id=(st.trip.trip_id) onclick="onStopDepartureClicked(this)" {
                        (format!("{:0>2?}", st.stop_time.departure_time.time().minute()))}}}}}}}}
}

fn departure_direction_tabs(stop_times: &Vec<&StopTimesExtra>, route_tab_id: usize) -> Markup {
    let stop_times_by_direction: HashMap<i32, Vec<&StopTimesExtra>> = stop_times
        .into_iter()
        .cloned()
        .into_group_map_by(|st| st.trip.direction_id);
    let mut ordered_routes: Vec<(i32, Vec<&StopTimesExtra>)> =
        stop_times_by_direction.into_iter().collect();
    ordered_routes.sort_by_key(|(direction, _)| *direction);

    html! {
        div class="tabs" {
            @for (tab, (_, route_stop_times)) in ordered_routes.into_iter().enumerate() {
                input
                    type="radio"
                    checked=[if tab == 0 {Some("checked")} else {None}]
                    name=(format!("direction-tabs-{}", route_tab_id))
                    id=(format!("direction-tabs-{}-{}", route_tab_id, tab));
                label
                    for=(format!("direction-tabs-{}-{}", route_tab_id, tab)) {
                    (route_stop_times.iter().map(|st| &st.trip.last_stop_name).counts().into_iter().max_by_key(|(_, v)| *v).unwrap().0)
                }
                div class="tab" style="padding: 0em" {
                    (departure_minutes(&route_stop_times))
                }
            }
        }
    }
}

fn departure_route_tabs(stop_times: &Vec<StopTimesExtra>) -> Markup {
    let stop_times_by_route_id: HashMap<&Route, Vec<&StopTimesExtra>> =
        stop_times.into_iter().into_group_map_by(|st| &st.route);
    let mut ordered_routes: Vec<(&Route, Vec<&StopTimesExtra>)> =
        stop_times_by_route_id.into_iter().collect();
    ordered_routes.sort_by_key(|(route, _)| route_sorting_key(route));

    html! {
        @if !ordered_routes.is_empty() {
            div class="tabs" {
                @for (tab, (route, route_stop_times)) in ordered_routes.into_iter().enumerate() {
                    input
                        checked=[if tab == 0 {Some("checked")} else {None}]
                        type="radio" name="route-tabs"
                        id=(format!("route-tabs-{}", tab));
                    label
                        for=(format!("route-tabs-{}", tab)) {(format_route(route))}
                    div class="tab" style="padding: 0em; border: none" {
                        (departure_direction_tabs(&route_stop_times, tab))}}}}
        @else {"There are no departures on this day"}
    }
}

pub fn stop_html(stop: &Stop, stop_times: &Vec<StopTimesExtra>) -> Markup {
    html! {
        article class="stop_info" {
            h1 {(stop.stop_name)}
            (departure_route_tabs(stop_times))
        }
    }
}

pub fn trip_html(trip: &Trip, route: &Route, trip_stops: &Vec<(StopTime, Stop, u32)>) -> Markup {
    html! {
        p {
            {(format_route(route))} " "
            (trip.first_stop_name) " ➜ " (trip.last_stop_name)
        }
        table style="font-size: 90%" {
            tr {
                th {"Station"}
                th {"Arrival"}
                th {"Departure"}
                th {"Platform"}
            }
            @for (st, stop, root_id) in trip_stops {
                tr {
                    td {
                        span style="color: #666" class="link" data-stop-id=(root_id)  onclick="onTripStopClicked(this)" {
                            (stop.stop_name)
                        }
                    }
                    td {(st.arrival_time.time().format("%H:%M"))}
                    td {(st.departure_time.time().format("%H:%M"))}
                    td {(stop.platform_code.as_ref().unwrap_or(&"-".to_string()))}
                }
            }
        }
        p {
            i {"Number of stations: " (trip_stops.len())}
        }
    }
}
