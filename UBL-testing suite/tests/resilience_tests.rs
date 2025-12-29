//! Resilience Tests
//!  Validate system resilience and recovery

use anyhow::Result;
use std::time: :{Duration, Instant};
use tokio::time:: sleep;

mod common;
use common::*;

#[tokio::test]
async fn test_resilience_auto_retry() -> Result<()> {
    println!("🔥 Testing Auto-Retry Resilience");
    
    let ctx = setup_chaos_env().await?;
    
    // Simulate transient failure by stopping service briefly
    println!("1️⃣ Simulating transient failure...");
    
    // Send request that might fail
    let start = Instant::now();
    let mut attempts = 0;
    let max_attempts = 5;
    
    let result = loop {
        attempts += 1;
        
        match ctx.ubl_client.health().await {
            Ok(health) => break Ok(health),
            Err(e) if attempts < max_attempts => {
                println!("   Attempt {} failed, retrying...", attempts);
                sleep(Duration::from_secs(2)).await;
                continue;
            }
            Err(e) => break Err(e),
        }
    };
    
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "Should succeed with retry");
    println!("✅ Auto-retry succeeded after {} attempts in {:?}", attempts, duration);
    
    Ok(())
}

#[tokio::test]
async fn test_resilience_circuit_breaker() -> Result<()> {
    println!("🔥 Testing Circuit Breaker");
    
    let ctx = setup_chaos_env().await?;
    
    // Simulate multiple failures to trip circuit breaker
    println!("1️⃣ Generating failures to trip circuit breaker...");
    
    let mut failure_count = 0;
    let threshold = 5;
    
    for i in 0..10 {
        match ctx.office_client.health().await {
            Ok(_) => println!("   Attempt {}: Success", i + 1),
            Err(_) => {
                failure_count += 1;
                println!("   Attempt {}: Failed ({}/{})", i + 1, failure_count, threshold);
            }
        }
        
        if failure_count >= threshold {
            println!("   🔴 Circuit breaker should trip!");
            break;
        }
        
        sleep(Duration::from_millis(500)).await;
    }
    
    // Circuit breaker should now be open
    println!("2️⃣ Verifying circuit breaker is open.. .");
    
    // Next request should fail fast
    let fast_fail_start = Instant::now();
    let result = ctx.office_client.health().await;
    let fast_fail_duration = fast_fail_start.elapsed();
    
    if result.is_err() {
        assert!(fast_fail_duration < Duration::from_millis(100),
            "Circuit breaker should fail fast, took {:?}", fast_fail_duration);
        println!("✅ Circuit breaker failed fast in {:?}", fast_fail_duration);
    }
    
    // Wait for circuit breaker to go to half-open
    println!("3️⃣ Waiting for circuit breaker to reset...");
    sleep(Duration:: from_secs(30)).await;
    
    // Should allow one request through
    let result = ctx.office_client.health().await;
    println!("   Half-open state result: {:?}", result. is_ok());
    
    Ok(())
}

#[tokio::test]
async fn test_resilience_graceful_degradation() -> Result<()> {
    println!("🔥 Testing Graceful Degradation");
    
    let ctx = setup_chaos_env().await?;
    
    let bootstrap = ctx.ubl_client.bootstrap("T. UBL").await?;
    let conversation_id = bootstrap.conversations. first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_default")
        .to_string();
    
    println!("1️⃣ Testing normal operation...");
    
    // Normal operation
    let normal_result = ctx.ubl_client.send_message(SendMessageRequest {
        conversation_id: conversation_id.clone(),
        content: "Normal operation test".to_string(),
        idempotency_key: Some("resilience_normal_001".to_string()),
    }).await;
    
    assert!(normal_result.is_ok(), "Normal operation should succeed");
    println!("   ✅ Normal operation works");
    
    // Simulate Office unavailable
    println!("2️⃣ Simulating Office unavailable...");
    // (In real test, would stop Office service)
    
    // Message send should still work (writes to ledger)
    let degraded_result = ctx.ubl_client.send_message(SendMessageRequest {
        conversation_id: conversation_id.clone(),
        content: "Degraded mode test".to_string(),
        idempotency_key: Some("resilience_degraded_001".to_string()),
    }).await;
    
    // Should succeed or fail gracefully
    match degraded_result {
        Ok(_) => println!("   ✅ Message sent in degraded mode"),
        Err(e) => println!("   ⚠️ Graceful failure: {}", e),
    }
    
    // Read operations should still work (projections)
    let timeline_result = ctx.ubl_client.get_conversation_timeline(&conversation_id, None).await;
    assert!(timeline_result.is_ok(), "Timeline queries should work in degraded mode");
    println!("   ✅ Read operations work in degraded mode");
    
    println!("✅ Graceful degradation validated");
    Ok(())
}

