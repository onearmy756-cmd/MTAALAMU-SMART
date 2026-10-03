#!/usr/bin/env node
/**
 * Lint for web-r/www/index.html (MTAALAMU browser dashboard).
 *
 * Zero-dependency checks (no npm install needed, works in CI):
 *   1. Document basics: <title>, <meta charset>, non-trivial size
 *   2. Balanced <script>/<style> blocks
 *   3. Every inline <script> parses as JavaScript (node --check) —
 *      the real safety net for a 100k+ char single-file dashboard
 *   4. Duplicate id="" attributes (fail — they break getElementById wiring)
 *   5. src/href reference integrity: local relative targets must exist
 *
 * Usage: node scripts/lint_dashboard.mjs [path-to-html]
 * Exit 0 = clean, 1 = problems found.
 */
import { readFileSync, writeFileSync, existsSync, statSync, unlinkSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join, dirname, resolve } from "node:path";

const file = resolve(process.argv[2] || "hermes-agent/web-r/www/index.html");
const problems = [];
const warns = [];

if (!existsSync(file)) {
  console.error(`FAIL: file not found: ${file}`);
  process.exit(1);
}
const html = readFileSync(file, "utf8");
const bytes = statSync(file).size;
if (bytes < 10_000) problems.push(`file suspiciously small (${bytes} bytes)`);

// 1. Document basics
if (!/<title>[\s\S]+?<\/title>/i.test(html)) problems.push("missing <title>");
if (!/<meta[^>]+charset/i.test(html)) problems.push("missing <meta charset>");
if (!/lang\s*=/i.test(html)) warns.push("<html lang=...> missing (a11y)");

// 2. Balanced script/style blocks
const openScripts = (html.match(/<script\b/gi) || []).length;
const closeScripts = (html.match(/<\/script>/gi) || []).length;
if (openScripts !== closeScripts)
  problems.push(`unbalanced <script>: ${openScripts} open vs ${closeScripts} close`);
const openStyles = (html.match(/<style\b/gi) || []).length;
const closeStyles = (html.match(/<\/style>/gi) || []).length;
if (openStyles !== closeStyles)
  problems.push(`unbalanced <style>: ${openStyles} open vs ${closeStyles} close`);

// 3. Inline <script> blocks must parse as JS
const scriptRe = /<script\b([^>]*)>([\s\S]*?)<\/script>/gi;
let m, idx = 0, jsBytes = 0;
while ((m = scriptRe.exec(html)) !== null) {
  const attrs = m[1] || "";
  const code = m[2] || "";
  if (/\bsrc\s*=/i.test(attrs)) continue; // external script — not lintable here
  if (code.trim().length === 0) continue;
  idx += 1;
  jsBytes += code.length;
  const tmp = join(tmpdir(), `mtaalamu_inline_${idx}.js`);
  writeFileSync(tmp, code, "utf8");
  try {
    execFileSync(process.execPath, ["--check", tmp], { stdio: "pipe" });
  } catch (e) {
    const tail = String(e.stderr || e.message || "").split("\n").slice(0, 4).join("\n");
    problems.push(`inline script #${idx} fails node --check:\n${tail}`);
  } finally {
    try { unlinkSync(tmp); } catch {}
  }
}
if (idx === 0) warns.push("no inline <script> blocks found — dashboard logic missing?");
if (jsBytes < 50_000) warns.push(`inline JS only ${jsBytes} chars — expected the full dashboard`);

// 4. Duplicate ids
const ids = [...html.matchAll(/\bid\s*=\s*["']([^"']+)["']/gi)].map((x) => x[1]);
const seen = new Map();
for (const id of ids) seen.set(id, (seen.get(id) || 0) + 1);
const dups = [...seen.entries()].filter(([, n]) => n > 1);
if (dups.length) problems.push(`duplicate id attributes: ${dups.map(([i, n]) => `${i}×${n}`).join(", ")}`);

// 5. Local relative references (src/href) must exist on disk.
// Template literals (href="${...}") are dynamic: only their static prefix is
// checkable (e.g. "ramani-3d.html${hash}" → check "ramani-3d.html").
const refRe = /\b(?:src|href)\s*=\s*["']([^"'#]+)["']/gi;
const base = dirname(file);
while ((m = refRe.exec(html)) !== null) {
  let ref = m[1].trim();
  if (!ref || /^(https?:|data:|mailto:|tel:|#|\/\/)/i.test(ref)) continue;
  if (ref.includes("${")) {
    ref = ref.split("${")[0].trim();
    if (!ref || /[()\s]/.test(ref)) continue; // template opens mid-expression
  }
  if (ref.includes("(") || ref.includes(" ")) continue; // not a plain path
  const target = resolve(base, ref.split("?")[0]);
  if (!existsSync(target)) problems.push(`broken local reference: "${m[1]}"`);
}

// Report
for (const w of warns) console.log(`WARN: ${w}`);
if (problems.length) {
  for (const p of problems) console.error(`FAIL: ${p}`);
  console.error(`\n${problems.length} problem(s) in ${file}`);
  process.exit(1);
}
console.log(`OK: ${file} (${(bytes / 1024).toFixed(0)} KB, ${idx} inline script(s), ${ids.length} ids) — lint clean`);
