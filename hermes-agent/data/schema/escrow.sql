-- ============================================================
-- MTAALAMU SMART — escrow.sql
-- Escrow & payments (D-118…D-119)
-- Tables: clients, technicians, jobs, escrow, transactions, disputes
-- SQLite. Schema only — no seed data.
-- Job status: pending | paid | in_progress | done | disputed | cancelled
-- Escrow status: held | released | refunded | disputed
-- ============================================================

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS clients (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  client_code   TEXT    NOT NULL UNIQUE,
  name          TEXT    NOT NULL,
  phone         TEXT    NOT NULL UNIQUE,
  email         TEXT    NOT NULL DEFAULT '',
  region        TEXT    NOT NULL DEFAULT '',
  zone          TEXT    NOT NULL DEFAULT '',
  language      TEXT    NOT NULL DEFAULT 'sw'
                  CHECK (language IN ('sw', 'en')),
  kyc_status    TEXT    NOT NULL DEFAULT 'pending'
                  CHECK (kyc_status IN ('pending', 'verified', 'rejected')),
  active        INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
  meta_json     TEXT    NOT NULL DEFAULT '{}' CHECK (json_valid(meta_json)),
  created_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  updated_at    TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS technicians (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  tech_code     TEXT    NOT NULL UNIQUE,
  name          TEXT    NOT NULL,
  phone         TEXT    NOT NULL UNIQUE,
  email         TEXT    NOT NULL DEFAULT '',
  trade         TEXT    NOT NULL DEFAULT '',
  region        TEXT    NOT NULL DEFAULT '',
  rating        REAL    NOT NULL DEFAULT 0.0 CHECK (rating BETWEEN 0.0 AND 5.0),
  jobs_done     INTEGER NOT NULL DEFAULT 0 CHECK (jobs_done >= 0),
  tier          TEXT    NOT NULL DEFAULT 'solo'
                  CHECK (tier IN ('solo', 'personal', 'pro', 'premium',
                                  'business', 'enterprise')),
  verified      INTEGER NOT NULL DEFAULT 0 CHECK (verified IN (0, 1)),
  active        INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
  skills_json   TEXT    NOT NULL DEFAULT '[]' CHECK (json_valid(skills_json)),
  meta_json     TEXT    NOT NULL DEFAULT '{}' CHECK (json_valid(meta_json)),
  created_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  updated_at    TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS jobs (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  job_code      TEXT    NOT NULL UNIQUE,
  client_id     INTEGER NOT NULL,
  technician_id INTEGER,
  trade         TEXT    NOT NULL DEFAULT '',
  title         TEXT    NOT NULL,
  description   TEXT    NOT NULL DEFAULT '',
  region        TEXT    NOT NULL DEFAULT '',
  amount_tzs    INTEGER NOT NULL DEFAULT 0 CHECK (amount_tzs >= 0),
  commission_tzs INTEGER NOT NULL DEFAULT 0 CHECK (commission_tzs >= 0),
  fee_tzs       INTEGER NOT NULL DEFAULT 0 CHECK (fee_tzs >= 0),
  total_tzs     INTEGER NOT NULL DEFAULT 0 CHECK (total_tzs >= 0),
  status        TEXT    NOT NULL DEFAULT 'pending'
                  CHECK (status IN ('pending', 'paid', 'in_progress', 'done',
                                    'disputed', 'cancelled')),
  priority      TEXT    NOT NULL DEFAULT 'normal'
                  CHECK (priority IN ('low', 'normal', 'high', 'urgent')),
  remote        INTEGER NOT NULL DEFAULT 0 CHECK (remote IN (0, 1)),
  scheduled_at  TEXT,
  started_at    TEXT,
  completed_at  TEXT,
  created_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  updated_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  meta_json     TEXT    NOT NULL DEFAULT '{}' CHECK (json_valid(meta_json)),
  FOREIGN KEY (client_id)     REFERENCES clients (id)     ON DELETE RESTRICT,
  FOREIGN KEY (technician_id) REFERENCES technicians (id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS escrow (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  escrow_code   TEXT    NOT NULL UNIQUE,
  job_id        INTEGER NOT NULL,
  client_id     INTEGER NOT NULL,
  technician_id INTEGER,
  amount_tzs    INTEGER NOT NULL DEFAULT 0 CHECK (amount_tzs >= 0),
  fee_tzs       INTEGER NOT NULL DEFAULT 0 CHECK (fee_tzs >= 0),
  net_tzs       INTEGER NOT NULL DEFAULT 0 CHECK (net_tzs >= 0),
  currency      TEXT    NOT NULL DEFAULT 'TZS',
  status        TEXT    NOT NULL DEFAULT 'held'
                  CHECK (status IN ('held', 'released', 'refunded', 'disputed')),
  hold_reason   TEXT    NOT NULL DEFAULT '',
  released_at   TEXT,
  refunded_at   TEXT,
  created_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  updated_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  meta_json     TEXT    NOT NULL DEFAULT '{}' CHECK (json_valid(meta_json)),
  FOREIGN KEY (job_id)        REFERENCES jobs (id)        ON DELETE RESTRICT,
  FOREIGN KEY (client_id)     REFERENCES clients (id)     ON DELETE RESTRICT,
  FOREIGN KEY (technician_id) REFERENCES technicians (id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS transactions (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  txn_code      TEXT    NOT NULL UNIQUE,
  escrow_id     INTEGER,
  job_id        INTEGER,
  client_id     INTEGER,
  technician_id INTEGER,
  type          TEXT    NOT NULL
                  CHECK (type IN ('deposit', 'hold', 'release', 'refund',
                                  'commission', 'fee', 'topup', 'withdrawal')),
  method        TEXT    NOT NULL DEFAULT 'M-Pesa'
                  CHECK (method IN ('M-Pesa', 'Tigo Pesa', 'Airtel Money',
                                    'Halopesa', 'Bank', 'Card', 'Wallet')),
  amount_tzs    INTEGER NOT NULL DEFAULT 0 CHECK (amount_tzs >= 0),
  fee_tzs       INTEGER NOT NULL DEFAULT 0 CHECK (fee_tzs >= 0),
  vat_tzs       INTEGER NOT NULL DEFAULT 0 CHECK (vat_tzs >= 0),
  status        TEXT    NOT NULL DEFAULT 'pending'
                  CHECK (status IN ('pending', 'completed', 'failed', 'reversed')),
  reference     TEXT    NOT NULL DEFAULT '',
  phone         TEXT    NOT NULL DEFAULT '',
  error_message TEXT    NOT NULL DEFAULT '',
  created_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  confirmed_at  TEXT,
  meta_json     TEXT    NOT NULL DEFAULT '{}' CHECK (json_valid(meta_json)),
  FOREIGN KEY (escrow_id)     REFERENCES escrow (id)      ON DELETE SET NULL,
  FOREIGN KEY (job_id)        REFERENCES jobs (id)        ON DELETE SET NULL,
  FOREIGN KEY (client_id)     REFERENCES clients (id)     ON DELETE SET NULL,
  FOREIGN KEY (technician_id) REFERENCES technicians (id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS disputes (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  dispute_code  TEXT    NOT NULL UNIQUE,
  job_id        INTEGER NOT NULL,
  escrow_id     INTEGER,
  opened_by     TEXT    NOT NULL DEFAULT 'client'
                  CHECK (opened_by IN ('client', 'technician', 'admin')),
  reason        TEXT    NOT NULL
                  CHECK (reason IN ('not_done', 'poor_quality', 'no_show',
                                    'overcharged', 'damage', 'late', 'other')),
  status        TEXT    NOT NULL DEFAULT 'open'
                  CHECK (status IN ('open', 'investigating', 'resolved',
                                    'escalated', 'closed')),
  amount_tzs    INTEGER NOT NULL DEFAULT 0 CHECK (amount_tzs >= 0),
  resolution    TEXT    NOT NULL DEFAULT '',
  outcome       TEXT    NOT NULL DEFAULT ''
                  CHECK (outcome IN ('', 'refunded', 'released', 'split')),
  evidence_json TEXT    NOT NULL DEFAULT '[]' CHECK (json_valid(evidence_json)),
  opened_at     TEXT    NOT NULL DEFAULT (datetime('now')),
  resolved_at   TEXT,
  meta_json     TEXT    NOT NULL DEFAULT '{}' CHECK (json_valid(meta_json)),
  FOREIGN KEY (job_id)    REFERENCES jobs (id)    ON DELETE RESTRICT,
  FOREIGN KEY (escrow_id) REFERENCES escrow (id)  ON DELETE SET NULL
);

-- ---------- indexes ----------
CREATE INDEX IF NOT EXISTS idx_jobs_client      ON jobs (client_id);
CREATE INDEX IF NOT EXISTS idx_jobs_technician  ON jobs (technician_id);
CREATE INDEX IF NOT EXISTS idx_jobs_status      ON jobs (status);
CREATE INDEX IF NOT EXISTS idx_jobs_created     ON jobs (created_at);
CREATE INDEX IF NOT EXISTS idx_escrow_job       ON escrow (job_id);
CREATE INDEX IF NOT EXISTS idx_escrow_status    ON escrow (status);
CREATE INDEX IF NOT EXISTS idx_txn_escrow       ON transactions (escrow_id);
CREATE INDEX IF NOT EXISTS idx_txn_status       ON transactions (status);
CREATE INDEX IF NOT EXISTS idx_txn_created      ON transactions (created_at);
CREATE INDEX IF NOT EXISTS idx_disputes_job     ON disputes (job_id);
CREATE INDEX IF NOT EXISTS idx_disputes_status  ON disputes (status);
