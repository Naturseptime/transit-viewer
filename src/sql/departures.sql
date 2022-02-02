WITH RECURSIVE
input(service_date, timezone) AS (
    VALUES ($1::DATE, $2::TEXT)
),
helper(start_of_day) AS (
    -- Time is measured from noon minus 12 hours (relevant for DST change)
    SELECT ((service_date + INTERVAL '12 hours') AT TIME ZONE input.timezone - INTERVAL '12 hours') FROM input
)
SELECT
    start_of_day + arrival_time * INTERVAL '1 second' AS arrival_time,
    start_of_day + departure_time * INTERVAL '1 second' AS departure_time,
    trip_id, trip_headsign, direction_id,
    route_id, route_short_name, route_type, first_stop_name, last_stop_name
FROM 
    stop_hierarchy NATURAL JOIN stop_times NATURAL JOIN trips NATURAL JOIN routes NATURAL JOIN trip_terminals, input, helper
WHERE stop_hierarchy.root_id = $3 AND 
    (EXISTS
       (SELECT *
        FROM calendar
        WHERE calendar.service_id = trips.service_id
          AND (service_date >= calendar.start_date
               AND service_date < calendar.end_date
               AND CASE EXTRACT(DOW FROM service_date)
                       WHEN '0' THEN sunday
                       WHEN '1' THEN monday
                       WHEN '2' THEN tuesday
                       WHEN '3' THEN wednesday
                       WHEN '4' THEN thursday
                       WHEN '5' THEN friday
                       WHEN '6' THEN saturday
                   END))
     OR EXISTS
       (SELECT *
        FROM calendar_dates
        WHERE calendar_dates.service_id = trips.service_id
          AND calendar_dates.date = service_date
          AND calendar_dates.exception_type = 1))
    AND NOT EXISTS
      (SELECT *
       FROM calendar_dates
       WHERE calendar_dates.service_id = trips.service_id
         AND calendar_dates.date = service_date
         AND calendar_dates.exception_type = 2)
    ORDER BY departure_time;