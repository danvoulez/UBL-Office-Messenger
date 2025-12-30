# 🔐 Cryptography Observability & Security Monitoring

## Critical Principle

**"Cryptography failures are silent killers."**

You must actively monitor: 
- Algorithm correctness
- Key management
- Signature validation
- Hash chain integrity
- Random number generation quality
- Timing attacks
- Side-channel vulnerabilities

---

## Must-Monitor Cryptographic Operations

### 1. Digital Signatures (Ed25519)

#### Signature Generation
```rust
// Metrics
signature_generation_total{algorithm, key_version, result}
signature_generation_duration_seconds{algorithm}
signature_generation_failures{algorithm, error_type}

// Validation
signature_key_usage_count{key_id}  // Monitor for key exhaustion
signature_entropy_quality{key_id}  // Randomness quality
```

#### Signature Verification
```rust
// Metrics
signature_verification_total{algorithm, result}
signature_verification_failures{algorithm, failure_reason}
signature_verification_duration_seconds{algorithm}

// Critical alerts
invalid_signature_rate > 0.001  // Should be near-zero
signature_verification_always_succeeds  // Broken validation! 
```

### 2. Hash Functions (SHA-256/BLAKE3)

#### Hash Computation
```rust
// Metrics
hash_computation_total{algorithm, context}
hash_computation_duration_seconds{algorithm}
hash_collision_detected{algorithm}  // MUST be zero

// Validation
hash_output_entropy{algorithm}  // Should be ~256 bits
hash_avalanche_effect_score{algorithm}  // Should be ~50%
```

#### Hash Chain Validation
```rust
// Metrics
hash_chain_validations_total{container_id, result}
hash_chain_breaks_detected{container_id, sequence}
hash_chain_length{container_id}

// Critical
hash_chain_integrity == 1. 0  // MUST be 100%
```

### 3. Random Number Generation

#### Entropy Source
```rust
// Metrics
rng_entropy_available_bits
rng_entropy_consumption_rate
rng_entropy_starvation_events  // CRITICAL

// Quality checks
rng_statistical_tests_passed{test_name}
rng_diehard_test_score
rng_monobit_test_result
rng_runs_test_result
```

### 4. Key Management

#### Key Lifecycle
```rust
// Metrics
cryptographic_keys_active{key_type, algorithm}
key_rotation_total{key_type, reason}
key_rotation_overdue{key_type}  // Keys past rotation date
key_age_seconds{key_id}

// Critical
key_compromise_detected{key_id}
key_material_exposure_attempts{key_id}
```

#### Key Storage
```rust
// Metrics
key_storage_access_total{key_id, operation}
key_storage_unauthorized_access_attempts{key_id}
key_storage_backup_age_seconds
key_storage_encryption_valid{storage_type}
```

---

## Cryptographic Invariants (MUST be monitored)

### Invariant 1: Signature Consistency
```rust
// For same message + key, signature MUST be deterministic (Ed25519)
invariant_signature_determinism_violations

// Example check: 
async fn verify_signature_determinism(key: &SigningKey, message: &[u8]) {
    let sig1 = key.sign(message);
    let sig2 = key.sign(message);
    
    if sig1 != sig2 {
        SIGNATURE_DETERMINISM_VIOLATIONS.inc();
        alert!("CRITICAL: Non-deterministic signature detected!");
    }
}
```

### Invariant 2: Hash Consistency
```rust
// Same input MUST always produce same hash
invariant_hash_consistency_violations

// Example check:
async fn verify_hash_consistency(data: &[u8]) {
    let hash1 = sha256(data);
    let hash2 = sha256(data);
    
    if hash1 != hash2 {
        HASH_CONSISTENCY_VIOLATIONS.inc();
        alert!("CRITICAL: Hash function inconsistency!");
    }
}
```

### Invariant 3: Key Uniqueness
```rust
// No two keys should ever be identical
invariant_duplicate_keys_detected

// Example check:
async fn verify_key_uniqueness(new_key: &PublicKey, existing_keys: &[PublicKey]) {
    if existing_keys.contains(new_key) {
        KEY_COLLISION_DETECTED.inc();
        alert!("CRITICAL:  Duplicate key generated - RNG failure!");
    }
}
```

