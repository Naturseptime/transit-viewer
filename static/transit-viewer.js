const urlParams = new URLSearchParams(window.location.search);
var feed = urlParams.get('feed')
var date = urlParams.get('date') || new Date().toISOString().slice(0, 10);

// ***** OSM base layer *****

var map = L.map('map', {maxZoom: 17});
var attributionText = null;
var tileLayer = L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
  attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
  detectRetina: true, maxNativeZoom: 19}).addTo(map);

// ***** Stops layer *****

var stopMarkers = null;
var stationPopupOptions = {minWidth: 300, minHeight: 100, maxHeight: 300};

var stopsLayer = L.layerGroup([]);
var frequencyLayer = L.layerGroup([]);
map.addLayer(stopsLayer);
map.addLayer(frequencyLayer);
addLayerSelection();

function reloadAll() {
  tripSidebar.hide();
  map.spin(true);
  reloadStops();
  reloadSegments();
}

function fitView(stopMarkers) {
  var coords = Object.values(stopMarkers).map(function(m) { return [m.position.lat, m.position.lng]; })
  var lats = coords.map(function(c) {return c[0];});
  var lngs = coords.map(function(c) {return c[1];});
  lats.sort();
  lngs.sort();
  var a = Math.floor(0.25 * coords.length);
  var b = Math.floor(0.75 * coords.length);
  var bounds = L.latLngBounds(
    L.latLng(lats[a], lngs[a]),
    L.latLng(lats[b], lngs[b]));
  map.fitBounds(bounds);
}

function reloadStops() {
  stopMarkers = {};
  stopsLayer.clearLayers();
  
  var pruneCluster = new PruneClusterForLeaflet(120, 20);

  pruneCluster.PrepareLeafletMarker = function (marker, data, category) {
    if(!marker.getTooltip()) {
      marker.bindTooltip(data.properties.stop_name);
      marker.bindPopup("Loading...", stationPopupOptions);
      marker.on('popupopen', function() {
        loadStopInformation(marker.getPopup(), data.properties.stop_id);});
    }
  };  
      
  $.ajax({url: "/" + encodeURIComponent(feed) + "/stops"}).done(function(geojsonData) {  
    L.geoJSON(geojsonData, {
      pointToLayer: function(feature, latLng) {
        var marker = new PruneCluster.Marker(latLng.lat, latLng.lng);
        marker.data.properties = feature.properties;
        pruneCluster.RegisterMarker(marker);
        stopMarkers[feature.properties.stop_id] = marker;
      }
    });
    pruneCluster.ProcessView();
    
    //~ var bounds = pruneCluster.Cluster.ComputeGlobalBounds();
    //~ map.fitBounds(L.latLngBounds(L.latLng(bounds.minLat, bounds.minLng), L.latLng(bounds.maxLat, bounds.maxLng)));
    fitView(stopMarkers);
    
    stopsLayer.addLayer(pruneCluster);
    map.spin(false);
  });
}

function loadStopInformation(popup, stop_id) {
  $.ajax({url: "/" + encodeURIComponent(feed) + "/" + encodeURIComponent(date) + "/stops/" + encodeURIComponent(stop_id)}).done(
    function(data) {
      popup.setContent(data);
      popup.update();
    });
};

// ***** Trips layer *****

var tripLayer = null;  

function onTripLoaded(data) {
  tripLayer = L.geoJson.vt(data, {
    maxZoom: 24,
    tolerance: 3,
    style: function(tags) {
      return {weight: 8, color: "#FF0000", opacity: 0.7};
    },
  });
  
  tripSidebar.setContent(data.features[0].properties.trip_info);
  tripSidebar.show();

  map.addLayer(tripLayer);  
}

function jumpToStopAndShowInfo(stop_id) {
  var marker = stopMarkers[stop_id];
  var coords = [marker.position.lat, marker.position.lng];
  map.panTo(coords);
  var popup = L.popup(stationPopupOptions).setLatLng(coords);
  popup.openOn(map);
  loadStopInformation(popup, stop_id);
}

function onStopDepartureClicked(element) {
  showTrip(element.dataset.tripId);
}

function onTripStopClicked(element) {
  jumpToStopAndShowInfo(element.dataset.stopId);
}

function showTrip(trip_id) {
  if(tripLayer) map.removeLayer(tripLayer);
  if(map) map.closePopup();

  $.ajax({url: "/" + encodeURIComponent(feed) + "/" + encodeURIComponent(date) + "/trips/" + encodeURIComponent(trip_id)}).done(onTripLoaded);
}  
  
var tripSidebar = L.control.sidebar('trip-sidebar', {
  closeButton: true,
  position: 'right'
}).addTo(map)

