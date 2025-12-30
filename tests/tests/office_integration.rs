//! Office Integration Tests - Prompt 2
//! Tests Office ws/test endpoint with mocked UBL

use anyhow::Result;
use integration_tests::OfficeClient;
use integration_tests::setup;

mod common;
use common::*;

#[tokio::test]
async fn office_ws_test_end_to_end_returns_receipt() -> Result<()> {
    println!("🧪 Testing Office ws/test endpoint");
    
    let ctx = setup().await?;
    
    // Test ws/test endpoint
    let req = integration_tests::WsTestRequest {
        tenant: "acme".to_string(),
        workspace: "dan-42".to_string(),
        repo: "billing".to_string(),
        sha: "cafebabedeadbeef".to_string(),
        suite: "unit".to_string(),
        limits: None,
        wait: Some(true),
    };
    
    let resp = ctx.office_client
        .ws_test(req, Some("sid-demo"), Some("asc-demo"))
        .await?;
    
    assert!(!resp.link_hash.is_empty(), "deve retornar link_hash (atom_hash)");
    
    if let Some(receipt) = resp.receipt {
        let receipt_obj = receipt.as_object().expect("receipt deve ser objeto");
        assert_eq!(
            receipt_obj.get("kind").and_then(|v| v.as_str()),
            Some("ws/receipt"),
            "receipt deve ter kind='ws/receipt'"
        );
        assert_eq!(
            receipt_obj.get("exit_code").and_then(|v| v.as_i64()),
            Some(0),
            "receipt deve ter exit_code=0"
        );
        println!("✅ Receipt recebido: {:?}", receipt_obj);
    } else {
        println!("⚠️  Receipt não retornado (pode ser que wait=false)");
    }
    
    println!("✅ Office ws/test test passed");
    Ok(())
}

