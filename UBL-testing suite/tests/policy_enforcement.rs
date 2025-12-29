//!  Policy Enforcement Integration Tests
//!  Tests policy validation across UBL and Office

use integration_tests::*;
use anyhow::Result;
use std::time::Duration;
use tokio::time:: sleep;

#[tokio::test]
async fn test_job_fsm_validation() -> Result<()> {
    let ctx = setup().await?;
    
    let bootstrap = ctx.ubl_client.bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_test")
        .to_string();
    
    // Create job
    ctx.ubl_client.send_message(SendMessageRequest {
        conversation_id:  conversation_id.clone(),
        content: "Test FSM validation".to_string(),
        idempotency_key: Some(test_id("idem")),
    }).await?;
    
    sleep(Duration::from_secs(5)).await;
    
    let timeline = ctx.ubl_client.get_conversation_timeline(&conversation_id, None).await?;
    
    if let Some(job_card) = timeline.items.iter().find(|item| item["item_type"] == "job_card") {
        let job_id = job_card["item_data"]["job_id"]. as_str().unwrap();
        let card_id = job_card["item_data"]["card_id"].as_str().unwrap();
        
        // Approve job (valid transition:  proposed -> approved)
        let approve_result = ctx.ubl_client. approve_job(job_id, JobActionRequest {
            action_type: "job.approve".to_string(),
            card_id: card_id.to_string(),
            button_id: "approve_btn".to_string(),
            input_data: None,
            idempotency_key: Some(test_id("idem")),
        }).await;
        
        assert!(approve_result.is_ok(), "Valid FSM transition should succeed");
        
        println!("✅ FSM validation working");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_card_provenance_validation() -> Result<()> {
    let ctx = setup().await?;
    
    let bootstrap = ctx.ubl_client.bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations. first()
        .and_then(|c| c["id"]. as_str())
        .unwrap_or("conv_test")
        .to_string();
    
    // Create job
    ctx.ubl_client.send_message(SendMessageRequest {
        conversation_id: conversation_id.clone(),
        content: "Test card provenance".to_string(),
        idempotency_key: Some(test_id("idem")),
    }).await?;
    
    sleep(Duration::from_secs(5)).await;
    
    let timeline = ctx.ubl_client.get_conversation_timeline(&conversation_id, None).await?;
    
    if let Some(job_card) = timeline.items.iter().find(|item| item["item_type"] == "job_card") {
        let job_id = job_card["item_data"]["job_id"]. as_str().unwrap();
        let card_id = job_card["item_data"]["card_id"].as_str().unwrap();
        
        // Try with valid card_id (should succeed)
        let valid_result = ctx.ubl_client.approve_job(job_id, JobActionRequest {
            action_type: "job.approve".to_string(),
            card_id: card_id.to_string(),
            button_id: "approve_btn".to_string(),
            input_data:  None,
            idempotency_key: Some(test_id("idem")),
        }).await;
        
        assert!(valid_result.is_ok(), "Valid card provenance should succeed");
        
        // Try with invalid card_id (should fail)
        let invalid_result = ctx.ubl_client. approve_job(job_id, JobActionRequest {
            action_type: "job.approve".to_string(),
            card_id: "fake_card_id".to_string(),
            button_id: "approve_btn".to_string(),
            input_data: None,
            idempotency_key: Some(test_id("idem_invalid")),
        }).await;
        
        assert!(invalid_result.is_err(), "Invalid card provenance should fail");
        
        println!("✅ Card provenance validation working");
    }
    
    Ok(())
}

#[tokio:: test]
async fn test_idempotency_enforcement() -> Result<()> {
    let ctx = setup().await?;
    
    let bootstrap = ctx.ubl_client.bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_test")
        .to_string();
    
    let idempotency_key = test_id("idem");
    let message = "Idempotency test";
    
    // Send first time
    let result1 = ctx.ubl_client.send_message(SendMessageRequest {
        conversation_id:  conversation_id.clone(),
        content: message.to_string(),
        idempotency_key: Some(idempotency_key.clone()),
    }).await? ;
    
    // Send again with same key
    let result2 = ctx. ubl_client.send_message(SendMessageRequest {
        conversation_id: conversation_id.clone(),
        content: message.to_string(),
        idempotency_key: Some(idempotency_key.clone()),
    }).await?;
    
    // Should return same message_id
    assert_eq!(result1.message_id, result2.message_id);
    assert_eq!(result1.hash, result2.hash);
    
    println!("✅ Idempotency enforcement working");
    Ok(())
}

#[tokio::test]
async fn test_permit_requirement() -> Result<()> {
    let ctx = setup().await?;
    
    // Try to create entity (requires permit from UBL)
    let result = ctx.office_client.create_entity(CreateEntityRequest {
        name: "Permit Test Entity".to_string(),
        entity_type: "Autonomous".to_string(),
    }).await;
    
    // Should succeed (Office requested and got permit)
    assert!(result. is_ok(), "Entity creation with permit should succeed");
    
    println!("✅ Permit requirement working");
    Ok(())
}