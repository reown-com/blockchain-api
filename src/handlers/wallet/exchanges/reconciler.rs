use {
    super::{
        binance::BinanceExchange,
        coinbase::CoinbaseExchange,
        transactions::{mark_failed, mark_succeeded, touch_pending},
        ExchangeType, GetBuyStatusParams,
    },
    crate::{
        database::exchange_transactions as db, handlers::wallet::exchanges::BuyTransactionStatus,
        state::AppState,
    },
    axum::extract::State,
    std::{sync::Arc, time::Duration},
    tokio::time::{interval, MissedTickBehavior},
    tracing::{debug, warn},
};

const POLL_INTERVAL: Duration = Duration::from_secs(600); // 10 minutes

const CLAIM_BATCH_SIZE: i64 = 200;
const EXPIRE_PENDING_AFTER_HOURS: i64 = 12;

pub async fn run(state: Arc<AppState>) {
    debug!("starting");
    let mut poll = interval(POLL_INTERVAL);
    poll.set_missed_tick_behavior(MissedTickBehavior::Delay);
    loop {
        poll.tick().await;
        debug!("polling new batch");
        let fetch_started = std::time::SystemTime::now();
        match db::claim_due_batch(&state.postgres, CLAIM_BATCH_SIZE).await {
            Ok(mut rows) => {
                state
                    .metrics
                    .add_exchange_reconciler_fetch_batch_latency(fetch_started);
                debug!("fetched {} exchange transactions", rows.len());
                if rows.is_empty() {
                    continue;
                }
                debug!("processing {} exchange transactions", rows.len());
                let mut rate = interval(Duration::from_millis(200));
                rate.set_missed_tick_behavior(MissedTickBehavior::Delay);

                let process_started = std::time::SystemTime::now();
                for row in rows.drain(..) {
                    rate.tick().await;

                    let exchange_id = row.exchange_id.as_str();
                    let internal_id = row.id.clone();
                    debug!(
                        "processing exchange transaction {} on {}",
                        internal_id, exchange_id
                    );
                    let res = match ExchangeType::from_id(exchange_id) {
                        Some(ExchangeType::Coinbase) => {
                            CoinbaseExchange
                                .get_buy_status(
                                    State(state.clone()),
                                    GetBuyStatusParams {
                                        session_id: internal_id.clone(),
                                    },
                                )
                                .await
                        }
                        Some(ExchangeType::Binance) => {
                            BinanceExchange
                                .get_buy_status(
                                    State(state.clone()),
                                    GetBuyStatusParams {
                                        session_id: internal_id.clone(),
                                    },
                                )
                                .await
                        }
                        _ => {
                            warn!(exchange_id, "unknown exchange id for reconciliation");
                            continue;
                        }
                    };

                    match res {
                        Ok(status) => match status.status {
                            BuyTransactionStatus::Success => {
                                debug!(
                                    exchange_id,
                                    internal_id, "marking transaction as succeeded"
                                );
                                let _ =
                                    mark_succeeded(&state, &internal_id, status.tx_hash.as_deref())
                                        .await;
                            }
                            BuyTransactionStatus::Failed => {
                                debug!(exchange_id, internal_id, "marking transaction as failed");
                                let _ = mark_failed(
                                    &state,
                                    &internal_id,
                                    Some("provider_failed"),
                                    status.tx_hash.as_deref(),
                                )
                                .await;
                            }
                            _ => {
                                let _ = touch_pending(&state, &internal_id).await;
                            }
                        },
                        Err(err) => {
                            debug!(exchange_id, internal_id, error = %err, "reconciler provider check failed");
                            let _ = touch_pending(&state, &internal_id).await;
                        }
                    }
                }

                state
                    .metrics
                    .add_exchange_reconciler_process_batch_latency(process_started);
                let _ = db::expire_old_pending(&state.postgres, EXPIRE_PENDING_AFTER_HOURS).await;
            }
            Err(e) => {
                warn!(error = %e, "failed to claim exchange transactions");
            }
        }
    }
}
