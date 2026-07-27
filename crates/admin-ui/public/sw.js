// AgroCore Admin PWA Service Worker
// Provides offline caching, background sync, and push notifications

const CACHE_NAME = 'agrocore-admin-v1';
const STATIC_CACHE_NAME = 'agrocore-static-v1';
const DYNAMIC_CACHE_NAME = 'agrocore-dynamic-v1';
const OFFLINE_CACHE_NAME = 'agrocore-offline-v1';

// Files to cache for offline use
const STATIC_ASSETS = [
  '/',
  '/index.html',
  '/manifest.json',
  '/styles/tailwind.css',
  '/styles/daisyui.css',
  // API endpoints that should be cached for offline
  '/api/v1/orders/my-tasks',
  '/api/v1/workforce/tasks',
];

// Install event - cache static assets
self.addEventListener('install', (event) => {
  event.waitUntil(
    Promise.all([
      caches.open(STATIC_CACHE_NAME).then((cache) => {
        return cache.addAll(STATIC_ASSETS.map(url => new Request(url, { credentials: 'include' })));
      }),
      caches.open(OFFLINE_CACHE_NAME).then((cache) => {
        return cache.addAll([
          '/offline.html',
          '/worker-tasks-offline.html',
        ]);
      }),
    ])
  );
  self.skipWaiting();
});

// Activate event - clean up old caches
self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames
          .filter((name) => name !== STATIC_CACHE_NAME && name !== DYNAMIC_CACHE_NAME && name !== OFFLINE_CACHE_NAME)
          .map((name) => caches.delete(name))
      );
    })
  );
  self.clients.claim();
});

// Fetch event - network first for API, cache first for static
self.addEventListener('fetch', (event) => {
  const { request } = event;
  const url = new URL(request.url);

  // Skip non-GET requests
  if (request.method !== 'GET') {
    return;
  }

  // API requests - network first with offline fallback
  if (url.pathname.startsWith('/api/')) {
    event.respondWith(networkFirstWithCache(request));
    return;
  }

  // Static assets - cache first
  event.respondWith(cacheFirst(request));
});

// Network first strategy with cache fallback
async function networkFirstWithCache(request) {
  try {
    const response = await fetch(request);
    if (response.ok) {
      const cache = await caches.open(DYNAMIC_CACHE_NAME);
      cache.put(request, response.clone());
    }
    return response;
  } catch (error) {
    const cachedResponse = await caches.match(request);
    if (cachedResponse) {
      return cachedResponse;
    }
    // Return offline page for API failures
    if (request.url.includes('/worker/tasks') || request.url.includes('/my-tasks')) {
      return caches.match('/worker-tasks-offline.html');
    }
    return new Response(JSON.stringify({ error: 'Offline', offline: true }), {
      status: 503,
      headers: { 'Content-Type': 'application/json' },
    });
  }
}

// Cache first strategy
async function cacheFirst(request) {
  const cachedResponse = await caches.match(request);
  if (cachedResponse) {
    return cachedResponse;
  }
  try {
    const response = await fetch(request);
    if (response.ok) {
      const cache = await caches.open(STATIC_CACHE_NAME);
      cache.put(request, response.clone());
    }
    return response;
  } catch (error) {
    // Return offline page for navigation requests
    if (request.mode === 'navigate') {
      return caches.match('/offline.html');
    }
    return new Response('Offline', { status: 503 });
  }
}

// Background sync for task updates
self.addEventListener('sync', (event) => {
  if (event.tag === 'sync-worker-tasks') {
    event.waitUntil(syncWorkerTasks());
  }
  if (event.tag === 'sync-task-status') {
    event.waitUntil(syncTaskStatus());
  }
});

async function syncWorkerTasks() {
  try {
    const cache = await caches.open(OFFLINE_CACHE_NAME);
    const requests = await cache.keys();
    
    for (const request of requests) {
      if (request.url.includes('/api/v1/workforce/tasks/') && 
          (request.method === 'POST' || request.method === 'PUT' || request.method === 'PATCH')) {
        try {
          await fetch(request);
          await cache.delete(request);
        } catch (error) {
          console.log('Sync failed for:', request.url);
        }
      }
    }
  } catch (error) {
    console.error('Background sync failed:', error);
  }
}

async function syncTaskStatus() {
  try {
    // Get pending task status updates from IndexedDB
    const db = await openDB();
    const tx = db.transaction('pendingStatusUpdates', 'readwrite');
    const store = tx.objectStore('pendingStatusUpdates');
    const updates = await store.getAll();
    
    for (const update of updates) {
      try {
        await fetch(update.url, {
          method: 'PUT',
          headers: update.headers,
          body: JSON.stringify(update.body),
          credentials: 'include',
        });
        await store.delete(update.id);
      } catch (error) {
        console.log('Failed to sync status update:', error);
      }
    }
  } catch (error) {
    console.error('Task status sync failed:', error);
  }
}

// IndexedDB helper for offline storage
function openDB() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open('AgroCoreOfflineDB', 1);
    request.onerror = () => reject(request.error);
    request.onsuccess = () => resolve(request.result);
    request.onupgradeneeded = (event) => {
      const db = event.target.result;
      if (!db.objectStoreNames.contains('pendingStatusUpdates')) {
        db.createObjectStore('pendingStatusUpdates', { keyPath: 'id', autoIncrement: true });
      }
      if (!db.objectStoreNames.contains('cachedTasks')) {
        db.createObjectStore('cachedTasks', { keyPath: 'id' });
      }
    };
  });
}

// Push notification handling
self.addEventListener('push', (event) => {
  if (!event.data) return;
  
  const data = event.data.json();
  const options = {
    body: data.body || 'New notification',
    icon: '/icons/icon-192x192.png',
    badge: '/icons/badge-72x72.png',
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

// Periodic background sync (if supported)
self.addEventListener('periodicsync', (event) => {
  if (event.tag === 'periodic-task-sync') {
    event.waitUntil(syncWorkerTasks());
  }
});