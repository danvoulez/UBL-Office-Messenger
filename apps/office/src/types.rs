//! Types for Office workspace and deployment operations
//! Prompt 1: Office front-door types

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct WsTestBody {
    pub tenant: String,
    pub workspace: String,
    pub repo: String,
    pub sha: String,
    pub suite: String,
    #[serde(default)]
    pub limits: Option<WsLimits>,
    #[serde(default)]
    pub wait: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct WsLimits {
    pub cpu: Option<u32>,
    pub mem_mb: Option<u32>,
    pub timeout_sec: Option<u32>,
    pub net: Option<bool>,
}

#[derive(Deserialize)]
pub struct WsBuildBody {
    pub tenant: String,
    pub workspace: String,
    pub repo: String,
    pub sha: String,
    pub target: String,
    #[serde(default)]
    pub limits: Option<WsLimits>,
    #[serde(default)]
    pub wait: Option<bool>,
}

#[derive(Deserialize)]
pub struct DeployBody {
    pub tenant: String,
    pub app: String,
    pub env: String,
    pub image_digest: String,
    pub strategy: String,
    #[serde(default)]
    pub desired_replicas: Option<u32>,
    #[serde(default)]
    pub wait: Option<bool>,
}

#[derive(Deserialize)]
pub struct RepoBundleApplyBody {
    pub tenant: String,
    pub repo: String,
    #[serde(rename = "ref")]
    pub r#ref: String,
}


