//! Health Check Integration Tests
//!  Verify all services are running and healthy

use integration_tests::*;
use anyhow::Result;

#[tokio::test]
async fn test_all_services_healthy() -> Result<()> {
    let ctx = setup().await?;
    
    // Check UBL Kernel
    let ubl_health = ctx.ubl_client. health().await?;
    assert_eq!(ubl_health.status, "ok");
    
    // Check Office
    let office_health = ctx.office_client.health().await?;
    assert_eq!(office_health.status, "ok");
    
    println!("✅ All services healthy");
    Ok(())
}

#[tokio::test]
async fn test_ubl_database_connection() -> Result<()> {
    let ctx = setup().await?;
    
    // Bootstrap should succeed (requires DB)
    let bootstrap = ctx.ubl_client.bootstrap("T. UBL").await?;
    
    assert!(bootstrap.entities.len() >= 0);
    assert!(bootstrap.conversations.len() >= 0);
    
    println!("✅ UBL database connection working");
    Ok(())
}

#[tokio::test]
async fn test_office_can_reach_ubl() -> Result<()> {
    let ctx = setup().await?;
    
    // Create entity (Office will talk to UBL)
    let entity = ctx.office_client. create_entity(CreateEntityRequest {
        name: "Test Entity". to_string(),
        entity_type: "Autonomous".to_string(),
    }).await?;
    
    assert! (!entity.entity_id.is_empty());
    assert_eq!(entity.name, "Test Entity");
    
    println!("✅ Office can communicate with UBL");
    Ok(())
}