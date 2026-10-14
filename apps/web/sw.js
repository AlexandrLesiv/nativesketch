// Runtime is wholly local after first successful cache installation.
const CACHE = "native-sketch-__CACHE_VERSION__";
const ASSETS = [
    "./",
    "./index.html",
    "./style.css",
    "./main.js",
    "./engine.js",
    "./sketch_wasm.wasm",
];
self.addEventListener("install", (e) =>
    e.waitUntil(caches.open(CACHE).then((c) => c.addAll(ASSETS))),
);
self.addEventListener("activate", (e) =>
    e.waitUntil(
        caches
            .keys()
            .then((keys) =>
                Promise.all(
                    keys
                        .filter(
                            (k) =>
                                k.startsWith("native-sketch-") && k !== CACHE,
                        )
                        .map((k) => caches.delete(k)),
                ),
            ),
    ),
);
self.addEventListener("fetch", (e) => {
    if (
        e.request.method === "GET" &&
        new URL(e.request.url).origin === self.location.origin
    )
        e.respondWith(
            caches.match(e.request).then((c) => c || fetch(e.request)),
        );
});
