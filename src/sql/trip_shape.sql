-- Use shape points in shapes.txt when possible
WITH path AS (
    SELECT public.ST_MakeLine(
        public.ST_SetSRID(public.ST_MakePoint(shape_pt_lon, shape_pt_lat), 4326)
        ORDER BY shape_pt_sequence ASC) AS path
    FROM shapes NATURAL JOIN trips
    WHERE trip_id = $1),
-- When no shapes.txt is provided we connected adjacent stations by straight lines
path_replacement AS (
    SELECT public.ST_MakeLine(
        public.ST_SetSRID(public.ST_MakePoint(stop_lon, stop_lat), 4326)
        ORDER BY stop_sequence ASC) AS path
    FROM stop_times NATURAL JOIN stops
    WHERE trip_id = $1),
paths_or_replacement AS (
    SELECT path FROM
    (SELECT * FROM path UNION SELECT * FROM path_replacement) AS t
    ORDER BY COALESCE(public.ST_NPoints(path), 0) DESC
    LIMIT 1)
SELECT ST_AsGeoJSON(paths_or_replacement.*)::json AS geojson FROM paths_or_replacement