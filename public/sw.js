// Ike Sales service worker.
//
// Its only job is to make the page installable and let the shell open with no
// signal. It never answers for data: every cross-origin request (the JSON on
// raw.githubusercontent.com), every non-GET and — since the Worker serves this
// page from its own origin (D27) — every call to the Worker's own areas goes
// straight to the network untouched, so a number on screen is never stale and
// a signed-in answer is never kept on the phone. The page itself is
// network-first, so a new deploy shows on the next open.
//
// v2: the first cache on the Worker's origin; v1 belonged to the retired
// github.io address.
var CACHE = 'ike-sales-shell-v2';
var NETWORK_ONLY = ['/auth/', '/expenses/', '/quotations/'];
var SHELL = ['./', 'manifest.webmanifest', 'icons/icon-192.png', 'icons/icon-512.png'];

self.addEventListener('install', function (e) {
  e.waitUntil(caches.open(CACHE).then(function (c) { return c.addAll(SHELL); }));
  self.skipWaiting();
});

self.addEventListener('activate', function (e) {
  e.waitUntil(
    caches.keys()
      .then(function (keys) {
        return Promise.all(keys.filter(function (k) { return k !== CACHE; })
          .map(function (k) { return caches.delete(k); }));
      })
      .then(function () { return self.clients.claim(); })
  );
});

self.addEventListener('fetch', function (e) {
  var req = e.request;
  if (req.method !== 'GET') return;
  var url = new URL(req.url);
  if (url.origin !== self.location.origin) return;
  if (NETWORK_ONLY.some(function (p) { return url.pathname.indexOf(p) === 0; })) return;
  e.respondWith(
    fetch(req)
      .then(function (res) {
        if (res.ok) {
          var copy = res.clone();
          caches.open(CACHE).then(function (c) { c.put(req, copy); });
        }
        return res;
      })
      .catch(function () {
        return caches.match(req).then(function (hit) { return hit || caches.match('./'); });
      })
  );
});
