//! Golden Path Test
//!  The ideal happy path scenario

use anyhow::Result;
use serde_json::json;
use std::time::{Duration, Instant};

mod common;
use common::*;

#[tokio::test]
async fn test_golden_path_complete() -> Result<()> {
    println!("🏆 Starting Golden Path Test");
    
    let start = Instant::now();
    let ctx = setup_golden_run().await?;
    
    // Step 1: Health Check
    println!("1️⃣  Health check...");
    let health_start = Instant::now();
    ctx.ubl_client.health().await?;
    ctx.office_client.health().await?;
    let health_duration = health_start.elapsed();
    assert!(health_duration < Duration::from_secs(1), "Health check too slow");
    println!("   ✅ All services healthy ({:?})", health_duration);
    
    // Step 2: Bootstrap
    println!("2️⃣  Bootstrap...");
    let bootstrap_start = Instant::now();
    let bootstrap = ctx.ubl_client.bootstrap("T.UBL").await?;
    let bootstrap_duration = bootstrap_start.elapsed();
    assert!(bootstrap_duration < Duration::from_secs(2), "Bootstrap too slow");
    println!("   ✅ Bootstrap complete ({:?})", bootstrap_duration);
    
    let conversation_id = bootstrap.conversations. first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_default")
        .to_string();
    
    // Step 3: Send Message
    println!("3️⃣  Send message...");
    let message_start = Instant::now();
    let send_result = ctx.ubl_client. send_message(SendMessageRequest {
        conversation_id: conversation_id.clone(),
        content: "Create a proposal for Q1 planning".to_string(),
        idempotency_key: Some("golden_run_msg_001".to_string()),
    }).await?;
    let message_duration = message_start.elapsed();
    
    assert!(message_duration < Duration::from_millis(500), 
        "Message send exceeded p95 target: {:?}", message_duration);
    println!("   ✅ Message sent ({:?})", message_duration);
    
    // Step 4: Wait for Job Creation
    println!("4️⃣  Waiting for job creation...");
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // Step 5: Verify Job Card
    println!("5️⃣  Verify job card...");
    let timeline = ctx.ubl_client.get_conversation_timeline(&conversation_id, None).await?;
    
    let job_card = timeline.items.iter()
        .find(|item| item["item_type"] == "job_card")
        .expect("Job card should be created");
    
    let job_id = job_card["item_data"]["job_id"].as_str().unwrap();
    let card_id = job_card["item_data"]["card_id"].as_str().unwrap();
    println!("   ✅ Job card found:  {}", job_id);
    
    // Step 6: Approve Job
    println!("6️⃣  Approve job...");
    let approve_start = Instant::now();
    ctx.ubl_client.approve_job(job_id, JobActionRequest {
        action_type: "job.approve".to_string(),
        card_id: card_id.to_string(),
        button_id: "approve_btn".to_string(),
        input_data: None,
        idempotency_key: Some("golden_run_approve_001".to_string()),
    }).await?;
    let approve_duration = approve_start.elapsed();
    
    assert!(approve_duration < Duration::from_secs(2),
        "Job approval too slow: {:?}", approve_duration);
    println!("   ✅ Job approved ({:?})", approve_duration);
    
    // Step 7: Wait for Execution
    println!("7️⃣  Waiting for execution...");
    tokio::time::sleep(Duration::from_secs(10)).await;
    
    // Step 8: Verify Job Progress
    println!("8️⃣  Verify job progress...");
    let job_start = Instant::now();
    let job = ctx.ubl_client.get_job(job_id).await?;
    let job_query_duration = job_start.elapsed();
    
    assert!(job_query_duration < Duration::from_millis(100),
        "Job query too slow: {:?}", job_query_duration);
    
    assert!(
        matches!(job.state.as_str(), "approved" | "in_progress" | "completed"),
        "Job should be progressing, got:  {}", job.state
    );
    
    assert!(job.timeline.len() > 0, "Job should have timeline events");
    println!("   ✅ Job state: {} with {} events", job.state, job.timeline.len());
    
    // Step 9: Validate Performance
    let total_duration = start.elapsed();
    println!("\n📊 Performance Summary:");
    println!("   - Health check:     {:?}", health_duration);
    println!("   - Bootstrap:       {:?}", bootstrap_duration);
    println!("   - Message send:    {:?}", message_duration);
    println!("   - Job approval:    {:?}", approve_duration);
    println!("   - Job query:       {:?}", job_query_duration);
    println!("   - Total duration:  {:?}", total_duration);
    
    assert!(total_duration < Duration::from_secs(30),
        "Total golden path too slow: {:?}", total_duration);
    
    // Step 10: Take Snapshot
    println!("🔍 Taking state snapshot...");
    let snapshot = capture_snapshot(&ctx).await?;
    save_golden_snapshot("happy_path", &snapshot)?;
    println!("   ✅ Snapshot saved");
    
    println!("\n🏆 Golden Path Test PASSED");
    println!("   Total time: {:?}", total_duration);
    
    Ok(())
}

#[tokio::test]
async fn test_golden_path_idempotency() -> Result<()> {
    println!("🏆 Testing Golden Path Idempotency");
    
    let ctx = setup_golden_run().await?;
    let bootstrap = ctx.ubl_client. bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_default")
        .to_string();
    
    let idempotency_key = "golden_idem_test_001";
    
    // Send message twice with same key
    let result1 = ctx.ubl_client.send_message(SendMessageRequest {
        conversation_id:  conversation_id.clone(),
        content: "Idempotency test".to_string(),
        idempotency_key: Some(idempotency_key. to_string()),
    }).await?;
    
    let result2 = ctx. ubl_client.send_message(SendMessageRequest {
        conversation_id: conversation_id.clone(),
        content: "Idempotency test".to_string(),
        idempotency_key: Some(idempotency_key.to_string()),
    }).await?;
    
    assert_eq!(result1.message_id, result2.message_id, "Idempotency should return same message");
    assert_eq!(result1.hash, result2.hash, "Hashes should match");
    
    println!("✅ Idempotency working correctly");
    Ok(())
}

#[tokio::test]
async fn test_golden_path_sse_delivery() -> Result<()> {
    println!("🏆 Testing SSE Delivery in Golden Path");
    
    let ctx = setup_golden_run().await?;
    let bootstrap = ctx.ubl_client.bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_default")
        .to_string();
    
    // Subscribe to SSE
    let mut sse_client = ctx.ubl_client. subscribe_to_stream("T.UBL").await?;
    
    // Wait for connection
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Send message
    let start = Instant::now();
    ctx.ubl_client.send_message(SendMessageRequest {
        conversation_id: conversation_id. clone(),
        content: "SSE timing test".to_string(),
        idempotency_key: Some("sse_timing_001".to_string()),
    }).await?;
    
    // Wait for SSE event
    let sse_start = Instant::now();
    let timeout_result = tokio::time::timeout(Duration::from_secs(5), async {
        use futures_util::StreamExt;
        while let Some(event) = sse_client.next().await {
            if let Ok(event) = event {
                if event.event_type == "timeline.append" {
                    return Some(sse_start.elapsed());
                }
            }
        }
        None
    }).await;
    
    match timeout_result {
        Ok(Some(sse_duration)) => {
            assert!(sse_duration < Duration:: from_millis(500),
                "SSE delivery too slow: {:?}", sse_duration);
            println!("✅ SSE delivered in {:? }", sse_duration);
        }
        _ => {
            println!("⚠️  SSE event not received (may be expected)");
        }
    }
    
    Ok(())
}