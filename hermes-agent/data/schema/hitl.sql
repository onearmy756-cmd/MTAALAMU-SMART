-- ============================================================
-- MTAALAMU SMART — hitl.sql
-- Human-In-The-Loop approval requests (N-148…N-150)
-- SQLite. Schema only — no seed data.
-- Status vocabulary: pending | approved | rejected | modified | timeout
-- ============================================================

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS hitl_requests (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  request_id      TEXT    NOT NULL UNIQUE,
  case_id         INTEGER,
  job_id          INTEGER,
  agent_step      TEXT    NOT NULL DEFAULT 'hitl',
  approval_type   TEXT    NOT NULL DEFAULT 'solution'
                    CHECK (approval_type IN ('solution', 'payment', 'action',
                                             'refund', 'remote_access', 'emergency')),
  status          TEXT    NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending', 'approved', 'rejected',
                                      'modified', 'timeout')),
  title           TEXT    NOT NULL,
  description     TEXT    NOT NULL DEFAULT '',
  proposal_json   TEXT    NOT NULL DEFAULT '{}' CHECK (json_valid(proposal_json)),
  original_json   TEXT    NOT NULL DEFAULT '{}' CHECK (json_valid(original_json)),
  modified_json   TEXT    NOT NULL DEFAULT '{}' CHECK (json_valid(modified_json)),
  decision_note   TEXT    NOT NULL DEFAULT '',
  reject_reason   TEXT    NOT NULL DEFAULT '',
  confidence      REAL    NOT NULL DEFAULT 0.0 CHECK (confidence BETWEEN 0.0 AND 1.0),
  risk_level      TEXT    NOT NULL DEFAULT 'medium'
                    CHECK (risk_level IN ('low', 'medium', 'high', 'critical')),
  customer_phone  TEXT    NOT NULL DEFAULT '',
  assigned_to     TEXT    NOT NULL DEFAULT '',
  attempts        INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
  timeout_sec     INTEGER NOT NULL DEFAULT 300 CHECK (timeout_sec >= 0),
  created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
  responded_at    TEXT,
  expires_at      TEXT
);

-- 3 indexes (N-148…N-150)
CREATE INDEX IF NOT EXISTS idx_hitl_status         ON hitl_requests (status);
CREATE INDEX IF NOT EXISTS idx_hitl_approval_type   ON hitl_requests (approval_type);
CREATE INDEX IF NOT EXISTS idx_hitl_status_type     ON hitl_requests (status, approval_type);