### Invariant 4: Signature Uniqueness (for non-deterministic schemes)
```rust
// Signatures should be unique (if randomized)
invariant_signature_reuse_detected

// For schemes with nonces
async fn verify_signature_uniqueness(sig: &Signature) {
    if signature_cache.contains(sig) {
        SIGNATURE_REUSE_DETECTED.inc();
        alert!("WARNING: Signature reused - possible replay");
    }
}
```

---

## Security Monitoring

### Timing Attack Detection
```rust
// Metrics
crypto_operation_timing_variance{operation, input_size}
crypto_operation_constant_time_violations{operation}

// Measure timing correlation
async fn monitor_timing_attack_vulnerability() {
    // Signature verification should be constant-time
    let timings:  Vec<Duration> = vec! [];
    
    for _ in 0..1000 {
        let start = Instant::now();
        verify_signature(msg, sig, key);
        timings.push(start.elapsed());
    }
    
    let variance = calculate_variance(&timings);
    
    if variance > THRESHOLD {
        TIMING_ATTACK_VULNERABILITY.inc();
        warn!("Potential timing attack vulnerability detected");
    }
}
```

### Side-Channel Monitoring
```rust
// Metrics
cache_timing_anomalies_detected
power_consumption_anomalies_detected  // In hardware deployments
electromagnetic_emission_anomalies_detected

// Monitor for unusual patterns
async fn monitor_side_channels() {
    // Check for cache-timing patterns
    let cache_misses = get_cache_miss_rate();
    
    if cache_misses_correlate_with_secret_data(cache_misses) {
        SIDE_CHANNEL_LEAK_DETECTED.inc();
        alert!("CRITICAL:  Potential side-channel leak!");
    }
}
```

### Cryptographic Downgrade Detection
```rust
// Metrics
crypto_downgrade_attempts_detected
weak_algorithm_usage{algorithm, context}
deprecated_algorithm_usage{algorithm}

// Monitor algorithm usage
async fn monitor_algorithm_strength() {
    if using_weak_algorithm() {
        WEAK_ALGORITHM_USAGE.inc();
        alert!("WARNING: Weak cryptographic algorithm in use");
    }
}
```

---

## Implementation

### `ubl/kernel/rust/ubl-server/src/metrics_crypto.rs`

