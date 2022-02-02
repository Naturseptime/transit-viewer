CREATE INDEX routes_route_id_index ON routes (route_id);

CREATE INDEX trips_trip_id_index ON trips (trip_id);
CREATE INDEX trips_route_id_index ON trips (route_id);
CREATE INDEX trips_service_id_index ON trips (service_id);
CREATE INDEX trips_route_id_service_id_index ON trips (route_id, service_id);

CREATE INDEX stops_stop_id_index ON stops (stop_id);
CREATE INDEX stops_parent_station_index ON stops (parent_station);

CREATE INDEX stop_times_stop_id_index ON stop_times (stop_id);
CREATE INDEX stop_times_trip_id_index ON stop_times (trip_id);
CREATE INDEX stop_times_trip_id_stop_sequence_index ON stop_times (trip_id, stop_sequence);
CREATE INDEX stop_times_trip_id_shape_dist_traveled_index ON stop_times (trip_id, shape_dist_traveled);

CREATE INDEX calendar_service_id_index ON calendar (service_id);
CREATE INDEX calendar_monday_index ON calendar (monday, service_id);
CREATE INDEX calendar_tuesday_index ON calendar (tuesday, service_id);
CREATE INDEX calendar_wednesday_index ON calendar (wednesday, service_id);
CREATE INDEX calendar_thursday_index ON calendar (thursday, service_id);
CREATE INDEX calendar_friday_index ON calendar (friday, service_id);
CREATE INDEX calendar_saturday_index ON calendar (saturday, service_id);
CREATE INDEX calendar_sunday_index ON calendar (sunday, service_id);
CREATE INDEX calendar_start_date_index ON calendar (start_date, service_id);
CREATE INDEX calendar_end_date_index ON calendar (end_date, service_id);

CREATE INDEX calendar_dates_index ON calendar_dates (service_id, date, exception_type);
CREATE INDEX calendar_dates_date_index ON calendar_dates (date);

CREATE INDEX shapes_shape_id_index ON shapes (shape_id);
CREATE INDEX shapes_shape_dist_traveled_index ON shapes (shape_id, shape_dist_traveled);
CREATE INDEX shapes_shape_id_shape_pt_sequence_index ON shapes (shape_id, shape_pt_sequence);

CREATE INDEX cache_tile_index ON frequency_tile_cache (date, z, x, y);
CREATE INDEX cache_hit_index ON frequency_tile_cache (last_hit);