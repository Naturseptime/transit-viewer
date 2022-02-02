SELECT json_build_object(
    'type', 'FeatureCollection',
    'features', json_agg(ST_AsGeoJSON(t.*)::json))::text AS geojson
FROM (SELECT ST_MakePoint(stop_lon, stop_lat), stop_id, stop_name FROM stops WHERE parent_station IS NULL) AS t;