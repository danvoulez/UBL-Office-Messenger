//!  Job Lifecycle Integration Tests
//!  Tests complete job flow:  Creation → Approval → Execution → Completion

use integration_tests::*;
use anyhow::Result;
use std::time::Duration;
use tokio::time:: sleep;
use serde_json::json;

#[tokio::test]
async fn test_complete_job_lifecycle() -> Result<()> {
    let ctx = setup().await?;
    
    // 1. Bootstrap
    let bootstrap = ctx.ubl_client.bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations. first()
        .and_then(|c| c["id"]. as_str())
        .unwrap_or("conv_test")
        .to_string();
    
    // 2. Send message that triggers job creation
    let message = "Please create a proposal for client ABC";
    ctx.ubl_client.send_message(SendMessageRequest {
        conversation_id:  conversation_id.clone(),
        content: message.to_string(),
        idempotency_key: Some(test_id("idem")),
    }).await?;
    
    println!("📨 Message sent, waiting for Office to create job...");
    
    // 3. Wait for Office to create job and job card
    sleep(Duration::from_secs(5)).await;
    
    let timeline = ctx.ubl_client.get_conversation_timeline(&conversation_id, None).await?;
    
    // Find job card in timeline
    let job_card = timeline.items.iter()
        .find(|item| item["item_type"] == "job_card")
        .expect("Job card should be created");
    
    let job_id = job_card["item_data"]["job_id"].as_str().unwrap();
    let card_id = job_card["item_data"]["card_id"].as_str().unwrap();
    
    println!("✅ Job created: {}", job_id);
    
    // 4. Get job details
    let job = ctx.ubl_client.get_job(job_id).await?;
    assert_eq!(job.state, "proposed");
    
    // 5. Approve job
    ctx.ubl_client.approve_job(job_id, JobActionRequest {
        action_type:  "job. approve".to_string(),
        card_id: card_id. to_string(),
        button_id: "approve_btn".to_string(),
        input_data: None,
        idempotency_key: Some(test_id("idem")),
    }).await?;
    
    println!("✅ Job approved");
    
    // 6. Wait for job execution
    sleep(Duration::from_secs(3)).await;
    
    let job_updated = ctx.ubl_client. get_job(job_id).await?;
    
    // Job should transition to approved or in_progress
    assert!(
        job_updated.state == "approved" || 
        job_updated.state == "in_progress" ||
        job_updated.state == "completed",
        "Job should progress from proposed state, got: {}", job_updated.state
    );
    
    println!("✅ Job lifecycle complete:  {}", job_updated.state);
    Ok(())
}

