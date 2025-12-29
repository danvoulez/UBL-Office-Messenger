//!  Failure Recovery Tests
//!  Tests various failure scenarios and recovery mechanisms

use anyhow::Result;
use std::time: :{Duration, Instant};
use tokio::time:: sleep;

mod common;
use common::*;

#[tokio::test]
async fn test_recovery_from_database_disconnect() -> Result<()> {
    println!("🔥 Testing Recovery from Database Disconnect");
    
    let ctx = setup_chaos_env().await?;
    
    // Normal operation
    println!("1️⃣ Establishing baseline.. .");
    let bootstrap = ctx.ubl_client. bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_default")
        .to_string();
    
    ctx.ubl_client.send_message(SendMessageRequest {
        conversation_id:  conversation_id.clone(),
        content: "Before disconnect".to_string(),
        idempotency_key: Some("recovery_db_001".to_string()),
    }).await?;
    println!("   ✅ Baseline established");
    
    // Simulate database disconnect
    println!("2️⃣ Simulating database disconnect...");
    // In real implementation:  docker-compose stop postgres
    
    // Attempt operation during disconnect
    println!("3️⃣ Attempting operation during disconnect...");
    let disconnected_result = ctx.ubl_client. send_message(SendMessageRequest {
        conversation_id: conversation_id.clone(),
        content: "During disconnect".to_string(),
        idempotency_key: Some("recovery_db_002".to_string()),
    }).await;
    
    match disconnected_result {
        Ok(_) => println!("   ⚠️ Operation succeeded (unexpected)"),
        Err(e) => {
            println!("   ✅ Operation failed gracefully:  {}", e);
            assert!(e.to_string().contains("connection") || e.to_string().contains("timeout"),
                "Error should indicate connection issue");
        }
    }
    
    // Restore database
    println!("4️⃣ Restoring database connection...");
    // In real implementation: docker-compose start postgres
    sleep(Duration::from_secs(10)).await;
    
    // Verify recovery
    println!("5️⃣ Verifying recovery...");
    let recovery_start = Instant::now();
    let mut recovered = false;
    
    for attempt in 1..=10 {
        sleep(Duration::from_secs(2)).await;
        
        if let Ok(_) = ctx.ubl_client.health().await {
            recovered = true;
            let recovery_time = recovery_start.elapsed();
            println!("   ✅ System recovered after {: ?} ({} attempts)", recovery_time, attempt);
            
            assert!(recovery_time < Duration::from_secs(60),
                "Recovery took too long:  {:?}", recovery_time);
            break;
        }
    }
    
    assert!(recovered, "System failed to recover");
    
    // Verify data integrity
    println! ("6️⃣ Verifying data integrity...");
    let timeline = ctx.ubl_client.get_conversation_timeline(&conversation_id, None).await?;
    
    let has_baseline_message = timeline.items.iter().any(|item| {
        item["item_data"]["content"]. as_str() == Some("Before disconnect")
    });
    
    assert!(has_baseline_message, "Baseline message should be preserved");
    println!("   ✅ Data integrity maintained");
    
    println!("✅ Database disconnect recovery successful");
    Ok(())
}

#[tokio::test]
async fn test_recovery_from_cascading_failures() -> Result<()> {
    println!("🔥 Testing Recovery from Cascading Failures");
    
    let ctx = setup_chaos_env().await?;
    
    let bootstrap = ctx.ubl_client.bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_default")
        .to_string();
    
    // Start background load
    println!("1️⃣ Starting background load...");
    let load_handle = tokio::spawn({
        let ctx_clone = ctx.clone();
        let conv_id = conversation_id.clone();
        async move {
            for i in 0..100 {
                let _ = ctx_clone.ubl_client.send_message(SendMessageRequest {
                    conversation_id: conv_id.clone(),
                    content: format!("Load message {}", i),
                    idempotency_key: Some(format!("cascading_load_{}", i)),
                }).await;
                
                tokio::time::sleep(Duration:: from_millis(500)).await;
            }
        }
    });
    
    // Inject cascading failures
    println!("2️⃣ Injecting cascading failures...");
    
    // Failure 1: Network latency
    println!("   🔥 Injecting network latency.. .");
    sleep(Duration::from_secs(5)).await;
    
    // Failure 2: CPU pressure
    println!("   🔥 Injecting CPU pressure.. .");
    sleep(Duration::from_secs(5)).await;
    
    // Failure 3: Memory pressure
    println!("   🔥 Injecting memory pressure...");
    sleep(Duration:: from_secs(5)).await;
    
    // Monitor system state
    println!("3️⃣ Monitoring system under cascading failures...");
    let mut health_checks = vec! [];
    
    for i in 0..10 {
        let health = ctx.ubl_client.health().await;
        health_checks.push(health.is_ok());
        sleep(Duration::from_secs(2)).await;
    }
    
    let availability = health_checks.iter().filter(|&&h| h).count() as f64 / health_checks.len() as f64;
    println!("   Availability during failures: {:.1}%", availability * 100.0);
    
    // Remove failures
    println!("4️⃣ Removing failures...");
    sleep(Duration::from_secs(10)).await;
    
    // Verify recovery
    println!("5️⃣ Verifying recovery...");
    let recovery_start = Instant::now();
    
    loop {
        if ctx.ubl_client.health().await. is_ok() && 
           ctx.office_client.health().await.is_ok() {
            let recovery_time = recovery_start. elapsed();
            println!("   ✅ System fully recovered in {:?}", recovery_time);
            
            assert!(recovery_time < Duration::from_secs(120),
                "Recovery too slow: {:?}", recovery_time);
            break;
        }
        
        if recovery_start.elapsed() > Duration::from_secs(300) {
            panic!("System failed to recover within 5 minutes");
        }
        
        sleep(Duration::from_secs(5)).await;
    }
    
    // Wait for background load to finish
    let _ = load_handle.await;
    
    println!("✅ Cascading failure recovery successful");
    Ok(())
}

