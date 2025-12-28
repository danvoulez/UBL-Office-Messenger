//! Ledger Integrity Verifier
//!
//! Verifies the chain integrity of the UBL ledger by:
//! 1. Checking that entry_hash is correctly computed
//! 2. Checking that previous_hash matches previous entry_hash
//! 3. Checking sequence is monotonically increasing
//!
//! Usage: cargo run --bin verify-ledger [-- --container <container_id>]

use blake3::Hasher;
use sqlx::PgPool;
use std::collections::HashMap;

#[derive(Debug)]
struct LedgerEntry {
    container_id: String,
    sequence: i64,
    link_hash: String,
    previous_hash: String,
    entry_hash: String,
    ts_unix_ms: i64,
}

#[derive(Debug)]
struct VerificationResult {
    container_id: String,
    total_entries: usize,
    valid_entries: usize,
    errors: Vec<String>,
}

async fn verify_container(pool: &PgPool, container_id: &str) -> VerificationResult {
    let entries: Vec<LedgerEntry> = sqlx::query_as!(
        LedgerEntry,
        r#"
        SELECT container_id, sequence, link_hash, previous_hash, entry_hash, ts_unix_ms
        FROM ledger_entry
        WHERE container_id = $1
        ORDER BY sequence ASC
        "#,
        container_id
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut errors = Vec::new();
    let total_entries = entries.len();
    let mut valid_entries = 0;
    let mut expected_prev_hash = "0x00".to_string();
    let mut expected_seq = 1i64;

    for entry in &entries {
        let mut entry_errors = Vec::new();

        // Check sequence
        if entry.sequence != expected_seq {
            entry_errors.push(format!(
                "Sequence mismatch: expected {}, got {}",
                expected_seq, entry.sequence
            ));
        }

        // Check previous_hash
        if entry.previous_hash != expected_prev_hash {
            entry_errors.push(format!(
                "Previous hash mismatch at seq {}: expected {}, got {}",
                entry.sequence,
                &expected_prev_hash[..16.min(expected_prev_hash.len())],
                &entry.previous_hash[..16.min(entry.previous_hash.len())]
            ));
        }

        // Verify entry_hash computation
        let mut h = Hasher::new();
        h.update(b"ubl:ledger\n");
        h.update(entry.container_id.as_bytes());
        h.update(&entry.sequence.to_be_bytes());
        h.update(entry.link_hash.as_bytes());
        h.update(entry.previous_hash.as_bytes());
        h.update(&entry.ts_unix_ms.to_be_bytes());
        let computed_hash = hex::encode(h.finalize().as_bytes());

        if entry.entry_hash != computed_hash {
            entry_errors.push(format!(
                "Entry hash mismatch at seq {}: computed {} != stored {}",
                entry.sequence,
                &computed_hash[..16],
                &entry.entry_hash[..16.min(entry.entry_hash.len())]
            ));
        }

        if entry_errors.is_empty() {
            valid_entries += 1;
        } else {
            errors.extend(entry_errors);
        }

        // Update for next iteration
        expected_prev_hash = entry.entry_hash.clone();
        expected_seq = entry.sequence + 1;
    }

    VerificationResult {
        container_id: container_id.to_string(),
        total_entries,
        valid_entries,
        errors,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost:5432/ubl_ledger".to_string());

    println!("🔌 Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;
    println!("✅ Connected\n");

    // Parse args
    let args: Vec<String> = std::env::args().collect();
    let container_filter = args
        .iter()
        .position(|a| a == "--container")
        .and_then(|i| args.get(i + 1))
        .cloned();

    // Get containers to verify
    let containers: Vec<String> = if let Some(ref container) = container_filter {
        vec![container.clone()]
    } else {
        sqlx::query_scalar!(
            r#"SELECT DISTINCT container_id FROM ledger_entry ORDER BY container_id"#
        )
        .fetch_all(&pool)
        .await?
    };

    println!("═══════════════════════════════════════════════════════════════");
    println!("📋 VERIFICATION RESULTS");
    println!("═══════════════════════════════════════════════════════════════\n");

    let mut all_valid = true;

    for container_id in containers {
        let result = verify_container(&pool, &container_id).await;

        if result.errors.is_empty() {
            println!(
                "✅ {} — {} entries, all valid",
                result.container_id, result.total_entries
            );
        } else {
            all_valid = false;
            println!(
                "❌ {} — {} entries, {} valid, {} ERRORS:",
                result.container_id,
                result.total_entries,
                result.valid_entries,
                result.errors.len()
            );
            for error in &result.errors[..5.min(result.errors.len())] {
                println!("   └─ {}", error);
            }
            if result.errors.len() > 5 {
                println!("   └─ ... and {} more errors", result.errors.len() - 5);
            }
        }
    }

    println!("\n═══════════════════════════════════════════════════════════════");
    
    if all_valid {
        println!("🏆 LEDGER INTEGRITY: VERIFIED");
        Ok(())
    } else {
        println!("⚠️  LEDGER INTEGRITY: ERRORS FOUND");
        std::process::exit(1);
    }
}



