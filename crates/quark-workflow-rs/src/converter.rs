//! Data converter — serializes Rust values to/from protobuf `Payload`s.
//!
//! The default converter uses JSON encoding. Users can implement the
//! `DataConverter` trait to use a different encoding (e.g., protobuf,
//! msgpack, or encrypted codecs).

use std::sync::Arc;

use quark_workflow_proto::temporal::api::common::v1 as commonpb;

use crate::errors::SdkError;

/// Trait for converting between Rust values and protobuf `Payload`s.
///
/// All values sent to the server (workflow args, signal args, query
/// args, results) pass through a `DataConverter`. The server stores
/// them as opaque `Payload` blobs; the converter controls the encoding.
pub trait DataConverter: Send + Sync {
    /// Convert a slice of JSON values into a `Payloads` message.
    fn to_payloads(&self, values: &[serde_json::Value]) -> Result<commonpb::Payloads, SdkError>;

    /// Convert a `Payloads` message back into a slice of JSON values.
    fn decode_payloads(&self, payloads: &commonpb::Payloads) -> Result<Vec<serde_json::Value>, SdkError>;

    /// Convert a single JSON value into a `Payload`.
    fn to_payload(&self, value: &serde_json::Value) -> Result<commonpb::Payload, SdkError>;

    /// Convert a single `Payload` back into a JSON value.
    fn decode_payload(&self, payload: &commonpb::Payload) -> Result<serde_json::Value, SdkError>;
}

/// JSON data converter — the default converter.
///
/// Encodes values as JSON with `encoding: "json/plain"` metadata.
pub struct JsonDataConverter;

impl JsonDataConverter {
    /// Creates a new `JsonDataConverter`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonDataConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl DataConverter for JsonDataConverter {
    fn to_payloads(&self, values: &[serde_json::Value]) -> Result<commonpb::Payloads, SdkError> {
        let mut payloads = Vec::with_capacity(values.len());
        for value in values {
            payloads.push(self.to_payload(value)?);
        }
        Ok(commonpb::Payloads { payloads })
    }

    fn decode_payloads(&self, payloads: &commonpb::Payloads) -> Result<Vec<serde_json::Value>, SdkError> {
        payloads
            .payloads
            .iter()
            .map(|p| self.decode_payload(p))
            .collect()
    }

    fn to_payload(&self, value: &serde_json::Value) -> Result<commonpb::Payload, SdkError> {
        let data = serde_json::to_vec(value)?;
        let mut metadata = std::collections::HashMap::new();
        metadata.insert(
            "encoding".to_string(),
            quark_workflow_proto::prost::bytes::Bytes::from("json/plain"),
        );
        Ok(commonpb::Payload {
            data: data.into(),
            metadata,
            ..Default::default()
        })
    }

    fn decode_payload(&self, payload: &commonpb::Payload) -> Result<serde_json::Value, SdkError> {
        if payload.data.is_empty() {
            return Ok(serde_json::Value::Null);
        }
        let value: serde_json::Value = serde_json::from_slice(&payload.data)?;
        Ok(value)
    }
}

/// Converts a typed Rust value to a `Payloads` containing a single
/// JSON-encoded payload.
pub(crate) fn to_single_payload(
    converter: &Arc<dyn DataConverter>,
    value: &serde_json::Value,
) -> Result<commonpb::Payloads, SdkError> {
    converter.to_payloads(std::slice::from_ref(value))
}

/// Extracts a single typed value from a `Payloads` message.
pub(crate) fn from_single_payload<T: serde::de::DeserializeOwned>(
    converter: &Arc<dyn DataConverter>,
    payloads: &commonpb::Payloads,
) -> Result<T, SdkError> {
    let values = converter.decode_payloads(payloads)?;
    if values.is_empty() {
        return serde_json::from_value(serde_json::Value::Null).map_err(SdkError::from);
    }
    serde_json::from_value(values[0].clone()).map_err(SdkError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_string() {
        let converter = JsonDataConverter::new();
        let value = serde_json::json!("hello world");
        let payload = converter.to_payload(&value).unwrap();
        let result = converter.decode_payload(&payload).unwrap();
        assert_eq!(value, result);
    }

    #[test]
    fn test_roundtrip_object() {
        let converter = JsonDataConverter::new();
        let value = serde_json::json!({ "orderId": "12345", "amount": 99.99 });
        let payload = converter.to_payload(&value).unwrap();
        let result = converter.decode_payload(&payload).unwrap();
        assert_eq!(value, result);
    }

    #[test]
    fn test_roundtrip_array() {
        let converter = JsonDataConverter::new();
        let values = vec![
            serde_json::json!("first"),
            serde_json::json!(42),
            serde_json::json!({ "nested": true }),
        ];
        let payloads = converter.to_payloads(&values).unwrap();
        let results = converter.decode_payloads(&payloads).unwrap();
        assert_eq!(values, results);
    }

    #[test]
    fn test_payload_metadata() {
        let converter = JsonDataConverter::new();
        let payload = converter.to_payload(&serde_json::json!("test")).unwrap();
        let encoding = payload
            .metadata
            .get("encoding")
            .expect("metadata should have encoding");
        assert_eq!(encoding.as_ref(), b"json/plain");
    }

    #[test]
    fn test_empty_payload() {
        let converter = JsonDataConverter::new();
        let payload = commonpb::Payload {
            data: vec![].into(),
            ..Default::default()
        };
        let result = converter.decode_payload(&payload).unwrap();
        assert_eq!(result, serde_json::Value::Null);
    }
}
