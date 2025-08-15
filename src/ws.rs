use {
    async_tungstenite::{tokio::ConnectStream, tungstenite, WebSocketStream},
    axum::extract::ws::{Message as AxumWsMessage, WebSocket},
    futures_util::{SinkExt, StreamExt},
    tracing::log::debug,
};

#[tracing::instrument(skip(client_ws, provider_ws), level = "debug")]
pub async fn proxy(
    project_id: String,
    client_ws: WebSocket,
    provider_ws: WebSocketStream<ConnectStream>,
) {
    let (mut client_ws_sender, mut client_ws_receiver) = client_ws.split();
    let (mut provider_ws_sender, mut provider_ws_receiver) = provider_ws.split();

    // Relay: client -> provider
    let write = async {
        while let Some(Ok(msg)) = client_ws_receiver.next().await {
            let tmsg = match msg {
                AxumWsMessage::Text(s) => tungstenite::Message::Text(s),
                AxumWsMessage::Binary(b) => tungstenite::Message::Binary(b.to_vec()),
                AxumWsMessage::Ping(b) => tungstenite::Message::Ping(b.to_vec()),
                AxumWsMessage::Pong(b) => tungstenite::Message::Pong(b.to_vec()),
                AxumWsMessage::Close(frame) => {
                    tungstenite::Message::Close(frame.map(|f| tungstenite::protocol::CloseFrame {
                        code: f.code.into(),
                        reason: f.reason,
                    }))
                }
            };
            if provider_ws_sender.send(tmsg).await.is_err() {
                break;
            }
        }
    };

    // Relay: provider -> client
    let read = async {
        while let Some(Ok(msg)) = provider_ws_receiver.next().await {
            let amsg = match msg {
                tungstenite::Message::Text(s) => AxumWsMessage::Text(s),
                tungstenite::Message::Binary(b) => AxumWsMessage::Binary(b),
                tungstenite::Message::Ping(b) => AxumWsMessage::Ping(b),
                tungstenite::Message::Pong(b) => AxumWsMessage::Pong(b),
                tungstenite::Message::Close(frame) => {
                    AxumWsMessage::Close(frame.map(|f| axum::extract::ws::CloseFrame {
                        code: f.code.into(),
                        reason: f.reason,
                    }))
                }
                tungstenite::Message::Frame(_) => continue,
            };
            if client_ws_sender.send(amsg).await.is_err() {
                break;
            }
        }
    };
    tokio::select! {
        _ = read => debug!("WebSocket relaying messages to the provider for client {project_id} died."),
        _ = write => debug!("WebSocket relaying messages from the provider to the client {project_id} died."),
    }
}