tripSidebar.on('hidden', function () {
  if(tripLayer) map.removeLayer(tripLayer);
});

// ***** Frequency layer *****

function styleForTripFrequency(t) {
  if(t >= 256)
    return {color: "#000000", opacity: 0.5}; // < 5min
  else if(t >= 192)
    return {color: "#000055", opacity: 0.5}; // 5min
  else if(t >= 128)
    return {color: "#0000AA", opacity: 0.5}; // 7.5min
  else if(t >= 96)
    return {color: "#0827FF", opacity: 0.5}; // 10min
  else if(t >= 64)
    return {color: "#115588", opacity: 0.4}; // 15min
  else if(t >= 48)
    return {color: "#188855", opacity: 0.4}; // 20min
  else if(t >= 32)
    return {color: "#22BB22", opacity: 0.4}; // 30min
  else if(t >= 24)
    return {color: "#88DD11", opacity: 0.3}; // 45min
  else if(t >= 16)
    return {color: "#EEEE11", opacity: 0.3}; // 1h
  else if(t >= 8)
    return {color: "#EE8811", opacity: 0.2}; // 2h
  else
    return {color: "#FF0000", opacity: 0.1}; // > 2h
};

var geojsonVTLayer = null;

function onFrequencyLayerLoaded(data) {
  geojsonVTLayer.redraw();
}


function reloadSegments() {
  uri = "/" + encodeURIComponent(feed) + "/frequency/" + encodeURIComponent(date) + "/{z}/{x}/{y}/tile.pbf"
  geojsonVTLayer = L.vectorGrid.protobuf(uri, {
    maxZoom: 24,
    tolerance: 1,
    rendererFactory: L.canvas.tile,
    vectorTileLayerStyles: {
      default:  function(properties, zoom) {
        s = styleForTripFrequency(properties.cnt);
        s.weight = 5;
        return s;
      }
    }
    
  });
  
  frequencyLayer.clearLayers();
  frequencyLayer.addLayer(geojsonVTLayer);
}



function onSegmentFrequenciesLoaded(data) {
  segmentFrequencies = {};
  data.forEach(function(entry) {
      segmentFrequencies[[entry[0], entry[1]]] = entry[2];
  });
  onFrequencyLayerLoaded(segmentsGeojson);
}

// ***** Layer selection *****

function addLayerSelection() {
  var baseMaps = {};
  var overlayMaps = {
    "Show all stops": stopsLayer,
    "Show frequencies": frequencyLayer};
  L.control.layers(baseMaps, overlayMaps, {position: 'topright'}).addTo(map);
}

// ***** UI controls *****

function updateURL() {
  var url = new URL(window.location.href);
  url.searchParams.set("feed", feed);
  url.searchParams.set("date", date);
  window.history.replaceState({}, '', url);
}

function updateAttribution() {
  map.attributionControl.removeAttribution(attributionText);
  attributionText = $("<a>", {href: feeds[feed].feed_publisher_url}).text(feeds[feed].feed_publisher_name)[0].outerHTML;
  map.attributionControl.addAttribution(attributionText);
}

function onChangeFeed(newfeed) {
  feed = newfeed;
  updateAttribution();
  updateURL();
  reloadAll();
}

function onChangeDate(newdate) {
  date = newdate;
  updateURL();
  reloadSegments();
}

function onFeedListLoaded(data) {
  feeds = [];

  var select = document.createElement('select');
  select.name = select.id = 'feed-select';
  data.forEach(function(entry) {
    feeds[entry.feed_uid] = entry;
    var option = document.createElement('option');
    option.value = entry.feed_uid;
    option.textContent = entry.feed_title;
    if(entry.feed_uid == feed)
      option.setAttribute('selected', 'selected');
    select.appendChild(option);});
  select.setAttribute('onchange', "onChangeFeed(this.value)");
  
  updateAttribution();

  var date_input = document.createElement('input');
  date_input.type = 'date';
  date_input.name = date_input.id = 'date-select';
  date_input.setAttribute('value', date);
  date_input.setAttribute('onchange', "onChangeDate(this.value)");
  
  html = "<div style='margin: 0.5em'><label>Select feed:<br>" + select.outerHTML + "</label>" +
    "<label>Select date:<br>" + date_input.outerHTML + "</label></div>";
  
  L.control.custom({
    position: 'bottomright',
    content : html,
    classes : 'leaflet-control-layers'}).addTo(map);
}

$.ajax({url: "/feeds"}).done(onFeedListLoaded);

L.control.fullscreen({position: 'topleft'}).addTo(map);
L.control.scale().addTo(map);
var feeds = null;


reloadAll();