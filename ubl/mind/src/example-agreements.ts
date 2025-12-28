/**
 * Example: Agreements and ABAC in Action
 * 
 * This demonstrates the UBL 2.0 approach to authorization:
 * - Every permission traces to an Agreement
 * - Roles are derived, not stored
 * - Authorization is auditable
 */

import { AgreementStore, AgreementTemplates, Permission } from './agreements.js';
import { ABAC, PolicyEngine, CommonPolicies } from './abac.js';

async function main() {
  console.log('🤝 UBL 2.0 - Agreements & ABAC Demo\n');
  console.log('═'.repeat(60));

  // ==========================================================================
  // 1. Create the store and engines
  // ==========================================================================
  
  const store = new AgreementStore();
  const abac = new ABAC(store);
  const policyEngine = new PolicyEngine(abac);

  // ==========================================================================
  // 2. Create some entities
  // ==========================================================================
  
  console.log('\n📋 Creating Entities...\n');

  const acmeCorp = store.createEntity({
    type: 'Organization',
    name: 'Acme Corporation'
  });
  console.log(`   ✓ Created: ${acmeCorp.name} (${acmeCorp.id})`);

  const alice = store.createEntity({
    type: 'Person',
    name: 'Alice Engineer'
  });
  console.log(`   ✓ Created: ${alice.name} (${alice.id})`);

  const bob = store.createEntity({
    type: 'Person',
    name: 'Bob Manager'
  });
  console.log(`   ✓ Created: ${bob.name} (${bob.id})`);

  const system = store.createEntity({
    type: 'System',
    name: 'UBL System'
  });
  console.log(`   ✓ Created: ${system.name} (${system.id})`);

  // ==========================================================================
  // 3. Create an Employment Agreement
  // ==========================================================================
  
  console.log('\n📝 Creating Employment Agreement...\n');

  const alicePermissions: Permission[] = [
    { action: 'read', resource: '*' },
    { action: 'write', resource: 'code' },
    { action: 'deploy', resource: 'staging' }
  ];

  const employmentData = AgreementTemplates.employment(
    acmeCorp.id,
    alice.id,
    { position: 'Software Engineer', permissions: alicePermissions },
    { from: Date.now() },  // Valid from now, indefinite
    acmeCorp.id
  );

  const employment = store.createAgreement(employmentData);
  console.log(`   ✓ Created: ${employment.name}`);
  console.log(`     ID: ${employment.id}`);
  console.log(`     Status: ${employment.status}`);
  console.log(`     Parties: ${employment.parties.map(p => p.role).join(', ')}`);

  // ==========================================================================
  // 4. Propose and consent to the agreement
  // ==========================================================================
  
  console.log('\n✍️  Agreement Lifecycle...\n');

  // Propose
  store.proposeAgreement(employment.id, acmeCorp.id);
  console.log(`   ✓ Agreement proposed by Employer`);

  // Employer consents
  store.consentToAgreement(employment.id, acmeCorp.id, 'employer_signature');
  console.log(`   ✓ Employer consented`);

  // Employee consents - this activates the agreement!
  const activatedAgreement = store.consentToAgreement(employment.id, alice.id, 'alice_signature');
  console.log(`   ✓ Employee consented`);
  console.log(`   ✓ Agreement Status: ${activatedAgreement.status}`);

  // ==========================================================================
  // 5. Check derived roles
  // ==========================================================================
  
  console.log('\n👤 Derived Roles for Alice...\n');

  const aliceRoles = store.getRolesForEntity(alice.id);
  for (const role of aliceRoles) {
    console.log(`   Role: ${role.role}`);
    console.log(`   Granted by: ${role.grantedBy}`);
    console.log(`   Permissions:`);
    for (const perm of role.permissions) {
      console.log(`     - ${perm.action}:${perm.resource}`);
    }
  }

  // ==========================================================================
  // 6. ABAC Authorization checks
  // ==========================================================================
  
  console.log('\n🔐 Authorization Checks...\n');

  const checks = [
    { actor: alice.id, action: 'read', resource: 'documents' },
    { actor: alice.id, action: 'write', resource: 'code' },
    { actor: alice.id, action: 'deploy', resource: 'staging' },
    { actor: alice.id, action: 'deploy', resource: 'production' },  // Should DENY
    { actor: bob.id, action: 'read', resource: 'documents' },       // Should DENY (no agreement)
  ];

  for (const check of checks) {
    const result = abac.authorize(check);
    const icon = result.decision === 'ALLOW' ? '✅' : '❌';
    const actor = check.actor === alice.id ? 'Alice' : 'Bob';
    console.log(`   ${icon} ${actor} ${check.action}:${check.resource}`);
    console.log(`      └─ ${result.reason}`);
  }

  // ==========================================================================
  // 7. Add policies
  // ==========================================================================
  
  console.log('\n📜 Adding Policies...\n');

  // System bypass policy
  policyEngine.addPolicy(CommonPolicies.systemBypass(system.id));
  console.log('   ✓ Added: System Bypass policy');

  // Deny dangerous actions
  policyEngine.addPolicy(CommonPolicies.denyDangerousActions(['delete', 'drop', 'truncate']));
  console.log('   ✓ Added: Deny Dangerous Actions policy');

  // ==========================================================================
  // 8. Policy + ABAC evaluation
  // ==========================================================================
  
  console.log('\n🔐 Policy + ABAC Evaluation...\n');

  const policyChecks = [
    { actor: alice.id, action: 'delete', resource: 'database' },    // DENY by policy
    { actor: system.id, action: 'delete', resource: 'database' },   // ALLOW by system bypass
    { actor: alice.id, action: 'read', resource: 'logs' },          // ALLOW by ABAC
  ];

  for (const check of policyChecks) {
    const result = policyEngine.evaluate(check);
    const icon = result.decision === 'ALLOW' ? '✅' : '❌';
    const actor = check.actor === alice.id ? 'Alice' : 
                  check.actor === system.id ? 'System' : 'Unknown';
    console.log(`   ${icon} ${actor} ${check.action}:${check.resource}`);
    console.log(`      └─ ${result.reason}`);
  }

  // ==========================================================================
  // 9. Show event trail
  // ==========================================================================
  
  console.log('\n📜 Agreement Event Trail...\n');

  const events = store.getEvents(employment.id);
  for (const event of events) {
    const time = new Date(event.timestamp).toISOString();
    console.log(`   [${time}] ${event.type}`);
  }

  // ==========================================================================
  // 10. Summary
  // ==========================================================================
  
  console.log('\n' + '═'.repeat(60));
  console.log('\n✨ Key Insights:\n');
  console.log('   • Roles are DERIVED from active Agreements');
  console.log('   • Every permission traces to an Agreement');
  console.log('   • Authorization decisions include provenance');
  console.log('   • Policies add layer above ABAC (fail-closed)');
  console.log('   • All events are recorded for audit');
  console.log('\n   "Every relationship is an Agreement."');
  console.log('   "Authorization is traceable, not opaque."');
}

// Run
const isMainModule = import.meta.url === `file://${process.argv[1]}`;
if (isMainModule) {
  main().catch(console.error);
}
