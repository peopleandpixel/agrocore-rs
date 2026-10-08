// AgroCore Admin PWA Service Worker
//
// Provides offline caching, background sync, and push notifications.
//
// This file was never registered: nothing in the application called
// `serviceWorker.register`, so none of it ran. It was also broken in ways that would
// have failed on the first registration attempt:
//
//   - `STATIC_ASSETS` named `/styles/tailwind.css` and `/styles/daisyui.css`, which
//     do not exist. The stylesheets live at `/vendor/` now, after the CDN links were
//     removed from `index.html`. `cache.addAll` rejects the whole install if any one
//     URL 404s, so a wrong path means no service worker at all.
//   - It tried to cache two API endpoints during install. A `GET /api/...` there
//     returns 401 for an unauthenticated request, `addAll` rejects on any non-2xx, and
//     the install fails. API responses belong in the dynamic cache, filled by the
//     fetch handler when a request actually succeeds.
//   - `cacheFirst` for every non-API request meant the application shell was served
//     from cache forever. After a deploy the browser would keep running the previous
//     build until the cache was cleared by hand, which is the wrong trade for an
//     application where workers enter data.
//   - `manifest.json` declares `/icons/icon-*.png`, and that directory did not exist.
//
// The strategy below is the standard one for a shell that must start offline:
//
//   * precache the shell at install, so a cold start with no network works;
//   * network-first for navigations and API calls, so a deployed change is picked up
//     as soon as there is a network, and the cache is the fallback rather than the
//     default;
//   * cache-first only for hashed, immutable assets — those cannot change under a
//     given name, so caching them is safe and makes repeat loads free.
//
// Cache names are versioned. Bumping the version invalidates everything at once,
// which is how a deploy reaches clients that were offline when it happened.

const VERSION = 'v2';
const SHELL_CACHE = `agrocore-shell-${VERSION}`;
const ASSET_CACHE = `agrocore-asset-${VERSION}`;
const DATA_CACHE = `agrocore-data-${VERSION}`;
const KEEP = [SHELL_CACHE, ASSET_CACHE, DATA_CACHE];

// The shell: what is needed to start the application with no network at all.
//
// Every entry is checked by `precacheAll`, which logs and skips what is missing
// rather than failing the install — a shell that is 95% present and starts is more
// useful than one that refuses to install because of a typo, and the failure mode of
// a missing asset is a visible 404 rather than a silent blank page.
const SHELL_ASSETS = [
  '/',
  // Emitted by Trunk from the crate-root `index.html`, not copied from `public/`, so
  // it is not a file this repository contains. `precacheAll` skips what is missing and
  // the shell still installs; it is listed because the path is what the navigation
  // handler falls back to.
  '/index.html',
  '/manifest.json',
  '/offline.html',
  // Stylesheets and scripts, vendored. See the note in index.html: these were CDN
  // URLs, which is why the UI could not start without a network.
  '/vendor/daisyui.css',
  '/vendor/tailwind.js',
  '/vendor/leaflet.css',
  '/vendor/leaflet-draw.css',
  '/vendor/leaflet.js',
  '/vendor/leaflet-draw.js',
  '/vendor/maplibre-gl.js',
  '/vendor/maplibre-gl.css',
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    (async () => {
      const cache = await caches.open(SHELL_CACHE);
      await precacheAll(cache, SHELL_ASSETS);
      await self.skipWaiting();
    })(),
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    (async () => {
      // Anything not in the current version set is deleted. This is the only thing
      // that evicts an old build for a client that was offline during a deploy, so it
      // must run on every activation rather than only when a name changes.
      const names = await caches.keys();
      await Promise.all(
        names.filter((name) => !KEEP.includes(name)).map((name) => caches.delete(name)),
      );
      await self.clients.claim();
    })(),
  );
});

async function precacheAll(cache, urls) {
  const results = await Promise.allSettled(
    urls.map(async (url) => {
      const response = await fetch(url, { credentials: 'same-origin' });
      if (!response.ok) {
        throw new Error(`${response.status} for ${url}`);
      }
      await cache.put(url, response);
    }),
  );
  const failed = results
    .map((r, i) => (r.status === 'rejected' ? urls[i] : null))
    .filter(Boolean);
  if (failed.length) {
    console.warn('[sw] not precached:', failed.join(', '));
  }
}

self.addEventListener('fetch', (event) => {
  const { request } = event;
  if (request.method !== 'GET') {
    return;
  }

  const url = new URL(request.url);
  if (url.origin !== self.location.origin) {
    // Tile servers and any other third party. Not cached and not intercepted: an
    // offline map is a product decision, not something the worker should fake by
    // serving a stale tile.
    return;
  }

  // API responses: network first, cache as the fallback. An API response is only
  // written to the cache when it actually succeeded, so a 401 or a 500 never
  // overwrites a good response with an error page.
  if (url.pathname.startsWith('/api/')) {
    event.respondWith(networkFirst(request, DATA_CACHE));
    return;
  }

  // Navigations: network first, so a deployed change is picked up on the next load
  // while still working offline. Falling back to `/index.html` keeps the client-side
  // router able to resolve a deep link like `/tasks`.
  if (request.mode === 'navigate') {
    event.respondWith(navigationHandler(request));
    return;
  }

  // Hashed assets are immutable under their own name: `admin-ui-<hash>.wasm` cannot
  // change without changing the name, so cache-first is correct and makes repeat
  // loads free.
  if (isHashedAsset(url.pathname)) {
    event.respondWith(cacheFirst(request, ASSET_CACHE));
    return;
  }

  event.respondWith(networkFirst(request, SHELL_CACHE));
});

