"""MTECH OS — config (env-driven, hakuna hard-code za siri)."""
import os
import shutil
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[3]


class Config:
    def __init__(self) -> None:
        # --- LLM (Ollama) ---
        self.ollama_url = os.environ.get("OLLAMA_URL", "http://127.0.0.1:11434")
        self.model = os.environ.get("MTECH_MODEL", "qwen2.5vl:3b")  # Qwen 2.5 VL 3B
        self.temperature = float(os.environ.get("MTECH_TEMP", "0.4"))
        self.max_steps = int(os.environ.get("MTECH_MAX_STEPS", "12"))

        # --- Akili ya MTAALAMU SMART (data + engine) ---
        repo = _repo_root()
        self.mtaalamu_root = Path(os.environ.get("MTAALAMU_ROOT", str(repo / "hermes-agent")))
        self.data_dir = self.mtaalamu_root / "data"
        self.skills_file = Path(os.environ.get("MTECH_SKILLS", str(self.data_dir / "skills" / "skills.json")))
        engine_bin = os.environ.get("MTECH_ENGINE_BIN", "")
        self.engine_bin = engine_bin or str(self._find_engine(repo))

        # --- Kernel bridge ---
        self.dev_mtech = Path(os.environ.get("MTECH_DEV", "/dev/mtech"))

        # --- Install layout (mfumo umewekwa kwenye OS) ---
        self.install_root = Path(os.environ.get("MTECH_ROOT", "/opt/mtech"))
        self.approve_all = os.environ.get("MTECH_APPROVE", "").lower() in ("1", "true", "yes")
        self.serve_port = int(os.environ.get("MTECH_PORT", "8790"))

    def _find_engine(self, repo: Path) -> Path:
        for cand in (
            repo / "dist" / "mtaalamu",
            repo / "hermes-agent" / "engine-rust" / "target" / "release" / "mtaalamu",
            Path("/usr/local/bin/mtaalamu"),
        ):
            if cand.exists():
                return cand
        return repo / "dist" / "mtaalamu"  # default (engine haijabuild bado)

    def adapted_paths(self) -> "Config":
        """Ikiwa tumewekwa kwenye /opt/mtech na repo haipo, tumia layout ya install."""
        if self.mtaalamu_root.exists() or not self.install_root.exists():
            return self
        alt = Config.__new__(Config)
        alt.__dict__.update(self.__dict__)
        alt.mtaalamu_root = self.install_root / "mtaalamu" / "hermes-agent"
        alt.data_dir = alt.mtaalamu_root / "data"
        alt.skills_file = alt.data_dir / "skills" / "skills.json"
        return alt


CONFIG = Config().adapted_paths()

SYSTEM_PROMPT = """Wewe ni MTECH — akili ya MTECH OS (Linux + kernel bridge).
Unadhani kama mtaalamu mwenye uzoefu (computer, simu, umeme, solar, mtandao, usalama).
Unaweza kutumia ZANA hizi (jibu kwa JSON pekee):
{"thought": "...", "tool": "<jina>", "args": {...}}  — tumia zana
{"thought": "...", "answer": "..."}                  — jibu la mwisho kwa Kiswahili
Zana: shell_exec, file_read, file_write, file_list, sys_probe, see_screen,
control_input, process_kill, kernel_events, mtaalamu_skills, mtaalamu_skill_run.
KANUNI: 1) Hesabu za kitaalamu tumia mtaalamu_skills/mtaalamu_skill_run, usihesabu mwenyewe. 2) Hatua hatari zitaulizwa kibali. 3) Jibu kwa Kiswahili fasaha mwishoni.
"""
