/**
 * MapLibre GL replacement for Leaflet-based map functionality.
 * Provides initPolygonEditor and initWorkerMap using MapLibre GL JS.
 */

// Map style — OpenStreetMap via MapLibre demo tiles
const MAP_STYLE = 'https://demotiles.maplibre.org/style.json';

/**
 * Calculate the centroid of a ring of coordinates [lng, lat].
 */
function ringCentroid(coords) {
    if (!coords || coords.length === 0) return null;
    let sumLng = 0, sumLat = 0;
    for (const [lng, lat] of coords) {
        sumLng += lng;
        sumLat += lat;
    }
    return [sumLng / coords.length, sumLat / coords.length];
}

/**
 * Calculate polygon area in hectares using the spherical excess formula.
 * Coords are [lng, lat] pairs.
 */
function polygonAreaHa(coords) {
    if (!coords || coords.length < 3) return null;

    const radius = 6378137;
    const toRad = (value) => (value * Math.PI) / 180;
    let area = 0;

    for (let i = 0, j = coords.length - 1; i < coords.length; j = i, i += 1) {
        const lat1 = toRad(coords[j][1]);
        const lat2 = toRad(coords[i][1]);
        const dLng = toRad(coords[i][0] - coords[j][0]);
        area += dLng * (2 + Math.sin(lat1) + Math.sin(lat2));
    }

    return Math.abs((area * radius * radius) / 2) / 10_000;
}

/**
 * Convert GeoJSON Polygon coordinates to {lat, lng} format expected by Rust.
 */
function toPoints(coords) {
    return coords.map(([lng, lat]) => ({ lat, lng }));
}

/**
 * Create a simple marker element.
 */
function createMarkerElement(color = '#ef4444') {
    const el = document.createElement('div');
    el.style.width = '20px';
    el.style.height = '20px';
    el.style.borderRadius = '50%';
    el.style.backgroundColor = color;
    el.style.border = '3px solid white';
    el.style.boxShadow = '0 2px 6px rgba(0,0,0,0.4)';
    return el;
}

/**
 * Initialize the worker map showing the user's current location.
 */
