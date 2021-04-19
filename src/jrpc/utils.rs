//! # A JsonRPC mini-framework.
//!
//! This module provides a small framework for building JsonRPC API on top of a `wrap` filter, this
//! is done through the builder methods of `JrpcFilter`.
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use warp::filters;
use warp::Filter;

pub const JRPC_VERSION: &str = "2.0";
const JRPC_CODE_METHOD_NOT_FOUND: i32 = -32601;
const JRPC_CODE_INVALID_PARAMS: i32 = -32605;
const JRPC_CODE_INVALID_REQUEST: i32 = -32600;
const JRPC_CODE_SERVER_ERROR: i32 = -32000;

/// JsonRPC envelope.
#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct JrpcEnvelope {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: Value,
}

/// JsonRPC context.
#[derive(Debug)]
struct JrpcCtx {
    jsonrpc: String,
    id: String,
    method: String,
}

/// JsonRPC errors.
#[derive(Debug)]
#[allow(dead_code)]
pub enum JrpcError {
    WrongMethod,
    WrongParameters,
    WrongVersion,
    ServerError,
    InvalidRequest,
}

/// JsonRPC error details.
#[derive(Serialize)]
pub struct JrpcErrorDetails {
    code: i32,
    message: String,
}

/// JsonRPC response.
#[derive(Serialize)]
pub struct JrpcResponse<T> {
    pub jsonrpc: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JrpcErrorDetails>,
}

pub type JrpcResult<T> = Result<T, JrpcError>;
type MethodMap<S> = HashMap<String, Box<dyn Fn(JrpcEnvelope, S) -> String + Send + Sync>>;
type SharedMethodMap<S> =
    std::sync::Arc<HashMap<String, Box<dyn Fn(JrpcEnvelope, S) -> String + Send + Sync>>>;

/// A JsonRPC `warp` filter, methods can be added through the builder method
/// `add` while the filter is built with the `build` method.
///
/// `warp` is an HTTP framework that enable building API from 'filters' composition,
/// check its documentation for more informations.
pub struct JrpcFilter<S> {
    methods: MethodMap<S>,
    shared_state: S,
}

impl<S: Send + Sync + Clone> JrpcFilter<S> {
    /// A builder object to easily create `wrap` filters that handle JsonRPC.
    pub fn new(shared_state: S) -> Self {
        JrpcFilter {
            methods: HashMap::new(),
            shared_state,
        }
    }

    /// Build a `wrap` filter that handles JRPC requests.
    /// To register methods use the `add` method on `JrpcFilter`.
    pub fn build(self) -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
        filters::body::json::<JrpcEnvelope>()
            .and(Self::with_methods(self.methods))
            .and(Self::with_state(self.shared_state))
            .and_then(
                |ctx: JrpcEnvelope, methods: SharedMethodMap<S>, state: S| async move {
                    if ctx.jsonrpc != JRPC_VERSION {
                        return Self::build_error(ctx.id, JrpcError::WrongVersion);
                    }
                    if let Some(method) = methods.get(&ctx.method) {
                        Ok(method(ctx, state))
                    } else {
                        Self::build_error(ctx.id, JrpcError::WrongMethod)
                    }
                },
            )
    }

    /// A builder method to register new JsonRPC methods.
    ///
    /// params:
    ///  - method: The method name.
    ///  - handler: The handler function for this method, it takes a `Serialize` struct
    ///             as input (which correspond to the jrpc parmeters) and must return a
    ///             `Result<R, JrpcError>`, where R is `Serialize` and correspond to the
    ///             request response.
    ///
    pub fn add<T, F, R>(mut self, method: &str, handler: F) -> Self
    where
        T: DeserializeOwned + Send,
        F: Fn(T, S) -> Result<R, JrpcError> + Send + Sync + Clone + 'static,
        R: Serialize,
    {
        let method_handler = move |ctx: JrpcEnvelope, state: S| {
            let result = match serde_json::from_value::<T>(ctx.params) {
                Ok(params) => handler(params, state),
                Err(_) => Err(JrpcError::WrongParameters),
            };
            let response = Self::build_response(ctx.id, result);
            serde_json::to_string(&response).unwrap()
        };
        self.methods
            .insert(method.to_string(), Box::new(method_handler));
        self
    }

    /// Build the JsonRPC response from a JsonRPC Result and a request ID.
    fn build_response<R: Serialize>(id: String, res: JrpcResult<R>) -> JrpcResponse<R> {
        match res {
            Ok(value) => JrpcResponse {
                id,
                jsonrpc: JRPC_VERSION.to_string(),
                result: Some(value),
                error: None,
            },
            Err(err) => JrpcResponse {
                id,
                jsonrpc: JRPC_VERSION.to_string(),
                result: None,
                error: Some(err.into()),
            },
        }
    }

    /// Build an error response from a given JrpcError.
    #[allow(clippy::unnecessary_wraps)]
    fn build_error(id: String, error: JrpcError) -> Result<String, std::convert::Infallible> {
        let response = Self::build_response::<()>(id, Err(error));
        Ok(serde_json::to_string(&response).unwrap())
    }

    /// A helper function that returns a filter extracting the methods.
    fn with_methods(
        methods: MethodMap<S>,
    ) -> impl Filter<Extract = (SharedMethodMap<S>,), Error = core::convert::Infallible> + Clone
    {
        let shared_methods = std::sync::Arc::new(methods);
        filters::any::any().map(move || std::sync::Arc::clone(&shared_methods))
    }

    /// A helper function that returns a filter extracting the shared state.
    fn with_state(
        shared_state: S,
    ) -> impl Filter<Extract = (S,), Error = core::convert::Infallible> + Clone {
        filters::any::any().map(move || shared_state.clone())
    }
}

impl From<JrpcError> for JrpcErrorDetails {
    fn from(err: JrpcError) -> Self {
        match err {
            JrpcError::WrongMethod => JrpcErrorDetails {
                code: JRPC_CODE_METHOD_NOT_FOUND,
                message: "Method does not exsists".to_string(),
            },
            JrpcError::WrongParameters => JrpcErrorDetails {
                code: JRPC_CODE_INVALID_PARAMS,
                message: "Invalid parameters".to_string(),
            },
            JrpcError::WrongVersion => JrpcErrorDetails {
                code: JRPC_CODE_INVALID_REQUEST,
                message: "Invalid jsonrpc version, expected '2.0'".to_string(),
            },
            JrpcError::ServerError => JrpcErrorDetails {
                code: JRPC_CODE_SERVER_ERROR,
                message: "Server error".to_string(),
            },
            JrpcError::InvalidRequest => JrpcErrorDetails {
                code: JRPC_CODE_INVALID_REQUEST,
                message: "Invalid request".to_string(),
            },
        }
    }
}
