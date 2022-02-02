WITH input(date) AS (
  VALUES($1::DATE)
),
paths_in_tile AS (
  SELECT ST_AsMVTGeom(path, ST_TileEnvelope($2, $3, $4), extent => 4096, buffer => 64) AS path, astop, bstop
  FROM segment_paths
  WHERE $2 >= zoom_threshold AND (path && ST_TileEnvelope($2, $3, $4, margin => (64.0 / 4096)))
),
segment_frequencies_at_day(astop, bstop, cnt, path) AS (
  SELECT astop, bstop, SUM(trip_count), path
  FROM paths_in_tile NATURAL JOIN segment_frequencies, input
    WHERE
  (EXISTS
       (SELECT *
        FROM calendar AS c
        WHERE c.service_id = segment_frequencies.service_id
          AND (input.date >= c.start_date
               AND input.date < c.end_date
               AND CASE EXTRACT(DOW FROM input.date)
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
        FROM calendar_dates AS cd
        WHERE cd.service_id = segment_frequencies.service_id
          AND cd.date = input.date
          AND cd.exception_type = 1))
    AND NOT EXISTS
      (SELECT *
       FROM calendar_dates AS cd
       WHERE cd.service_id = segment_frequencies.service_id
         AND cd.date = input.date
         AND cd.exception_type = 2)
  GROUP BY astop, bstop, path),
paths_with_frequency AS (
  SELECT path, astop, bstop, cnt FROM segment_frequencies_at_day
  WHERE cnt >= 16
  ORDER BY cnt ASC
)
SELECT ST_AsMVT(paths_with_frequency.*) AS freqmvt
FROM paths_with_frequency;