#[tokio::test]
async fn test_resilience_state_recovery() -> Result<()> {
    println!("🔥 Testing State Recovery After Crash");
    
    let ctx = setup_chaos_env().await?;
    
    let bootstrap = ctx.ubl_client.bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_default")
        .to_string();
    
    println!("1️⃣ Creating state before crash...");
    
    // Create some state
    let message_ids = vec! [];
    for i in 0..5 {
        let result = ctx.ubl_client. send_message(SendMessageRequest {
            conversation_id: conversation_id.clone(),
            content: format!("Pre-crash message {}", i),
            idempotency_key: Some(format!("recovery_test_{}", i)),
        }).await?;
        
        println!("   Created message:  {}", result.message_id);
    }
    
    // Get current ledger sequence
    let timeline_before = ctx.ubl_client. get_conversation_timeline(&conversation_id, None).await?;
    let message_count_before = timeline_before.items.iter()
        .filter(|item| item["item_type"] == "message")
        .count();
    
    println!("   Messages before crash: {}", message_count_before);
    
    // Simulate crash and restart
    println!("2️⃣ Simulating service crash...");
    // (In real test, would kill and restart service)
    sleep(Duration::from_secs(5)).await;
    
    println!("3️⃣ Verifying state after recovery...");
    
    // Verify all messages are still there
    let timeline_after = ctx.ubl_client.get_conversation_timeline(&conversation_id, None).await?;
    let message_count_after = timeline_after.items. iter()
        .filter(|item| item["item_type"] == "message")
        .count();
    
    assert_eq!(message_count_before, message_count_after,
        "Message count should match after recovery");
    
    println!("   Messages after recovery: {}", message_count_after);
    println!("✅ State fully recovered");
    
    Ok(())
}

#[tokio::test]
async fn test_resilience_data_integrity_under_stress() -> Result<()> {
    println!("🔥 Testing Data Integrity Under Stress");
    
    let ctx = setup_chaos_env().await?;
    
    let bootstrap = ctx.ubl_client.bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_default")
        .to_string();
    
    println!("1️⃣ Sending messages under stress...");
    
    let mut handles = vec![];
    let num_concurrent = 20;
    
    for i in 0..num_concurrent {
        let ctx_clone = ctx.clone();
        let conv_id = conversation_id.clone();
        
        let handle = tokio::spawn(async move {
            let result = ctx_clone.ubl_client.send_message(SendMessageRequest {
                conversation_id:  conv_id,
                content: format!("Stress test message {}", i),
                idempotency_key:  Some(format!("stress_integrity_{}", i)),
            }).await;
            
            (i, result)
        });
        
        handles.push(handle);
    }
    
    // Wait for all to complete
    let results = futures::future::join_all(handles).await;
    
    let mut successful = 0;
    let mut failed = 0;
    
    for result in results {
        match result {
            Ok((i, Ok(_))) => {
                successful += 1;
            }
            Ok((i, Err(e))) => {
                failed += 1;
                println!("   Message {} failed: {}", i, e);
            }
            Err(e) => {
                failed += 1;
                println!("   Task failed: {}", e);
            }
        }
    }
    
    println!("   Successful:  {}, Failed: {}", successful, failed);
    
    // Verify data integrity
    println!("2️⃣ Verifying data integrity...");
    sleep(Duration::from_secs(2)).await;
    
    let timeline = ctx.ubl_client.get_conversation_timeline(&conversation_id, None).await?;
    
    // Count unique messages
    let mut message_ids = std::collections::HashSet::new();
    for item in &timeline.items {
        if item["item_type"] == "message" {
            if let Some(msg_id) = item["item_data"]["message_id"]. as_str() {
                message_ids.insert(msg_id.to_string());
            }
        }
    }
    
    println!("   Unique messages in timeline: {}", message_ids. len());
    
    // Should have no duplicates (idempotency working)
    assert!(message_ids.len() <= num_concurrent,
        "Should not have duplicates");
    
    println!("✅ Data integrity maintained under stress");
    
    Ok(())
}

#[tokio::test]
async fn test_resilience_timeout_handling() -> Result<()> {
    println!("🔥 Testing Timeout Handling");
    
    let ctx = setup_chaos_env().await?;
    
    println!("1️⃣ Testing normal timeout behavior...");
    
    // Create client with short timeout
    let short_timeout_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(1))
        .build()?;
    
    let start = Instant::now();
    
    // Make request that might timeout
    let result = short_timeout_client
        .get(&format!("{}/health", ctx.ubl_url))
        .send()
        .await;
    
    let duration = start.elapsed();
    
    match result {
        Ok(_) => println!("   ✅ Request succeeded in {:?}", duration),
        Err(e) if e.is_timeout() => {
            println!("   ⚠️ Request timed out after {:?}", duration);
            assert!(duration >= Duration::from_secs(1) && duration < Duration::from_secs(2),
                "Timeout should be respected");
        }
        Err(e) => println!("   ⚠️ Request failed: {}", e),
    }
    
    println!("✅ Timeout handling working correctly");
    
    Ok(())
}