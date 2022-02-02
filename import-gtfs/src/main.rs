use gtfs_structures::{Calendar, CalendarDate, RawStopTime, RawTrip, Route, Shape, Stop};
use postgres::{Client, Transaction};
use rgb::RGB8;
use serde_plain::to_string;
use std::path::PathBuf;
use string_interner::{backend::StringBackend, symbol::SymbolU32, StringInterner, Symbol};
use structopt::StructOpt;
use uuid::Uuid;

type Interner = StringInterner<StringBackend<SymbolU32>>;

pub fn optional_color_to_string(color: &Option<RGB8>) -> Option<String> {
    match color {
        None => None,
        Some(RGB8 { r, g, b }) => Some(format!("{:02X}{:02X}{:02X}", r, g, b))
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "Import a zipped GTFS feed into postgresql")]
struct Opt {
    #[structopt(parse(from_os_str))]
    input_gtfs_data: PathBuf,

    output_database: String,

    #[structopt(short = "f", long = "feed-title")]
    feed_title: String
}

fn insert_routes(tx: &mut Transaction, interner: &mut Interner, routes: &Vec<Route>) {
    println!("Import {} routes...", &routes.len());

    let stmt = tx.prepare(
      "INSERT INTO routes (route_id, route_short_name, route_long_name, route_desc, route_type, route_text_color, route_color, route_sort_order)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8)").unwrap();

    for r in routes {
        tx.execute(
            &stmt,
            &[
                &(interner.get_or_intern(&r.id).to_usize() as u32),
                &r.short_name,
                &r.long_name,
                &r.desc,
                &to_string(&r.route_type)
                    .ok()
                    .and_then(|x| x.parse::<i32>().ok()),
                &optional_color_to_string(&r.route_text_color),
                &optional_color_to_string(&r.route_color),
                &r.route_order.map(|x| x as i32)
            ]
        )
        .unwrap();
    }
}

fn insert_trips(tx: &mut Transaction, interner: &mut Interner, trips: &Vec<RawTrip>) {
    println!("Import {} trips...", &trips.len());

    let stmt = tx.prepare(
      "INSERT INTO trips (trip_id, route_id, service_id, shape_id, trip_headsign, direction_id) VALUES ($1, $2, $3, $4, $5, $6)").unwrap();

    for trip in trips {
        tx.execute(
            &stmt,
            &[
                &(interner.get_or_intern(&trip.id).to_usize() as u32),
                &(interner.get_or_intern(&trip.route_id).to_usize() as u32),
                &(interner.get_or_intern(&trip.service_id).to_usize() as u32),
                &trip
                    .shape_id
                    .as_ref()
                    .map(|shape_id| interner.get_or_intern(&shape_id).to_usize() as u32),
                &trip.trip_headsign,
                &to_string(&trip.direction_id)
                    .ok()
                    .and_then(|x| x.parse::<i32>().ok())
            ]
        )
        .unwrap();
    }
}

fn insert_stops(tx: &mut Transaction, interner: &mut Interner, stops: &Vec<Stop>) {
    println!("Import {} stops...", &stops.len());

    let stmt = tx.prepare(
      "INSERT INTO stops (stop_id, stop_name, stop_lat, stop_lon, parent_station, platform_code) VALUES ($1, $2, $3, $4, $5, $6)").unwrap();

    for stop in stops {
        tx.execute(
            &stmt,
            &[
                &(interner.get_or_intern(&stop.id).to_usize() as u32),
                &stop.name,
                &stop.latitude,
                &stop.longitude,
                &stop
                    .parent_station
                    .as_ref()
                    .map(|parent_id| interner.get_or_intern(&parent_id).to_usize() as u32),
                &stop.platform_code
            ]
        )
        .unwrap();
    }
}

fn insert_stop_times(tx: &mut Transaction, interner: &mut Interner, stop_times: &Vec<RawStopTime>) {
    println!("Import {} stop times...", &stop_times.len());

    let stmt = tx.prepare(
      "INSERT INTO stop_times (trip_id, arrival_time, departure_time, stop_id, stop_sequence, shape_dist_traveled) VALUES ($1, $2, $3, $4, $5, $6)").unwrap();

    for st in stop_times {
        tx.execute(
            &stmt,
            &[
                &(interner.get_or_intern(&st.trip_id).to_usize() as u32),
                &st.arrival_time.map(|x| x as i32),
                &st.departure_time.map(|x| x as i32),
                &(interner.get_or_intern(&st.stop_id).to_usize() as u32),
                &(st.stop_sequence as i32),
                &st.shape_dist_traveled
            ]
        )
        .unwrap();
    }
}

fn insert_calendar(tx: &mut Transaction, interner: &mut Interner, calendar: &Vec<Calendar>) {
    println!("Import {} calendars...", &calendar.len());

    let stmt = tx.prepare(
      "INSERT INTO calendar (service_id, monday, tuesday, wednesday, thursday, friday, saturday, sunday, start_date, end_date)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)").unwrap();

    for c in calendar {
        tx.execute(
            &stmt,
            &[
                &(interner.get_or_intern(&c.id).to_usize() as u32),
                &c.monday,
                &c.tuesday,
                &c.wednesday,
                &c.thursday,
                &c.friday,
                &c.saturday,
                &c.sunday,
                &c.start_date,
                &c.end_date
            ]
        )
        .unwrap();
    }
}

