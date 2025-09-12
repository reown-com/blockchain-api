use {
    crate::database::error::DatabaseError,
    chrono::{DateTime, Utc},
    sqlx::{FromRow, PgExecutor, Postgres},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "exchange_transaction_status", rename_all = "lowercase")]
pub enum TxStatus {
    Pending,
    Succeeded,
    Failed,
}

#[derive(Debug, FromRow, Clone)]
pub struct ExchangeTransaction {
    pub id: String,
    pub exchange_id: String,
    pub project_id: Option<String>,
    pub asset: Option<String>,
    pub amount: Option<f64>,
    pub recipient: Option<String>,
    pub pay_url: Option<String>,
    pub status: TxStatus,
    pub failure_reason: Option<String>,
    pub tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub locked_at: Option<DateTime<Utc>>,
}

pub struct NewExchangeTransaction<'a> {
    pub id: &'a str,
    pub exchange_id: &'a str,
    pub project_id: Option<&'a str>,
    pub asset: Option<&'a str>,
    pub amount: Option<f64>,
    pub recipient: Option<&'a str>,
    pub pay_url: Option<&'a str>,
}

pub async fn insert_new(
    executor: impl PgExecutor<'_>,
    tx: NewExchangeTransaction<'_>,
) -> Result<ExchangeTransaction, DatabaseError> {
    let query = r#"
        INSERT INTO exchange_reconciliation_ledger
            (id, exchange_id, project_id, asset, amount, recipient, pay_url)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, exchange_id, project_id, asset, amount, recipient, pay_url, status,
                  failure_reason, tx_hash, created_at, updated_at, last_checked_at, completed_at, locked_at
    "#;

    let row = sqlx::query_as::<Postgres, ExchangeTransaction>(query)
        .bind(tx.id)
        .bind(tx.exchange_id)
        .bind(tx.project_id)
        .bind(tx.asset)
        .bind(tx.amount)
        .bind(tx.recipient)
        .bind(tx.pay_url)
        .fetch_one(executor)
        .await?;
    Ok(row)
}

pub struct UpdateExchangeStatus<'a> {
    pub id: &'a str,
    pub status: TxStatus,
    pub tx_hash: Option<&'a str>,
    pub failure_reason: Option<&'a str>,
}

pub async fn update_status(
    executor: impl PgExecutor<'_>,
    tx: UpdateExchangeStatus<'_>,
) -> Result<ExchangeTransaction, DatabaseError> {
    let query = r#"
        UPDATE exchange_reconciliation_ledger SET
            status = $2,
            tx_hash = $3,
            failure_reason = $4,
            last_checked_at = NOW(),
            completed_at = CASE WHEN $2 IN ('succeeded','failed') THEN NOW() ELSE NULL END,
            updated_at = NOW(),
            locked_at = NULL
        WHERE id = $1
        RETURNING id, exchange_id, project_id, asset, amount, recipient, pay_url, status,
                  failure_reason, tx_hash, created_at, updated_at, last_checked_at, completed_at, locked_at
    "#;

    let row = sqlx::query_as::<Postgres, ExchangeTransaction>(query)
        .bind(tx.id)
        .bind(tx.status)
        .bind(tx.tx_hash)
        .bind(tx.failure_reason)
        .fetch_one(executor)
        .await?;
    Ok(row)
}

pub async fn touch_non_terminal(
    executor: impl PgExecutor<'_>,
    id: &str,
) -> Result<(), DatabaseError> {
    let query = r#"
        UPDATE exchange_reconciliation_ledger SET
            last_checked_at = NOW(),
            updated_at = NOW(),
            locked_at = NULL
        WHERE id = $1
    "#;
    sqlx::query::<Postgres>(query)
        .bind(id)
        .execute(executor)
        .await?;
    Ok(())
}

pub async fn claim_due_batch(
    executor: impl PgExecutor<'_>,
    max_claim: i64,
) -> Result<Vec<ExchangeTransaction>, DatabaseError> {
    let query = r#"
        WITH candidates AS (
            SELECT id FROM exchange_reconciliation_ledger
            WHERE status = 'pending'
              AND (locked_at IS NULL OR locked_at < NOW() - INTERVAL '15 minutes')
              AND (last_checked_at IS NULL OR last_checked_at < NOW() - INTERVAL '5 minutes')
              AND created_at < NOW() - INTERVAL '3 hours'
            ORDER BY last_checked_at NULLS FIRST, created_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
        ), claimed AS (
            UPDATE exchange_reconciliation_ledger t
            SET locked_at = NOW(), updated_at = NOW()
            WHERE t.id IN (SELECT id FROM candidates)
            RETURNING t.*
        )
        SELECT * FROM claimed
    "#;

    let rows = sqlx::query_as::<Postgres, ExchangeTransaction>(query)
        .bind(max_claim)
        .fetch_all(executor)
        .await?;
    Ok(rows)
}

pub async fn expire_old_pending(
    executor: impl PgExecutor<'_>,
    max_age_hours: i64,
) -> Result<u64, DatabaseError> {
    let query = r#"
        UPDATE exchange_reconciliation_ledger SET
            status = 'failed'::exchange_transaction_status,
            failure_reason = COALESCE(failure_reason, 'expired'),
            completed_at = NOW(),
            updated_at = NOW()
        WHERE status = 'pending'
          AND created_at < NOW() - ($1 || ' hours')::INTERVAL
          AND (locked_at IS NULL OR locked_at < NOW() - INTERVAL '20 minutes')
    "#;

    let res = sqlx::query::<Postgres>(query)
        .bind(max_age_hours)
        .execute(executor)
        .await?;
    Ok(res.rows_affected())
}
