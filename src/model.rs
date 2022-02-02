use chrono::{DateTime, Local};
use postgres::Row;
use serde::Serialize;

pub struct Stop {
    pub stop_id: u32,
    pub stop_name: String,
    pub stop_lon: f64,
    pub stop_lat: f64,
    pub platform_code: Option<String>
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Route {
    pub route_id: u32,
    pub route_short_name: String,
    pub route_type: i32
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct StopTime {
    pub arrival_time: DateTime<Local>,
    pub departure_time: DateTime<Local>
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct StopTimesExtra {
    pub stop_time: StopTime,
    pub trip: Trip,
    pub route: Route
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Trip {
    pub trip_id: u32,
    pub direction_id: i32,
    pub first_stop_name: String,
    pub last_stop_name: String
}


#[derive(Debug)]
pub struct ShapePoint {
    pub latitude: f64,
    pub longitude: f64
}

#[derive(Serialize)]
pub struct FeedInfo {
    pub feed_uid : String,
    pub feed_title : String,
    pub feed_publisher_name : String,
    pub feed_publisher_url : String
}

pub fn trip_from_row(row: &Row) -> Result<Trip, postgres::error::Error> {
    Ok(Trip {
        trip_id: row.try_get("trip_id")?,
        direction_id: row.try_get("direction_id")?,
        first_stop_name: row.try_get("first_stop_name")?,
        last_stop_name: row.try_get("last_stop_name")?
    })
}

pub fn route_from_row(row: &Row) -> Result<Route, postgres::error::Error> {
    Ok(Route {
        route_id: row.try_get("route_id")?,
        route_short_name: row.try_get("route_short_name")?,
        route_type: row.try_get("route_type")?
    })
}

pub fn stop_time_from_row(row: &Row) -> Result<StopTime, postgres::error::Error> {
    Ok(StopTime {
        arrival_time: row.try_get("arrival_time")?,
        departure_time: row.try_get("departure_time")?
    })
}

pub fn stop_from_row(row: &Row) -> Result<Stop, postgres::error::Error> {
    Ok(Stop {
        stop_id: row.try_get("stop_id")?,
        stop_name: row.try_get("stop_name")?,
        stop_lon: row.try_get("stop_lon")?,
        stop_lat: row.try_get("stop_lat")?,
        platform_code: row.try_get("platform_code")?
    })
}

pub fn feed_from_row(row: &Row) -> Result<FeedInfo, postgres::error::Error> {
    Ok(FeedInfo {
        feed_uid: row.try_get("feed_uid")?,
        feed_title: row.try_get("feed_title")?,
        feed_publisher_name: row.try_get("feed_publisher_name")?,
        feed_publisher_url: row.try_get("feed_publisher_url")?
    })
}