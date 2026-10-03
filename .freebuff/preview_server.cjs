#!/usr/bin/env node
/* MTAALAMU SMART — preview server (node, stdlib pekee)
   + directory listing kama JSON (studio inasoma upstream/ trees) */
const http = require("http");
const fs = require("fs");
const path = require("path");
const ROOT = path.resolve(__dirname, "..");
const PORT = parseInt(process.env.PORT || "8000", 10);
const MIME = {
  ".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8", ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8", ".geojson": "application/json; charset=utf-8",
  ".png": "image/png", ".jpg": "image/jpeg", ".jpeg": "image/jpeg", ".gif": "image/gif",
  ".svg": "image/svg+xml", ".ico": "image/x-icon", ".txt": "text/plain; charset=utf-8",
  ".woff": "font/woff", ".woff2": "font/woff2", ".map": "application/json"
};
http.createServer((req, res) => {
  const u = new URL(req.url, "http://localhost");
  let p = decodeURIComponent(u.pathname);
  if (p === "/" || p === "/index.html") {
    res.writeHead(302, { Location: "/hermes-agent/web-r/www/index.html" });
    res.end();
    return;
  }
  const file = path.normalize(path.join(ROOT, p));
  if (!file.startsWith(ROOT)) { res.writeHead(403); res.end(); return; }
  fs.stat(file, (serr, st) => {
    if (!serr && st.isDirectory()) {
      /* directory listing kama JSON — studio inasoma upstream/ trees */
      fs.readdir(file, { withFileTypes: true }, (derr, ents) => {
        if (derr) { res.writeHead(404, { "Content-Type": "text/plain" }); res.end("404 " + p); return; }
        res.writeHead(200, { "Content-Type": "application/json; charset=utf-8" });
        res.end(JSON.stringify({ dir: p, entries: ents.map((e) => ({ name: e.name, dir: e.isDirectory() })) }));
      });
      return;
    }
    fs.readFile(file, (err, buf) => {
      if (err) { res.writeHead(404, { "Content-Type": "text/plain" }); res.end("404 " + p); return; }
      res.writeHead(200, { "Content-Type": MIME[path.extname(file).toLowerCase()] || "application/octet-stream" });
      res.end(buf);
    });
  });
}).listen(PORT, "0.0.0.0", () => console.log("MTAALAMU preview (node): http://0.0.0.0:" + PORT + "/hermes-agent/web-r/www/index.html"));
