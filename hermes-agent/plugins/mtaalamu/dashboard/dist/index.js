/**
 * MTAALAMU dashboard plugin bundle — Hermes web plugin SDK v1.1.0 IIFE.
 *
 * Registers the "MTAALAMU" tab inside the official Hermes Agent dashboard:
 *   - Chat-first workspace (uses the built-in Hermes chat route)
 *   - Problem solver (solve / agentic run with HITL approve flow)
 *   - Fundi Deploy console (discover → select → HITL start → jobs)
 *   - Fundi Mobile-style actions (telecom calc, gov apps launcher)
 *   - Home Assistant panel (real entities + HITL toggles, our dark-cyan theme)
 *   - OpenMRS panel (real patient search + HITL registration, our theme)
 *   - Channels (WhatsApp / SMS / email dispatch)
 *   - Predictive maintenance + background scanner
 *   - Remote jobs queue
 *   - RBAC switch (admin / specialist / user) enforced server-side
 *
 * All data comes from /api/plugins/mtaalamu/* (real engine). No fake data:
 * errors are rendered honestly, per HERMES kanuni.
 */
(function () {
  "use strict";

  var SDK = window.__HERMES_PLUGIN_SDK__;
  var REG = window.__HERMES_PLUGINS__;
  if (!SDK || !REG) return;

  var React = SDK.React;
  var hooks = SDK.hooks;
  var C = SDK.components;
  var fetchJSON = SDK.fetchJSON;
  var cn = SDK.utils.cn;
  var useState = hooks.useState, useEffect = hooks.useEffect, useCallback = hooks.useCallback;

  var API = "/api/plugins/mtaalamu";

  // ---------------------------------------------------------------- primitives

  function useJson(url, opt) {
    var _a = useState({ loading: true, data: null, error: null }), s = _a[0], set = _a[1];
    var reload = useCallback(function () {
      set({ loading: true, data: s.data, error: null });
      fetchJSON(url, opt)
        .then(function (d) { set({ loading: false, data: d, error: null }); })
        .catch(function (e) { set({ loading: false, data: null, error: String(e.message || e) }); });
    }, [url, JSON.stringify(opt || {})]);
    useEffect(function () { reload(); }, [reload]);
    return [s, reload];
  }

  function post(url, body) {
    return fetchJSON(url, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body || {}),
    });
  }

  function errText(e) {
    var m = (e && e.message) || String(e);
    if (e && e.body) {
      try {
        var b = JSON.parse(e.body);
        if (b && b.hint) m += " — 💡 " + b.hint;
        else if (b && b.error && m.indexOf(b.error) < 0) m = b.error + (b.hint ? " — 💡 " + b.hint : "");
      } catch (ex) { /* body si JSON */ }
    }
    return m;
  }

  function H3(p) { return React.createElement("h3", { className: "mta-h3" }, p.children); }

  function Dot(p) {
    return React.createElement("span", {
      className: "mta-dot " + (p.ok ? "ok" : "bad"),
      title: p.title || "",
    });
  }

  function Err(p) {
    if (!p.e) return null;
    return React.createElement("div", { className: "mta-err" }, "⚠️ " + p.e);
  }

  function Btn(p) {
    return React.createElement(
      "button",
      {
        className: "mta-btn" + (p.primary ? " primary" : "") + (p.danger ? " danger" : "") + (p.sm ? " sm" : ""),
        title: p.title || undefined,
        disabled: !!p.disabled,
        onClick: p.onClick,
      },
      p.children
    );
  }

  function Field(p) {
    return React.createElement(
      "label",
      { className: "mta-field" },
      React.createElement("span", null, p.label),
      p.children
    );
  }

  function Pre(p) {
    return React.createElement("pre", { className: "mta-pre" },
      typeof p.v === "string" ? p.v : JSON.stringify(p.v, null, 2));
  }

  // ---------------------------------------------------------------- App

  function MtaalamuApp() {
    var _r = useState("specialist"), role = _r[0], setRole = _r[1];
    var _t = useState("solve"), tab = _t[0], setTab = _t[1];
    var _h = useState(null), health = _h[0], setHealth = _h[1];

    var loadHealth = useCallback(function () {
      fetchJSON(API + "/health").then(setHealth).catch(function (e) {
        setHealth({ engine: false, error: String(e.message || e) });
      });
    }, []);
    useEffect(function () { loadHealth(); var t = setInterval(loadHealth, 30000); return function () { clearInterval(t); }; }, [loadHealth]);

    var sec = { role: role };

    return React.createElement(
      "div",
      { className: "mta-wrap" },
      React.createElement(
        "header",
        { className: "mta-top" },
        React.createElement("div", { className: "mta-brand" },
          "🔧 MTAALAMU SMART", React.createElement(Dot, { ok: !!(health && health.engine), title: health && health.engine ? "engine tayari" : "engine haipo" }),
          React.createElement("code", { className: "mta-path" }, (health && health.engine_path) || "engine-rust/target/release/mtaalamu")
        ),
        React.createElement(
          "div",
          { className: "mta-rbac" },
          ["admin", "specialist", "user"].map(function (r) {
            return React.createElement("button", {
              key: r, className: "mta-btn sm" + (role === r ? " primary" : ""),
              onClick: function () { setRole(r); },
            }, r);
          })
        )
      ),
      React.createElement(
        "nav",
        { className: "mta-tabs" },
        [["solve", "🛠️ Tatua"], ["deploy", "💻 Fundi Deploy"], ["mobile", "📱 Fundi Mobile"], ["ha", "🏠 Home Assistant"], ["omrs", "🏥 OpenMRS"], ["channels", "📨 Channels"], ["maintain", "🔮 Maintenance"], ["jobs", "🗂️ Remote Jobs"]].map(function (x) {
          return React.createElement("button", {
            key: x[0], className: "mta-tab" + (tab === x[0] ? " on" : ""),
            onClick: function () { setTab(x[0]); },
          }, x[1]);
        })
      ),
      React.createElement("main", { className: "mta-main" },
        tab === "solve" ? React.createElement(SolvePanel, sec) :
        tab === "deploy" ? React.createElement(DeployPanel, sec) :
        tab === "mobile" ? React.createElement(MobilePanel, sec) :
        tab === "ha" ? React.createElement(RealUiPanel, { role: role, kind: "ha" }) :
        tab === "omrs" ? React.createElement(RealUiPanel, { role: role, kind: "omrs" }) :
        tab === "channels" ? React.createElement(ChannelsPanel, null) :
        tab === "maintain" ? React.createElement(MaintainPanel, sec) :
        React.createElement(JobsPanel, sec)
      ),
      React.createElement("footer", { className: "mta-foot" },
        "R-1: hesabu zote ni Rust — LLM anaeleza tu · HITL: hakuna hatua bila idhini ya mteja · chat ya Hermes ndiyo mbele")
    );
  }

  // ---------------------------------------------------------------- Solve

  function SolvePanel(p) {
    var _m = useState(""), msg = _m[0], setMsg = _m[1];
    var _res = useState(null), res = _res[0], setRes = _res[1];
    var _busy = useState(false), busy = _busy[0], setBusy = _busy[1];
    var _err = useState(null), err = _err[0], setErr = _err[1];

    function run(kind) {
      setBusy(true); setErr(null);
      post(API + "/" + kind, { msg: msg, role: p.role, approve: kind === "agentic" })
        .then(function (d) { setRes(d); })
        .catch(function (e) { setErr(String(e.message || e)); })
        .finally(function () { setBusy(false); });
    }

    return React.createElement(
      "div", null,
      React.createElement(H3, null, "🛠️ Tatua Tatizo (solve / agentic)"),
      React.createElement(Field, { label: "Tatizo la mteja (Kiswahili au English)" },
        React.createElement("textarea", {
          className: "mta-input", rows: 3, value: msg,
          placeholder: "mf: kompyuta inaenda polepole na inazima yenyewe",
          onChange: function (e) { setMsg(e.target.value); },
        })),
      React.createElement("div", { className: "mta-row" },
        React.createElement(Btn, { primary: true, disabled: busy || !msg, onClick: function () { run("solve"); } }, busy ? "…" : "Solve"),
        React.createElement(Btn, { disabled: busy || !msg, onClick: function () { run("agentic"); } }, "Agentic (scan→plan→report)")),
      React.createElement(Err, { e: err }),
      res && React.createElement("div", { className: "mta-card" },
        React.createElement(Pre, { v: res })),
      res && res.error && res.error.indexOf("HITL") >= 0 &&
      React.createElement("div", { className: "mta-hitl" },
        "🔒 HITL: hatua hii inahitaji idhini ya mteja. Uliza mteja kisha (kama admin/specialist) rudia.")
    );
  }

  // ---------------------------------------------------------------- Deploy

  function DeployPanel(p) {
    var _h = useJson(API + "/deploy/summary"), sum = _h[0], reloadSum = _h[1];
    var _hosts = useState(null), hosts = _hosts[0], setHosts = _hosts[1];
    var _sel = useState({}), sel = _sel[0], setSel = _sel[1];
    var _os = useState("auto"), os = _os[0], setOs = _os[1];
    var _need = useState("office"), need = _need[0], setNeed = _need[1];
    var _err = useState(null), err = _err[0], setErr = _err[1];
    var _msg = useState(""), okMsg = _msg[0], setOk = _msg[1];

    function discover() {
      setErr(null);
      fetchJSON(API + "/deploy/hosts").then(function (d) { setHosts(d.hosts || d); }).catch(function (e) { setErr(String(e.message || e)); });
    }
    function start() {
      var macs = Object.keys(sel).filter(function (k) { return sel[k]; });
      if (!macs.length) { setErr("Chagua host angalau moja"); return; }
      setErr(null);
      post(API + "/deploy/start", { macs: macs.join(","), os: os, need: need, role: p.role, approved: true })
        .then(function (d) { setOk("Job imetumwa ✓"); reloadSum(); })
        .catch(function (e) { setErr(String(e.message || e)); });
    }

    var hostList = Array.isArray(hosts) ? hosts : (hosts && (hosts.hosts || hosts.devices)) || [];

    return React.createElement(
      "div", null,
      React.createElement(H3, null, "💻 Fundi Deploy"),
      React.createElement("div", { className: "mta-row" },
        React.createElement(Btn, { onClick: discover }, "🔍 Discover hosts"),
        React.createElement(Btn, { onClick: reloadSum }, "↻ Summary")),
      sum && sum.data && React.createElement("div", { className: "mta-card" }, React.createElement(Pre, { v: sum.data })),
      hostList.length > 0 && React.createElement(
        "div", { className: "mta-card" },
        React.createElement("table", { className: "mta-table" },
          React.createElement("thead", null, React.createElement("tr", null,
            React.createElement("th", null, ""), React.createElement("th", null, "MAC"), React.createElement("th", null, "Jina"))),
          React.createElement("tbody", null, hostList.map(function (h, i) {
            var mac = h.mac || h.id || String(i);
            return React.createElement("tr", { key: mac },
              React.createElement("td", null, React.createElement("input", {
                type: "checkbox", checked: !!sel[mac],
                onChange: function (e) { var n = Object.assign({}, sel); n[mac] = e.target.checked; setSel(n); },
              })),
              React.createElement("td", null, React.createElement("code", null, mac)),
              React.createElement("td", null, h.name || h.hostname || "-"));
          })))
      ),
      hostList.length > 0 && React.createElement(
        "div", { className: "mta-row" },
        React.createElement(Field, { label: "OS" },
          React.createElement("select", { className: "mta-input", value: os, onChange: function (e) { setOs(e.target.value); } },
            ["auto", "windows", "ubuntu"].map(function (o) { return React.createElement("option", { key: o, value: o }, o); }))),
        React.createElement(Field, { label: "Need" },
          React.createElement("select", { className: "mta-input", value: need, onChange: function (e) { setNeed(e.target.value); } },
            ["office", "design", "dev"].map(function (o) { return React.createElement("option", { key: o, value: o }, o); }))),
        React.createElement(Btn, { primary: true, danger: true, onClick: start }, "🚀 Anza (HITL)")),
      React.createElement(Err, { e: err || (sum && sum.error) }),
      okMsg && React.createElement("div", { className: "mta-ok" }, okMsg)
    );
  }

  // ---------------------------------------------------------------- Mobile (manual-first)

  function MobilePanel() {
    var _f = useState({ width: "", dist: "", freq: "2400" }), f = _f[0], setF = _f[1];
    var _out = useState(null), out = _out[0], setOut = _out[1];
    function calc() {
      var c = 3e8, fr = parseFloat(f.freq) * 1e6 || 2.4e9, d = parseFloat(f.dist) || 0;
      var fspl = d > 0 ? 20 * Math.log10(d) + 20 * Math.log10(fr) + 32.44 : 0;
      setOut({ FSPL_dB: fspl.toFixed(1), wavelength_m: (c / fr).toFixed(3), note: "FSPL + wavelength — hesabu za network" });
    }
    var gov = ["NIDA", "TIN", "BRELA", "TRA", "NSSF", "WCF", "Business Permit", "Land Form"];
    return React.createElement(
      "div", null,
      React.createElement(H3, null, "📱 Fundi Mobile — manual-first (bila Hermes pia inafanya kazi)"),
      React.createElement("div", { className: "mta-row" },
        React.createElement(Field, { label: "Urefu (m)" },
          React.createElement("input", { className: "mta-input", type: "number", value: f.dist, onChange: function (e) { setF(Object.assign({}, f, { dist: e.target.value })); } })),
        React.createElement(Field, { label: "Frequency (MHz)" },
          React.createElement("input", { className: "mta-input", type: "number", value: f.freq, onChange: function (e) { setF(Object.assign({}, f, { freq: e.target.value })); } })),
        React.createElement(Btn, { primary: true, onClick: calc }, "Hisabu")),
      out && React.createElement("div", { className: "mta-card" }, React.createElement(Pre, { v: out })),
      React.createElement(H3, null, "🏛️ Government applications (forms)"),
      React.createElement("div", { className: "mta-grid" },
        gov.map(function (g) {
          return React.createElement("button", { key: g, className: "mta-btn" }, g);
        }))
    );
  }

  // ---------------------------------------------------------------- Channels

  function ChannelsPanel() {
    var _f = useState({ to: "", msg: "", subject: "" }), f = _f[0], setF = _f[1];
    var _st = useState(null), st = _st[0], setSt = _st[1];
    function send(ch) {
      setSt(null);
      post(API + "/channels/" + ch, f).then(setSt).catch(function (e) { setSt({ ok: false, error: String(e.message || e) }); });
    }
    return React.createElement(
      "div", null,
      React.createElement(H3, null, "📨 Channels — WhatsApp / SMS / Email (halisi)"),
      React.createElement(Field, { label: "Kwa (namba/email)" },
        React.createElement("input", { className: "mta-input", value: f.to, onChange: function (e) { setF(Object.assign({}, f, { to: e.target.value })); } })),
      React.createElement(Field, { label: "Subject (email)" },
        React.createElement("input", { className: "mta-input", value: f.subject, onChange: function (e) { setF(Object.assign({}, f, { subject: e.target.value })); } })),
      React.createElement(Field, { label: "Ujumbe" },
        React.createElement("textarea", { className: "mta-input", rows: 3, value: f.msg, onChange: function (e) { setF(Object.assign({}, f, { msg: e.target.value })); } })),
      React.createElement("div", { className: "mta-row" },
        React.createElement(Btn, { primary: true, onClick: function () { send("whatsapp"); } }, "💬 WhatsApp"),
        React.createElement(Btn, { onClick: function () { send("sms"); } }, "📱 SMS"),
        React.createElement(Btn, { onClick: function () { send("email"); } }, "✉️ Email")),
      st && React.createElement("div", { className: st.ok ? "mta-ok" : "mta-err" }, st.ok ? "✔ Imetumwa" : "⚠️ " + (st.error || "imeshindikana"))
    );
  }

  // ---------------------------------------------------------------- Maintenance

  function MaintainPanel(p) {
    var _d = useState(null), dat = _d[0], setD = _d[1];
    var _busy = useState(false), busy = _busy[0], setBusy = _busy[1];
    var _err = useState(null), err = _err[0], setErr = _err[1];
    function runOnce() {
      setBusy(true); setErr(null);
      post(API + "/scanner", { mode: "run_once", role: p.role, approved: p.role !== "user" })
        .then(setD).catch(function (e) { setErr(String(e.message || e)); })
        .finally(function () { setBusy(false); });
    }
    function forecast() {
      setBusy(true); setErr(null);
      fetchJSON(API + "/maintenance?horizon=30&top=10").then(setD).catch(function (e) { setErr(String(e.message || e)); })
        .finally(function () { setBusy(false); });
    }
    return React.createElement(
      "div", null,
      React.createElement(H3, null, "🔮 Predictive maintenance + 🔎 background scanner"),
      React.createElement("div", { className: "mta-row" },
        React.createElement(Btn, { primary: true, disabled: busy, onClick: forecast }, "Bashiri hatari (30 siku)"),
        React.createElement(Btn, { disabled: busy, onClick: runOnce }, "Scanner: run once (HITL)")),
      React.createElement(Err, { e: err || (dat && dat.error) }),
      dat && React.createElement("div", { className: "mta-card" }, React.createElement(Pre, { v: dat }))
    );
  }

  // ---------------------------------------------------------------- Jobs

  function JobsPanel(p) {
    var _l = useJson(API + "/jobs"), list = _l[0], reload = _l[1];
    var _k = useState("solve"), kind = _k[0], setKind = _k[1];
    var _m = useState(""), msg = _m[0], setMsg = _m[1];
    function submit() {
      post(API + "/jobs", { kind: kind, msg: msg, role: p.role, approved: true }).then(function () { setMsg(""); reload(); });
    }
    var kindSel = React.createElement(
      Field, { label: "Aina" },
      React.createElement("select", { className: "mta-input", value: kind, onChange: function (e) { setKind(e.target.value); } },
        ["solve", "scan", "deploy_summary", "agentic"].map(function (k) { return React.createElement("option", { key: k }, k); }))
    );
    var msgField = React.createElement(
      Field, { label: "Maelezo" },
      React.createElement("input", { className: "mta-input", value: msg, onChange: function (e) { setMsg(e.target.value); } })
    );
    return React.createElement(
      "div", null,
      React.createElement(H3, null, "🗂️ Remote jobs"),
      React.createElement("div", { className: "mta-row" },
        kindSel,
        msgField,
        React.createElement(Btn, { primary: true, onClick: submit }, "Tuma kazi")),
      React.createElement("div", { className: "mta-row" }, React.createElement(Btn, { onClick: reload }, "↻ Rejesha")),
      list.data && list.data.jobs && list.data.jobs.length === 0 && React.createElement("p", null, "Hakuna kazi bado."),
      list.data && list.data.jobs && list.data.jobs.slice().reverse().slice(0, 10).map(function (j) {
        return React.createElement("div", { key: j.id, className: "mta-card" },
          React.createElement("b", null, j.id, " · ", j.kind, " · ", j.status),
          j.msg && React.createElement("div", { className: "mta-dim" }, j.msg));
      })
    );
  }

  // ------------------------------ UI HALISI chini ya Hermes (source halisi, si iframe si URL za nje)

  var REALUI_META = {
    ha: { ico: '🏠', title: 'Home Assistant — UI HALISI (Lovelace)', src: 'hermes-agent/home-assistant/', route: '/ha/' },
    omrs: { ico: '🏥', title: 'OpenMRS — UI HALISI (o3)', src: 'hermes-agent/openmrs/', route: '/openmrs/' },
    hermes: { ico: '🤖', title: 'Hermes Agent — UI HALISI yake', src: 'hermes-agent/web/', route: '/hermes-ui/' },
  };

  function RealUiPanel(p) {
    var meta = REALUI_META[p.kind];
    var _s = useJson(API + '/integrations'), st = _s[0];
    var info = st.data && st.data.integrations && st.data.integrations[p.kind === 'omrs' ? 'openmrs' : p.kind];
    var present = !!(info && info.present);
    function openReal() { window.location.assign(meta.route); }

    return React.createElement(
      'div', null,
      React.createElement('div', { className: 'mta-row' },
        React.createElement(H3, null, meta.ico + ' ' + meta.title),
        React.createElement('div', { style: { marginLeft: 'auto' } },
          React.createElement('span', { className: 'mta-pill' + (present ? ' ok' : ' dim') },
            'source: ' + meta.src + (present ? ' ✅' : ' (inavutwa kwenye server)')))),
      React.createElement('div', { className: 'mta-card' },
        React.createElement('p', { className: 'mta-dim' },
          'Hii ni UI HALISI ya mfumo huo — source yake ipo NDANI ya hermes-agent. Hermes inapitisha UI hiyo HALISI chini ya route ' + meta.route + ' (ukurasa KAMILI — si iframe, si URL ya nje). Kwa sasa: ', React.createElement('b', null, info && info.ui)),
        React.createElement('div', { className: 'mta-row' },
          React.createElement(Btn, { primary: true, onClick: openReal }, meta.ico + ' FUNGUA UI HALISI (' + meta.route + ')')),
        info && !present && React.createElement('p', { className: 'mta-dim' },
          'Kwenye PC hii source haijavutwa kamili — kwenye server yako endesha: node hermes-agent/scripts/mtaalamu/fetch_integrations.mjs, kisha docker compose -f hermes-agent/deploy/docker-compose.mtaalamu.yml up -d --build')),
      st.error && React.createElement(Err, { e: st.error }),
      React.createElement('div', { className: 'mta-card' },
        React.createElement(H3, null, '🤝 HERMES anaitumia kama agent'),
        React.createElement('p', { className: 'mta-dim' },
          p.kind === 'ha'
            ? 'Tools: mtaalamu_ha — states / turn_on / turn_off / toggle (REST halisi ya HA, HITL + RBAC).'
            : 'Tools: mtaalamu_openmrs — find / create patient (REST halisi ya OpenMRS, HITL + RBAC).'),
        p.kind === 'ha' && React.createElement(HAMiniPanel, p))
    );
  }

  function HAMiniPanel(p) {
    var _s = useJson(API + '/ha/state'), st = _s[0], reload = _s[1];
    var _pend = useState(null), pend = _pend[0], setPend = _pend[1];
    var _e = useState(null), err = _e[0], setErr = _e[1];
    var ents = (st.data && st.data.entities) || [];
    function approve() {
      if (!pend) return;
      post(API + '/ha/service', { domain: pend.domain, action: pend.action, entity: pend.entity, role: p.role, approved: true })
        .then(function () { setPend(null); reload(); })
        .catch(function (ex) { setErr(errText(ex)); setPend(null); });
    }
    return React.createElement(
      'div', null,
      React.createElement('div', { className: 'mta-row' },
        React.createElement('span', { className: 'mta-pill' + (st.data && st.data.ok ? ' ok' : st.error ? ' bad' : ' dim') },
          st.loading ? 'inapakia…' : st.error ? 'HA server haipatikani (weka HASS_URL/HASS_TOKEN)' : ((st.data.count || 0) + ' entities')),
        React.createElement(Btn, { sm: true, onClick: reload }, '↻ Rejesha')),
      React.createElement(Err, { e: err || st.error }),
      React.createElement('div', { className: 'mta-grid' },
        ents.slice(0, 12).map(function (e) {
          var d = e.id.split('.')[0];
          var canDo = d === 'light' || d === 'switch' || d === 'climate';
          var on = e.state === 'on';
          return React.createElement('div', { key: e.id, className: 'mta-ent' },
            React.createElement('div', { className: 'mta-ent-body' },
              React.createElement('b', { title: e.id }, e.name),
              React.createElement('div', { className: 'mta-ent-state' },
                React.createElement('span', { className: 'val' + (on ? '' : ' off') }, e.state), e.unit ? ' ' + e.unit : '')),
            canDo && React.createElement(Btn, { sm: true, onClick: function () { setPend({ entity: e.id, name: e.name, domain: d, action: on ? 'turn_off' : 'turn_on' }); } }, on ? 'Zima' : 'Washa'));
        })),
      pend && React.createElement(
        'div', { className: 'mta-hitl' },
        '🔒 HITL: ', React.createElement('b', null, pend.name), ' → ', React.createElement('b', null, pend.action),
        React.createElement('div', { className: 'mta-row' },
          React.createElement(Btn, { primary: true, disabled: p.role === 'user', onClick: approve }, '✅ Idhinisha'),
          React.createElement(Btn, { sm: true, onClick: function () { setPend(null); } }, '❌ Ghairi')))
    );
  }

  // ---------------------------------------------------------------- styles

  /* MTAALAMU dark-cyan theme — sawa na web-r/www/index.html (--bg #04070d, --card #0a1628, --cyan #00e5ff, --green #00ff88) */
  var css = [
    ".mta-wrap{display:flex;flex-direction:column;gap:10px;padding:14px;height:100%;overflow:auto;background:#04070d;color:#cfe9f5;font-family:ui-monospace,Consolas,monospace}",
    ".mta-top{display:flex;justify-content:space-between;align-items:center;gap:10px;flex-wrap:wrap}",
    ".mta-brand{font-weight:700;font-size:15px;display:flex;align-items:center;gap:8px;color:#eafcff;letter-spacing:1px;text-shadow:0 0 12px rgba(0,229,255,.45)}",
    ".mta-path{font-size:11px;color:#5e8aa3;max-width:340px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}",
    ".mta-dot{width:9px;height:9px;border-radius:50%;display:inline-block}.mta-dot.ok{background:#00ff88;box-shadow:0 0 8px #00ff88}.mta-dot.bad{background:#ff3b3b;box-shadow:0 0 8px #ff3b3b}",
    ".mta-rbac{display:flex;gap:6px}",
    ".mta-tabs{display:flex;gap:6px;flex-wrap:wrap;position:sticky;top:0;z-index:5;background:rgba(4,7,13,.94);padding:4px 0;backdrop-filter:blur(4px)}",
    ".mta-tab{padding:7px 12px;border-radius:8px;border:1px solid rgba(0,229,255,.35);background:#0f2233;color:#00e5ff;cursor:pointer;font-size:12.5px;font-weight:700;letter-spacing:.5px}",
    ".mta-tab:hover{background:#143048}",
    ".mta-tab.on{background:rgba(0,229,255,.16);color:#00ff88;border-color:#00ff88;box-shadow:0 0 12px rgba(0,255,136,.18)}",
    ".mta-btn{padding:8px 14px;border-radius:6px;border:1px solid rgba(0,229,255,.35);background:#0f2233;color:#00e5ff;cursor:pointer;font-size:12.5px;font-weight:700;letter-spacing:.4px}",
    ".mta-btn:hover{background:#143048}",
    ".mta-btn.primary{background:rgba(0,229,255,.18);border-color:#00e5ff;color:#eafcff;box-shadow:0 0 12px rgba(0,229,255,.22)}",
    ".mta-btn.danger{border-color:rgba(255,59,59,.55);color:#ff3b3b}.mta-btn.danger:hover{background:rgba(255,59,59,.12)}",
    ".mta-btn.sm{padding:4px 10px;font-size:11.5px}.mta-btn:disabled{opacity:.45;cursor:default}",
    ".mta-main{flex:1;min-height:220px}.mta-row{display:flex;gap:8px;align-items:end;flex-wrap:wrap;margin:8px 0}",
    ".mta-field{display:flex;flex-direction:column;gap:4px;font-size:11.5px;color:#5e8aa3}",
    ".mta-input{background:#0d1b2e;border:1px solid rgba(0,229,255,.35);color:#cfe9f5;border-radius:6px;padding:8px 10px;font-size:12.5px;min-width:180px}",
    ".mta-input:focus{outline:none;border-color:#00e5ff;box-shadow:0 0 8px rgba(0,229,255,.3)}",
    ".mta-card{border:1px solid rgba(0,229,255,.22);border-radius:10px;padding:11px;margin:8px 0;background:#0f2233}",
    ".mta-pre{margin:0;font-size:11.5px;white-space:pre-wrap;word-break:break-word;max-height:380px;overflow:auto;color:#9fd8e8}",
    ".mta-err{border:1px solid rgba(255,59,59,.55);color:#ff3b3b;border-radius:8px;padding:9px 11px;font-size:12.5px;margin:6px 0;background:rgba(255,59,59,.06)}",
    ".mta-ok{border:1px solid rgba(0,255,136,.5);color:#00ff88;border-radius:8px;padding:9px 11px;font-size:12.5px;margin:6px 0;background:rgba(0,255,136,.05)}",
    ".mta-hitl{border:1px dashed #ffb300;color:#ffb300;border-radius:8px;padding:9px 11px;font-size:12.5px;margin:6px 0;background:rgba(255,179,0,.05)}",
    ".mta-pill{display:inline-flex;align-items:center;gap:6px;padding:4px 10px;border-radius:999px;font-size:11px;border:1px solid rgba(0,229,255,.35);background:#0f2233;color:#00e5ff}",
    ".mta-pill.ok{color:#00ff88;border-color:rgba(0,255,136,.4)}.mta-pill.bad{color:#ff3b3b;border-color:rgba(255,59,59,.5)}.mta-pill.dim{color:#5e8aa3}",
    ".mta-chip{padding:4px 10px;border-radius:999px;border:1px solid rgba(0,229,255,.3);background:#0f2233;color:#cfe9f5;cursor:pointer;font-size:11.5px}",
    ".mta-chip:hover{background:#143048}",
    ".mta-chip.on{color:#00ff88;border-color:#00ff88;background:rgba(0,255,136,.12)}",
    ".mta-table{width:100%;border-collapse:collapse;font-size:12.5px}.mta-table th{text-align:left;color:#00e5ff;font-weight:700;letter-spacing:.6px;font-size:11.5px;padding:5px}",
    ".mta-table td{padding:6px 5px;border-bottom:1px solid rgba(0,229,255,.12)}",
    ".mta-table tbody tr:hover td{background:rgba(0,229,255,.05)}",
    ".mta-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(150px,1fr));gap:8px;margin-top:8px}",
    ".mta-ent{display:flex;gap:8px;align-items:flex-start;border:1px solid rgba(0,229,255,.22);border-radius:10px;background:#0f2233;padding:9px}",
    ".mta-ent-ico{font-size:18px;line-height:1.2}",
    ".mta-ent-body{flex:1;min-width:0}.mta-ent-body b{display:block;color:#eafcff;font-size:12px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}",
    ".mta-ent-state{font-size:11.5px;color:#5e8aa3}.mta-ent-state .val{color:#00ff88}.mta-ent-state .val.off{color:#5e8aa3}",
    ".mta-bar{height:6px;border-radius:3px;background:#0d1b2e;overflow:hidden;margin-top:5px}.mta-bar i{display:block;height:100%;background:#00e5ff}",
    ".mta-dim{color:#5e8aa3;font-size:12px}.mta-h3{margin:6px 0;font-size:13.5px;color:#00e5ff;letter-spacing:1px}",
    ".mta-foot{color:#5e8aa3;font-size:11px;opacity:.9}",
  ].join("\n");

  var style = document.createElement("style");
  style.textContent = css;
  document.head.appendChild(style);

  REG.register("mtaalamu", MtaalamuApp);
})();