```rust
//! Cryptography-Specific Metrics

use lazy_static::lazy_static;
use prometheus::{
    IntCounterVec, IntGaugeVec, HistogramVec,
    Opts, Registry,
};

lazy_static! {
    // Signature Operations
    pub static ref SIGNATURE_GENERATION_TOTAL: IntCounterVec = IntCounterVec:: new(
        Opts::new("signature_generation_total", "Signature generation operations"),
        &["algorithm", "key_version", "result"]
    ).unwrap();
    
    pub static ref SIGNATURE_GENERATION_DURATION:  HistogramVec = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "signature_generation_duration_seconds",
            "Signature generation duration"
        ).buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05]),
        &["algorithm"]
    ).unwrap();
    
    pub static ref SIGNATURE_VERIFICATION_TOTAL: IntCounterVec = IntCounterVec:: new(
        Opts::new("signature_verification_total", "Signature verification operations"),
        &["algorithm", "result"]
    ).unwrap();
    
    pub static ref SIGNATURE_VERIFICATION_FAILURES: IntCounterVec = IntCounterVec:: new(
        Opts::new("signature_verification_failures_total", "Signature verification failures"),
        &["algorithm", "failure_reason"]
    ).unwrap();
    
    pub static ref SIGNATURE_VERIFICATION_DURATION: HistogramVec = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "signature_verification_duration_seconds",
            "Signature verification duration"
        ).buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05]),
        &["algorithm"]
    ).unwrap();
    
    // Hash Operations
    pub static ref HASH_COMPUTATION_TOTAL:  IntCounterVec = IntCounterVec::new(
        Opts::new("hash_computation_total", "Hash computation operations"),
        &["algorithm", "context"]
    ).unwrap();
    
    pub static ref HASH_COMPUTATION_DURATION: HistogramVec = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "hash_computation_duration_seconds",
            "Hash computation duration"
        ).buckets(vec![0.00001, 0.00005, 0.0001, 0.0005, 0.001, 0.005]),
        &["algorithm"]
    ).unwrap();
    
    pub static ref HASH_COLLISION_DETECTED: IntCounterVec = IntCounterVec:: new(
        Opts::new("hash_collision_detected", "Hash collisions detected (MUST be zero)"),
        &["algorithm"]
    ).unwrap();
    
    // Hash Chain
    pub static ref HASH_CHAIN_VALIDATIONS:  IntCounterVec = IntCounterVec::new(
        Opts::new("hash_chain_validations_total", "Hash chain validation checks"),
        &["container_id", "result"]
    ).unwrap();
    
    pub static ref HASH_CHAIN_BREAKS:  IntCounterVec = IntCounterVec::new(
        Opts::new("hash_chain_breaks_detected", "Hash chain breaks detected"),
        &["container_id", "sequence"]
    ).unwrap();
    
    pub static ref HASH_CHAIN_LENGTH:  IntGaugeVec = IntGaugeVec::new(
        Opts::new("hash_chain_length", "Current hash chain length"),
        &["container_id"]
    ).unwrap();
    
    // RNG Quality
    pub static ref RNG_ENTROPY_AVAILABLE:  IntGaugeVec = IntGaugeVec::new(
        Opts::new("rng_entropy_available_bits", "Available entropy in bits"),
        &["source"]
    ).unwrap();
    
    pub static ref RNG_ENTROPY_STARVATION: IntCounterVec = IntCounterVec::new(
        Opts::new("rng_entropy_starvation_events", "RNG entropy starvation events"),
        &["source"]
    ).unwrap();
    
    pub static ref RNG_STATISTICAL_TESTS:  IntCounterVec = IntCounterVec::new(
        Opts::new("rng_statistical_tests_total", "RNG statistical test results"),
        &["test_name", "result"]
    ).unwrap();
    
    // Key Management
    pub static ref CRYPTOGRAPHIC_KEYS_ACTIVE: IntGaugeVec = IntGaugeVec::new(
        Opts::new("cryptographic_keys_active", "Active cryptographic keys"),
        &["key_type", "algorithm"]
    ).unwrap();
    
    pub static ref KEY_ROTATION_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new("key_rotation_total", "Key rotation operations"),
        &["key_type", "reason"]
    ).unwrap();
    
    pub static ref KEY_ROTATION_OVERDUE: IntGaugeVec = IntGaugeVec::new(
        Opts::new("key_rotation_overdue", "Keys past rotation deadline"),
        &["key_type"]
    ).unwrap();
    
    pub static ref KEY_AGE_SECONDS: IntGaugeVec = IntGaugeVec::new(
        Opts::new("key_age_seconds", "Key age in seconds"),
        &["key_id"]
    ).unwrap();
    
    pub static ref KEY_COMPROMISE_DETECTED: IntCounterVec = IntCounterVec::new(
        Opts::new("key_compromise_detected", "Potential key compromise detected"),
        &["key_id", "detection_method"]
    ).unwrap();
    
    pub static ref KEY_STORAGE_UNAUTHORIZED_ACCESS: IntCounterVec = IntCounterVec::new(
        Opts::new("key_storage_unauthorized_access_attempts", "Unauthorized key access attempts"),
        &["key_id", "source"]
    ).unwrap();
    
    // Cryptographic Invariants
    pub static ref SIGNATURE_DETERMINISM_VIOLATIONS: IntCounterVec = IntCounterVec::new(
        Opts::new("signature_determinism_violations", "Non-deterministic signature violations"),
        &["key_id"]
    ).unwrap();
    
    pub static ref HASH_CONSISTENCY_VIOLATIONS: IntCounterVec = IntCounterVec::new(
        Opts::new("hash_consistency_violations", "Hash consistency violations"),
        &["algorithm"]
    ).unwrap();
    
    pub static ref KEY_COLLISION_DETECTED: IntCounterVec = IntCounterVec:: new(
        Opts::new("key_collision_detected", "Duplicate key detected"),
        &["key_type"]
    ).unwrap();
    
    pub static ref SIGNATURE_REUSE_DETECTED:  IntCounterVec = IntCounterVec::new(
        Opts::new("signature_reuse_detected", "Signature reuse detected"),
        &["context"]
    ).unwrap();
    
    // Security Monitoring
    pub static ref TIMING_ATTACK_VULNERABILITY:  IntCounterVec = IntCounterVec::new(
        Opts::new("timing_attack_vulnerability_detected", "Potential timing attack vulnerability"),
        &["operation"]
    ).unwrap();
    
    pub static ref SIDE_CHANNEL_LEAK_DETECTED: IntCounterVec = IntCounterVec::new(
        Opts::new("side_channel_leak_detected", "Potential side-channel leak"),
        &["channel_type", "operation"]
    ).unwrap();
    
    pub static ref CRYPTO_DOWNGRADE_ATTEMPTS:  IntCounterVec = IntCounterVec::new(
        Opts::new("crypto_downgrade_attempts_detected", "Cryptographic downgrade attempts"),
        &["algorithm", "source"]
    ).unwrap();
    
    pub static ref WEAK_ALGORITHM_USAGE: IntCounterVec = IntCounterVec::new(
        Opts::new("weak_algorithm_usage", "Weak cryptographic algorithm usage"),
        &["algorithm", "context"]
    ).unwrap();
    
    // Timing Analysis
    pub static ref CRYPTO_OPERATION_TIMING_VARIANCE: HistogramVec = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "crypto_operation_timing_variance",
            "Timing variance for crypto operations"
        ).buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1]),
        &["operation", "input_size"]
    ).unwrap();
    
    pub static ref CONSTANT_TIME_VIOLATIONS: IntCounterVec = IntCounterVec::new(
        Opts::new("crypto_operation_constant_time_violations", "Constant-time violations"),
        &["operation"]
    ).unwrap();
}

pub fn register_crypto_metrics(registry: &Registry) {
    registry.register(Box::new(SIGNATURE_GENERATION_TOTAL.clone())).unwrap();
    registry.register(Box::new(SIGNATURE_GENERATION_DURATION.clone())).unwrap();
    registry.register(Box::new(SIGNATURE_VERIFICATION_TOTAL. clone())).unwrap();
    registry.register(Box::new(SIGNATURE_VERIFICATION_FAILURES.clone())).unwrap();
    registry.register(Box::new(SIGNATURE_VERIFICATION_DURATION.clone())).unwrap();
    
    registry.register(Box::new(HASH_COMPUTATION_TOTAL.clone())).unwrap();
    registry.register(Box::new(HASH_COMPUTATION_DURATION.clone())).unwrap();
    registry.register(Box::new(HASH_COLLISION_DETECTED.clone())).unwrap();
    
    registry.register(Box::new(HASH_CHAIN_VALIDATIONS.clone())).unwrap();
    registry.register(Box::new(HASH_CHAIN_BREAKS.clone())).unwrap();
    registry.register(Box::new(HASH_CHAIN_LENGTH.clone())).unwrap();
    
    registry. register(Box::new(RNG_ENTROPY_AVAILABLE.clone())).unwrap();
    registry.register(Box::new(RNG_ENTROPY_STARVATION.clone())).unwrap();
    registry.register(Box:: new(RNG_STATISTICAL_TESTS.clone())).unwrap();
    
    registry.register(Box::new(CRYPTOGRAPHIC_KEYS_ACTIVE.clone())).unwrap();
    registry.register(Box::new(KEY_ROTATION_TOTAL.clone())).unwrap();
    registry.register(Box::new(KEY_ROTATION_OVERDUE. clone())).unwrap();
    registry.register(Box::new(KEY_AGE_SECONDS. clone())).unwrap();
    registry.register(Box::new(KEY_COMPROMISE_DETECTED. clone())).unwrap();
    registry.register(Box::new(KEY_STORAGE_UNAUTHORIZED_ACCESS.clone())).unwrap();
    
    registry.register(Box::new(SIGNATURE_DETERMINISM_VIOLATIONS.clone())).unwrap();
    registry.register(Box::new(HASH_CONSISTENCY_VIOLATIONS.clone())).unwrap();
    registry.register(Box::new(KEY_COLLISION_DETECTED.clone())).unwrap();
    registry.register(Box::new(SIGNATURE_REUSE_DETECTED.clone())).unwrap();
    
    registry.register(Box::new(TIMING_ATTACK_VULNERABILITY.clone())).unwrap();
    registry.register(Box::new(SIDE_CHANNEL_LEAK_DETECTED. clone())).unwrap();
    registry.register(Box::new(CRYPTO_DOWNGRADE_ATTEMPTS.clone())).unwrap();
    registry.register(Box:: new(WEAK_ALGORITHM_USAGE.clone())).unwrap();
    
    registry.register(Box::new(CRYPTO_OPERATION_TIMING_VARIANCE.clone())).unwrap();
    registry.register(Box:: new(CONSTANT_TIME_VIOLATIONS.clone())).unwrap();
}
```

