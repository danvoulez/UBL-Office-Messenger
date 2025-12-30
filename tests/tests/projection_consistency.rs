//! Projection Consistency Integration Tests
//! Tests that projections stay consistent with ledger

use integration_tests::*;
use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use sqlx::PgPool;

#[tokio::test]
async fn test_message_projection_consistency() -> Result<()> {
    let ctx = setup().await?;
    
    let bootstrap = ctx.ubl_client.bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_test")
        .to_string();
    
    // Send message
    let message_content = format!("Projection test {}", uuid::Uuid::new_v4());
    let send_result = ctx.ubl_client.send_message(SendMessageRequest {
        conversation_id:  conversation_id.clone(),
        content: message_content.clone(),
        idempotency_key: Some(test_id("idem")),
    }).await?;
    
    // Wait for projection update
    sleep(Duration::from_secs(2)).await;
    
    // Query timeline (uses projection)
    let timeline = ctx.ubl_client.get_conversation_timeline(&conversation_id, None).await?;
    
    // Message should be in timeline
    let message_in_timeline = timeline.items.iter().any(|item| {
        item["item_type"] == "message" &&
        item["item_data"]["message_id"] == send_result. message_id
    });
    
    assert!(message_in_timeline, "Message should be in projection");
    
    println!("✅ Message projection consistent");
    Ok(())
}

#[tokio::test]
async fn test_job_projection_consistency() -> Result<()> {
    let ctx = setup().await?;
    
    let bootstrap = ctx.ubl_client.bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_test")
        .to_string();
    
    // Trigger job creation
    ctx.ubl_client.send_message(SendMessageRequest {
        conversation_id: conversation_id.clone(),
        content: "Create projection test job".to_string(),
        idempotency_key: Some(test_id("idem")),
    }).await?;
    
    sleep(Duration::from_secs(5)).await;
    
    let timeline = ctx. ubl_client.get_conversation_timeline(&conversation_id, None).await?;
    
    if let Some(job_card) = timeline.items.iter().find(|item| item["item_type"] == "job_card") {
        let job_id = job_card["item_data"]["job_id"].as_str().unwrap();
        
        // Query job (uses projection)
        let job = ctx.ubl_client.get_job(job_id).await?;
        
        // Job data should match timeline
        assert_eq!(job. job_id, job_id);
        assert! (!job.title.is_empty());
        
        println!("✅ Job projection consistent");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_timeline_ordering() -> Result<()> {
    let ctx = setup().await?;
    
    let bootstrap = ctx.ubl_client.bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_test")
        .to_string();
    
    // Send multiple messages
    for i in 0..3 {
        ctx.ubl_client.send_message(SendMessageRequest {
            conversation_id: conversation_id. clone(),
            content: format! ("Message {}", i),
            idempotency_key: Some(test_id(&format!("idem_{}", i))),
        }).await?;
        
        sleep(Duration::from_millis(500)).await;
    }
    
    sleep(Duration::from_secs(2)).await;
    
    // Query timeline
    let timeline = ctx.ubl_client.get_conversation_timeline(&conversation_id, None).await?;
    
    // Timeline should be ordered by creation time
    let cursors: Vec<String> = timeline.items.iter()
        .map(|item| item["cursor"].as_str().unwrap().to_string())
        .collect();
    
    // Cursors should be in ascending order
    for i in 0..cursors. len().saturating_sub(1) {
        assert!(
            cursors[i] < cursors[i + 1],
            "Timeline should be ordered:  {} should be < {}",
            cursors[i], cursors[i + 1]
        );
    }
    
    println!("✅ Timeline ordering consistent");
    Ok(())
}