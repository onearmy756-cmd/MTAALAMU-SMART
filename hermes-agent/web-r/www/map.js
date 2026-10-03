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
    routeInfo: null, pending: 0,
    /* mpya: GPS / search / weather / tracking / hazards / cities */
    me: null, meAccuracy: null, track: null, trackLine: null, watchId: null,
    searchMarker: null, weatherPanel: null, citiesLayer: null, hazardsLayer: null,
    altRoutes: [], lastRoute: null,
    /* live: magari + watu + sauti + picha halisi */
    liveLayer: null, liveTimer: null, liveObjs: [], voiceOn: false,
    photoLayer: null, autoLocateDone: false
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

  /* njia ya mteja (kutoka → kwenye) — miji + MIKOA YOTE */
  function placeByName(name) {
    var out = null;
    (MT.data.cities || []).forEach(function (c) { if (c.name === name) out = c; });
    if (out) return out;
    if (MT.data.regions && MT.data.regions[name]) {
      var r = MT.data.regions[name];
      return { name: name, lat: r.lat, lng: r.lng };
    }
    return null;
  }
  function drawUserRoute() {
    var fromSel = el("map_from"), toSel = el("map_to"), info = el("route_info");
    if (!fromSel || !toSel || !MT.data) return;
    var a = placeByName(fromSel.value);
    var b = placeByName(toSel.value);
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
      speakSw("Njia ya " + a.name + " hadi " + b.name + ", kilomita " + Math.round(rt.distance / 100) / 10 + ".");
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

    if (id === "live") { if (on) startLive(); else stopLive(); return; }
    if (id === "photos") { /* picha: zinawashwa kwa kitufe/photosAt — hii ni alama tu */
      var panel = el("map_photos");
      if (panel) panel.style.display = on ? "block" : "none";
      return;
    }
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
    if (id === "cities") {
      if (on) {
        if (!MT.citiesLayer) {
          MT.citiesLayer = L.layerGroup();
          (MT.data.cities || []).forEach(function (c) {
            var mk = L.circleMarker([c.lat, c.lng], {
              radius: 4, color: "#0b1220", weight: 1.5,
              fillColor: "#e2e8f0", fillOpacity: 0.9
            });
            mk.bindTooltip(c.name, { permanent: true, direction: "top", className: "mt-city-tip" });
            mk.bindPopup("<b>" + esc(c.name) + "</b><br/><button onclick=\"MTMap.routeTo(" + c.lat + "," + c.lng + ",\'" + esc(c.name) + "\')\" style=\"background:#00e5ff;border:0;padding:4px 10px;border-radius:4px;cursor:pointer;font-weight:700\">Nenda hapa</button>");
            MT.citiesLayer.addLayer(mk);
          });
        }
        MT.citiesLayer.addTo(MT.map);
      } else if (MT.citiesLayer) MT.map.removeLayer(MT.citiesLayer);
      return;
    }
    if (id === "hazards") {
      if (on) {
        if (!MT.hazardsLayer) {
          MT.hazardsLayer = L.layerGroup();
          (MT.data.hazards || []).forEach(function (h) {
            var col = h.severity === "high" ? "#ff1744" : h.severity === "medium" ? "#ff9100" : "#ffc107";
            var ico = h.type === "mafuriko" ? "\uD83C\uDF0A" : h.type === "umeme" ? "\u26A1" :
                      h.type === "uhalifu" ? "\uD83D\uDEA8" : "\u26A0";
            var mk = L.marker([h.lat, h.lng], {
              icon: L.divIcon({ className: "mt-hz", html: '<div style="font-size:22px;filter:drop-shadow(0 0 4px ' + col + ')">' + ico + "</div>", iconSize: [26, 26], iconAnchor: [13, 13] })
            });
            mk.bindPopup('<div class="mt-pop"><b style="color:' + col + '">' + ico + " " + esc(h.name) + "</b>" +
              '<div class="mt-pop-rows">' +
              "<div><span>Aina</span><b>" + esc(h.type) + "</b></div>" +
              "<div><span>Mahali</span><b>" + esc(h.place) + "</b></div>" +
              "<div><span>Ukali</span><b>" + esc(h.severity) + "</b></div>" +
              "</div><i>" + esc(h.note_sw || "") + "</i></div>");
            MT.hazardsLayer.addLayer(mk);
          });
        }
        MT.hazardsLayer.addTo(MT.map);
      } else if (MT.hazardsLayer) MT.map.removeLayer(MT.hazardsLayer);
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
    // NEW buttons + search + keyboard Enter
    var loc = el("map_locate_btn");
    if (loc && !loc.__b) { loc.__b = true; loc.addEventListener("click", locateMe); }
    var vox = el("map_voice_btn");
    if (vox && !vox.__b) { vox.__b = true; vox.addEventListener("click", toggleVoice); }
    var lv = el("map_live_btn");
    if (lv && !lv.__b) { lv.__b = true; lv.addEventListener("click", toggleLive); }
    var ph = el("map_photo_btn");
    if (ph && !ph.__b) {
      ph.__b = true;
      ph.addEventListener("click", function () {
        var c = MT.map.getCenter();
        photosAt(c.lat, c.lng, "eneo la katikati ya ramani");
      });
    }
    var near = el("map_near_btn");
    if (near && !near.__b) {
      near.__b = true;
      near.addEventListener("click", function () {
        if (MT.me && MT.me.getLatLng) {
          var p = MT.me.getLatLng();
          nearestTech(p.lat, p.lng);
        } else locateMe();
      });
    }
    var full = el("map_full_btn");
    if (full && !full.__b) { full.__b = true; full.addEventListener("click", fullscreen); }
    var sb = el("map_search_btn");
    if (sb && !sb.__b) { sb.__b = true; sb.addEventListener("click", searchPlace); }
    var si = el("map_search");
    if (si && !si.__b) {
      si.__b = true;
      si.addEventListener("keydown", function (e) { if (e.key === "Enter") { e.preventDefault(); searchPlace(); } });
    }
    // double-click map = tracking on/off
    if (MT.map && !MT.map.__track) {
      MT.map.__track = true;
      MT.map.on("dblclick", function () { toggleTrack(); });
    }
  }

  /* ============ FEATURES MPYA: GPS · SEARCH · WEATHER · TRACKING ============ */

  function setInfo(html) {
    var info = el("route_info");
    if (info) info.innerHTML = html;
  }

  /* --- GPS: niko wapi --- */
  function locateMe() {
    if (!navigator.geolocation) { setInfo('<span class="mt-err">GPS haipatikani kwenye kifaa hiki</span>'); return; }
    setInfo('<span class="mt-busy">\uD83D\uDCCD Inatafuta eneo lako…</span>');
    navigator.geolocation.getCurrentPosition(function (pos) {
      var lat = pos.coords.latitude, lng = pos.coords.longitude;
      showMe(lat, lng, pos.coords.accuracy);
      MT.map.setView([lat, lng], 14);
      setInfo('\u2705 Eneo lako limepatikana (±' + Math.round(pos.coords.accuracy) + " m)");
      speakSw("Eneo lako limepatikana.");
      nearestTech(lat, lng);
      weatherAt(lat, lng);
      photosAt(lat, lng, "eneo lako");
    }, function (err) {
      setInfo('<span class="mt-err">GPS imeshindikana: ' + esc(err.message) + "</span>");
      speakSw("GPS imeshindikana.");
    }, { enableHighAccuracy: true, timeout: 10000 });
  }

  /* --- Live tracking (njia yako inachorwa mtiririko) --- */
  function toggleTrack() {
    if (MT.watchId) {
      navigator.geolocation.clearWatch(MT.watchId);
      MT.watchId = null;
      setInfo("\u23f9 Tracking imesimama");
      return;
    }
    if (!navigator.geolocation) return;
    MT.track = [];
    MT.watchId = navigator.geolocation.watchPosition(function (pos) {
      var ll = [pos.coords.latitude, pos.coords.longitude];
      MT.track.push(ll);
      if (!MT.trackLine) {
        MT.trackLine = L.polyline(MT.track, { color: "#76ff03", weight: 4, opacity: 0.9, dashArray: "6 4" }).addTo(MT.map);
      } else MT.trackLine.setLatLngs(MT.track);
      if (MT.me) MT.me.setLatLng(ll);
    }, null, { enableHighAccuracy: true });
    setInfo('\uD83D\uDFE2 Tracking IMEWASHWA — njia yako inachorwa (bonyeza tena ku-simama)');
  }

  /* --- Hali ya hewa (Open-Meteo: bure, hakuna key) --- */
  function weatherAt(lat, lng) {
    setInfo('<span class="mt-busy">\u2601 Inapakua hali ya hewa…</span>');
    fetch("https://api.open-meteo.com/v1/forecast?latitude=" + lat + "&longitude=" + lng +
          "&current=temperature_2m,precipitation,weather_code,wind_speed_10m")
      .then(function (r) { return r.json(); })
      .then(function (w) {
        var c = w.current || {};
        var codes = { 0: "\u2600 Wazi", 1: "\ud83c\udf24 Kidogo wazi", 2: "\u2601 Mawingu", 3: "\u2601\u2601 Mawingu mengi",
          45: "\ud83c\udf2b Ukimu", 51: "\ud83c\udf27 Manyunyu", 61: "\ud83c\udf27 Mvua", 63: "\ud83c\udf27\ud83c\udf27 Mvua nyingi",
          65: "\ud83c\udf27 Mvua kubwa", 80: "\ud83c\udf27\u26a1 Manyunyu", 95: "\u26c8\ufe0f Dhoruba", 96: "\u26c8\ufe0f Dhoruba na mvua ya barafu" };
        var desc = codes[c.weather_code] || ("Kodi " + c.weather_code);
        var risk = (c.precipitation || 0) > 2.5 ? "\u26a0\ufe0f MVUA — hatari ya mafuriko kwenye njia!" :
                   (c.precipitation || 0) > 0.5 ? "Mvua nyepesi — waangalie barabara" : "Hali njema za njia";
        setInfo("\u2601 <b>" + (Math.round(c.temperature_2m * 10) / 10) + "°C</b> · " + desc +
          " · \uD83C\uDF2C " + Math.round(c.wind_speed_10m) + " km/h · \uD83D\uDCA7 " + (c.precipitation || 0) + "mm — " + risk +
          ' <span class="mt-src">Open-Meteo</span>');
      })
      .catch(function () { setInfo('<span class="mt-err">Hali ya hewa haipatikani (offline?)</span>'); });
  }

  /* --- Search mahali (Nominatim / OSM: bure) --- */
  function searchPlace() {
    var box = el("map_search");
    if (!box || !box.value.trim()) return;
    var q = box.value.trim() + ", Tanzania";
    setInfo('<span class="mt-busy">\uD83D\uDD0E Inatafuta "' + esc(box.value.trim()) + '"…</span>');
    fetch("https://nominatim.openstreetmap.org/search?format=json&limit=1&countrycodes=tz&q=" + encodeURIComponent(q))
      .then(function (r) { return r.json(); })
      .then(function (res) {
        if (!res || !res.length) { setInfo('\u274c Mahali hakupatikana'); return; }
        var r0 = res[0];
        var lat = parseFloat(r0.lat), lng = parseFloat(r0.lon);
        if (MT.searchMarker) MT.map.removeLayer(MT.searchMarker);
        MT.searchMarker = L.marker([lat, lng], {
          icon: L.divIcon({ className: "mt-pin", html: '\uD83D\uDCCC', iconSize: [26, 26], iconAnchor: [13, 26] })
        }).bindPopup('<div class="mt-pop"><b>\uD83D\uDCCC ' + esc(r0.display_name.split(",").slice(0, 3).join(", ")) + "</b>" +
          '<div class="mt-pop-rows">' +
          "<div><span>Lat</span><b>" + lat.toFixed(5) + "</b></div>" +
          "<div><span>Lng</span><b>" + lng.toFixed(5) + "</b></div></div>" +
          '<button onclick="MTMap.routeTo(' + lat + "," + lng + ')" style="background:#00e5ff;border:0;padding:4px 10px;border-radius:4px;cursor:pointer;font-weight:700;margin-top:4px">Nenda hapa</button>' +
          '</div>').addTo(MT.map);
        MT.map.setView([lat, lng], 14);
        setInfo('\u2705 ' + esc(r0.display_name));
        weatherAt(lat, lng);
      })
      .catch(function () { setInfo('<span class="mt-err">Utafutaji umeshindikana (offline?)</span>'); });
  }

  /* --- Karibu nami: fundi/customer/job wa karibu (haversine) --- */
  function haversine(a, b) {
    function rad(x) { return x * Math.PI / 180; }
    var dLat = rad(b[0] - a[0]), dLng = rad(b[1] - a[1]);
    var s = Math.sin(dLat / 2) * Math.sin(dLat / 2) +
      Math.cos(rad(a[0])) * Math.cos(rad(b[0])) * Math.sin(dLng / 2) * Math.sin(dLng / 2);
    return 6371000 * 2 * Math.atan2(Math.sqrt(s), Math.sqrt(1 - s));
  }
  function nearestTech(lat, lng) {
    var best = null, bd = Infinity;
    (MT.data.markers || []).forEach(function (m) {
      if (m.kind !== "tech" || m.status === "offline") return;
      var d = haversine([lat, lng], [m.lat, m.lng]);
      if (d < bd) { bd = d; best = m; }
    });
    if (!best) return;
    var note = "\uD83E\uDD16 " + esc(best.name) + " ni " + fmtDist(bd) + " kutoka kwako (" + esc(best.trade) + ", ★" + best.rating + ")";
    setInfo(note + ' — <a href="#" onclick="MTMap.routeTo(' + best.lat + "," + best.lng + ');return false" style="color:#00e5ff">chora njia</a>');
    L.circleMarker([best.lat, best.lng], { radius: 12, color: "#76ff03", weight: 3, fill: false })
      .bindTooltip("\u2705 Karibu nawe: " + best.name, { permanent: false })
      .addTo(MT.map);
  }

  /* --- Nenda hapo (kutoka popup yoyote) + njia mbadala --- */
  function routeTo(lat, lng, name) {
    var dest = { lat: lat, lng: lng, name: name || "Ziara" };
    var start = null;
    if (MT.me && MT.me.getLatLng) {
      var p = MT.me.getLatLng();
      start = { lat: p.lat, lng: p.lng };
    } else if ((MT.data.cities || []).length) {
      start = MT.data.cities[0];
    }
    if (!start) return;
    // njia kuu + mbadala (OSRM alternatives=true)
    MT.altRoutes.forEach(function (l) { if (MT.map.hasLayer(l)) MT.map.removeLayer(l); });
    MT.altRoutes = [];
    fetch("https://router.project-osrm.org/route/v1/driving/" + start.lng + "," + start.lat + ";" + lng + "," + lat + "?overview=full&geometries=geojson&alternatives=true")
      .then(function (r) { return r.json(); })
      .then(function (j) {
        if (!j || j.code !== "Ok" || !j.routes || !j.routes.length) throw new Error("no-route");
        var cols = ["#a78bfa", "#38bdf8"];
        j.routes.forEach(function (rt, i) {
          var line = drawRouteLine(rt.geometry.coordinates, {
            color: cols[i] || "#a78bfa", weight: i === 0 ? 5 : 3,
            opacity: 0.95, dashArray: i === 0 ? null : "10 6"
          });
          line.bindPopup("<b>" + esc(dest.name) + "</b><br/>" +
            (i === 0 ? "Njia kuu: " : "Njia mbadala: ") + fmtDist(rt.distance) + " · " + fmtDur(rt.duration));
          line.addTo(MT.map);
          MT.altRoutes.push(line);
          if (i === 0) {
            setInfo("\uD83D\uDE97 <b>" + esc(dest.name) + "</b> · " + fmtDist(rt.distance) + " · " + fmtDur(rt.duration) +
              (j.routes.length > 1 ? " (njia " + j.routes.length + " zimepatikana)" : "") +
              ' <span class="mt-src">OSRM</span>');
            MT.map.fitBounds(line.getBounds(), { padding: [40, 40] });
          }
        });
      })
      .catch(function () { setInfo('<span class="mt-err">Njia haipatikani (OSRM offline?)</span>'); });
  }

  /* --- Fullscreen --- */
  function fullscreen() {
    var host = el("tzmap");
    if (!host) return;
    if (!document.fullscreenElement) {
      (host.requestFullscreen || host.webkitRequestFullscreen || function () {}).call(host);
    } else {
      (document.exitFullscreen || function () {}).call(document);
    }
    setTimeout(resize, 300);
  }

  /* ============ MIPYA: SAUTI (VOICE) ============
   * Kiswahili kwa Web Speech API (sw-TZ). Kitufe kwenye UI: 'Sauti ON/OFF'.
   * Inasema: hali ya hewa, mafanikio ya GPS, njia, tahadhari.
   */
  function speakSw(text) {
    if (!MT.voiceOn || !window.speechSynthesis || !text) return;
    try {
      speechSynthesis.cancel();
      var u = new SpeechSynthesisUtterance(String(text));
      u.lang = "sw-TZ"; u.rate = 0.95;
      var vs = speechSynthesis.getVoices() || [];
      for (var i = 0; i < vs.length; i++) {
        var l = (vs[i].lang || "").toLowerCase();
        if (l.indexOf("sw") === 0 || l.indexOf("swahili") >= 0) { u.voice = vs[i]; break; }
      }
      speechSynthesis.speak(u);
    } catch (e) { /* ignore */ }
  }

  function toggleVoice() {
    MT.voiceOn = !MT.voiceOn;
    var btn = el("map_voice_btn");
    if (btn) btn.style.borderColor = MT.voiceOn ? "#76ff03" : "";
    if (MT.voiceOn) {
      if (window.speechSynthesis) { try { speechSynthesis.getVoices(); } catch (e) {} }
      speakSw("Sauti imewashwa. Karibu Mtaalamu Smart Ramani.");
      setInfo("\uD83D\uDD0a Sauti IMEWASHWA — taarifa zitasemwa kwa Kiswahili");
    } else {
      try { if (window.speechSynthesis) speechSynthesis.cancel(); } catch (e) {}
      setInfo("\uD83D\uDD07 Sauti IMEZIMWA");
    }
  }

  /* ============ MIPYA: AUTO-LOCATE (maria moja ukurasa unapofunguka) ============ */
  function autoLocate() {
    if (MT.autoLocateDone || !navigator.geolocation || !MT.map) return;
    MT.autoLocateDone = true;
    navigator.geolocation.getCurrentPosition(function (pos) {
      var lat = pos.coords.latitude, lng = pos.coords.longitude;
      showMe(lat, lng, pos.coords.accuracy);
      MT.map.setView([lat, lng], 13);
      setInfo("\uD83D\uDCCD Eneo lako limepatikana kiotomatiki (±" + Math.round(pos.coords.accuracy) + " m)");
      speakSw("Eneo lako limepatikana.");
      nearestTech(lat, lng);
      weatherAt(lat, lng);
    }, function () { /* hakuna ruhusa — kimya */ }, { enableHighAccuracy: true, timeout: 8000 });
  }

  function showMe(lat, lng, accuracy) {
    if (!MT.map) return;
    if (MT.me) MT.map.removeLayer(MT.me);
    if (MT.meAccuracy) MT.map.removeLayer(MT.meAccuracy);
    MT.meAccuracy = L.circle([lat, lng], { radius: accuracy || 40, color: "#00e5ff", weight: 1, fillColor: "#00e5ff", fillOpacity: 0.08 }).addTo(MT.map);
    MT.me = L.circleMarker([lat, lng], { radius: 9, color: "#ffffff", weight: 2.5, fillColor: "#00e5ff", fillOpacity: 1 })
      .bindPopup('<div class="mt-pop"><b>\uD83D\uDCCD Eneo lako</b><div class="mt-pop-rows">' +
        "<div><span>Lat</span><b>" + lat.toFixed(5) + "</b></div>" +
        "<div><span>Lng</span><b>" + lng.toFixed(5) + "</b></div></div></div>").addTo(MT.map);
  }

  /* ============ MIPYA: PICHA HALISI ZA SEHEMU (Wikimedia) ============
   * Bila API key: GeoNames Wikipedia? Hapana — tunatumia Wikipedia REST
   * 'geosearch' (bure, CORS imefunguliwa): picha za karibu na sehemu husika.
   */
  function photosAt(lat, lng, name) {
    setInfo('<span class="mt-busy">\U0001f4f8 Inatafuta picha halisi za sehemu hii…</span>');
    var url = "https://en.wikipedia.org/w/api.php?action=query&format=json&origin=*&generator=geosearch&ggscoord=" +
      lat + "%7C" + lng + "&ggsradius=10000&ggslimit=6&prop=pageimages%7Ccoordinates&piprop=thumbnail&pithumbsize=400";
    fetch(url)
      .then(function (r) { return r.json(); })
      .then(function (j) {
        var pages = (j && j.query && j.query.pages) ? j.query.pages : {};
        var cards = [];
        Object.keys(pages).forEach(function (k) {
          var p = pages[k];
          if (p.thumbnail && p.thumbnail.source) {
            cards.push(
              '<div style="display:inline-block;margin:4px;border:1px solid #00e5ff33;border-radius:8px;overflow:hidden;background:#0a1628;vertical-align:top;width:150px">' +
              '<img src="' + p.thumbnail.source + '" style="width:150px;height:100px;object-fit:cover" loading="lazy"/>' +
              '<div style="font-size:10px;color:#b2ebf2;padding:4px 6px;max-height:44px;overflow:hidden">' + esc(p.title) + '</div>' +
              '<a href="https://en.wikipedia.org/wiki/' + encodeURIComponent(p.title.replace(/ /g, "_")) + '" target="_blank" style="font-size:9px;color:#00e5ff;display:block;padding:0 6px 4px">Wikipedia \u2197</a></div>');
          }
        });
        if (!cards.length) {
          setInfo('\u2139 Hakuna picha za karibu ("' + esc(name || "sehemu") + '") kwenye Wikipedia — jaribu sehemu kuu kama Zanzibar au Kilimanjaro.');
          return;
        }
        var panel = el("map_photos");
        if (panel) {
          panel.innerHTML = '<div style="font-size:11px;color:#00e5ff;margin:2px 0">\U0001f4f8 PICHA HALISI: ' + esc(name || "") + ' (Wikipedia)</div>' + cards.join("");
          panel.style.display = "block";
        }
        setInfo("\u2705 Picha " + cards.length + " za sehemu hii zimeonyeshwa chini ya ramani");
        speakSw("Picha za sehemu hii zimeonyeshwa.");
      })
      .catch(function () { setInfo('<span class="mt-err">Picha hazipatikani (offline?)</span>'); });
  }

  /* ============ MIPYA: LIVE — MAGARI + WATU WANAPOTEMBEA ============
   * Simulasi ya kuhama kwenye njia kuu (corridors) + kazi: magari yanayotembea
   * na watu wanaotembea. Real-time (interval kutoka data.live.tick_ms).
   */
  function vehicleIcon() {
    return L.divIcon({ className: "mt-live-veh", html: '<div style="font-size:18px;filter:drop-shadow(0 0 4px #ff9100)">\uD83D\uDE97</div>', iconSize: [22, 22], iconAnchor: [11, 11] });
  }
  function personIcon() {
    return L.divIcon({ className: "mt-live-per", html: '<div style="font-size:15px;filter:drop-shadow(0 0 4px #76ff03)">\uD83D\uDEB6</div>', iconSize: [18, 18], iconAnchor: [9, 9] });
  }

  function stopLive() {
    if (MT.liveTimer) { clearInterval(MT.liveTimer); MT.liveTimer = null; }
    if (MT.liveLayer && MT.map) MT.map.removeLayer(MT.liveLayer);
    MT.liveLayer = null; MT.liveObjs = [];
  }

  function startLive() {
    stopLive();
    if (!MT.map) return;
    MT.liveLayer = L.layerGroup().addTo(MT.map);
    var cfg = MT.data.live || { vehicles: 6, people: 8, tick_ms: 900 };
    var pts = [];
    /* pointi za mwendo: corridors (kama zipo) + miji yote */
    (MT.data.routes || []).forEach(function (r) {
      pts.push([[r.from.lat, r.from.lng], [r.to.lat, r.to.lng]]);
    });
    (MT.data.cities || []).forEach(function (c) {
      pts.push([[c.lat, c.lng], [c.lat + 0.25, c.lng + 0.25]]);
    });
    if (!pts.length) pts.push([[-6.816, 39.285], [-6.163, 35.751]]);
    var nV = cfg.vehicles || 6, nP = cfg.people || 8, tick = cfg.tick_ms || 900;
    for (var i = 0; i < nV; i++) {
      var seg = pts[i % pts.length];
      var mk = L.marker(seg[0], { icon: vehicleIcon() }).addTo(MT.liveLayer)
        .bindPopup("\uD83D\uDE97 Gari la huduma (live simulasi)");
      MT.liveObjs.push({ m: mk, a: seg[0], b: seg[1], t: Math.random(), sp: 0.004 + Math.random() * 0.008 });
    }
    for (var j = 0; j < nP; j++) {
      var seg2 = pts[(j * 3 + 1) % pts.length];
      var mk2 = L.marker(seg2[0], { icon: personIcon() }).addTo(MT.liveLayer)
        .bindPopup("\uD83D\uDEB6 Mtu kwenye njia (live simulasi)");
      MT.liveObjs.push({ m: mk2, a: seg2[0], b: seg2[1], t: Math.random(), sp: 0.002 + Math.random() * 0.005 });
    }
    MT.liveTimer = setInterval(function () {
      MT.liveObjs.forEach(function (o) {
        o.t += o.sp * (Math.random() * 0.6 + 0.7);
        if (o.t >= 1) { o.t = 0; var tmp = o.a; o.a = o.b; o.b = tmp; }
        var lat = o.a[0] + (o.b[0] - o.a[0]) * o.t;
        var lng = o.a[1] + (o.b[1] - o.a[1]) * o.t;
        o.m.setLatLng([lat, lng]);
      });
    }, tick);
    setInfo("\uD83D\uDFE2 LIVE IMEWASHWA — magari " + nV + " + watu " + nP + " wakitembea (simulasi ya kweli ya mtiririko)");
    speakSw("Live imewashwa. Unaona magari na watu wakitembea kwenye ramani.");
  }

  function toggleLive() {
    if (MT.liveTimer) { stopLive(); setInfo("\u23f8 Live IMESIMAMISHWA"); }
    else startLive();
    var btn = el("map_live_btn");
    if (btn) btn.style.borderColor = MT.liveTimer ? "#76ff03" : "";
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

    /* AUTO-LOCATE: mara moja baada ya ramani kuwa tayari (GPS binafsi, kwa ruhusa ya browser) */
    setTimeout(autoLocate, 800);
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

  window.MTMap = {
    init: init, resize: resize, setBase: setBase, toggle: toggleOverlay,
    routeTo: routeTo, weatherAt: weatherAt, locate: locateMe, track: toggleTrack, search: searchPlace,
    photosAt: photosAt, voice: toggleVoice, live: toggleLive
  };

  if (document.readyState === "complete" || document.readyState === "interactive") {
    setTimeout(function () { if (el("tzmap")) init(); }, 0);
  } else {
    document.addEventListener("DOMContentLoaded", function () { if (el("tzmap")) init(); });
  }
})();
