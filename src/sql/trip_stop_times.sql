WITH RECURSIVE
input(idate, timezone) AS (
    VALUES ($1::DATE, $2::TEXT)
),
helper(start_of_day) AS (
    -- Time is measured from noon minus 12 hours (relevant for DST change)
    SELECT ((input.idate + INTERVAL '12 hours') AT TIME ZONE input.timezone - INTERVAL '12 hours') FROM input
)
SELECT 
    start_of_day + arrival_time * INTERVAL '1 second' AS arrival_time,
    start_of_day + departure_time * INTERVAL '1 second' AS departure_time,
    stop_hierarchy.root_id, stop_hierarchy.stop_id, stops.stop_name, stops.platform_code, stops.stop_lon, stops.stop_lat
FROM stop_times NATURAL JOIN stop_hierarchy NATURAL JOIN stops, input, helper
WHERE trip_id = $3 ORDER BY stop_sequence