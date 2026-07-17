const TILE_URL = "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png";
const TILE_ATTRIBUTION = '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors';

function leaflet() {
    const L = globalThis.L;
    if (!L) {
        throw new Error("Leaflet is not loaded");
    }
    return L;
}

function toLatLng(point) {
    return [point.lat, point.lng];
}

function toPoints(latlngs) {
    return latlngs.map((point) => ({ lat: point.lat, lng: point.lng }));
}

function ringCentroid(points) {
    if (points.length === 0) {
        return null;
    }

    const totals = points.reduce(
        (acc, point) => {
            acc.lat += point.lat;
            acc.lng += point.lng;
            return acc;
        },
        { lat: 0, lng: 0 },
    );

    return {
        lat: totals.lat / points.length,
        lng: totals.lng / points.length,
    };
}

function polygonAreaHa(points) {
    if (points.length < 3) {
        return null;
    }

    const radius = 6378137;
    const toRad = (value) => (value * Math.PI) / 180;
    let area = 0;

    for (let i = 0, j = points.length - 1; i < points.length; j = i, i += 1) {
        const p1 = points[j];
        const p2 = points[i];
        area += toRad(p2.lng - p1.lng) * (2 + Math.sin(toRad(p1.lat)) + Math.sin(toRad(p2.lat)));
    }

    return Math.abs((area * radius * radius) / 2) / 10_000;
}

function initializeMap(el, center, zoom) {
    const L = leaflet();
    const map = L.map(el, {
        zoomControl: true,
        preferCanvas: true,
    }).setView(center, zoom);

    L.tileLayer(TILE_URL, {
        attribution: TILE_ATTRIBUTION,
        maxZoom: 20,
    }).addTo(map);

    return { L, map };
}

function emitPolygonState(onChange, layer) {
    const polygon = layer.getLatLngs()[0] || [];
    const points = toPoints(polygon);

    if (points.length < 3) {
        onChange(null, null, null);
        return;
    }

    onChange(points, polygonAreaHa(points), ringCentroid(points));
}

export function initPolygonEditor(el, onChange, onLocation) {
    if (el.__agrocoreLeafletInitialized) {
        return;
    }

    // Get user's current location
    const defaultCenter = [38.7223, -9.1393]; // Lisbon fallback

    function setupMap(center) {
        const { L, map } = initializeMap(el, center, 16);
        const drawnItems = L.featureGroup().addTo(map);

        // Add user location marker (red marker)
        L.marker(center, {
            icon: L.icon({
                iconUrl: 'https://raw.githubusercontent.com/pointhi/leaflet-color-markers/master/img/marker-icon-red.png',
                shadowUrl: 'https://cdnjs.cloudflare.com/ajax/libs/leaflet/1.9.4/images/marker-shadow.png',
                iconSize: [25, 41],
                iconAnchor: [12, 41],
                popupAnchor: [1, -34],
            })
        }).addTo(map);

        L.popup().setLatLng(center).setContent('Ihr Standort').openOn(map);

        // Notify parent about user location
        if (typeof onLocation === 'function') {
            onLocation({ lat: center[0], lng: center[1] });
        }

        // Handle container resize for proper map rendering in modal
        setTimeout(() => {
            map.invalidateSize();
        }, 300);

        const drawControl = new L.Control.Draw({
            draw: {
                polygon: true,
                polyline: false,
                rectangle: false,
                circle: false,
                circlemarker: false,
                marker: false,
            },
            edit: {
                featureGroup: drawnItems,
                edit: true,
                remove: true,
            },
        });

        map.addControl(drawControl);

        map.on(L.Draw.Event.CREATED, (event) => {
            drawnItems.clearLayers();
            drawnItems.addLayer(event.layer);
            emitPolygonState(onChange, event.layer);
        });

        // Handle polygon completion when clicking first point
        map.on('draw:drawstop', () => {
            setTimeout(() => {
                const layers = drawnItems.getLayers();
                if (layers.length > 0) {
                    emitPolygonState(onChange, layers[0]);
                }
            }, 100);
        });

        map.on(L.Draw.Event.EDITED, (event) => {
            event.layers.eachLayer((layer) => emitPolygonState(onChange, layer));
        });

        map.on(L.Draw.Event.DELETED, () => {
            onChange(null, null, null);
        });

        el.__agrocoreLeafletInitialized = true;
    }

    // Try to get user location
    if (navigator.geolocation) {
        navigator.geolocation.getCurrentPosition(
            (position) => {
                setupMap([position.coords.latitude, position.coords.longitude]);
            },
            () => {
                // Geolocation failed, use default
                setupMap(defaultCenter);
            },
            { enableHighAccuracy: true, timeout: 5000, maximumAge: 0 }
        );
    } else {
        setupMap(defaultCenter);
    }
}

export function initWorkerMap(el) {
    if (el.__agrocoreLeafletInitialized) {
        return;
    }

    const { L, map } = initializeMap(el, [48.8566, 2.3522], 13);

    // Handle container resize for proper map rendering
    setTimeout(() => {
        map.invalidateSize();
    }, 300);

    el.__agrocoreLeafletInitialized = true;
}