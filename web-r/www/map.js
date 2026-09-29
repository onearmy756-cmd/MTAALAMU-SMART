/* ============================================================
 * map.js — MTAALAMU SMART: ramani halisi (Tanzania)
 *
 *   Vyanzo vya picha (tiles):
 *     - Google Satellite / Hybrid      (mt1.google.com)
 *     - NASA EOSDIS GIBS MODIS Terra   (gibs.earthdata.nasa.gov)
 *     - Esri World Imagery             (arcgisonline.com)
 *     - OpenStreetMap                  (tile.openstreetmap.org)
 *   Njia (routing): OSRM  router.project-osrm.org
 *   Mipaka ya mikoa: geoBoundaries TZA ADM1 (cc-by / ODbL data)
 *
 *   Data yote inasomwa kwenye <script id="tz-geo"> kilichojazwa na R
 *   kutoka web-r/data/geo.json + data/tz_regions.geojson  (JSON = chanzo)
 * ============================================================ */
(function () {
  "use strict";

  var MT = {
    map: null, base: null, extra: null, baseId: null,
    data: null, labels: {}, bound: null, chor: null, routes: [],
    ovLayers: {}, ovOn: {}, pendingOv: {}, boundState: "none",
    routeInfo: null, pending: 0
  };

  function el(id) { return document.getElementById(id); }

  function esc(s) {
    return String(s === undefined || s === null ? "" : s)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function Lb(k, fallback) {
    var v = MT.labels[k];
    return v === undefined || v === null || v === "" ? (fallback || k) : v;
  }

  /* ---------- tarikhito ya NASA GIBS (siku moja nyuma + fallback) ---------- */
  function dateStr(d) {
    var m = ("0" + (d.getMonth() + 1)).slice(-2), day = ("0" + d.getDate()).slice(-2);
    return d.getFullYear() + "-" + m + "-" + day;
  }

  function makeBase(cfg) {
    var opts = {
      attribution: cfg.attr || "",
      maxZoom: cfg.maxZoom || 19,
      minZoom: 4
    };
    if (cfg.dated) opts.date = dateStr(new Date(Date.now() - 86400000));
    var lyr = L.tileLayer(cfg.tiles, opts);
    if (cfg.dated) {
      var tries = 0;
      lyr.on("tileerror", function () {
        if (tries >= 6) return;           /* GIBS bado haijachapisha siku hii */
        tries += 1;
        lyr.options.date = dateStr(new Date(Date.now() - 86400000 * (tries + 1)));
        lyr.redraw();
      });
    }
    return lyr;
  }

  function setBase(id) {
    if (!MT.map || !MT.data) return;
    var cfg = null;
    MT.data.basemaps.forEach(function (b) { if (b.id === id) cfg = b; });
    if (!cfg) return;

    if (MT.base) { MT.map.removeLayer(MT.base); MT.base = null; }
    if (MT.extra) { MT.map.removeLayer(MT.extra); MT.extra = null; }

    if (cfg.overlay && MT.baseId && MT.base) {
      /* kiunzi cha lebo: kinaongezwa juu ya msingi uliopo */
      MT.extra = L.tileLayer(cfg.tiles, { attribution: cfg.attr || "", maxZoom: cfg.maxZoom || 19 });
      MT.extra.addTo(MT.map);
      return;
    }

    MT.base = makeBase(cfg);
    MT.base.addTo(MT.map);
    MT.baseId = id;

    /* Google + Esri hazina lebo za majina — ongeza OSM juu kama mbadala wa hybrid */
    if (id === "hybrid" || id === "google") {
      /* hakuna kitu: Google Satellite ni kioo chenye lebo kwenye hybrid */
    }
  }

  /* ---------- rangi ya choropleth (kazi kwa mkoa) ---------- */
  function statsOf(name) {
    var r = MT.data.regions && MT.data.regions[name];
    return r || null;
  }

  function jobRange() {
    var min = Infinity, max = -Infinity;
    Object.keys(MT.data.regions || {}).forEach(function (k) {
      var j = MT.data.regions[k].jobs;
      if (j < min) min = j;
      if (j > max) max = j;
    });
    if (!isFinite(min)) { min = 0; max = 1; }
    return { min: min, max: max };
  }

  function jobColor(v, r) {
    var t = (r.max === r.min) ? 0.5 : (v - r.min) / (r.max - r.min);
    /* njano (kidogo) → nyekundu (kingi) */
    var stops = [[254, 240, 138], [254, 178, 76], [253, 141, 60], [240, 59, 32], [165, 15, 21]];
    var x = Math.max(0, Math.min(0.9999, t)) * (stops.length - 1);
    var i = Math.floor(x), f = x - i;
    var a = stops[i], b = stops[Math.min(stops.length - 1, i + 1)];
    return "rgb(" +
      Math.round(a[0] + (b[0] - a[0]) * f) + "," +
      Math.round(a[1] + (b[1] - a[1]) * f) + "," +
      Math.round(a[2] + (b[2] - a[2]) * f) + ")";
  }

  function regionPopup(name) {
    var s = statsOf(name);
    var h = '<div class="mt-pop"><b>' + esc(name) + "</b>";
    if (s) {
      h += '<div class="mt-pop-rows">' +
        "<div><span>" + esc(Lb("jobs", "Jobs")) + "</span><b>" + s.jobs + "</b></div>" +
        "<div><span>" + esc(Lb("techs", "Technicians")) + "</span><b>" + s.techs + "</b></div>" +
        "<div><span>" + esc(Lb("customers", "Customers")) + "</span><b>" + s.customers + "</b></div>" +
        "</div>";
    }
    return h + "</div>";
  }

  function styleBoundary() {
    return { color: "#7dd3fc", weight: 1.1, opacity: 0.75, fill: false };
  }

  function styleChoropleth() {
    var r = jobRange();
    return function (feat) {
      var name = feat.properties && feat.properties.shapeName;
      var s = statsOf(name);
      var v = s ? s.jobs : r.min;
      return {
        fillColor: jobColor(v, r),
        weight: 1,
        color: "#0b1220",
        opacity: 0.6,
        fillOpacity: 0.45
      };
    };
  }

  /* ---------- alama (wataalamu / wateja / kazi) ---------- */
  function markerStyle(kind) {
    if (kind === "tech") return { c: "#22c55e", r: 8, ico: "\uD83E\uDD16" };
    if (kind === "customer") return { c: "#38bdf8", r: 7, ico: "\uD83C\uDFE2" };
    return { c: "#f59e0b", r: 8, ico: "\uD83D\uDD27" };
  }

  function statusLabel(st) {
    return Lb("st." + st, st);
  }

  function markerPopup(m) {
    var st = markerStyle(m.kind);
    var h = '<div class="mt-pop mt-pop-' + esc(m.kind) + '">' +
      '<div class="mt-pop-h">' + st.ico + " <b>" + esc(m.name) + "</b></div>" +
      '<div class="mt-pop-id">' + esc(m.id) + " · " + esc(m.trade) + "</div>" +
      '<div class="mt-pop-rows">' +
      "<div><span>" + esc(Lb("place", "Place")) + "</span><b>" + esc(m.place) + "</b></div>" +
      "<div><span>" + esc(Lb("status", "Status")) + '</span><b class="mt-st">' + esc(statusLabel(m.status)) + "</b></div>" +
      "<div><span>" + esc(Lb("jobs", "Jobs")) + "</span><b>" + m.jobs + "</b></div>" +
      (m.rating ? "<div><span>" + esc(Lb("rating", "Rating")) + "</span><b>★ " + m.rating + "</b></div>" : "") +
      "</div>" +
      '<div class="mt-pop-env"><i>' + esc(Lb("env", "Environment")) + ":</i> " + esc(m.env) + "</div>" +
      '<div class="mt-pop-env"><i>' + esc(Lb("tools", "Tools")) + ":</i> " + esc(m.tools) + "</div>" +
      "</div>";
    return h;
  }

  /* ---------- OSRM: njia ---------- */
  function osrmUrl(a, b, full) {
    return "https://router.project-osrm.org/route/v1/driving/" +
      a.lng + "," + a.lat + ";" + b.lng + "," + b.lat +
      "?overview=" + (full ? "full" : "false") + "&geometries=geojson";
  }

  function fetchRoute(a, b) {
    return fetch(osrmUrl(a, b, true))
      .then(function (r) { return r.json(); })
      .then(function (j) {
        if (!j || j.code !== "Ok" || !j.routes || !j.routes.length) throw new Error("no-route");
        return j.routes[0];
      });
  }

  function fmtDist(m) { return m >= 1000 ? (m / 1000).toFixed(1) + " km" : Math.round(m) + " m"; }
  function fmtDur(s) {
    var mn = Math.round(s / 60);
    if (mn < 60) return mn + " min";
    return Math.floor(mn / 60) + " h " + (mn % 60) + " min";
  }

  function drawRouteLine(coords, opts) {
    var latlngs = coords.map(function (c) { return [c[1], c[0]]; });
    return L.polyline(latlngs, opts);
  }

  /* njia kuu (corridors) — zote zinachukuliwa kwa mpangilio */
  function drawCorridors() {
    if (!MT.map || !MT.data || !MT.data.routes) return;
    if (MT.routes.length) { MT.routes.forEach(function (l) { l.addTo(MT.map); }); return; }
    MT.data.routes.forEach(function (r, i) {
      MT.pending += 1;
      setTimeout(function () {
        fetchRoute(r.from, r.to).then(function (rt) {
          var line = drawRouteLine(rt.geometry.coordinates, {
            color: "#f472b6", weight: 3, opacity: 0.85, dashArray: "8 6"
          });
          line.bindPopup("<b>" + esc(r.name) + "</b><br/>" + fmtDist(rt.distance) + " · " + fmtDur(rt.duration));
          MT.routes.push(line);
          if (MT.ovOn.corridors) line.addTo(MT.map);
        }).catch(function () { /* OSRM haikubali — kuacha kimya */ })
          .then(function () { MT.pending -= 1; });
      }, i * 260);
    });
  }

  function clearCorridors() {
    MT.routes.forEach(function (l) { if (MT.map && MT.map.hasLayer(l)) MT.map.removeLayer(l); });
  }

  /* njia ya mteja (kutoka → kwenye) */
  function drawUserRoute() {
    var fromSel = el("map_from"), toSel = el("map_to"), info = el("route_info");
    if (!fromSel || !toSel || !MT.data) return;
    var a = null, b = null;
    MT.data.cities.forEach(function (c) {
      if (c.name === fromSel.value) a = c;
      if (c.name === toSel.value) b = c;
    });
    if (!a || !b) return;
    if (MT.userRoute && MT.map) { MT.map.removeLayer(MT.userRoute); MT.userRoute = null; }
    if (info) info.innerHTML = '<span class="mt-busy">' + esc(Lb("route_wait", "Calculating…")) + "</span>";
    fetchRoute(a, b).then(function (rt) {
      var line = drawRouteLine(rt.geometry.coordinates, {
        color: "#a78bfa", weight: 5, opacity: 0.95
      });
      line.bindPopup("<b>" + esc(a.name) + " → " + esc(b.name) + "</b><br/>" +
        fmtDist(rt.distance) + " · " + fmtDur(rt.duration));
      MT.userRoute = line;
      if (MT.map) line.addTo(MT.map);
      if (info) {
        info.innerHTML = "<b>" + esc(a.name) + " → " + esc(b.name) + "</b> · " +
          fmtDist(rt.distance) + " · " + fmtDur(rt.duration) +
          ' <span class="mt-src">OSRM</span>';
      }
      if (MT.map && MT.userRoute) MT.map.fitBounds(MT.userRoute.getBounds(), { padding: [30, 30] });
    }).catch(function () {
      if (info) info.innerHTML = '<span class="mt-err">' + esc(Lb("route_err", "Routing unavailable")) + "</span>";
    });
  }

  /* ---------- mipaka ya mikoa: kupakua GeoJSON kama faili ---------- */
  function fetchBoundaries() {
    if (MT.boundState === "loading" || MT.boundState === "error") return;
    MT.boundState = "loading";
    var url = (MT.data && MT.data.boundaries_url) ? MT.data.boundaries_url : "tz_regions.geojson";
    fetch(url, { cache: "force-cache" })
      .then(function (r) { if (!r.ok) throw new Error("http " + r.status); return r.json(); })
      .then(function (g) {
        MT.bound = g;
        MT.boundState = "ready";
        ["boundaries", "choropleth"].forEach(function (id) {
          if (MT.pendingOv[id]) toggleOverlay(id, true);
        });
      })
      .catch(function () { MT.boundState = "error"; });
  }

  /* ---------- tabaka (overlays) ---------- */
  function toggleOverlay(id, on) {
    if (!MT.map) return;
    MT.ovOn[id] = on;
    var lyr = MT.ovLayers[id];

    if (id === "boundaries" || id === "choropleth") {
      MT.pendingOv[id] = !!on;
      if (on) {
        if (MT.bound) {
          var lyr2 = MT.ovLayers[id];
          if (!lyr2) lyr2 = MT.ovLayers[id] = (id === "boundaries" ? MT.makeBoundaries() : MT.makeChoropleth());
          lyr2.addTo(MT.map);
        } else {
          fetchBoundaries();          /* GeoJSON bado haijafika — itaongezwa baada ya kupakua */
        }
      } else if (MT.ovLayers[id]) MT.map.removeLayer(MT.ovLayers[id]);
      return;
    }
    if (id === "techs" || id === "customers" || id === "jobs") {
      if (on) {
        if (!lyr) {
          lyr = L.layerGroup();
          MT.data.markers.forEach(function (m) {
            if (m.kind !== (id === "techs" ? "tech" : id === "customers" ? "customer" : "job")) return;
            var st = markerStyle(m.kind);
            var mk = L.circleMarker([m.lat, m.lng], {
              radius: st.r, color: "#0b1220", weight: 1.5,
              fillColor: st.c, fillOpacity: 0.95
            });
            mk.bindPopup(markerPopup(m));
            lyr.addLayer(mk);
          });
          MT.ovLayers[id] = lyr;
        }
        lyr.addTo(MT.map);
      } else if (lyr) MT.map.removeLayer(lyr);
      return;
    }
    if (id === "corridors") {
      if (on) drawCorridors(); else clearCorridors();
      return;
    }
    if (id === "labels") {
      var cfg = null;
      MT.data.basemaps.forEach(function (b) { if (b.id === "esri_labels") cfg = b; });
      if (!cfg) return;
      if (on) {
        if (!MT.extra) {
          MT.extra = L.tileLayer(cfg.tiles, { attribution: cfg.attr || "", maxZoom: cfg.maxZoom || 19 });
          MT.extra.addTo(MT.map);
        }
      } else if (MT.extra) { MT.map.removeLayer(MT.extra); MT.extra = null; }
    }
  }

  MT.makeBoundaries = function () {
    return L.geoJSON(MT.bound, { style: styleBoundary, onEachFeature: function (f, l) {
      l.bindPopup(regionPopup(f.properties.shapeName));
    }});
  };

  MT.makeChoropleth = function () {
    return L.geoJSON(MT.bound, { style: styleChoropleth(), onEachFeature: function (f, l) {
      l.bindPopup(regionPopup(f.properties.shapeName));
    }});
  };

  /* ---------- udhibiti (controls) ---------- */
  function bindControls() {
    var base = el("map_base");
    if (base && !base.__b) {
      base.__b = true;
      base.addEventListener("change", function () { setBase(base.value); });
    }
    var box = el("map_ov");
    if (box && !box.__b) {
      box.__b = true;
      box.addEventListener("change", function (e) {
        if (e.target && e.target.type === "checkbox") toggleOverlay(e.target.value, e.target.checked);
      });
    }
    var btn = el("map_route_btn");
    if (btn && !btn.__b) { btn.__b = true; btn.addEventListener("click", drawUserRoute); }
    ["map_from", "map_to"].forEach(function (id) {
      var s = el(id);
      if (s && !s.__b) { s.__b = true; s.addEventListener("change", drawUserRoute); }
    });
  }

  /* ---------- kuanzisha ---------- */
  function init() {
    if (typeof L === "undefined") return;
    var host = el("tzmap");
    if (!host) return;

    var payload = readPayload();
    if (!payload) return;

    MT.data = payload.data || {};
    MT.labels = payload.labels || {};
    MT.bound = null;
    MT.boundState = "none";
    MT.pendingOv = {};

    if (MT.map) { MT.map.remove(); MT.map = null; MT.ovLayers = {}; MT.routes = []; MT.userRoute = null; }

    var c = MT.data.center || { lat: -6.37, lng: 34.89, zoom: 6 };
    host.innerHTML = "";
    MT.map = L.map(host, { zoomControl: true, scrollWheelZoom: true, minZoom: 4, maxZoom: 18 })
      .setView([c.lat, c.lng], c.zoom);

    var first = (MT.data.basemaps && MT.data.basemaps[0]) ? MT.data.basemaps[0].id : "google";
    setBase(first);
    var sel = el("map_base");
    if (sel) sel.value = first;

    /* tabaka zilizowashwa tayari kutoka geo.json */
    (MT.data.overlays || []).forEach(function (o) {
      var box = el("map_ov");
      var cb = box ? box.querySelector('input[value="' + o.id + '"]') : null;
      var wantOn = cb ? cb.checked : !!o.on;
      if (wantOn) toggleOverlay(o.id, true);
    });

    bindControls();

    MT.map.on("click", function () { /* kubonyeza baraza la maelezo */ });

    setTimeout(function () { if (MT.map) MT.map.invalidateSize(); }, 120);
  }

  function readPayload() {
    var n = el("tz-geo");
    if (!n) return null;
    try { return JSON.parse(n.textContent); } catch (e) { return null; }
  }

  function resize() { if (MT.map) { try { MT.map.invalidateSize(); } catch (e) {} } }

  /* kazi ya tab — inaitwa na vitufe vya tabs kwenye views.R */
  window.mtTab = function (id) {
    if (window.Shiny && Shiny.setInputValue) Shiny.setInputValue("tab", id, { priority: "event" });
    if (id === "map") setTimeout(resize, 90);
  };

  window.MTMap = { init: init, resize: resize, setBase: setBase, toggle: toggleOverlay };

  if (document.readyState === "complete" || document.readyState === "interactive") {
    setTimeout(function () { if (el("tzmap")) init(); }, 0);
  } else {
    document.addEventListener("DOMContentLoaded", function () { if (el("tzmap")) init(); });
  }
})();
