CREATE TABLE IF NOT EXISTS public.feeds (
    feed_uid TEXT PRIMARY KEY,
    feed_title TEXT NOT NULL,
    feed_publisher_name TEXT NOT NULL,
    feed_publisher_url TEXT NOT NULL
);

CREATE TABLE routes (
    route_id OID NOT NULL,
    route_short_name TEXT,
    route_long_name TEXT,
    route_desc TEXT,
    route_type INT,
    route_color TEXT,
    route_text_color TEXT,
    route_sort_order INT
);

CREATE TABLE trips (
    trip_id OID NOT NULL,
    route_id OID NOT NULL,
    service_id OID NOT NULL,
    shape_id OID,
    trip_headsign TEXT,
    direction_id INT
);

CREATE TABLE stops (
    stop_id OID NOT NULL,
    stop_name TEXT NOT NULL,
    stop_lat FLOAT NOT NULL,
    stop_lon FLOAT NOT NULL,
    location_type INT,
    parent_station OID,
    platform_code TEXT
);

CREATE TABLE stop_times (
    trip_id OID NOT NULL,
    arrival_time INT,
    departure_time INT,
    stop_id OID NOT NULL,
    stop_sequence INT,
    shape_dist_traveled REAL
);

CREATE TABLE calendar (
    service_id OID NOT NULL,
    monday BOOL NOT NULL,
    tuesday BOOL NOT NULL,
    wednesday BOOL NOT NULL,
    thursday BOOL NOT NULL,
    friday BOOL NOT NULL,
    saturday BOOL NOT NULL,
    sunday BOOL NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL
);

CREATE TABLE calendar_dates (
    service_id OID NOT NULL,
    date DATE NOT NULL,
    exception_type INT NOT NULL
);

CREATE TABLE shapes (
    shape_id OID NOT NULL,
    shape_pt_lat FLOAT NOT NULL,
    shape_pt_lon FLOAT NOT NULL,
    shape_pt_sequence INT NOT NULL,
    shape_dist_traveled REAL
);

CREATE TABLE frequency_tile_cache (
    date DATE NOT NULL,
    z INT NOT NULL,
    x INT NOT NULL,
    y INT NOT NULL,
    tile BYTEA NOT NULL,
    last_hit TIMESTAMP NOT NULL
);