export function initWorkerMap(el) {
    if (el.__agrocoreMapInitialized) {
        return;
    }

    const maplibregl = window.maplibregl;
    if (!maplibregl) {
        console.error('MapLibre GL is not loaded');
        return;
    }

    const defaultCenter = [-9.1393, 38.7223]; // Lisbon fallback [lng, lat]

    function setupMap(center) {
        const map = new maplibregl.Map({
            container: el,
            style: MAP_STYLE,
            center: center,
            zoom: 16,
            attributionControl: true,
        });

        map.addControl(new maplibregl.NavigationControl(), 'top-right');

        // Add user location marker
        const markerEl = createMarkerElement('#ef4444');
        const marker = new maplibregl.Marker(markerEl)
            .setLngLat(center)
            .setPopup(new maplibregl.Popup({ offset: 25 }).setHTML('Ihr Standort'))
            .addTo(map);

        // Open popup after map loads
        map.on('load', () => {
            marker.togglePopup();
        });

        // Handle container resize for proper map rendering
        setTimeout(() => {
            map.resize();
        }, 300);

        el.__agrocoreMapInitialized = true;
        el.__agrocoreMap = map;
    }

    // Try to get user location
    if (navigator.geolocation) {
        navigator.geolocation.getCurrentPosition(
            (position) => {
                const center = [position.coords.longitude, position.coords.latitude];
                setupMap(center);
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

/**
 * Initialize the polygon editor for drawing field boundaries.
 * Uses click-to-draw interaction instead of Leaflet Draw plugin.
 */
export function initPolygonEditor(el, onChange) {
    if (el.__agrocoreMapInitialized) {
        return;
    }

    const maplibregl = window.maplibregl;
    if (!maplibregl) {
        console.error('MapLibre GL is not loaded');
        return;
    }

    const defaultCenter = [-9.1393, 38.7223]; // Lisbon fallback [lng, lat]

    function setupMap(center) {
        const map = new maplibregl.Map({
            container: el,
            style: MAP_STYLE,
            center: center,
            zoom: 16,
            attributionControl: true,
        });

        map.addControl(new maplibregl.NavigationControl(), 'top-right');

        // Add user location marker (red marker)
        const userMarkerEl = createMarkerElement('#ef4444');
        new maplibregl.Marker(userMarkerEl)
            .setLngLat(center)
            .setPopup(new maplibregl.Popup({ offset: 25 }).setHTML('Ihr Standort'))
            .addTo(map);

        // Polygon drawing state
        let drawingPoints = [];
        let drawingMarkers = [];
        let drawingLine = null;
        let polygonSourceId = null;
        let polygonLayerId = null;
        let outlineLayerId = null;

        // Emit polygon state to Rust
        function emitState(coords) {
            if (!coords || coords.length < 3) {
                onChange(null, null, null);
                return;
            }
            const points = toPoints(coords);
            const area = polygonAreaHa(coords);
            const centroid = ringCentroid(coords);
            // Convert centroid to {lat, lng} format
            const centerObj = centroid ? { lat: centroid[1], lng: centroid[0] } : null;
            onChange(points, area, centerObj);
        }

        // Clear existing polygon
        function clearPolygon() {
            if (polygonSourceId && map.getSource(polygonSourceId)) {
                map.removeSource(polygonSourceId);
            }
            if (polygonLayerId && map.getLayer(polygonLayerId)) {
                map.removeLayer(polygonLayerId);
            }
            if (outlineLayerId && map.getLayer(outlineLayerId)) {
                map.removeLayer(outlineLayerId);
            }
            polygonSourceId = null;
            polygonLayerId = null;
            outlineLayerId = null;
        }

        // Render polygon from coordinates
        function renderPolygon(coords) {
            clearPolygon();

            if (!coords || coords.length < 3) return;

            // Close the ring if not already closed
            const closed = [...coords];
            if (closed.length >= 3) {
                const first = closed[0];
                const last = closed[closed.length - 1];
                if (first[0] !== last[0] || first[1] !== last[1]) {
                    closed.push([...first]);
                }
            }

            const geojson = {
                type: 'Feature',
                geometry: {
                    type: 'Polygon',
                    coordinates: [closed],
                },
            };

            polygonSourceId = 'polygon-source';
            map.addSource(polygonSourceId, {
                type: 'geojson',
                data: geojson,
            });

            polygonLayerId = 'polygon-fill';
            map.addLayer({
                id: polygonLayerId,
                type: 'fill',
                source: polygonSourceId,
                paint: {
                    'fill-color': '#3b82f6',
                    'fill-opacity': 0.3,
                },
            });

            outlineLayerId = 'polygon-outline';
            map.addLayer({
                id: outlineLayerId,
                type: 'line',
                source: polygonSourceId,
                paint: {
                    'line-color': '#3b82f6',
                    'line-width': 2,
                },
            });
        }

        // Update drawing line
        function updateDrawingLine(mousePoint) {
            // Remove existing line
            const existingLineId = 'drawing-line';
            if (map.getLayer(existingLineId)) {
                map.removeLayer(existingLineId);
                map.removeSource(existingLineId);
            }

            if (drawingPoints.length < 1 || !mousePoint) return;

            const lineCoords = [...drawingPoints, mousePoint];
            map.addSource(existingLineId, {
                type: 'geojson',
                data: {
                    type: 'Feature',
                    geometry: {
                        type: 'LineString',
                        coordinates: lineCoords,
                    },
                },
            });

            map.addLayer({
                id: existingLineId,
                type: 'line',
                source: existingLineId,
                paint: {
                    'line-color': '#3b82f6',
                    'line-width': 2,
                    'line-dasharray': [5, 5],
                },
            });
        }

        // Clear drawing line
        function clearDrawingLine() {
            const lineId = 'drawing-line';
            if (map.getLayer(lineId)) {
                map.removeLayer(lineId);
            }
            if (map.getSource(lineId)) {
                map.removeSource(lineId);
            }
        }

        // Add a point marker
        function addPointMarker(coord) {
            const el = document.createElement('div');
            el.style.width = '8px';
            el.style.height = '8px';
            el.style.borderRadius = '50%';
            el.style.backgroundColor = '#3b82f6';
            el.style.border = '2px solid white';

            const marker = new maplibregl.Marker(el).setLngLat(coord).addTo(map);
            drawingMarkers.push(marker);
            return marker;
        }

        // Clear all drawing markers
        function clearDrawingMarkers() {
            drawingMarkers.forEach((m) => m.remove());
            drawingMarkers = [];
        }

        // Handle map click for drawing
        map.on('click', (e) => {
            const point = [e.lngLat.lng, e.lngLat.lat];

            // Check if clicking on the first point to close the polygon
            if (drawingPoints.length >= 3) {
                const firstPoint = drawingPoints[0];
                const distance = Math.sqrt(
                    Math.pow(point[0] - firstPoint[0], 2) + Math.pow(point[1] - firstPoint[1], 2)
                );

                // If close to first point (within ~0.002 degrees ≈ 200m), close the polygon
                if (distance < 0.002) {
                    clearDrawingLine();
                    clearDrawingMarkers();
                    drawingPoints = [];

                    // Render final polygon
                    renderPolygon(drawingPoints);
                    emitState(drawingPoints);
                    return;
                }
            }

            drawingPoints.push(point);
            addPointMarker(point);

            if (drawingPoints.length >= 3) {
                // Preview the polygon
                renderPolygon(drawingPoints);
                emitState(drawingPoints);
            }
        });

        // Handle mouse move for drawing line
        map.on('mousemove', (e) => {
            if (drawingPoints.length > 0) {
                clearDrawingLine();
                updateDrawingLine([e.lngLat.lng, e.lngLat.lat]);
            }
        });

        // Double click to finish/clear drawing
        map.on('dblclick', (e) => {
            e.preventDefault();
            clearDrawingLine();
            clearDrawingMarkers();
            clearPolygon();
            drawingPoints = [];
            onChange(null, null, null);
        });

        // Right click to clear current drawing
        map.on('contextmenu', (e) => {
            e.preventDefault();
            clearDrawingLine();
            clearDrawingMarkers();
            clearPolygon();
            drawingPoints = [];
            onChange(null, null, null);
        });

        // Handle container resize
        setTimeout(() => {
            map.resize();
        }, 300);

        el.__agrocoreMapInitialized = true;
        el.__agrocoreMap = map;
    }

    // Try to get user location
    if (navigator.geolocation) {
        navigator.geolocation.getCurrentPosition(
            (position) => {
                const center = [position.coords.longitude, position.coords.latitude];
                setupMap(center);
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
