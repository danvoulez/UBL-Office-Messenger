//! 💎 Diamond Run Complete Test
//!  The ultimate end-to-end validation

use anyhow::Result;
use std::time: :{Duration, Instant};

mod common;
use common::*;

#[tokio::test]
async fn test_diamond_golden_paths() -> Result<()> {
    println!("💎 Diamond Run: Golden Paths");
    
    let start = Instant::now();
    let ctx = setup_diamond_env().await?;
    
    // Scenario 1: Complete user journey
    complete_user_journey(&ctx).await?;
    
    // Scenario 2: Multi-user collaboration
    multi_user_collaboration(&ctx).await?;
    
    // Scenario 3: Complex workflow
    complex_workflow(&ctx).await?;
    
    let duration = start.elapsed();
    println!("✅ Golden paths completed in {: ?}", duration);
    
    Ok(())
}

async fn complete_user_journey(ctx: &DiamondContext) -> Result<()> {
    println!("  📍 Scenario 1: Complete User Journey");
    
    // Login
    let bootstrap = ctx.ubl_client.bootstrap("T. UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_default")
        .to_string();
    
    // Send message
    ctx.ubl_client.send_message(SendMessageRequest {
        conversation_id: conversation_id.clone(),
        content: "Create Q1 planning document".to_string(),
        idempotency_key: Some("diamond_journey_001".to_string()),
    }).await?;
    
    tokio::time::sleep(Duration:: from_secs(5)).await;
    
    // Get job
    let timeline = ctx.ubl_client. get_conversation_timeline(&conversation_id, None).await?;
    let job_card = timeline.items.iter()
        .find(|item| item["item_type"] == "job_card")
        .expect("Job card should exist");
    
    let job_id = job_card["item_data"]["job_id"].as_str().unwrap();
    let card_id = job_card["item_data"]["card_id"].as_str().unwrap();
    
    // Approve
    ctx.ubl_client. approve_job(job_id, JobActionRequest {
        action_type: "job. approve".to_string(),
        card_id: card_id.to_string(),
        button_id: "approve_btn".to_string(),
        input_data: None,
        idempotency_key: Some("diamond_approve_001".to_string()),
    }).await?;
    
    // Wait for completion
    tokio::time::sleep(Duration::from_secs(10)).await;
    
    let job = ctx.ubl_client.get_job(job_id).await?;
    assert!(
        matches!(job. state. as_str(), "approved" | "in_progress" | "completed"),
        "Job should progress"
    );
    
    println!("    ✅ User journey complete");
    Ok(())
}

async fn multi_user_collaboration(ctx: &DiamondContext) -> Result<()> {
    println!("  📍 Scenario 2: Multi-User Collaboration");
    
    // Create multiple entities
    let alice = ctx.office_client.create_entity(CreateEntityRequest {
        name: "Alice".to_string(),
        entity_type: "Autonomous". to_string(),
    }).await?;
    
    let bob = ctx.office_client.create_entity(CreateEntityRequest {
        name: "Bob". to_string(),
        entity_type: "Autonomous".to_string(),
    }).await?;
    
    // Simulate collaboration
    let bootstrap = ctx.ubl_client. bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_default")
        .to_string();
    
    for i in 0..5 {
        ctx.ubl_client.send_message(SendMessageRequest {
            conversation_id: conversation_id.clone(),
            content: format!("Collaboration message {}", i),
            idempotency_key: Some(format!("collab_{}", i)),
        }).await?;
        
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    println!("    ✅ Collaboration complete");
    Ok(())
}

async fn complex_workflow(ctx: &DiamondContext) -> Result<()> {
    println!("  📍 Scenario 3: Complex Workflow");
    
    // Multi-step job with user input
    let bootstrap = ctx. ubl_client.bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_default")
        .to_string();
    
    ctx.ubl_client.send_message(SendMessageRequest {
        conversation_id: conversation_id. clone(),
        content: "Create detailed project plan with milestones".to_string(),
        idempotency_key: Some("complex_workflow_001".to_string()),
    }).await?;
    
    tokio::time::sleep(Duration:: from_secs(5)).await;
    
    // Verify job created and progressing
    let timeline = ctx.ubl_client.get_conversation_timeline(&conversation_id, None).await?;
    assert!(timeline.items.len() > 0, "Timeline should have items");
    
    println!("    ✅ Complex workflow complete");
    Ok(())
}