function isHashedAsset(pathname) {
  return /\.[0-9a-f]{8,}\.(js|css|wasm)$/i.test(pathname);
}

async function navigationHandler(request) {
  try {
    const response = await fetch(request);
    if (response.ok) {
      const cache = await caches.open(SHELL_CACHE);
      cache.put('/index.html', response.clone());
    }
    return response;
  } catch {
    // Deep link → the shell, so the router can resolve it. Otherwise the dedicated
    // offline page, which explains the situation instead of showing a blank body.
    const shell = await caches.match('/index.html', { cacheName: SHELL_CACHE });
    if (shell) {
      return shell;
    }
    const offline = await caches.match('/offline.html', { cacheName: SHELL_CACHE });
    if (offline) {
      return offline;
    }
    return new Response('Offline', {
      status: 503,
      headers: { 'Content-Type': 'text/plain; charset=utf-8' },
    });
  }
}

async function networkFirst(request, cacheName) {
  try {
    const response = await fetch(request);
    if (response.ok) {
      const cache = await caches.open(cacheName);
      cache.put(request, response.clone());
    }
    return response;
  } catch {
    const cached = await caches.match(request);
    if (cached) {
      return cached;
    }
    // No cached copy. A 503 with JSON, so a client that parses the body gets a
    // parseable error rather than an HTML error page it cannot read — and, unlike an
    // empty body, one it can tell apart from an empty result.
    return new Response(JSON.stringify({ error: 'offline', message: 'No network and no cached response' }), {
      status: 503,
      headers: { 'Content-Type': 'application/json' },
    });
  }
}

async function cacheFirst(request, cacheName) {
  const cached = await caches.match(request);
  if (cached) {
    return cached;
  }
  const response = await fetch(request);
  if (response.ok) {
    const cache = await caches.open(cacheName);
    cache.put(request, response.clone());
  }
  return response;
}

// Background sync for task updates.
//
// The queued requests live in a cache keyed by a synthetic URL, because the Cache API
// stores Requests and this keeps the queued item readable for inspection. A queued
// entry is only deleted after the server accepted it.
self.addEventListener('sync', (event) => {
  if (event.tag === 'agrocore-sync-tasks') {
    event.waitUntil(flushQueue());
  }
});

const QUEUE_PREFIX = '/__agrocore-queue__/';

async function flushQueue() {
  const cache = await caches.open(DATA_CACHE);
  const requests = await cache.keys();
  for (const request of requests) {
    if (!request.url.includes(QUEUE_PREFIX)) {
      continue;
    }
    try {
      const stored = await cache.match(request);
      if (!stored) {
        continue;
      }
      const { url, method, headers, body } = await stored.json();
      const response = await fetch(url, {
        method,
        headers,
        body,
        credentials: 'same-origin',
      });
      // Only 2xx means the change landed. A 4xx or 5xx keeps the entry queued, so a
      // rejected change is retried rather than silently dropped — which is the whole
      // point of a queue.
      if (response.ok) {
        await cache.delete(request);
      } else {
        console.warn('[sw] queued change rejected:', response.status, url);
      }
    } catch (error) {
      // Still offline: leave it queued and let the next sync pick it up.
      console.log('[sw] queued change still offline:', request.url);
    }
  }
}

// Push notification handling
self.addEventListener('push', (event) => {
  if (!event.data) return;
  
  const data = event.data.json();
  const options = {
    body: data.body || 'New notification',
    icon: '/logo.png',
    badge: '/logo_trans.png',
    vibrate: [200, 100, 200],
    data: data.data || {},
    actions: data.actions || [
      { action: 'view', title: 'View' },
      { action: 'dismiss', title: 'Dismiss' },
    ],
    requireInteraction: true,
  };
  
  event.waitUntil(
    self.registration.showNotification(data.title || 'AgroCore', options)
  );
});

self.addEventListener('notificationclick', (event) => {
  event.notification.close();
  
  if (event.action === 'view') {
    event.waitUntil(
      clients.matchAll({ type: 'window', includeUncontrolled: true }).then((clientList) => {
        for (const client of clientList) {
          if (client.url.includes('/worker/tasks') && 'focus' in client) {
            return client.focus();
          }
        }
        return clients.openWindow('/worker/tasks');
      })
    );
  }
});

// Periodic background sync (if supported).
//
// Uses `flushQueue`, the function above. The previous version called
// `syncWorkerTasks`, which read the offline cache for POST/PUT requests — but the
// Cache API does not store POST requests, so that loop could never have found
// anything to replay.
self.addEventListener('periodicsync', (event) => {
  if (event.tag === 'agrocore-periodic-sync') {
    event.waitUntil(flushQueue());
  }
});