use {
    crate::{
        analytics::exchange_event_info::{ExchangeEventInfo, ExchangeEventType},
        database::{
            error::DatabaseError,
            exchange_reconciliation::{
                self as exchange_transactions, NewExchangeTransaction, TxStatus,
            },
        },
        metrics::ExchangeReconciliationQueryType,
        state::AppState,
    },
    std::{sync::Arc, time::Instant},
};

pub async fn create(
    state: &Arc<AppState>,
    args: NewExchangeTransaction<'_>,
) -> Result<(), DatabaseError> {
    let mut db_tx = state.postgres.begin().await?;
    let q_start = Instant::now();
    let row = exchange_transactions::insert_new(&mut *db_tx, args).await?;
    state.metrics.add_exchange_reconciliation_query_latency(
        ExchangeReconciliationQueryType::InsertNew,
        q_start,
    );

    state
        .analytics
        .exchange_transaction_event(ExchangeEventInfo::new(
            ExchangeEventType::Started,
            row.session_id,
            row.exchange_id,
            row.project_id,
            row.asset,
            row.amount,
            row.recipient,
            row.pay_url,
            None,
            None,
        ))
        .map_err(|e| DatabaseError::BadArgument(e.to_string()))?;
    db_tx.commit().await?;
    Ok(())
}

pub async fn mark_succeeded(
    state: &Arc<AppState>,
    session_id: &str,
    tx_hash: Option<&str>,
) -> Result<(), DatabaseError> {
    let mut db_tx = state.postgres.begin().await?;
    let q_start = Instant::now();
    let row = exchange_transactions::update_status(
        &mut *db_tx,
        exchange_transactions::UpdateExchangeStatus {
            session_id,
            status: TxStatus::Succeeded,
            tx_hash,
            failure_reason: None,
        },
    )
    .await?;
    state.metrics.add_exchange_reconciliation_query_latency(
        ExchangeReconciliationQueryType::UpdateStatus,
        q_start,
    );

    state
        .analytics
        .exchange_transaction_event(ExchangeEventInfo::new(
            ExchangeEventType::Completed,
            row.session_id,
            row.exchange_id,
            row.project_id,
            row.asset,
            row.amount,
            row.recipient,
            row.pay_url,
            tx_hash.map(|s| s.to_string()),
            None,
        ))
        .map_err(|e| DatabaseError::BadArgument(e.to_string()))?;
    db_tx.commit().await?;
    Ok(())
}

pub async fn mark_failed(
    state: &Arc<AppState>,
    session_id: &str,
    failure_reason: Option<&str>,
    tx_hash: Option<&str>,
) -> Result<(), DatabaseError> {
    let mut db_tx = state.postgres.begin().await?;
    let q_start = Instant::now();
    let row = exchange_transactions::update_status(
        &mut *db_tx,
        exchange_transactions::UpdateExchangeStatus {
            session_id,
            status: TxStatus::Failed,
            tx_hash,
            failure_reason,
        },
    )
    .await?;
    state.metrics.add_exchange_reconciliation_query_latency(
        ExchangeReconciliationQueryType::UpdateStatus,
        q_start,
    );

    state
        .analytics
        .exchange_transaction_event(ExchangeEventInfo::new(
            ExchangeEventType::Failed,
            row.session_id,
            row.exchange_id,
            row.project_id,
            row.asset,
            row.amount,
            row.recipient,
            row.pay_url,
            tx_hash.map(|s| s.to_string()),
            row.failure_reason,
        ))
        .map_err(|e| DatabaseError::BadArgument(e.to_string()))?;
    db_tx.commit().await?;
    Ok(())
}

pub async fn touch_pending(
    state: &Arc<AppState>,
    session_id: &str,
) -> Result<(), crate::database::error::DatabaseError> {
    let mut db_tx = state.postgres.begin().await?;
    let q_start = Instant::now();
    exchange_transactions::touch_non_terminal(&mut *db_tx, session_id).await?;
    state.metrics.add_exchange_reconciliation_query_latency(
        ExchangeReconciliationQueryType::TouchNonTerminal,
        q_start,
    );
    db_tx.commit().await?;
    Ok(())
}
