use {
    crate::{
        analytics::exchange_event_info::{ExchangeEventInfo, ExchangeEventType},
        database::{
            error::DatabaseError,
            exchange_transactions::{self, NewExchangeTransaction, TxStatus},
        },
        state::AppState,
    },
    std::sync::Arc,
};

pub async fn create(
    state: &Arc<AppState>,
    args: NewExchangeTransaction<'_>,
) -> Result<(), DatabaseError> {
    let mut db_tx = state.postgres.begin().await?;
    let row = exchange_transactions::upsert_new(&mut *db_tx, args).await?;

    state
    .analytics
    .exchange_transaction_event(ExchangeEventInfo::new(
        ExchangeEventType::Started,
        row.id,
        row.exchange_id,
        row.project_id,
        row.asset,
        row.amount,
        row.recipient,
        row.pay_url,
        None,
        None,
    )).map_err(|e| DatabaseError::BadArgument(e.to_string()))?;     
    db_tx.commit().await?;
    Ok(())
}

pub async fn mark_succeeded(
    state: &Arc<AppState>,
    id: &str,
    tx_hash: Option<&str>,
) -> Result<(), DatabaseError> {
    let mut db_tx = state.postgres.begin().await?;
    let row = exchange_transactions::update_status(
        &mut *db_tx,
        exchange_transactions::UpdateExchangeStatus {
            id,
            status: TxStatus::Succeeded,
            tx_hash,
            failure_reason: None,
        },
    )
    .await?;

    state
        .analytics
        .exchange_transaction_event(ExchangeEventInfo::new(
            ExchangeEventType::Completed,
            row.id,
            row.exchange_id,
            row.project_id,
            row.asset,
            row.amount,
            row.recipient,
            row.pay_url,
            tx_hash.map(|s| s.to_string()),
            None,
        )).map_err(|e| DatabaseError::BadArgument(e.to_string()))?;
    db_tx.commit().await?;
    Ok(())
}

pub async fn mark_failed(
    state: &Arc<AppState>,
    id: &str,
    failure_reason: Option<&str>,
    tx_hash: Option<&str>,
) -> Result<(), DatabaseError> {
    let mut db_tx = state.postgres.begin().await?;
    let row = exchange_transactions::update_status(
        &mut *db_tx,
        exchange_transactions::UpdateExchangeStatus {
            id,
            status: TxStatus::Failed,
            tx_hash,
            failure_reason,
        },
    )
    .await?;

    state
        .analytics
        .exchange_transaction_event(ExchangeEventInfo::new(
            ExchangeEventType::Failed,
            row.id,
            row.exchange_id,
            row.project_id,
            row.asset,
            row.amount,
            row.recipient,
            row.pay_url,
            tx_hash.map(|s| s.to_string()),
            row.failure_reason,
        )).map_err(|e| DatabaseError::BadArgument(e.to_string()))?;
    db_tx.commit().await?;
    Ok(())
}

pub async fn touch_pending(
    state: &Arc<AppState>,
    id: &str,
) -> Result<(), crate::database::error::DatabaseError> {
    let mut db_tx = state.postgres.begin().await?;
    exchange_transactions::touch_non_terminal(&mut *db_tx, id).await?;
    db_tx.commit().await?;
    Ok(())
}
