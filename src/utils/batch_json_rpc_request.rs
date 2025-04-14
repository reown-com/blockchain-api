use alloy::rpc::json_rpc::Id;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum MaybeBatchRequest {
    Single(Request),
    Batch(Vec<Request>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Request {
    pub jsonrpc: String,
    pub method: String,
    // params are optional
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    // id is technically optional too, but requiring it for now since we need it for analytics and it seems all EVM methods require it
    pub id: Id,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_most_permissive() {
        let request = r#"{"jsonrpc":"2.0","method":"eth_chainId","id":1}"#;
        let single_request: MaybeBatchRequest = serde_json::from_str(request).unwrap();
        assert_eq!(
            single_request,
            MaybeBatchRequest::Single(Request {
                jsonrpc: "2.0".to_string(),
                method: "eth_chainId".to_string(),
                params: None,
                id: Id::Number(1),
            })
        );
    }

    #[test]
    fn test_deserialize_single_request() {
        let request = r#"{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}"#;
        let single_request: MaybeBatchRequest = serde_json::from_str(request).unwrap();
        assert_eq!(
            single_request,
            MaybeBatchRequest::Single(Request {
                jsonrpc: "2.0".to_string(),
                method: "eth_chainId".to_string(),
                params: Some(serde_json::Value::Array(vec![])),
                id: Id::Number(1),
            })
        );
    }

    #[test]
    fn test_deserialize_batch_request() {
        let request = r#"[
            {"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1},
            {"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":2}
        ]"#;
        let batch_request: MaybeBatchRequest = serde_json::from_str(request).unwrap();
        assert_eq!(
            batch_request,
            MaybeBatchRequest::Batch(vec![
                Request {
                    jsonrpc: "2.0".to_string(),
                    method: "eth_chainId".to_string(),
                    params: Some(serde_json::Value::Array(vec![])),
                    id: Id::Number(1),
                },
                Request {
                    jsonrpc: "2.0".to_string(),
                    method: "eth_chainId".to_string(),
                    params: Some(serde_json::Value::Array(vec![])),
                    id: Id::Number(2),
                },
            ])
        );
    }

    #[test]
    fn test_deserialize_single_request_raw() {
        let request = r#"{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}"#;
        let single_request: MaybeBatchRequest = serde_json::from_str(request).unwrap();
        assert!(matches!(single_request, MaybeBatchRequest::Single(_)));
        let single_request = match single_request {
            MaybeBatchRequest::Single(request) => request,
            _ => panic!("Expected a single request"),
        };
        assert_eq!(single_request.id, Id::Number(1));
        assert_eq!(single_request.method, "eth_chainId");
        assert_eq!(
            single_request.params,
            Some(serde_json::Value::Array(vec![]))
        );
    }

    #[test]
    fn test_deserialize_batch_request_raw() {
        let request = r#"[
            {"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1},
            {"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":2}
        ]"#;
        let batch_request: MaybeBatchRequest = serde_json::from_str(request).unwrap();
        assert_eq!(
            batch_request,
            MaybeBatchRequest::Batch(vec![
                Request {
                    jsonrpc: "2.0".to_string(),
                    method: "eth_chainId".to_string(),
                    params: Some(serde_json::Value::Array(vec![])),
                    id: Id::Number(1),
                },
                Request {
                    jsonrpc: "2.0".to_string(),
                    method: "eth_chainId".to_string(),
                    params: Some(serde_json::Value::Array(vec![])),
                    id: Id::Number(2),
                },
            ])
        );
    }

    #[test]
    fn test_deserialize_single_request_no_params() {
        let request = r#"{"jsonrpc":"2.0","method":"eth_chainId","id":1}"#;
        let single_request: MaybeBatchRequest = serde_json::from_str(request).unwrap();
        assert_eq!(
            single_request,
            MaybeBatchRequest::Single(Request {
                jsonrpc: "2.0".to_string(),
                method: "eth_chainId".to_string(),
                params: None,
                id: Id::Number(1),
            })
        );
    }
}
