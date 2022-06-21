#[macro_use]
extern crate rocket;
use chrono::NaiveDate;
use rocket::fs::FileServer;
use rocket::http::Header;
use rocket::http::RawStr;
use rocket::response::{content, Redirect, Responder};
use rocket::serde::json::Json;
use rocket_async_compression::Compression;
use rocket_sync_db_pools::database;
use serde_json::value::RawValue;
use serde_json::{json, Value};
use std::time::Instant;

mod model;
use model::*;
mod html;
use html::{stop_html, trip_html};

#[derive(Responder)]
struct CachedResponder<T> {
    inner: T,
    cached_header: Header<'static>
}
impl<'r, 'o: 'r, T: Responder<'r, 'o>> CachedResponder<T> {
    fn new(inner: T) -> Self {
        CachedResponder {
            inner,
            cached_header: Header::new("Cache-Control", "public, max-age=2592000")
        }
    }
}

#[database("gtfs_data")]
struct Database(postgres::Client);

fn alphanumeric_string(input: &str) -> Result<String, &str> {
    if input
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        Ok(input.to_string())
    } else {
        Err("Feed name must be alphanumeric")
    }
}

fn set_db_schema(conn: &mut postgres::Client, feed: &String) {
    let feed = alphanumeric_string(&feed).unwrap();

    let stmt = conn
        .prepare("SELECT COUNT(*)::int AS cnt FROM feeds WHERE feed_uid = $1")
        .unwrap();

    let row = conn.query_one(&stmt, &[&feed]).unwrap();
    let cnt: i32 = row.get("cnt");
    if cnt != 1 {
        panic!("Feed uid not found in database");
    }

    conn.execute(&format!("SET search_path=\"{}\", public", feed), &[])
        .expect("Error setting default schema path");
}

#[get("/<feed>/stops")]
async fn stops(db: Database, feed: String) -> CachedResponder<Json<Box<RawValue>>> {
    db.run(move |conn| {
        set_db_schema(conn, &feed);

        let timer = Instant::now();

        let stmt = conn.prepare(include_str!("sql/stops_geojson.sql")).unwrap();

        let row = conn.query_one(&stmt, &[]).unwrap();
        let stops_geojson: Box<RawValue> = RawValue::from_string(row.get("geojson")).unwrap();

        println!("Time elapsed in stops is: {:?}", timer.elapsed());

        CachedResponder::new(Json(stops_geojson))
    })
    .await
}

#[get("/<feed>/<date>/stops/<stop_id>")]
async fn stop_information(
    db: Database,
    feed: String,
    date: String,
    stop_id: u32
) -> content::RawHtml<String> {
    db.run(move |conn| {
        set_db_schema(conn, &feed);

        let stmt_stop = conn.prepare(include_str!("sql/stop.sql")).unwrap();
        let stop = conn
            .query_one(&stmt_stop, &[&stop_id])
            .and_then(|row| Ok(stop_from_row(&row)?))
            .unwrap();

        let start = Instant::now();
        let ddate = NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
        let stmt = conn.prepare(include_str!("sql/departures.sql")).unwrap();
        let stop_times = conn
            .query(&stmt, &[&ddate, &"Europe/Berlin", &stop_id])
            .unwrap()
            .iter()
            .map(|row| {
                Ok(StopTimesExtra {
                    stop_time: stop_time_from_row(row)?,
                    trip: trip_from_row(row)?,
                    route: route_from_row(row)?
                })
            })
            .collect::<Result<Vec<StopTimesExtra>, postgres::Error>>()
            .unwrap();

        let duration = start.elapsed();

        println!("Time elapsed in stop_information() is: {:?}", duration);

        content::RawHtml(stop_html(&stop, &stop_times).into_string())
    })
    .await
}

