/**
 * fetch_integrations.mjs — vuta SOURCE KAMILI ya miradi ya MTAALAMU kwenye server yako.
 *
 * Inasoma hermes-agent/integrations.lock.json (commits zilizopimwa) na
 * kuweka source halisi ndani ya hermes-agent/<name>/ (mfano:
 * hermes-agent/home-assistant/ = Home Assistant core KAMILI, Python+frontend).
 *
 * Matumizi (kwenye server yako, git inahitajika):
 *   node hermes-agent/scripts/mtaalamu/fetch_integrations.mjs              # vuta/kemsha zote
 *   node hermes-agent/scripts/mtaalamu/fetch_integrations.mjs home-assistant  # moja tu
 *
 * Baada ya hapo UI halisi zinaendeshwa kwenye server (ona
 * hermes-agent/deploy/docker-compose.mtaalamu.yml) na Hermes inaziunganisha:
 *   - tools: mtaalamu_ha / mtaalamu_openmrs (REST, HITL)
 *   - dashboard: /api/plugins/mtaalamu/{ha,openmrs}/proxy/ (UI halisi kupitia Hermes)
 */
import { readFileSync, existsSync, mkdirSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url)); // hermes-agent/scripts/mtaalamu
const hermesRoot = resolve(here, "..", "..");          // hermes-agent/
const lockPath = join(hermesRoot, "integrations.lock.json");

const lock = JSON.parse(readFileSync(lockPath, "utf8"));
const only = process.argv[2] || null;

function git(cwd, ...args) {
  return execFileSync("git", args, { cwd, stdio: ["ignore", "pipe", "pipe"] }).toString().trim();
}

for (const it of lock.integrations) {
  if (only && it.name !== only) continue;
  const dest = join(hermesRoot, it.name); // hermes-agent/home-assistant | hermes-agent/openmrs
  console.log(`\n=== ${it.name} (${it.branch} @ ${it.commit.slice(0, 10)}) ===`);

  if (!existsSync(dest)) {
    mkdirSync(dirname(dest), { recursive: true });
    console.log(`clone ${it.repo} -> ${dest}`);
    git(dirname(dest), "clone", "--no-checkout", it.repo, dest);
  }
  if (!existsSync(join(dest, ".git"))) {
    console.error(`❌ ${dest} haina .git — funga folda ya tarball kwanza (rm -rf) kisha rudia.`);
    process.exitCode = 1;
    continue;
  }

  // Kemsha commit iliyopimwa (GitHub inaruhusu fetch-by-sha); fallback: branch head.
  try {
    git(dest, "fetch", "--depth", "1", "origin", it.commit);
    git(dest, "checkout", "--detach", "--force", it.commit);
    console.log(`✅ ${it.name} @ ${git(dest, "rev-parse", "HEAD").slice(0, 10)} (pimwa)`);
  } catch {
    git(dest, "fetch", "--depth", "1", "origin", it.branch);
    git(dest, "checkout", "--detach", "--force", "FETCH_HEAD");
    console.log(`⚠️ ${it.name}: commit ya lock haikupatikana — nimekemsha kichwa cha ${it.branch}`);
  }
  console.log(`   UI: ${it.ui}`);
}
console.log("\nHalima: endesha UI halisi kwenye server → hermes-agent/deploy/docker-compose.mtaalamu.yml");
