/**
 * MapLibre GL wrapper for admin-ui
 * Replaces Leaflet-based map with MapLibre GL JS v4
 */

// MapLibre GL types and constants
const MAPLIBRE_CSS = '/vendor/maplibre-gl.css';
const MAPLIBRE_JS = '/vendor/maplibre-gl.js';

// Map style options
export const MAP_STYLES = {
    'osm': {
        name: 'OpenStreetMap',
        url: 'https://demotiles.maplibre.org/style.json',
        attribution: '© OpenStreetMap contributors'
    },
    'satellite': {
        name: 'Satellite',
        url: 'https://api.maptiler.com/maps/satellite/style.json?key=ubiG2xzaMqYn6g2LkeI8',
        attribution: '© MapTiler © OpenStreetMap contributors'
    },
    'basic': {
        name: 'Basic',
        url: 'https://demotiles.maplibre.org/style.json',
        attribution: '© OpenStreetMap contributors'
    }
};

export class MapLibreMap {
    constructor(containerId, options = {}) {
        this.containerId = containerId;
        this.options = {
            style: options.style || MAP_STYLES.basic.url,
            center: options.center || [38.7223, -9.1393], // Lisbon default
            zoom: options.zoom || 13,
            pitch: options.pitch || 0,
            bearing: options.bearing || 0,
            ...options
        };
        this.map = null;
        this.workerMarkers = new Map();
        this.taskLayers = new Map();
        this.siteLayers = new Map();
        this.isInitialized = false;
    }

    async init() {
        // Load MapLibre CSS if not already loaded
        if (!document.querySelector(`link[href="${MAPLIBRE_CSS}"]`)) {
            const link = document.createElement('link');
            link.rel = 'stylesheet';
            link.href = MAPLIBRE_CSS;
            document.head.appendChild(link);
        }

        // Load MapLibre JS if not already loaded
        if (!window.maplibregl) {
            await this.loadScript(MAPLIBRE_JS);
        }

        const maplibregl = window.maplibregl;
        if (!maplibregl) {
            throw new Error('MapLibre GL failed to load');
        }

        const container = document.getElementById(this.containerId);
        if (!container) {
            throw new Error(`Container ${this.containerId} not found`);
        }

        this.map = new maplibregl.Map({
            container: this.containerId,
            style: this.options.style,
            center: this.options.center,
            zoom: this.options.zoom,
            pitch: this.options.pitch,
            bearing: this.options.bearing
        });

        this.map.addControl(new maplibregl.NavigationControl(), 'top-right');
        this.map.addControl(new maplibregl.ScaleControl({ unit: 'metric' }), 'bottom-right');

        this.isInitialized = true;
        return this;
    }

    loadScript(src) {
        return new Promise((resolve, reject) => {
            if (document.querySelector(`script[src="${src}"]`)) {
                resolve();
                return;
            }
            const script = document.createElement('script');
            script.src = src;
            script.async = true;
            script.onload = resolve;
            script.onerror = reject;
            document.head.appendChild(script);
        });
    }

    // Worker location methods
    addWorkerLocation(workerId, lat, lng, options = {}) {
        if (!this.isInitialized) return;

        const maplibregl = window.maplibregl;
        const el = document.createElement('div');
        el.className = 'worker-marker';
        el.innerHTML = `
            <div class="worker-marker-inner ${options.inTask ? 'in-task' : 'idle'}">
                <div class="worker-pulse"></div>
            </div>
        `;

        const marker = new window.maplibregl.Marker(el)
            .setLngLat([lng, lat])
            .addTo(this.map);

        if (options.popup) {
            marker.setPopup(new maplibregl.Popup({ offset: 25 }).setHTML(options.popup));
        }

        this.workerMarkers.set(workerId, { marker, ...options });
        return marker;
    }

    updateWorkerLocation(workerId, lat, lng, options = {}) {
        const existing = this.workerMarkers.get(workerId);
        if (existing) {
            existing.marker.setLngLat([lng, lat]);
            if (options.popup && existing.marker.getPopup()) {
                existing.marker.getPopup().setHTML(options.popup);
            }
            Object.assign(existing, options);
        } else {
            this.addWorkerLocation(workerId, lat, lng, options);
        }
    }

    removeWorkerLocation(workerId) {
        const existing = this.workerMarkers.get(workerId);
        if (existing) {
            existing.marker.remove();
            this.workerMarkers.delete(workerId);
        }
    }

