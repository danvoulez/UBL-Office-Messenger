//! HTTP Client with Unix Socket support (Prompt 3)
//! REQUIRED for security - no TCP fallback when UBL_UNIX is set
//! 
//! Unix sockets significantly reduce latency for the cryptographic data pipeline.

use anyhow::Result;
use hyper::{Body, Client, Request, Method, body::HttpBody};
use hyperlocal::{UnixConnector, Uri as UnixUri};
use std::path::Path;

pub enum UblEndpoint {
    Tcp { base: String },          // ex: http://127.0.0.1:8080
    Unix { socket: String },       // ex: /run/ubl/ubl-server.sock
}

impl UblEndpoint {
    pub fn from_env() -> Self {
        if let Ok(sock) = std::env::var("UBL_UNIX") {
            UblEndpoint::Unix { socket: sock }
        } else {
            let base = std::env::var("UBL_BASE")
                .or_else(|_| std::env::var("UBL_ENDPOINT"))
                .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
            UblEndpoint::Tcp { base }
        }
    }

    pub async fn post_json<T: serde::Serialize>(&self, path: &str, body: &T) -> Result<hyper::Response<Body>> {
        match self {
            UblEndpoint::Tcp { base } => {
                // For TCP, use reqwest for simplicity then convert
                let url = format!("{base}{path}");
                let resp = reqwest::Client::new().post(&url).json(body).send().await?;
                
                let status = resp.status();
                let headers = resp.headers().clone();
                let body_bytes = resp.bytes().await?;
                
                let mut hyper_resp = hyper::Response::builder()
                    .status(status.as_u16())
                    .body(Body::from(body_bytes.to_vec()))?;
                
                // Copy headers
                for (key, value) in headers.iter() {
                    hyper_resp.headers_mut().insert(key.clone(), value.clone());
                }
                Ok(hyper_resp)
            }
            UblEndpoint::Unix { socket } => {
                // Unix Socket - low latency path for crypto pipeline
                let connector = UnixConnector;
                let client: Client<UnixConnector, Body> = Client::builder().build(connector);
                
                let socket_path = Path::new(socket);
                let uri = UnixUri::new(socket_path, path);
                
                let body_json = serde_json::to_string(body)?;
                let req = Request::builder()
                    .method(Method::POST)
                    .uri(uri)
                    .header("content-type", "application/json")
                    .body(Body::from(body_json))?;
                
                let resp = client.request(req).await
                    .map_err(|e| anyhow::anyhow!("Unix Socket request failed: {}", e))?;
                
                Ok(resp)
            }
        }
    }

    pub async fn get(&self, path: &str) -> Result<hyper::Response<Body>> {
        match self {
            UblEndpoint::Tcp { base } => {
                let url = format!("{base}{path}");
                let resp = reqwest::Client::new().get(&url).send().await?;
                
                let status = resp.status();
                let headers = resp.headers().clone();
                let body_bytes = resp.bytes().await?;
                
                let mut hyper_resp = hyper::Response::builder()
                    .status(status.as_u16())
                    .body(Body::from(body_bytes.to_vec()))?;
                
                for (key, value) in headers.iter() {
                    hyper_resp.headers_mut().insert(key.clone(), value.clone());
                }
                Ok(hyper_resp)
            }
            UblEndpoint::Unix { socket } => {
                let connector = UnixConnector;
                let client: Client<UnixConnector, Body> = Client::builder().build(connector);
                
                let socket_path = Path::new(socket);
                let uri = UnixUri::new(socket_path, path);
                
                let req = Request::builder()
                    .method(Method::GET)
                    .uri(uri)
                    .body(Body::empty())?;
                
                let resp = client.request(req).await
                    .map_err(|e| anyhow::anyhow!("Unix Socket request failed: {}", e))?;
                
                Ok(resp)
            }
        }
    }
}

/// Helper to read body bytes from hyper 0.14 Response
pub async fn body_to_bytes(body: Body) -> Result<hyper::body::Bytes> {
    let bytes = hyper::body::to_bytes(body).await
        .map_err(|e| anyhow::anyhow!("Failed to read body: {}", e))?;
    Ok(bytes)
}
