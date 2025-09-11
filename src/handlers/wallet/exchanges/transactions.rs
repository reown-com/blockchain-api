use {
    crate::{
        analytics::exchange_event_info::{ExchangeEventInfo, ExchangeEventType},
        database::{
            error::DatabaseError,
            exchange_transactions::{self, TxStatus},
        },
        state::AppState,
    },
    std::sync::Arc,
};

pub async fn create(
    state: &Arc<AppState>,
    id: &str,
    exchange_id: &str,
    project_id: &str,
    asset: &str,
    amount: f64,
    recipient: &str,
    pay_url: &str,
) -> Result<(), crate::database::error::DatabaseError> {
    let row = exchange_transactions::upsert_new(
        &state.postgres,
        id,
        exchange_id,
        Some(project_id),
        Some(asset),
        Some(amount),
        Some(recipient),
        Some(pay_url),
    )
    .await?;

    state.analytics.exchange_event(ExchangeEventInfo::new(
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
    ));

    Ok(())
}

pub async fn mark_succeeded(
    state: &Arc<AppState>,
    id: &str,
    tx_hash: Option<&str>,
) -> Result<(), DatabaseError> {
    let row = exchange_transactions::update_status(
        &state.postgres,
        id,
        TxStatus::Succeeded,
        tx_hash,
        None,
    )
    .await?;

    state.analytics.exchange_event(ExchangeEventInfo::new(
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
    ));

    Ok(())
}

pub async fn mark_failed(
    state: &Arc<AppState>,
    id: &str,
    failure_reason: Option<&str>,
    tx_hash: Option<&str>,
) -> Result<(), DatabaseError> {
    let row = exchange_transactions::update_status(
        &state.postgres,
        id,
        TxStatus::Failed,
        tx_hash,
        failure_reason,
    )
    .await?;

    state.analytics.exchange_event(ExchangeEventInfo::new(
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
    ));

    Ok(())
}

pub async fn touch_pending(
    state: &Arc<AppState>,
    id: &str,
) -> Result<(), crate::database::error::DatabaseError> {
    exchange_transactions::touch_non_terminal(&state.postgres, id).await
}
