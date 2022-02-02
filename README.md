# Transit Viewer

View public transit networks with an interactive map based on GTFS data.

Current features:
* Show departures for a selected station
* Show trips with intermediate stops and route on the map.
* Colorful map with service frequencies per day

transit-viewer is written in Rust and Javascript. It uses Postgres/Postgis as database, Rocket as REST-API and Leaflet for the slippy map.

![screenshot-munich](https://user-images.githubusercontent.com/24992664/152240586-e9623475-9def-4989-b8e5-f4347d21393b.png)
![screenshot-departures](https://user-images.githubusercontent.com/24992664/152240584-c3154fa7-c7dc-4022-ad75-224d61f5658b.png)
![screenshot-nl](https://user-images.githubusercontent.com/24992664/152240588-070352ae-9ef5-4975-8666-b09fb6d0e26f.png)
![screenshot-trip](https://user-images.githubusercontent.com/24992664/152240594-8053698b-ef52-4e54-80ca-6d7c8273fab6.png)

## Setup 

Install postgresql and the postgis extension.

Create user and database in postgresql

    CREATE USER transitviewer;
    CREATE DATABASE transitviewer OWNER transitviewer;
    \connect transitviewer
    CREATE EXTENSION postgis;

Adjust database name in Rocket.toml

    [default.databases.gtfs_data]
    url = "postgresql://transitviewer:mypassword@localhost/transitviewer"

Download your favorite GTFS feed, e.g. from here https://transitfeeds.com

Import the GTFS feed into the database. This can take a few minutes for large datasets.

    cargo install --path import-gtfs
    import-gtfs MY_GTFS.zip "host=localhost user=transitviewer" --feed-title "My GTFS Feed"

## Running

Start REST API server

    cargo run

In the web browser:

    http://localhost:8000