    // Task/Subtask layer methods
    addTaskLayer(taskId, geojson, options = {}) {
        if (!this.isInitialized || this.taskLayers.has(taskId)) return;

        const layerId = `task-${taskId}`;
        const sourceId = `task-source-${taskId}`;

        this.map.addSource(sourceId, {
            type: 'geojson',
            data: geojson
        });

        const layer = {
            id: layerId,
            type: geojson.geometry?.type === 'Point' ? 'circle' : 
                  geojson.geometry?.type === 'LineString' ? 'line' : 'fill',
            source: sourceId,
            paint: this.getTaskPaint(options.status || 'pending')
        };

        this.map.addLayer(layer);
        this.taskLayers.set(taskId, { layerId, sourceId, ...options });
        return layer;
    }

    getTaskPaint(status) {
        const colors = {
            'new': { fill: '#6b7280', line: '#6b7280', circle: '#6b7280' },
            'pending': { fill: '#3b82f6', line: '#3b82f6', circle: '#3b82f6' },
            'started': { fill: '#f59e0b', line: '#f59e0b', circle: '#f59e0b' },
            'in_progress': { fill: '#f59e0b', line: '#f59e0b', circle: '#f59e0b' },
            'paused': { fill: '#8b5cf6', line: '#8b5cf6', circle: '#8b5cf6' },
            'completed': { fill: '#10b981', line: '#10b981', circle: '#10b981' },
            'stopped': { fill: '#ef4444', line: '#ef4444', circle: '#ef4444' },
            'done': { fill: '#10b981', line: '#10b981', circle: '#10b981' }
        };
        const c = colors[status] || colors.pending;
        return {
            'fill-color': c.fill,
            'fill-opacity': 0.4,
            'fill-outline-color': c.line,
            'line-color': c.line,
            'line-width': 2,
            'circle-color': c.circle,
            'circle-radius': 8,
            'circle-stroke-width': 2,
            'circle-stroke-color': '#ffffff'
        };
    }

    updateTaskLayer(taskId, geojson, options = {}) {
        const sourceId = `task-source-${taskId}`;
        if (this.map.getSource(sourceId)) {
            this.map.getSource(sourceId).setData(geojson);
        }
        const layerId = `task-${taskId}`;
        if (this.map.getLayer(layerId) && options.status) {
            const paint = this.getTaskPaint(options.status);
            Object.entries(paint).forEach(([key, value]) => {
                this.map.setPaintProperty(`task-${taskId}`, key, value);
            });
        }
    }

    removeTaskLayer(taskId) {
        const layerId = `task-${taskId}`;
        const sourceId = `task-source-${taskId}`;
        if (this.map.getLayer(layerId)) this.map.removeLayer(layerId);
        if (this.map.getSource(sourceId)) this.map.removeSource(sourceId);
        this.taskLayers.delete(taskId);
    }

    // Site/Parcel layers
    addSiteLayer(siteId, geojson, options = {}) {
        const layerId = `site-${siteId}`;
        const sourceId = `site-source-${siteId}`;

        this.map.addSource(sourceId, {
            type: 'geojson',
            data: geojson
        });

        this.map.addLayer({
            id: layerId,
            type: 'fill',
            source: sourceId,
            paint: {
                'fill-color': options.color || '#3b82f6',
                'fill-opacity': 0.3,
                'fill-outline-color': options.color || '#3b82f6'
            }
        });

        this.siteLayers.set(siteId, { layerId, sourceId, ...options });
    }

    // Fit map to bounds
    fitBounds(bounds, padding = 50) {
        this.map.fitBounds(bounds, { padding });
    }

    // Fly to location
    flyTo(center, zoom, options = {}) {
        this.map.flyTo({ center, zoom, ...options });
    }

    // Cleanup
    destroy() {
        this.workerMarkers.forEach(({ marker }) => marker.remove());
        this.taskLayers.forEach(({ layerId, sourceId }) => {
            if (this.map.getLayer(layerId)) this.map.removeLayer(layerId);
            if (this.map.getSource(sourceId)) this.map.removeSource(sourceId);
        });
        this.siteLayers.forEach(({ layerId, sourceId }) => {
            if (this.map.getLayer(layerId)) this.map.removeLayer(layerId);
            if (this.map.getSource(sourceId)) this.map.removeSource(sourceId);
        });
        if (this.map) {
            this.map.remove();
            this.map = null;
        }
        this.isInitialized = false;
    }
}

// React/Leptos integration helper
window.MapLibreMap = MapLibreMap;

export default MapLibreMap;