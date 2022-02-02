DELETE FROM frequency_tile_cache WHERE (date, z, x, y) IN 
    (SELECT date, z, x, y FROM frequency_tile_cache 
     ORDER BY last_hit ASC LIMIT GREATEST((SELECT COUNT(*) FROM frequency_tile_cache) - $1::INT, 0));