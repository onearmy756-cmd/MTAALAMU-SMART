-- ============================================================
-- MTAALAMU SMART — trades.sql
-- Schema: trades, skills, problems, solutions, cases, learning
-- SQLite. Schema only — no seed data (seed comes from JSON).
-- ============================================================

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS trades (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  code          TEXT    NOT NULL UNIQUE,
  name_sw       TEXT    NOT NULL,
  name_en       TEXT    NOT NULL,
  icon          TEXT    NOT NULL DEFAULT '',
  color         TEXT    NOT NULL DEFAULT '#FF6B00',
  category      TEXT    NOT NULL DEFAULT 'general',
  active        INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
  meta_json     TEXT    NOT NULL DEFAULT '{}' CHECK (json_valid(meta_json)),
  created_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  updated_at    TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS skills (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  trade_id      INTEGER NOT NULL,
  name          TEXT    NOT NULL,
  name_en       TEXT    NOT NULL DEFAULT '',
  difficulty    INTEGER NOT NULL DEFAULT 1 CHECK (difficulty BETWEEN 1 AND 5),
  formula_id    TEXT,
  tools_json    TEXT    NOT NULL DEFAULT '[]' CHECK (json_valid(tools_json)),
  steps_json    TEXT    NOT NULL DEFAULT '[]' CHECK (json_valid(steps_json)),
  active        INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
  created_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  FOREIGN KEY (trade_id) REFERENCES trades (id) ON DELETE CASCADE,
  UNIQUE (trade_id, name)
);

CREATE TABLE IF NOT EXISTS problems (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  trade_id      INTEGER NOT NULL,
  code          TEXT    NOT NULL UNIQUE,
  description   TEXT    NOT NULL,
  description_en TEXT   NOT NULL DEFAULT '',
  symptoms_json TEXT    NOT NULL DEFAULT '[]' CHECK (json_valid(symptoms_json)),
  causes_json   TEXT    NOT NULL DEFAULT '{}' CHECK (json_valid(causes_json)),
  formula       TEXT,
  time_min      INTEGER NOT NULL DEFAULT 0 CHECK (time_min >= 0),
  cost_tzs      INTEGER NOT NULL DEFAULT 0 CHECK (cost_tzs >= 0),
  success_rate  REAL    NOT NULL DEFAULT 0.0 CHECK (success_rate BETWEEN 0.0 AND 1.0),
  severity      TEXT    NOT NULL DEFAULT 'medium'
                  CHECK (severity IN ('low', 'medium', 'high', 'critical')),
  active        INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
  meta_json     TEXT    NOT NULL DEFAULT '{}' CHECK (json_valid(meta_json)),
  created_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  updated_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  FOREIGN KEY (trade_id) REFERENCES trades (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS solutions (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  problem_id    INTEGER NOT NULL,
  trade_id      INTEGER NOT NULL,
  title_sw      TEXT    NOT NULL,
  title_en      TEXT    NOT NULL DEFAULT '',
  steps_json    TEXT    NOT NULL DEFAULT '[]' CHECK (json_valid(steps_json)),
  tools_json    TEXT    NOT NULL DEFAULT '[]' CHECK (json_valid(tools_json)),
  parts_json    TEXT    NOT NULL DEFAULT '[]' CHECK (json_valid(parts_json)),
  cost_tzs      INTEGER NOT NULL DEFAULT 0 CHECK (cost_tzs >= 0),
  time_min      INTEGER NOT NULL DEFAULT 0 CHECK (time_min >= 0),
  success_rate  REAL    NOT NULL DEFAULT 0.0 CHECK (success_rate BETWEEN 0.0 AND 1.0),
  priority      INTEGER NOT NULL DEFAULT 1 CHECK (priority >= 1),
  standards_json TEXT   NOT NULL DEFAULT '[]' CHECK (json_valid(standards_json)),
  created_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  FOREIGN KEY (problem_id) REFERENCES problems (id) ON DELETE CASCADE,
  FOREIGN KEY (trade_id)   REFERENCES trades (id)    ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS cases (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  case_code     TEXT    NOT NULL UNIQUE,
  problem_id    INTEGER,
  solution_id   INTEGER,
  trade_id      INTEGER,
  customer_name TEXT    NOT NULL DEFAULT '',
  customer_phone TEXT   NOT NULL DEFAULT '',
  region        TEXT    NOT NULL DEFAULT '',
  status        TEXT    NOT NULL DEFAULT 'open'
                  CHECK (status IN ('open', 'diagnosing', 'in_progress',
                                    'solved', 'failed', 'cancelled')),
  severity      TEXT    NOT NULL DEFAULT 'medium'
                  CHECK (severity IN ('low', 'medium', 'high', 'critical')),
  confidence    REAL    NOT NULL DEFAULT 0.0 CHECK (confidence BETWEEN 0.0 AND 1.0),
  cost_tzs      INTEGER NOT NULL DEFAULT 0 CHECK (cost_tzs >= 0),
  time_min      INTEGER NOT NULL DEFAULT 0 CHECK (time_min >= 0),
  symptoms_json TEXT    NOT NULL DEFAULT '[]' CHECK (json_valid(symptoms_json)),
  evidence_json TEXT    NOT NULL DEFAULT '{}' CHECK (json_valid(evidence_json)),
  opened_at     TEXT    NOT NULL DEFAULT (datetime('now')),
  closed_at     TEXT,
  FOREIGN KEY (problem_id)  REFERENCES problems (id)  ON DELETE SET NULL,
  FOREIGN KEY (solution_id) REFERENCES solutions (id) ON DELETE SET NULL,
  FOREIGN KEY (trade_id)    REFERENCES trades (id)    ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS learning (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  case_id       INTEGER,
  problem_id    INTEGER,
  solution_id   INTEGER,
  trade_id      INTEGER,
  event         TEXT    NOT NULL
                  CHECK (event IN ('symptom_observed', 'cause_confirmed',
                                    'solution_applied', 'solution_failed',
                                    'feedback', 'correction')),
  prior_json    TEXT    NOT NULL DEFAULT '{}' CHECK (json_valid(prior_json)),
  posterior_json TEXT   NOT NULL DEFAULT '{}' CHECK (json_valid(posterior_json)),
  weights_json  TEXT    NOT NULL DEFAULT '{}' CHECK (json_valid(weights_json)),
  delta         REAL    NOT NULL DEFAULT 0.0,
  source        TEXT    NOT NULL DEFAULT 'agent'
                  CHECK (source IN ('agent', 'human', 'import', 'test')),
  note          TEXT    NOT NULL DEFAULT '',
  created_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  FOREIGN KEY (case_id)     REFERENCES cases (id)     ON DELETE CASCADE,
  FOREIGN KEY (problem_id)  REFERENCES problems (id)  ON DELETE SET NULL,
  FOREIGN KEY (solution_id) REFERENCES solutions (id) ON DELETE SET NULL,
  FOREIGN KEY (trade_id)    REFERENCES trades (id)    ON DELETE SET NULL
);

-- ---------- indexes ----------
CREATE INDEX IF NOT EXISTS idx_skills_trade       ON skills (trade_id);
CREATE INDEX IF NOT EXISTS idx_problems_trade     ON problems (trade_id);
CREATE INDEX IF NOT EXISTS idx_problems_severity  ON problems (severity);
CREATE INDEX IF NOT EXISTS idx_solutions_problem  ON solutions (problem_id);
CREATE INDEX IF NOT EXISTS idx_solutions_trade    ON solutions (trade_id);
CREATE INDEX IF NOT EXISTS idx_cases_status       ON cases (status);
CREATE INDEX IF NOT EXISTS idx_cases_problem      ON cases (problem_id);
CREATE INDEX IF NOT EXISTS idx_cases_opened       ON cases (opened_at);
CREATE INDEX IF NOT EXISTS idx_learning_case      ON learning (case_id);
CREATE INDEX IF NOT EXISTS idx_learning_event     ON learning (event);
CREATE INDEX IF NOT EXISTS idx_learning_trade     ON learning (trade_id);