---

### Cryptographic Validation Service

### `ubl/kernel/rust/ubl-server/src/crypto/validator.rs`

```rust name=src/crypto/validator.rs
//! Cryptographic Validation and Monitoring

use ed25519_dalek: :{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use sha2::{Sha256, Digest};
use rand::rngs::OsRng;
use std::time:: Instant;
use tracing::{warn, error, info};

use crate::metrics_crypto::*;

pub struct CryptoValidator {
    // Configuration
    timing_sample_size: usize,
    timing_variance_threshold: f64,
}

impl CryptoValidator {
    pub fn new() -> Self {
        Self {
            timing_sample_size: 1000,
            timing_variance_threshold: 0.05, // 5% variance threshold
        }
    }
    
    /// Validate signature determinism (Ed25519 should be deterministic)
    pub async fn validate_signature_determinism(&self, key: &SigningKey) -> bool {
        let message = b"determinism test message";
        
        let sig1 = key.sign(message);
        let sig2 = key.sign(message);
        
        let is_deterministic = sig1.to_bytes() == sig2.to_bytes();
        
        if ! is_deterministic {
            error!("CRITICAL: Non-deterministic signature detected!");
            SIGNATURE_DETERMINISM_VIOLATIONS
                .with_label_values(&["ed25519"])
                .inc();
        }
        
        is_deterministic
    }
    
    /// Validate hash consistency
    pub async fn validate_hash_consistency(&self, data: &[u8]) -> bool {
        let hash1 = Sha256::digest(data);
        let hash2 = Sha256::digest(data);
        
        let is_consistent = hash1 == hash2;
        
        if !is_consistent {
            error!("CRITICAL: Hash function inconsistency detected!");
            HASH_CONSISTENCY_VIOLATIONS
                . with_label_values(&["sha256"])
                .inc();
        }
        
        is_consistent
    }
    
    /// Check for timing attack vulnerabilities
    pub async fn check_timing_attack_vulnerability(&self) -> bool {
        let key = SigningKey::generate(&mut OsRng);
        let public_key = key.verifying_key();
        let message = b"timing test message";
        
        // Generate valid signature
        let valid_sig = key.sign(message);
        
        // Generate invalid signature
        let mut invalid_sig_bytes = valid_sig.to_bytes();
        invalid_sig_bytes[0] ^= 0xFF; // Flip bits
        let invalid_sig = Signature::from_bytes(&invalid_sig_bytes);
        
        // Measure timing for valid signatures
        let mut valid_timings = Vec::with_capacity(self.timing_sample_size);
        for _ in 0..self.timing_sample_size {
            let start = Instant::now();
            let _ = public_key.verify(message, &valid_sig);
            valid_timings.push(start. elapsed());
        }
        
        // Measure timing for invalid signatures
        let mut invalid_timings = Vec::with_capacity(self.timing_sample_size);
        for _ in 0..self.timing_sample_size {
            let start = Instant::now();
            let _ = public_key.verify(message, &invalid_sig);
            invalid_timings.push(start.elapsed());
        }
        
        // Calculate means and variance
        let valid_mean = valid_timings.iter().sum::<std::time::Duration>().as_secs_f64() 
            / valid_timings. len() as f64;
        let invalid_mean = invalid_timings.iter().sum::<std::time::Duration>().as_secs_f64() 
            / invalid_timings.len() as f64;
        
        let timing_difference = (valid_mean - invalid_mean).abs() / valid_mean;
        
        // Record variance
        CRYPTO_OPERATION_TIMING_VARIANCE
            .with_label_values(&["signature_verification", "32"])
            .observe(timing_difference);
        
        let is_vulnerable = timing_difference > self.timing_variance_threshold;
        
        if is_vulnerable {
            warn!(
                "Potential timing attack vulnerability:  {:.2}% difference",
                timing_difference * 100.0
            );
            TIMING_ATTACK_VULNERABILITY
                .with_label_values(&["signature_verification"])
                .inc();
        }
        
        ! is_vulnerable
    }
    
    /// Validate key uniqueness
    pub async fn validate_key_uniqueness(&self, new_key: &VerifyingKey, existing_keys: &[VerifyingKey]) -> bool {
        let is_unique = ! existing_keys.iter().any(|k| k.as_bytes() == new_key.as_bytes());
        
        if ! is_unique {
            error! ("CRITICAL:  Duplicate key detected - RNG failure!");
            KEY_COLLISION_DETECTED
                .with_label_values(&["ed25519"])
                .inc();
        }
        
        is_unique
    }
    
    /// Test RNG quality with basic statistical tests
    pub async fn test_rng_quality(&self) -> bool {
        let sample_size = 10000;
        let mut rng = OsRng;
        let mut samples = vec![0u8; sample_size];
        
        use rand::RngCore;
        rng.fill_bytes(&mut samples);
        
        // Monobit test (should be ~50% ones)
        let ones_count = samples.iter()
            .flat_map(|byte| (0..8).map(move |i| (byte >> i) & 1))
            .filter(|&bit| bit == 1)
            .count();
        
        let total_bits = sample_size * 8;
        let ones_ratio = ones_count as f64 / total_bits as f64;
        
        let monobit_pass = (ones_ratio - 0.5).abs() < 0.01; // Within 1%
        
        RNG_STATISTICAL_TESTS
            .with_label_values(&["monobit", if monobit_pass { "pass" } else { "fail" }])
            .inc();
        
        if !monobit_pass {
            warn!("RNG monobit test failed:  {:.4}", ones_ratio);
        }
        
        // Runs test (check for patterns)
        let mut runs = 0;
        let bits:  Vec<u8> = samples. iter()
            .flat_map(|byte| (0..8).map(move |i| (byte >> i) & 1))
            .collect();
        
        for i in 1..bits.len() {
            if bits[i] != bits[i-1] {
                runs += 1;
            }
        }
        
        let expected_runs = total_bits / 2;
        let runs_ratio = runs as f64 / expected_runs as f64;
        let runs_pass = (runs_ratio - 1.0).abs() < 0.1; // Within 10%
        
        RNG_STATISTICAL_TESTS
            .with_label_values(&["runs", if runs_pass { "pass" } else { "fail" }])
            .inc();
        
        if !runs_pass {
            warn!("RNG runs test failed:  {:.4}", runs_ratio);
        }
        
        monobit_pass && runs_pass
    }
    
    /// Validate hash chain integrity
    pub async fn validate_hash_chain(
        &self,
        entries: &[(Vec<u8>, Vec<u8>)], // (hash, previous_hash)
    ) -> bool {
        for i in 1..entries.len() {
            let (current_hash, previous_hash) = &entries[i];
            let (expected_previous, _) = &entries[i-1];
            
            if previous_hash != expected_previous {
                error!(
                    "Hash chain break at position {}: expected {: ?}, got {:?}",
                    i, hex::encode(expected_previous), hex::encode(previous_hash)
                );
                
                HASH_CHAIN_BREAKS
                    .with_label_values(&["unknown", &i.to_string()])
                    .inc();
                
                return false;
            }
        }
        
        HASH_CHAIN_VALIDATIONS
            .with_label_values(&["unknown", "pass"])
            .inc();
        
        true
    }
    
    /// Monitor constant-time operations
    pub async fn verify_constant_time_operation<F>(&self, operation: F, operation_name: &str) -> bool
    where
        F:  Fn() -> (),
    {
        let mut timings = Vec::with_capacity(self.timing_sample_size);
        
        for _ in 0..self.timing_sample_size {
            let start = Instant::now();
            operation();
            timings.push(start.elapsed());
        }
        
        // Calculate variance
        let mean = timings. iter().sum::<std::time::Duration>().as_secs_f64() / timings.len() as f64;
        let variance:  f64 = timings.iter()
            .map(|t| {
                let diff = t.as_secs_f64() - mean;
                diff * diff
            })
            .sum::<f64>() / timings.len() as f64;
        
        let std_dev = variance.sqrt();
        let coefficient_of_variation = std_dev / mean;
        
        let is_constant_time = coefficient_of_variation < 0.1; // Less than 10% variation
        
        if !is_constant_time {
            warn!(
                "Operation '{}' is not constant-time: CoV = {:.4}",
                operation_name, coefficient_of_variation
            );
            CONSTANT_TIME_VIOLATIONS
                .with_label_values(&[operation_name])
                .inc();
        }
        
        is_constant_time
    }
    
    /// Run comprehensive crypto validation suite
    pub async fn run_validation_suite(&self) -> ValidationReport {
        info!("Running comprehensive cryptographic validation suite");
        
        let key = SigningKey::generate(&mut OsRng);
        
        let signature_determinism = self.validate_signature_determinism(&key).await;
        let hash_consistency = self. validate_hash_consistency(b"test data").await;
        let timing_security = self.check_timing_attack_vulnerability().await;
        let rng_quality = self.test_rng_quality().await;
        
        let all_passed = signature_determinism && hash_consistency && timing_security && rng_quality;
        
        ValidationReport {
            signature_determinism,
            hash_consistency,
            timing_security,
            rng_quality,
            overall_status: if all_passed { "PASS" } else { "FAIL" },
        }
    }
}

#[derive(Debug)]
pub struct ValidationReport {
    pub signature_determinism: bool,
    pub hash_consistency: bool,
    pub timing_security:  bool,
    pub rng_quality: bool,
    pub overall_status: &'static str,
}
```