#[tokio::test]
async fn test_job_rejection_flow() -> Result<()> {
    let ctx = setup().await?;
    
    let bootstrap = ctx.ubl_client.bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_test")
        .to_string();
    
    // Send message to create job
    ctx.ubl_client.send_message(SendMessageRequest {
        conversation_id: conversation_id. clone(),
        content: "Create a test document".to_string(),
        idempotency_key: Some(test_id("idem")),
    }).await?;
    
    sleep(Duration::from_secs(5)).await;
    
    let timeline = ctx.ubl_client.get_conversation_timeline(&conversation_id, None).await?;
    
    if let Some(job_card) = timeline.items.iter().find(|item| item["item_type"] == "job_card") {
        let job_id = job_card["item_data"]["job_id"].as_str().unwrap();
        let card_id = job_card["item_data"]["card_id"].as_str().unwrap();
        
        // Reject job
        ctx.ubl_client.approve_job(job_id, JobActionRequest {
            action_type: "job.reject".to_string(),
            card_id:  card_id.to_string(),
            button_id: "reject_btn".to_string(),
            input_data: Some(json!({"reason": "Not needed"})),
            idempotency_key: Some(test_id("idem")),
        }).await?;
        
        sleep(Duration::from_secs(2)).await;
        
        let job = ctx.ubl_client.get_job(job_id).await?;
        assert_eq!(job.state, "rejected");
        
        println!("✅ Job rejection flow complete");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_job_with_user_input() -> Result<()> {
    let ctx = setup().await?;
    
    let bootstrap = ctx.ubl_client.bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_test")
        .to_string();
    
    // Send message that requires input
    ctx.ubl_client.send_message(SendMessageRequest {
        conversation_id: conversation_id.clone(),
        content: "Schedule a meeting - what time works for you?".to_string(),
        idempotency_key: Some(test_id("idem")),
    }).await?;
    
    sleep(Duration::from_secs(5)).await;
    
    let timeline = ctx.ubl_client. get_conversation_timeline(&conversation_id, None).await?;
    
    if let Some(job_card) = timeline.items.iter().find(|item| item["item_type"] == "job_card") {
        let job_id = job_card["item_data"]["job_id"].as_str().unwrap();
        let card_id = job_card["item_data"]["card_id"].as_str().unwrap();
        
        // Approve job
        ctx.ubl_client. approve_job(job_id, JobActionRequest {
            action_type: "job.approve".to_string(),
            card_id: card_id.to_string(),
            button_id: "approve_btn".to_string(),
            input_data: None,
            idempotency_key: Some(test_id("idem")),
        }).await?;
        
        sleep(Duration::from_secs(3)).await;
        
        // Job might be waiting for input
        let job = ctx.ubl_client.get_job(job_id).await?;
        
        if job.state == "waiting_input" {
            // Provide input
            ctx.ubl_client.approve_job(job_id, JobActionRequest {
                action_type: "job.provide_input".to_string(),
                card_id: card_id.to_string(),
                button_id: "input_btn".to_string(),
                input_data: Some(json!({"time": "2pm tomorrow"})),
                idempotency_key: Some(test_id("idem")),
            }).await?;
            
            println!("✅ User input provided");
        }
    }
    
    Ok(())
}

#[tokio:: test]
async fn test_job_timeline_events() -> Result<()> {
    let ctx = setup().await?;
    
    let bootstrap = ctx.ubl_client.bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_test")
        .to_string();
    
    ctx.ubl_client.send_message(SendMessageRequest {
        conversation_id: conversation_id. clone(),
        content: "Create a report". to_string(),
        idempotency_key: Some(test_id("idem")),
    }).await?;
    
    sleep(Duration::from_secs(5)).await;
    
    let timeline = ctx.ubl_client.get_conversation_timeline(&conversation_id, None).await?;
    
    if let Some(job_card) = timeline.items.iter().find(|item| item["item_type"] == "job_card") {
        let job_id = job_card["item_data"]["job_id"].as_str().unwrap();
        
        let job = ctx.ubl_client.get_job(job_id).await?;
        
        // Job should have timeline events
        assert!(job.timeline. len() > 0, "Job should have timeline events");
        
        // Should have job. created event
        let has_created_event = job.timeline.iter()
            .any(|e| e["event_type"] == "job.created");
        assert!(has_created_event, "Should have job.created event");
        
        println!("✅ Job timeline has {} events", job.timeline.len());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_job_artifacts() -> Result<()> {
    let ctx = setup().await?;
    
    let bootstrap = ctx.ubl_client.bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_test")
        .to_string();
    
    ctx.ubl_client.send_message(SendMessageRequest {
        conversation_id: conversation_id.clone(),
        content: "Generate a PDF report".to_string(),
        idempotency_key: Some(test_id("idem")),
    }).await?;
    
    sleep(Duration::from_secs(5)).await;
    
    let timeline = ctx.ubl_client.get_conversation_timeline(&conversation_id, None).await?;
    
    if let Some(job_card) = timeline.items.iter().find(|item| item["item_type"] == "job_card") {
        let job_id = job_card["item_data"]["job_id"].as_str().unwrap();
        let card_id = job_card["item_data"]["card_id"]. as_str().unwrap();
        
        // Approve and wait for completion
        ctx.ubl_client.approve_job(job_id, JobActionRequest {
            action_type: "job.approve". to_string(),
            card_id: card_id.to_string(),
            button_id:  "approve_btn".to_string(),
            input_data: None,
            idempotency_key: Some(test_id("idem")),
        }).await?;
        
        // Wait for job to complete (longer timeout for artifact generation)
        sleep(Duration::from_secs(10)).await;
        
        let job = ctx.ubl_client.get_job(job_id).await?;
        
        // If job completed, should have artifacts
        if job.state == "completed" {
            println!("✅ Job has {} artifacts", job.artifacts.len());
        }
    }
    
    Ok(())
}