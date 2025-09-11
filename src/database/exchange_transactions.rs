use {
    crate::database::error::DatabaseError,
    chrono::{DateTime, Utc},
    sqlx::{FromRow, PgPool, Postgres},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxStatus {
    Pending,
    Succeeded,
    Failed,
}

impl TxStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TxStatus::Pending => "pending",
            TxStatus::Succeeded => "succeeded",
            TxStatus::Failed => "failed",
        }
    }
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
    pub status: String,
    pub failure_reason: Option<String>,
    pub tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub locked_at: Option<DateTime<Utc>>,
}

pub async fn upsert_new(
    postgres: &PgPool,
    id: &str,
    exchange_id: &str,
    project_id: Option<&str>,
    asset: Option<&str>,
    amount: Option<f64>,
    recipient: Option<&str>,
    pay_url: Option<&str>,
) -> Result<ExchangeTransaction, DatabaseError> {
    let query = r#"
        INSERT INTO exchange_transactions
            (id, exchange_id, project_id, asset, amount, recipient, pay_url, status, last_checked_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending', NOW())
        RETURNING id, exchange_id, project_id, asset, amount, recipient, pay_url, status,
                  failure_reason, tx_hash, created_at, updated_at, last_checked_at, completed_at, locked_at
    "#;

    let row = sqlx::query_as::<Postgres, ExchangeTransaction>(query)
        .bind(id)
        .bind(exchange_id)
        .bind(project_id)
        .bind(asset)
        .bind(amount)
        .bind(recipient)
        .bind(pay_url)
        .fetch_one(postgres)
        .await?;
    Ok(row)
}

pub async fn update_status(
    postgres: &PgPool,
    id: &str,
    status: TxStatus,
    tx_hash: Option<&str>,
    failure_reason: Option<&str>,
) -> Result<ExchangeTransaction, DatabaseError> {
    let (completed_at_set, failure_reason_bind) = match status {
        TxStatus::Succeeded | TxStatus::Failed => ("NOW()", failure_reason),
        TxStatus::Pending => ("NULL", None),
    };

    let query = format!(
        r#"
        UPDATE exchange_transactions SET
            status = $2,
            tx_hash = $3,
            failure_reason = $4,
            last_checked_at = NOW(),
            completed_at = {completed_at_set},
            updated_at = NOW(),
            locked_at = NULL
        WHERE id = $1
        RETURNING id, exchange_id, project_id, asset, amount, recipient, pay_url, status,
                  failure_reason, tx_hash, created_at, updated_at, last_checked_at, completed_at, locked_at
    "#
    );

    let row = sqlx::query_as::<Postgres, ExchangeTransaction>(&query)
        .bind(id)
        .bind(status.as_str())
        .bind(tx_hash)
        .bind(failure_reason_bind)
        .fetch_one(postgres)
        .await?;
    Ok(row)
}

pub async fn touch_non_terminal(postgres: &PgPool, id: &str) -> Result<(), DatabaseError> {
    let query = r#"
        UPDATE exchange_transactions SET
            last_checked_at = NOW(),
            updated_at = NOW(),
            locked_at = NULL
        WHERE id = $1
    "#;
    sqlx::query::<Postgres>(query)
        .bind(id)
        .execute(postgres)
        .await?;
    Ok(())
}

pub async fn claim_due_batch(
    postgres: &PgPool,
    max_claim: i64,
) -> Result<Vec<ExchangeTransaction>, DatabaseError> {
    let query = r#"
        WITH candidates AS (
            SELECT id FROM exchange_transactions
            WHERE status = 'pending'
              AND (locked_at IS NULL OR locked_at < NOW() - INTERVAL '15 minutes')
              AND (last_checked_at IS NULL OR last_checked_at < NOW() - INTERVAL '10 minutes')
            ORDER BY last_checked_at NULLS FIRST, created_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
        ), claimed AS (
            UPDATE exchange_transactions t
            SET locked_at = NOW(), updated_at = NOW()
            WHERE t.id IN (SELECT id FROM candidates)
            RETURNING t.*
        )
        SELECT * FROM claimed
    "#;

    let rows = sqlx::query_as::<Postgres, ExchangeTransaction>(query)
        .bind(max_claim)
        .fetch_all(postgres)
        .await?;
    Ok(rows)
}

pub async fn expire_old_pending(
    postgres: &PgPool,
    max_age_hours: i64,
) -> Result<u64, DatabaseError> {
    let query = r#"
        UPDATE exchange_transactions SET
            status = 'failed',
            failure_reason = COALESCE(failure_reason, 'expired'),
            completed_at = NOW(),
            updated_at = NOW()
        WHERE status = 'pending' AND created_at < NOW() - ($1 || ' hours')::INTERVAL
    "#;

    let res = sqlx::query::<Postgres>(query)
        .bind(max_age_hours)
        .execute(postgres)
        .await?;
    Ok(res.rows_affected())
}
