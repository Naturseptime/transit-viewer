-- Precompute start and end stations for a trip
CREATE MATERIALIZED VIEW trip_terminals AS
SELECT trip_id,
    (SELECT st.stop_id FROM stop_times AS st WHERE st.trip_id = trips.trip_id ORDER BY stop_sequence ASC LIMIT 1) AS first_stop_id,
    (SELECT s.stop_name FROM stop_times AS st NATURAL JOIN stops AS s WHERE st.trip_id = trips.trip_id ORDER BY stop_sequence ASC LIMIT 1) AS first_stop_name,
    (SELECT st.departure_time FROM stop_times AS st WHERE st.trip_id = trips.trip_id ORDER BY stop_sequence ASC LIMIT 1) AS first_departure,
    (SELECT st.stop_id FROM stop_times AS st WHERE st.trip_id = trips.trip_id ORDER BY stop_sequence DESC LIMIT 1) AS last_stop_id,
    (SELECT s.stop_name FROM stop_times AS st NATURAL JOIN stops AS s WHERE st.trip_id = trips.trip_id ORDER BY stop_sequence DESC LIMIT 1) AS last_stop_name,
    (SELECT st.arrival_time FROM stop_times AS st WHERE st.trip_id = trips.trip_id ORDER BY stop_sequence DESC LIMIT 1) AS last_arrival,
    (SELECT COUNT(*) FROM stop_times AS st WHERE st.trip_id = trips.trip_id) AS number_of_stations,
    (SELECT MAX(st.shape_dist_traveled) FROM stop_times AS st WHERE st.trip_id = trips.trip_id) AS dist_traveled
FROM trips;

CREATE INDEX trip_terminals_trip_id_index
  ON trip_terminals (trip_id);

-- Fast queries for root station <-> substations (i.e. platforms)
CREATE MATERIALIZED VIEW stop_hierarchy AS
WITH RECURSIVE stop_hierarchy(root_id, stop_id) AS
(SELECT stop_id, stop_id FROM stops WHERE parent_station IS NULL
    UNION
SELECT stop_hierarchy.root_id, stops.stop_id FROM stop_hierarchy, stops WHERE stops.parent_station = stop_hierarchy.stop_id)
SELECT * FROM stop_hierarchy;

CREATE INDEX stop_hierarchy_root_id_index ON stop_hierarchy (root_id);
CREATE INDEX stop_hierarchy_stop_id_index ON stop_hierarchy (stop_id);

-- Precompute trip segments between two stations astop -> bstop
-- If no shapes are provided in the GTFS feed, we use a direct straight segment
-- between the stops as replacement
CREATE MATERIALIZED VIEW segment_paths AS (
WITH
consecutive_stops(astop, bstop, adist, bdist, shape_id) AS  (
    SELECT stop_id, 
        LEAD(stop_id) OVER (PARTITION BY trip_id ORDER BY stop_sequence),
        shape_dist_traveled, 
        LEAD(shape_dist_traveled) OVER (PARTITION BY trip_id ORDER BY stop_sequence),
        shape_id
    FROM stop_times NATURAL JOIN trips
),
most_frequent_shape(astop, bstop, adist, bdist, shape_id) AS (
    SELECT DISTINCT ON (astop, bstop) astop, bstop, adist, bdist, shape_id
    FROM consecutive_stops
    WHERE bstop IS NOT NULL
    ORDER BY astop, bstop, shape_id),
paths AS (
    SELECT astop, bstop,
        public.ST_Transform(
            public.ST_MakeLine(
                public.ST_SetSRID(public.ST_MakePoint(shape_pt_lon, shape_pt_lat), 4326) 
                ORDER BY shape_pt_sequence ASC),
            3857) AS path
    FROM most_frequent_shape NATURAL JOIN shapes
    WHERE shape_dist_traveled BETWEEN adist AND bdist
    GROUP BY astop, bstop),
paths_replacement AS (
    SELECT astop, bstop,
        public.ST_Transform(
            public.ST_MakeLine(
                public.ST_SetSRID(public.ST_MakePoint(a.stop_lon, a.stop_lat), 4326),
                public.ST_SetSRID(public.ST_MakePoint(b.stop_lon, b.stop_lat), 4326)),
            3857)
    FROM consecutive_stops, stops AS a, stops AS b
    WHERE a.stop_id = astop AND b.stop_id = bstop
),
paths_or_replacement AS (
    SELECT DISTINCT ON (astop, bstop) astop, bstop, path FROM
    (SELECT * FROM paths UNION SELECT * FROM paths_replacement) AS t
    ORDER BY astop, bstop, public.ST_NPoints(path) DESC
)
SELECT astop, bstop, path,
    LEAST(GREATEST(FLOOR(LOG(0.5, public.ST_Length(path)::NUMERIC / 200000.0 + 1e-9))::INTEGER, 0), 10) AS zoom_threshold FROM paths_or_replacement
);

CREATE INDEX segment_paths_index ON segment_paths (astop, bstop);
CREATE INDEX segment_paths_spatial_index ON segment_paths USING GIST (path);
CREATE INDEX segment_paths_zoom_index ON segment_paths (zoom_threshold, astop, bstop);

-- Helper table for quickly computing the number of trips for this segment on a day
-- We save a trip_count for a triple (astop, bstop, service_id).
CREATE MATERIALIZED VIEW segment_frequencies AS (
WITH segment_frequencies_helper(astop, bstop, trip_id, service_id) AS (
    SELECT stop_id, LEAD(stop_id) OVER (PARTITION BY trip_id ORDER BY stop_sequence),
        trip_id, service_id
    FROM stop_times NATURAL JOIN trips)
SELECT astop, bstop, COUNT(trip_id) AS trip_count, service_id
FROM segment_frequencies_helper WHERE bstop IS NOT NULL
GROUP BY astop, bstop, service_id
);

CREATE INDEX segment_frequencies_stops_index ON segment_frequencies (astop, bstop, service_id);
CREATE INDEX segment_frequencies_service_id_index ON segment_frequencies (service_id, astop, bstop);

--We do not need these indices anymore (and they require a lot of memory)
DROP INDEX stop_times_trip_id_stop_sequence_index;
DROP INDEX stop_times_trip_id_shape_dist_traveled_index;