fn insert_calendar_dates(
    tx: &mut Transaction,
    interner: &mut Interner,
    calendar_dates: &Vec<CalendarDate>
) {
    println!("Import {} calendar dates...", &calendar_dates.len());

    let stmt = tx
        .prepare(
            "INSERT INTO calendar_dates (service_id, date, exception_type) VALUES ($1, $2, $3)"
        )
        .unwrap();

    for cd in calendar_dates {
        tx.execute(
            &stmt,
            &[
                &(interner.get_or_intern(&cd.service_id).to_usize() as u32),
                &cd.date,
                &to_string(&cd.exception_type)
                    .ok()
                    .and_then(|x| x.parse::<i32>().ok())
            ]
        )
        .unwrap();
    }
}

fn insert_shapes(tx: &mut Transaction, interner: &mut Interner, shape_points: &Vec<Shape>) {
    println!("Import {} shape points...", &shape_points.len());
    
    let stmt = tx.prepare(
      "INSERT INTO shapes (shape_id, shape_pt_lat, shape_pt_lon, shape_pt_sequence, shape_dist_traveled) VALUES ($1, $2, $3, $4, $5)").unwrap();
    for sp in shape_points {
        tx.execute(
            &stmt,
            &[
                &(interner.get_or_intern(&sp.id).to_usize() as u32),
                &sp.latitude,
                &sp.longitude,
                &(sp.sequence as i32),
                &sp.dist_traveled
            ]
        )
        .unwrap();
    }
}

fn insert_feed(
    tx: &mut Transaction,
    feed_uid: &str,
    feed_title: &str,
    publisher_name: &str,
    publisher_url: &str
) {
    let stmt = tx.prepare(
      "INSERT INTO public.feeds (feed_uid, feed_title, feed_publisher_name, feed_publisher_url) VALUES ($1, $2, $3, $4)").unwrap();
    tx.execute(
        &stmt,
        &[&feed_uid, &feed_title, &publisher_name, &publisher_url]
    )
    .unwrap();
}

fn main() {
    let opt = Opt::from_args();

    let mut interner = Interner::default();
    let mut conn = Client::connect(&opt.output_database, postgres::NoTls)
        .expect("Cannot connect to postgresql");

    println!("Reading GTFS zip file...");

    let gtfs = gtfs_structures::GtfsReader::default()
        .raw()
        .read(opt.input_gtfs_data.to_str().unwrap())
        .expect("Error reading GTFS data");

    let feed_uid = "gtfs_".to_string() + &Uuid::new_v4().to_simple().to_string();
    println!("Feed-UID is {}", feed_uid);

    println!("Importing data to postgresql...");

    let mut tx = conn.transaction().unwrap();

    if !feed_uid.is_empty() {
        tx.execute(&format!("CREATE SCHEMA \"{}\"", feed_uid), &[])
            .expect("Error creating schema");
        tx.execute(&format!("SET search_path=\"{}\"", feed_uid), &[])
            .expect("Error setting default schema path");
    }

    let create_tables_query = include_str!("sql/tables.sql");
    tx.batch_execute(create_tables_query)
        .expect("Error creating SQL tables");

    if let Some(fitemp) = gtfs.feed_info {
        let fitemp2 = fitemp.expect("Error reading GTFS feed info");
        let fi = fitemp2.first().expect("Empty feed_info.txt");
        insert_feed(&mut tx, &feed_uid, &opt.feed_title, &fi.name, &fi.url);
    } else {
        insert_feed(&mut tx, &feed_uid, &opt.feed_title, "", "");
    }

    insert_routes(
        &mut tx,
        &mut interner,
        &gtfs.routes.expect("Error reading routes from GTFS data")
    );

    insert_trips(
        &mut tx,
        &mut interner,
        &gtfs.trips.expect("Error reading trips from GTFS data")
    );

    insert_stops(
        &mut tx,
        &mut interner,
        &gtfs.stops.expect("Error reading stops from GTFS data")
    );

    insert_stop_times(
        &mut tx,
        &mut interner,
        &gtfs
            .stop_times
            .expect("Error reading stop_times from GTFS data")
    );

    if let Some(c) = &gtfs.calendar {
        insert_calendar(
            &mut tx,
            &mut interner,
            &c.as_ref().expect("Error reading GTFS calendar")
        );
    }

    if let Some(cd) = &gtfs.calendar_dates {
        insert_calendar_dates(
            &mut tx,
            &mut interner,
            &cd.as_ref().expect("Error reading GTFS calendar exceptions")
        );
    }

    if let Some(s) = &gtfs.shapes {
        insert_shapes(
            &mut tx,
            &mut interner,
            &s.as_ref().expect("Error reading GTFS shape data")
        );
    }

    println!("Create database indices...");
    let create_indices_query = include_str!("sql/indices.sql");
    tx.batch_execute(create_indices_query)
        .expect("Error creating SQL indices");

    println!("Create materialized views...");
    let create_trip_information_query = include_str!("sql/views.sql");
    tx.batch_execute(create_trip_information_query)
        .expect("Error computing trip information");

    tx.commit().unwrap();

    println!("Complete!");
}
