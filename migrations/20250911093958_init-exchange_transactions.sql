-- Exchange transactions tracking table
CREATE TABLE IF NOT EXISTS exchange_transactions (
  id VARCHAR(64) PRIMARY KEY,
  exchange_id VARCHAR(64) NOT NULL,

  project_id VARCHAR(255),
  asset VARCHAR(255),
  amount DOUBLE PRECISION,
  recipient VARCHAR(255),
  pay_url TEXT,

  status VARCHAR(16) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'succeeded', 'failed')),
  failure_reason VARCHAR(64),
  tx_hash VARCHAR(255),

  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_checked_at TIMESTAMPTZ,
  completed_at TIMESTAMPTZ,

  locked_at TIMESTAMPTZ
);

-- Indexes to speed up reconciliation scans and locking
CREATE INDEX IF NOT EXISTS idx_exchange_tx_pending_due
  ON exchange_transactions (last_checked_at)
  WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS idx_exchange_tx_lock
  ON exchange_transactions (locked_at)
  WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS idx_exchange_tx_status_created
  ON exchange_transactions (status, created_at);

