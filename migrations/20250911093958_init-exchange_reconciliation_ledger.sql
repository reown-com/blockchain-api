-- Status enum for reconciliation rows
DO $$ BEGIN
    CREATE TYPE exchange_transaction_status AS ENUM ('pending', 'succeeded', 'failed');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

-- Exchange transactions tracking table
CREATE TABLE IF NOT EXISTS exchange_reconciliation_ledger (
  id CHAR(32) PRIMARY KEY,
  exchange_id VARCHAR(64) NOT NULL,

  project_id VARCHAR(255),
  asset VARCHAR(255),
  amount DOUBLE PRECISION,
  recipient VARCHAR(255),
  pay_url TEXT,

  status exchange_transaction_status NOT NULL DEFAULT 'pending',
  failure_reason VARCHAR(64),
  tx_hash VARCHAR(255),

  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_checked_at TIMESTAMPTZ,
  completed_at TIMESTAMPTZ,

  locked_at TIMESTAMPTZ
);

-- Indexes to speed up reconciliation scans and locking
CREATE INDEX IF NOT EXISTS idx_exchange_recon_pending_due
  ON exchange_reconciliation_ledger (last_checked_at)
  WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS idx_exchange_recon_lock
  ON exchange_reconciliation_ledger (locked_at)
  WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS idx_exchange_recon_status_created
  ON exchange_reconciliation_ledger (status, created_at);

CREATE INDEX IF NOT EXISTS idx_exchange_recon_pending_due_composite
  ON exchange_reconciliation_ledger (last_checked_at, created_at)
  WHERE status = 'pending';

