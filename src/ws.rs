use {
    async_tungstenite::{tokio::ConnectStream, WebSocketStream},
    futures_util::{select, StreamExt},
    tracing::log::info,
};

#[tracing::instrument(skip(client_ws, provider_ws))]
pub async fn proxy(
    project_id: String,
    client_ws: axum_tungstenite::WebSocket,
    provider_ws: WebSocketStream<ConnectStream>,
) {
    let (client_ws_sender, client_ws_receiver) = client_ws.split();
    let (provider_ws_sender, provider_ws_receiver) = provider_ws.split();

    let mut write = client_ws_receiver.forward(provider_ws_sender);
    let mut read = provider_ws_receiver.forward(client_ws_sender);
    select! {
        _ = read => info!("WebSocket relaying messages to the provider for client {project_id} died.") ,
        _ = write => info!("WebSocket relaying messages from the provider to the client {project_id} died.") ,
    }
}