---

### Periodic Validation Task

### `ubl/kernel/rust/ubl-server/src/crypto/validation_task.rs`

```rust name=src/crypto/validation_task.rs
//! Periodic cryptographic validation task

use tokio::time::{interval, Duration};
use tracing: :{info, error};

use super::validator::CryptoValidator;

pub async fn run_periodic_crypto_validation() {
    let validator = CryptoValidator::new();
    let mut interval = interval(Duration::from_secs(3600)); // Every hour
    
    loop {
        interval.tick().await;
        
        info!("Running periodic cryptographic validation");
        
        let report = validator.run_validation_suite().await;
        
        if report.overall_status == "FAIL" {
            error!(
                "Cryptographic validation FAILED: {: ?}",
                report
            );
            // Send alert
        } else {
            info! ("Cryptographic validation passed");
        }
    }
}
```

---

### Alert Rules for Cryptography

### `observability/prometheus/alerts/cryptography.yml`

```yaml name=cryptography.yml
groups:
  - name: cryptography_alerts
    interval: 30s
    rules:
      # CRITICAL - Zero Tolerance
      - alert: SignatureDeterminismViolation
        expr: increase(signature_determinism_violations[5m]) > 0
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "Non-deterministic signature detected"
          description: "Ed25519 signatures are non-deterministic - critical crypto failure!"
          runbook_url: "https://runbooks/crypto-determinism-violation"
      
      - alert: HashConsistencyViolation
        expr: increase(hash_consistency_violations[5m]) > 0
        for: 0m
        labels:
          severity: critical
        annotations: 
          summary: "Hash function inconsistency detected"
          description: "Hash function producing inconsistent results - critical failure!"
      
      - alert: KeyCollisionDetected
        expr: increase(key_collision_detected[5m]) > 0
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "Duplicate cryptographic key detected"
          description:  "RNG failure - duplicate key generated!"
      
      - alert: HashChainBreak
        expr:  increase(hash_chain_breaks_detected[5m]) > 0
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "Hash chain integrity broken"
          description: "Hash chain break detected - possible tampering!"
      
      - alert: HashCollision
        expr: increase(hash_collision_detected[1h]) > 0
        for: 0m
        labels:
          severity:  critical
        annotations:
          summary: "Hash collision detected"
          description: "Hash collision detected - cryptographic failure!"
      
      # HIGH
      - alert: HighSignatureVerificationFailureRate
        expr: rate(signature_verification_failures_total[5m]) > 0. 01
        for: 5m
        labels:
          severity: high
        annotations:
          summary: "High signature verification failure rate"
          description: "Signature verification failing at {{ $value }}/sec"
      
      - alert: RNGEntropyStarvation
        expr: increase(rng_entropy_starvation_events[5m]) > 0
        for: 1m
        labels:
          severity: high
        annotations:
          summary: "RNG entropy starvation"
          description: "Random number generator running low on entropy"
      
      - alert: RNGStatisticalTestFailure
        expr: rng_statistical_tests_total{result="fail"} > 0
        for: 5m
        labels:
          severity: high
        annotations:
          summary: "RNG statistical test failure"
          description: "RNG failing statistical quality tests:  {{ $labels.test_name }}"
      
      # WARNING
      - alert: KeyRotationOverdue
        expr: key_rotation_overdue > 0
        for: 1h
        labels:
          severity:  warning
        annotations:
          summary: "Cryptographic key rotation overdue"
          description: "{{ $value }} keys past rotation deadline"
      
      - alert: TimingAttackVulnerability
        expr: increase(timing_attack_vulnerability_detected[1h]) > 0
        for: 0m
        labels:
          severity: warning
        annotations:
          summary: "Potential timing attack vulnerability"
          description:  "Timing variance detected in {{ $labels.operation }}"
      
      - alert: WeakAlgorithmUsage
        expr: increase(weak_algorithm_usage[10m]) > 0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Weak cryptographic algorithm in use"
          description: "Algorithm {{ $labels.algorithm }} is considered weak"
      
      - alert: UnauthorizedKeyAccess
        expr: increase(key_storage_unauthorized_access_attempts[5m]) > 0
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "Unauthorized key storage access attempt"
          description: "Unauthorized access to key {{ $labels.key_id }}"
```

---

## Summary

Cryptography observability MUST include:

### Zero-Tolerance Alerts (CRITICAL)
✅ Signature determinism violations:  **0**
✅ Hash consistency violations:  **0**
✅ Key collisions: **0**
✅ Hash chain breaks: **0**
✅ Hash collisions:  **0**

### Continuous Monitoring
✅ Signature generation/verification metrics
✅ Hash computation metrics
✅ RNG quality tests (monobit, runs tests)
✅ Key lifecycle tracking
✅ Timing attack detection
✅ Side-channel monitoring
✅ Constant-time operation verification

### Periodic Validation (Hourly)
✅ Signature determinism check
✅ Hash consistency check
✅ Timing attack vulnerability scan
✅ RNG statistical tests

### Security Monitoring
✅ Key compromise detection
✅ Unauthorized access attempts
✅ Cryptographic downgrade attempts
✅ Weak algorithm usage

**"In cryptography, you can't be too paranoid."** 🔐

This ensures your cryptographic operations are **continuously validated and monitored** for correctness! 