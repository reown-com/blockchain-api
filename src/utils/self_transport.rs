use {
    crate::{
        handlers::{proxy::rpc_call, RpcQueryParams},
        state::AppState,
    },
    alloy_json_rpc::{RequestPacket, Response, ResponsePacket},
    alloy_transport::{TransportError, TransportFut},
    hyper::{body::to_bytes, HeaderMap},
    std::{net::SocketAddr, sync::Arc, task::Poll},
    tower::Service,
};

#[derive(Clone)]
pub struct SelfTransport {
    pub state: Arc<AppState>,
    pub connect_info: SocketAddr,
    pub query: RpcQueryParams,
    pub headers: HeaderMap,
}

impl Service<RequestPacket> for SelfTransport {
    type Error = TransportError;
    type Future = TransportFut<'static>;
    type Response = ResponsePacket;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: RequestPacket) -> Self::Future {
        let state = self.state.clone();
        let connect_info = self.connect_info;
        let query = self.query.clone();
        let headers = self.headers.clone();

        Box::pin(async move {
            // TODO handle batch
            let req = match req {
                RequestPacket::Single(req) => req,
                RequestPacket::Batch(_) => unimplemented!(),
            };
            // let id = SystemTime::now()
            //     .duration_since(UNIX_EPOCH)
            //     .expect("Time should't go backwards")
            //     .as_millis()
            //     .to_string();

            let body = req.serialized().to_string().into_bytes().into();
            let response = rpc_call(state, connect_info, query, headers, body)
            .await
            // .map_err(SelfProviderError::RpcError)?;
            .unwrap();

            // TODO handle error response status

            // if response.status() != StatusCode::OK {
            //     return Err(SelfProviderError::ProviderError {
            //         status: response.status(),
            //         body: format!("{:?}", response.body()),
            //     });
            // }

            // response.body().

            let bytes = to_bytes(response.into_body()).await.unwrap();
            // .map_err(SelfProviderError::ProviderBody)?;
            let body = String::from_utf8(bytes.to_vec()).unwrap();

            // let response = serde_json::from_slice::<JsonRpcResponse>(&bytes)
            // .unwrap();
            //     // .map_err(SelfProviderError::ProviderBodySerde)?;

            // let result = match response {
            //     JsonRpcResponse::Error(e) => return
            // Err(SelfProviderError::JsonRpcError(e)),
            //     JsonRpcResponse::Result(r) => {
            //         // We shouldn't process with `0x` result because this leads to the
            // ethers-rs         // panic when looking for an avatar
            //         if r.result == EMPTY_RPC_RESPONSE {
            //             return Err(SelfProviderError::ProviderError {
            //                 status: StatusCode::METHOD_NOT_ALLOWED,
            //                 body: format!("JSON-RPC result is {}", EMPTY_RPC_RESPONSE),
            //             });
            //         } else {
            //             r.result
            //         }
            //     }
            // };
            // let result = serde_json::from_value(result).unwrap();
            // // .map_err(|_| {
            // //     SelfProviderError::GenericParameterError(
            // //         "Caller always provides generic parameter R=Bytes".into(),
            // //     )
            // // })?;
            // Ok(result)

            Ok(ResponsePacket::Single(Response {
                id: req.id().clone(),
                payload: alloy_json_rpc::ResponsePayload::Success(
                    serde_json::value::RawValue::from_string(body).unwrap(),
                ),
            }))
        })
    }
}