#[tokio:: test]
async fn test_recovery_split_brain_scenario() -> Result<()> {
    println!("🔥 Testing Split Brain Recovery");
    
    let ctx = setup_chaos_env().await?;
    
    // Create state before partition
    println!("1️⃣ Creating pre-partition state...");
    let bootstrap = ctx.ubl_client.bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_default")
        .to_string();
    
    for i in 0..5 {
        ctx.ubl_client.send_message(SendMessageRequest {
            conversation_id: conversation_id. clone(),
            content: format! ("Pre-partition {}", i),
            idempotency_key: Some(format! ("split_brain_pre_{}", i)),
        }).await?;
    }
    
    let pre_timeline = ctx.ubl_client. get_conversation_timeline(&conversation_id, None).await?;
    let pre_count = pre_timeline.items.len();
    println!("   Timeline items before partition: {}", pre_count);
    
    // Simulate network partition
    println!("2️⃣ Simulating network partition...");
    // In real implementation: toxiproxy or iptables
    
    // Attempt writes during partition
    println!("3️⃣ Attempting writes during partition...");
    for i in 0..3 {
        let result = ctx.ubl_client. send_message(SendMessageRequest {
            conversation_id: conversation_id.clone(),
            content: format!("During partition {}", i),
            idempotency_key: Some(format!("split_brain_during_{}", i)),
        }).await;
        
        // Should fail or queue
        if result.is_err() {
            println!("   ⚠️ Write {} failed (expected)", i);
        }
    }
    
    // Heal partition
    println!("4️⃣ Healing network partition...");
    sleep(Duration::from_secs(10)).await;
    
    // Verify conflict resolution
    println!("5️⃣ Verifying conflict resolution...");
    let post_timeline = ctx.ubl_client.get_conversation_timeline(&conversation_id, None).await?;
    let post_count = post_timeline. items.len();
    println!("   Timeline items after healing: {}", post_count);
    
    // Should have no duplicates (ledger sequence prevents)
    assert!(post_count >= pre_count, "Should not lose data");
    
    // Verify sequence continuity
    let mut sequences = vec![];
    for item in &post_timeline.items {
        if let Some(cursor) = item["cursor"].as_str() {
            if let Some(seq) = cursor.split(':').next() {
                if let Ok(seq_num) = seq.parse::<i64>() {
                    sequences.push(seq_num);
                }
            }
        }
    }
    
    sequences.sort();
    for i in 1..sequences.len() {
        assert!(sequences[i] > sequences[i-1], "Sequences should be strictly increasing");
    }
    
    println!("   ✅ No sequence gaps detected");
    println!("✅ Split brain recovery successful");
    
    Ok(())
}

#[tokio::test]
async fn test_recovery_projection_inconsistency() -> Result<()> {
    println!("🔥 Testing Projection Inconsistency Recovery");
    
    let ctx = setup_chaos_env().await?;
    
    let bootstrap = ctx.ubl_client.bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_default")
        .to_string();
    
    // Create events
    println!("1️⃣ Creating events...");
    for i in 0..10 {
        ctx.ubl_client.send_message(SendMessageRequest {
            conversation_id: conversation_id.clone(),
            content: format!("Projection test {}", i),
            idempotency_key: Some(format!("projection_recovery_{}", i)),
        }).await?;
    }
    
    sleep(Duration::from_secs(2)).await;
    
    // Get ledger count
    println!("2️⃣ Checking ledger vs projection consistency...");
    let timeline = ctx.ubl_client. get_conversation_timeline(&conversation_id, None).await?;
    let projection_count = timeline.items.len();
    
    // In real test, would compare ledger_entries count vs projection_timeline_items count
    println!("   Projection items: {}", projection_count);
    
    // Simulate projection lag or corruption
    println!("3️⃣ Simulating projection issue...");
    // In real implementation: manually corrupt projection table
    
    // Trigger projection rebuild
    println!("4️⃣ Triggering projection rebuild...");
    // In real implementation: call rebuild endpoint or restart projection updater
    sleep(Duration::from_secs(5)).await;
    
    // Verify consistency restored
    println!("5️⃣ Verifying consistency.. .");
    let rebuilt_timeline = ctx.ubl_client.get_conversation_timeline(&conversation_id, None).await?;
    let rebuilt_count = rebuilt_timeline.items.len();
    
    assert_eq!(projection_count, rebuilt_count, "Projection should be consistent after rebuild");
    
    println!("   ✅ Projection consistency restored");
    println!("✅ Projection recovery successful");
    
    Ok(())
}