#[get("/<feed>/<date>/trips/<trip_id>")]
async fn trip_information(db: Database, feed: String, date: String, trip_id: u32) -> Json<Value> {
    db.run(move |conn| {
        set_db_schema(conn, &feed);
        let stmt_trip = conn.prepare(include_str!("sql/trip.sql")).unwrap();
        let stmt_trip_stop_times = conn
            .prepare(include_str!("sql/trip_stop_times.sql"))
            .unwrap();

        let (trip, route) = conn
            .query_one(&stmt_trip, &[&trip_id])
            .and_then(|row| Ok((trip_from_row(&row)?, route_from_row(&row)?)))
            .unwrap();
        let mut features = vec![];

        let ddate = NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
        let trip_stops = conn
            .query(&stmt_trip_stop_times, &[&ddate, &"Europe/Berlin", &trip_id])
            .unwrap()
            .iter()
            .map(|row| {
                Ok((
                    stop_time_from_row(row)?,
                    stop_from_row(row)?,
                    row.try_get("root_id")?
                ))
            })
            .collect::<Result<Vec<(StopTime, Stop, u32)>, postgres::Error>>()
            .unwrap();

        // Query trip shape
        let stmt_trip_shape = conn.prepare(include_str!("sql/trip_shape.sql")).unwrap();

        let row = conn.query_one(&stmt_trip_shape, &[&trip_id]).unwrap();
        let mut trip_shape_geojson: Value = row.get("geojson");
        trip_shape_geojson.as_object_mut().unwrap().insert(
            "properties".to_string(),
            json!({
                "type": "Trip",
                "route_short_name": route.route_short_name,
                "trip_info": trip_html(&trip, &route, &trip_stops).into_string()})
        );

        features.push(trip_shape_geojson);
        features.extend(trip_stops.iter().map(|(_, stop, _)| {
            json!({
                "type": "Feature",
                "geometry": {
                    "type": "Point",
                    "coordinates": [stop.stop_lon, stop.stop_lat]
                },
                "properties": {
                    "stop_name": stop.stop_name,
                    "stop_id": stop.stop_id,
                    "platform_code": stop.platform_code
                },
            })
        }));

        let feature_collection = json!({
        "type": "FeatureCollection",
        "features": features});
        Json(feature_collection)
    })
    .await
}

#[get("/<feed>/frequency/<date>/<z>/<x>/<y>/tile.pbf")]
async fn segment_frequencies(
    db: Database,
    feed: String,
    date: String,
    z: i32,
    x: i32,
    y: i32
) -> CachedResponder<Vec<u8>> {
    db.run(move |conn| {
        set_db_schema(conn, &feed);

        let ddate = NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
        let opt = conn
            .query_opt(
                include_str!("sql/frequency_tile_query_cache.sql"),
                &[&ddate, &z, &x, &y]
            )
            .unwrap();

        match opt {
            Some(row) => {
                println!(
                    "Cache: hit: date = {}, z = {}, x = {}, y = {})",
                    date, z, x, y
                );

                conn.execute(
                    include_str!("sql/frequency_tile_hit_cache.sql"),
                    &[&ddate, &z, &x, &y]
                )
                .unwrap();
                conn.execute(
                    include_str!("sql/frequency_tile_cleanup_cache.sql"),
                    &[&10000]
                )
                .unwrap();

                let freqmvt: Vec<u8> = row.get("tile");
                CachedResponder::new(freqmvt)
            }
            None => {
                let stmt = conn
                    .prepare(include_str!("sql/frequency_tile.sql"))
                    .unwrap();

                let start = Instant::now();
                let row = conn.query_one(&stmt, &[&ddate, &z, &x, &y]).unwrap();
                let freqmvt: Vec<u8> = row.get("freqmvt");

                if z <= 15 && !freqmvt.is_empty() {
                    conn.execute(
                        include_str!("sql/frequency_tile_insert_into_cache.sql"),
                        &[&ddate, &z, &x, &y, &freqmvt]
                    )
                    .unwrap();
                }

                let duration = start.elapsed();

                println!(
                    "Time elapsed in segment_frequencies(z = {}, x = {}, y = {}) is: {:?}",
                    z, x, y, duration
                );
                CachedResponder::new(freqmvt)
            }
        }
    })
    .await
}

#[get("/feeds")]
async fn feeds(db: Database) -> Json<Vec<FeedInfo>> {
    db.run(move |conn| {
        let stmt = conn
            .prepare("SELECT * FROM feeds ORDER BY feed_title;")
            .unwrap();

        let feeds = conn
            .query(&stmt, &[])
            .unwrap()
            .iter()
            .map(|x| feed_from_row(x))
            .collect::<Result<Vec<FeedInfo>, _>>()
            .unwrap();

        Json(feeds)
    })
    .await
}

#[get("/")]
async fn index(db: Database) -> Redirect {
    db.run(move |conn| {
        let stmt = conn
            .prepare("SELECT feed_uid FROM feeds ORDER BY feed_title LIMIT 1;")
            .unwrap();

        let row = conn.query_one(&stmt, &[]).unwrap();
        let feed_uid: String = row.get("feed_uid");

        Redirect::to(format!(
            "/public/transit-viewer.html?feed={}",
            RawStr::new(&feed_uid).percent_encode().as_str()
        ))
    })
    .await
}

#[launch]
fn rocket() -> _ {
    let server = rocket::build()
        .attach(Database::fairing())
        .mount(
            "/",
            routes![
                stops,
                stop_information,
                trip_information,
                segment_frequencies,
                feeds,
                index
            ]
        )
        .mount("/public", FileServer::from("static"));
    server.attach(Compression::fairing())
}
