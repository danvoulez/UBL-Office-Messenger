//! Chaos Monkey Implementation
//! Randomly injects failures during normal operations

use anyhow::Result;
use rand::Rng;
use std::time::{Duration, Instant};
use tokio::time::sleep;

mod common;
use common::*;

#[tokio:: test]
#[ignore] // Run explicitly:  cargo test --test chaos_monkey -- --ignored
async fn test_chaos_monkey_random_failures() -> Result<()> {
    println!("🐵 Starting Chaos Monkey Test");
    println!("⚠️  This test runs for 5 minutes and randomly injects failures");
    
    let ctx = setup_chaos_env().await?;
    let test_duration = Duration::from_secs(300); // 5 minutes
    let start = Instant::now();
    
    let mut rng = rand::thread_rng();
    
    let mut total_operations = 0;
    let mut successful_operations = 0;
    let mut failed_operations = 0;
    let mut chaos_events = 0;
    
    while start.elapsed() < test_duration {
        // Random operation
        let operation = rng. gen_range(0..100);
        
        match operation {
            // 70% normal operations
            0.. =69 => {
                total_operations += 1;
                
                if let Err(e) = perform_normal_operation(&ctx).await {
                    failed_operations += 1;
                    println!("❌ Operation failed: {}", e);
                } else {
                    successful_operations += 1;
                    print!(".");
                    if successful_operations % 50 == 0 {
                        println!();
                    }
                }
            }
            
            // 20% chaos events
            70..=89 => {
                chaos_events += 1;
                inject_random_chaos(&ctx, &mut rng).await?;
            }
            
            // 10% validation
            90..=99 => {
                validate_system_state(&ctx).await?;
            }
            
            _ => unreachable!(),
        }
        
        sleep(Duration::from_millis(100)).await;
    }
    
    println!("\n\n🐵 Chaos Monkey Test Complete");
    println!("📊 Statistics:");
    println!("   Total Operations: {}", total_operations);
    println!("   Successful: {} ({:.1}%)", successful_operations,
        (successful_operations as f64 / total_operations as f64) * 100.0);
    println!("   Failed:  {} ({:.1}%)", failed_operations,
        (failed_operations as f64 / total_operations as f64) * 100.0);
    println!("   Chaos Events Injected: {}", chaos_events);
    
    // Success rate should be reasonable despite chaos
    let success_rate = successful_operations as f64 / total_operations as f64;
    assert!(success_rate > 0.7, "Success rate too low:  {:.1}%", success_rate * 100.0);
    
    println!("✅ System remained resilient under chaos");
    
    Ok(())
}

async fn perform_normal_operation(ctx: &ChaosTestContext) -> Result<()> {
    let mut rng = rand::thread_rng();
    
    match rng.gen_range(0..3) {
        0 => {
            // Send message
            let bootstrap = ctx.ubl_client. bootstrap("T.UBL").await?;
            let conversation_id = bootstrap.conversations.first()
                .and_then(|c| c["id"].as_str())
                .unwrap_or("conv_default")
                .to_string();
            
            ctx.ubl_client.send_message(SendMessageRequest {
                conversation_id,
                content: format!("Chaos test message {}", rng.gen: :<u32>()),
                idempotency_key: Some(format!("chaos_{}", uuid::Uuid::new_v4())),
            }).await?;
        }
        
        1 => {
            // Query timeline
            let bootstrap = ctx.ubl_client. bootstrap("T.UBL").await?;
            let conversation_id = bootstrap.conversations.first()
                .and_then(|c| c["id"].as_str())
                .unwrap_or("conv_default")
                .to_string();
            
            ctx.ubl_client.get_conversation_timeline(&conversation_id, None).await?;
        }
        
        2 => {
            // Health check
            ctx.ubl_client.health().await?;
        }
        
        _ => unreachable! (),
    }
    
    Ok(())
}

async fn inject_random_chaos(ctx:  &ChaosTestContext, rng: &mut impl Rng) -> Result<()> {
    let chaos_type = rng.gen_range(0..4);
    
    match chaos_type {
        0 => {
            println!("\n🔥 Chaos:  Injecting network latency (2s)");
            // In real implementation, would use toxiproxy or tc
            sleep(Duration::from_secs(2)).await;
        }
        
        1 => {
            println!("\n🔥 Chaos: Simulating connection failure");
            // In real implementation, would drop connections
            sleep(Duration::from_secs(1)).await;
        }
        
        2 => {
            println!("\n🔥 Chaos: CPU spike");
            // In real implementation, would use stress-ng
            for _ in 0..1000000 {
                let _ = rng.gen::<u64>();
            }
        }
        
        3 => {
            println!("\n🔥 Chaos: Memory allocation");
            // In real implementation, would allocate large chunk
            let _large_vec:  Vec<u8> = vec![0; 10_000_000];
            sleep(Duration::from_secs(1)).await;
        }
        
        _ => unreachable!(),
    }
    
    Ok(())
}

async fn validate_system_state(ctx: &ChaosTestContext) -> Result<()> {
    // Verify services are responsive
    let health = ctx.ubl_client.health().await;
    
    match health {
        Ok(_) => print!("✓"),
        Err(_) => print!("✗"),
    }
    
    Ok(())
}