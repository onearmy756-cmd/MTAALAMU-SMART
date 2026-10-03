/**
 * MTAALAMU dashboard plugin bundle — Hermes web plugin SDK v1.1.0 IIFE.
 *
 * Registers the "MTAALAMU" tab inside the official Hermes Agent dashboard:
 *   - Chat-first workspace (uses the built-in Hermes chat route)
 *   - Problem solver (solve / agentic run with HITL approve flow)
 *   - Fundi Deploy console (discover → select → HITL start → jobs)
 *   - Fundi Mobile-style actions (telecom calc, gov apps launcher)
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
        className: "mta-btn" + (p.primary ? " primary" : "") + (p.danger ? " danger" : ""),
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
        [["solve", "🛠️ Tatua"], ["deploy", "💻 Fundi Deploy"], ["mobile", "📱 Fundi Mobile"], ["channels", "📨 Channels"], ["maintain", "🔮 Maintenance"], ["jobs", "🗂️ Remote Jobs"]].map(function (x) {
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

  // ---------------------------------------------------------------- styles

  var css = [
    ".mta-wrap{display:flex;flex-direction:column;gap:10px;padding:14px;height:100%;overflow:auto}",
    ".mta-top{display:flex;justify-content:space-between;align-items:center;gap:10px}",
    ".mta-brand{font-weight:700;font-size:15px;display:flex;align-items:center;gap:8px}",
    ".mta-path{font-size:11px;opacity:.6;max-width:340px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}",
    ".mta-dot{width:9px;height:9px;border-radius:50%;display:inline-block}.mta-dot.ok{background:#3ecf6f}.mta-dot.bad{background:#e5484d}",
    ".mta-rbac{display:flex;gap:6px}.mta-tabs{display:flex;gap:6px;flex-wrap:wrap}",
    ".mta-tab{padding:6px 12px;border-radius:8px;border:1px solid var(--border,#333);background:transparent;cursor:pointer;font-size:13px}",
    ".mta-tab.on{background:var(--primary,#4f6ef7);color:#fff;border-color:transparent}",
    ".mta-btn{padding:7px 14px;border-radius:8px;border:1px solid var(--border,#333);background:var(--bg-elev,#1c1c1f);color:inherit;cursor:pointer;font-size:13px}",
    ".mta-btn.primary{background:var(--primary,#4f6ef7);color:#fff;border-color:transparent}.mta-btn.danger{border-color:#e5484d;color:#e5484d}",
    ".mta-btn.sm{padding:4px 10px;font-size:12px}.mta-btn:disabled{opacity:.5;cursor:default}",
    ".mta-main{flex:1;min-height:220px}.mta-row{display:flex;gap:8px;align-items:end;flex-wrap:wrap;margin:8px 0}",
    ".mta-field{display:flex;flex-direction:column;gap:4px;font-size:12px;opacity:.9}",
    ".mta-input{background:var(--bg-elev,#1c1c1f);border:1px solid var(--border,#333);color:inherit;border-radius:8px;padding:7px 10px;font-size:13px;min-width:180px}",
    ".mta-card{border:1px solid var(--border,#333);border-radius:10px;padding:10px;margin:8px 0;background:var(--bg-elev,#151517)}",
    ".mta-pre{margin:0;font-size:11.5px;white-space:pre-wrap;word-break:break-word;max-height:380px;overflow:auto}",
    ".mta-err{border:1px solid #e5484d;color:#e5484d;border-radius:8px;padding:8px 10px;font-size:13px;margin:6px 0}",
    ".mta-ok{border:1px solid #3ecf6f;color:#3ecf6f;border-radius:8px;padding:8px 10px;font-size:13px;margin:6px 0}",
    ".mta-hitl{border:1px dashed #f5a524;color:#f5a524;border-radius:8px;padding:8px 10px;font-size:13px;margin:6px 0}",
    ".mta-table{width:100%;border-collapse:collapse;font-size:13px}.mta-table th{text-align:left;opacity:.6;font-weight:500;padding:4px}",
    ".mta-table td{padding:5px 4px;border-top:1px solid var(--border,#2a2a2e)}",
    ".mta-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(140px,1fr));gap:8px;margin-top:8px}",
    ".mta-dim{opacity:.6;font-size:12px}.mta-h3{margin:6px 0;font-size:14px}.mta-foot{opacity:.5;font-size:11px}",
  ].join("\n");

  var style = document.createElement("style");
  style.textContent = css;
  document.head.appendChild(style);

  REG.register("mtaalamu", MtaalamuApp);
})();
