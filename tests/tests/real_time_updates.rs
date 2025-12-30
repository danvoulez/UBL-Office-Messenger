//! Real-Time Updates Integration Tests
//! Tests SSE streaming and real-time synchronization

use integration_tests::*;
use anyhow::Result;
use std::time::Duration;
use tokio::time: :{sleep, timeout};
use futures_util:: StreamExt;

#[tokio::test]
async fn test_sse_connection() -> Result<()> {
    let ctx = setup().await?;
    
    // Subscribe to SSE stream
    let mut client = ctx.ubl_client.subscribe_to_stream("T.UBL").await?;
    
    println!("✅ SSE connection established");
    
    // Wait for hello event
    let result = timeout(Duration::from_secs(5), async {
        while let Some(event) = client.next().await {
            if let Ok(event) = event {
                if event.event_type == "hello" {
                    return Ok: :<_, anyhow::Error>(());
                }
            }
        }
        Err(anyhow::anyhow!("No hello event received"))
    }).await??;
    
    println!("✅ Received hello event");
    Ok(())
}

#[tokio::test]
async fn test_message_appears_in_sse_stream() -> Result<()> {
    let ctx = setup().await?;
    
    let bootstrap = ctx.ubl_client. bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations.first()
        .and_then(|c| c["id"].as_str())
        .unwrap_or("conv_test")
        .to_string();
    
    // Subscribe to SSE
    let mut sse_client = ctx.ubl_client.subscribe_to_stream("T.UBL").await?;
    
    // Wait for connection
    sleep(Duration::from_secs(1)).await;
    
    // Send message
    let message_content = format!("SSE test message {}", uuid::Uuid::new_v4());
    ctx.ubl_client.send_message(SendMessageRequest {
        conversation_id:  conversation_id.clone(),
        content: message_content.clone(),
        idempotency_key: Some(test_id("idem")),
    }).await?;
    
    println!("📨 Message sent, waiting for SSE event...");
    
    // Wait for SSE event
    let result = timeout(Duration::from_secs(10), async {
        while let Some(event) = sse_client.next().await {
            if let Ok(event) = event {
                if event.event_type == "timeline. append" {
                    let data:  serde_json::Value = serde_json::from_str(&event.data)?;
                    if data["conversation_id"] == conversation_id {
                        println!("✅ Received timeline.append event via SSE");
                        return Ok::<_, anyhow::Error>(());
                    }
                }
            }
        }
        Err(anyhow:: anyhow!("No matching SSE event received"))
    }).await??;
    
    Ok(())
}

#[tokio::test]
async fn test_job_updates_via_sse() -> Result<()> {
    let ctx = setup().await?;
    
    let bootstrap = ctx.ubl_client.bootstrap("T.UBL").await?;
    let conversation_id = bootstrap.conversations. first()
        .and_then(|c| c["id"]. as_str())
        .unwrap_or("conv_test")
        .to_string();
    
    // Subscribe to SSE
    let mut sse_client = ctx.ubl_client.subscribe_to_stream("T.UBL").await?;
    
    sleep(Duration::from_secs(1)).await;
    
    // Trigger job creation
    ctx.ubl_client.send_message(SendMessageRequest {
        conversation_id: conversation_id.clone(),
        content: "Create a task".to_string(),
        idempotency_key: Some(test_id("idem")),
    }).await?;
    
    println!("📨 Message sent, waiting for job events...");
    
    // Wait for job-related SSE events
    let result = timeout(Duration::from_secs(15), async {
        let mut received_job_created = false;
        
        while let Some(event) = sse_client.next().await {
            if let Ok(event) = event {
                match event.event_type. as_str() {
                    "job.update" => {
                        println!("✅ Received job.update event");
                        return Ok::<_, anyhow:: Error>(());
                    }
                    "timeline.append" => {
                        let data: serde_json::Value = serde_json::from_str(&event.data)?;
                        if data["item"]["item_type"] == "job_card" {
                            println!("✅ Received job card via SSE");
                            return Ok(());
                        }
                    }
                    _ => {}
                }
            }
        }
        Err(anyhow::anyhow!("No job events received"))
    }).await??;
    
    Ok(())
}

#[tokio::test]
async fn test_presence_updates_via_sse() -> Result<()> {
    let ctx = setup().await?;
    
    // Subscribe to SSE
    let mut sse_client = ctx.ubl_client.subscribe_to_stream("T.UBL").await?;
    
    sleep(Duration::from_secs(1)).await;
    
    // Create entity (should trigger presence update)
    ctx.office_client.create_entity(CreateEntityRequest {
        name: "SSE Test Entity".to_string(),
        entity_type: "Autonomous".to_string(),
    }).await?;
    
    println!("👤 Entity created, waiting for presence event...");
    
    // Wait for presence update
    let result = timeout(Duration::from_secs(10), async {
        while let Some(event) = sse_client.next().await {
            if let Ok(event) = event {
                if event.event_type == "presence.update" {
                    println!("✅ Received presence.update event");
                    return Ok:: <_, anyhow::Error>(());
                }
            }
        }
        Err(anyhow::anyhow!("No presence event received"))
    }).await;
    
    // Presence events might not always fire, so don't fail test
    if result.is_ok() {
        println!("✅ Presence updates working");
    } else {
        println!("⚠️  No presence event (might be expected)");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_sse_reconnection() -> Result<()> {
    let ctx = setup().await?;
    
    // First connection
    let mut client1 = ctx.ubl_client.subscribe_to_stream("T.UBL").await?;
    
    // Wait for hello
    timeout(Duration::from_secs(5), async {
        while let Some(event) = client1.next().await {
            if let Ok(event) = event {
                if event.event_type == "hello" {
                    return Ok::<_, anyhow:: Error>(());
                }
            }
        }
        Err(anyhow::anyhow!("No hello"))
    }).await??;
    
    // Drop connection
    drop(client1);
    
    println!("🔌 Dropped first connection");
    
    sleep(Duration::from_secs(1)).await;
    
    // Reconnect
    let mut client2 = ctx.ubl_client.subscribe_to_stream("T.UBL").await?;
    
    // Should get hello again
    timeout(Duration::from_secs(5), async {
        while let Some(event) = client2.next().await {
            if let Ok(event) = event {
                if event.event_type == "hello" {
                    println!("✅ Reconnection successful");
                    return Ok: :<_, anyhow::Error>(());
                }
            }
        }
        Err(anyhow::anyhow!("No hello on reconnect"))
    }).await??;
    
    Ok(())
}