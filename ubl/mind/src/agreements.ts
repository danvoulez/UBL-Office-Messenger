/**
 * UBL Agreements - The Universal Primitive
 * 
 * "Every relationship is an Agreement."
 * 
 * In the new UBL 2.0 architecture, Agreements emerge naturally from the
 * Container/Commit model:
 * 
 * 1. An Agreement IS a Container (with its own ledger)
 * 2. Parties are linked via commits to the Agreement container
 * 3. Roles are derived state, not stored objects
 * 4. Permissions flow from Agreement terms
 * 
 * This is more fractal and elegant than treating Agreements as special objects.
 */

// =============================================================================
// CORE TYPES
// =============================================================================

/** Unique identifier for entities */
export type EntityId = string;

/** Unique identifier for agreements (also a container_id) */
export type AgreementId = string;

/** Unix timestamp in milliseconds */
export type Timestamp = number;

/** Time period with explicit bounds */
export interface Validity {
  readonly from: Timestamp;
  readonly until?: Timestamp;  // undefined = indefinite
}

// =============================================================================
// ENTITIES - Anything that can participate in agreements
// =============================================================================

export type EntityType = 'Person' | 'Organization' | 'System' | 'Agent';

export interface Entity {
  readonly id: EntityId;
  readonly type: EntityType;
  readonly name: string;
  readonly publicKey?: string;  // For signature verification
  readonly metadata?: Record<string, unknown>;
  readonly createdAt: Timestamp;
}

// =============================================================================
// AGREEMENTS - The fundamental force
// =============================================================================

/**
 * Agreement types - extensible registry
 * Each type defines the shape of terms and the roles it grants
 */
export type AgreementType = 
  | 'License'        // System ↔ Tenant
  | 'Employment'     // Employer ↔ Employee
  | 'Service'        // Provider ↔ Client
  | 'Custody'        // Custodian ↔ Owner
  | 'Authorization'  // Grantor ↔ Grantee
  | 'Membership'     // Organization ↔ Member
  | 'Testimony'      // Declarant ↔ Witness
  | 'Amendment'      // Modifies another agreement
  | 'Termination'    // Ends another agreement
  | string;          // Custom types

/**
 * Agreement status - lifecycle states
 */
export type AgreementStatus = 
  | 'Draft'      // Being prepared
  | 'Proposed'   // Awaiting consent
  | 'Active'     // In effect
  | 'Suspended'  // Temporarily inactive
  | 'Fulfilled'  // Completed successfully
  | 'Terminated' // Ended early
  | 'Expired';   // Past validity

/**
 * A party to an agreement with their role
 */
export interface Party {
  readonly entityId: EntityId;
  readonly role: string;  // e.g., 'Employer', 'Employee', 'Witness'
  readonly consentedAt?: Timestamp;
  readonly signature?: string;
}

/**
 * Permission granted by an agreement
 */
export interface Permission {
  readonly action: string;    // e.g., 'read', 'write', 'deploy', '*'
  readonly resource: string;  // e.g., 'code', 'staging', '*'
  readonly conditions?: Record<string, unknown>;  // Optional constraints
}

/**
 * The Agreement - fundamental primitive
 * 
 * Note: An Agreement IS also a Container. The agreement_id is a container_id.
 * The agreement's history lives in its own ledger.
 */
export interface Agreement {
  readonly id: AgreementId;
  readonly type: AgreementType;
  readonly name: string;
  readonly description?: string;
  
  // Parties
  readonly parties: readonly Party[];
  
  // Terms
  readonly terms: {
    readonly permissions: readonly Permission[];
    readonly obligations?: readonly string[];
    readonly custom?: Record<string, unknown>;
  };
  
  // Lifecycle
  readonly status: AgreementStatus;
  readonly validity: Validity;
  
  // Provenance
  readonly parentAgreementId?: AgreementId;  // For amendments
  readonly createdAt: Timestamp;
  readonly createdBy: EntityId;
  
  // Proof (from Body)
  readonly atomHash?: string;    // Hash of canonical agreement
  readonly entryHash?: string;   // Receipt from ledger
}

// =============================================================================
// ROLES - Derived relationships, not stored objects
// =============================================================================

/**
 * A Role is a relationship, not an attribute.
 * 
 * WRONG: "John is an Employee"
 * RIGHT: "John holds Employee role via Agreement #123, valid 2024-01-01 to 2025-01-01"
 */
export interface Role {
  readonly holderId: EntityId;           // Who holds this role
  readonly role: string;                  // The role name
  readonly grantedBy: AgreementId;       // Which agreement established it
  readonly scope: EntityId;               // In what context (realm/org)
  readonly validity: Validity;            // When is it valid
  readonly permissions: readonly Permission[];  // What can they do
}

/**
 * Derive roles from an agreement
 * Roles are NOT stored - they are computed from agreements
 */
export function deriveRoles(agreement: Agreement): Role[] {
  if (agreement.status !== 'Active') {
    return [];  // Only active agreements grant roles
  }
  
  return agreement.parties.map(party => ({
    holderId: party.entityId,
    role: party.role,
    grantedBy: agreement.id,
    scope: agreement.parties[0]?.entityId || agreement.id,  // First party is usually the org
    validity: agreement.validity,
    permissions: [...agreement.terms.permissions]
  }));
}

// =============================================================================
// AGREEMENT EVENTS - What gets committed to the ledger
// =============================================================================

export type AgreementEventType =
  | 'AgreementCreated'
  | 'AgreementProposed'
  | 'PartyConsented'
  | 'AgreementActivated'
  | 'AgreementSuspended'
  | 'AgreementResumed'
  | 'AgreementAmended'
  | 'AgreementTerminated'
  | 'AgreementExpired';

export interface AgreementEvent {
  readonly type: AgreementEventType;
  readonly agreementId: AgreementId;
  readonly timestamp: Timestamp;
  readonly actor: EntityId;
  readonly payload: Record<string, unknown>;
}

// =============================================================================
// AGREEMENT STORE - In-memory for now
// =============================================================================

export class AgreementStore {
  private entities: Map<EntityId, Entity> = new Map();
  private agreements: Map<AgreementId, Agreement> = new Map();
  private events: AgreementEvent[] = [];

  // --- Entities ---
  
  createEntity(entity: Omit<Entity, 'id' | 'createdAt'>): Entity {
    const id = `ent_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
    const full: Entity = {
      ...entity,
      id,
      createdAt: Date.now()
    };
    this.entities.set(id, full);
    return full;
  }

  getEntity(id: EntityId): Entity | undefined {
    return this.entities.get(id);
  }

  // --- Agreements ---

  createAgreement(
    input: Omit<Agreement, 'id' | 'status' | 'createdAt' | 'atomHash' | 'entryHash'>
  ): Agreement {
    const id = `agr_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
    const agreement: Agreement = {
      ...input,
      id,
      status: 'Draft',
      createdAt: Date.now()
    };
    
    this.agreements.set(id, agreement);
    this.recordEvent({
      type: 'AgreementCreated',
      agreementId: id,
      timestamp: Date.now(),
      actor: input.createdBy,
      payload: { type: input.type, parties: input.parties.map(p => p.entityId) }
    });
    
    return agreement;
  }

  proposeAgreement(id: AgreementId, actor: EntityId): Agreement {
    const agreement = this.agreements.get(id);
    if (!agreement) throw new Error(`Agreement ${id} not found`);
    if (agreement.status !== 'Draft') throw new Error('Can only propose draft agreements');
    
    const updated: Agreement = { ...agreement, status: 'Proposed' };
    this.agreements.set(id, updated);
    this.recordEvent({
      type: 'AgreementProposed',
      agreementId: id,
      timestamp: Date.now(),
      actor,
      payload: {}
    });
    
    return updated;
  }

  consentToAgreement(id: AgreementId, entityId: EntityId, signature?: string): Agreement {
    const agreement = this.agreements.get(id);
    if (!agreement) throw new Error(`Agreement ${id} not found`);
    if (agreement.status !== 'Proposed') throw new Error('Can only consent to proposed agreements');
    
    const partyIndex = agreement.parties.findIndex(p => p.entityId === entityId);
    if (partyIndex === -1) throw new Error(`Entity ${entityId} is not a party to this agreement`);
    
    const updatedParties = [...agreement.parties];
    updatedParties[partyIndex] = {
      ...updatedParties[partyIndex],
      consentedAt: Date.now(),
      signature
    };
    
    // Check if all parties have consented
    const allConsented = updatedParties.every(p => p.consentedAt !== undefined);
    
    const updated: Agreement = {
      ...agreement,
      parties: updatedParties,
      status: allConsented ? 'Active' : 'Proposed'
    };
    
    this.agreements.set(id, updated);
    this.recordEvent({
      type: 'PartyConsented',
      agreementId: id,
      timestamp: Date.now(),
      actor: entityId,
      payload: { allConsented }
    });
    
    if (allConsented) {
      this.recordEvent({
        type: 'AgreementActivated',
        agreementId: id,
        timestamp: Date.now(),
        actor: entityId,
        payload: {}
      });
    }
    
    return updated;
  }

  terminateAgreement(id: AgreementId, actor: EntityId, reason: string): Agreement {
    const agreement = this.agreements.get(id);
    if (!agreement) throw new Error(`Agreement ${id} not found`);
    
    const updated: Agreement = { ...agreement, status: 'Terminated' };
    this.agreements.set(id, updated);
    this.recordEvent({
      type: 'AgreementTerminated',
      agreementId: id,
      timestamp: Date.now(),
      actor,
      payload: { reason }
    });
    
    return updated;
  }

  getAgreement(id: AgreementId): Agreement | undefined {
    return this.agreements.get(id);
  }

  // --- Queries ---

  /** Get all agreements where an entity is a party */
  getAgreementsForEntity(entityId: EntityId): Agreement[] {
    return Array.from(this.agreements.values())
      .filter(a => a.parties.some(p => p.entityId === entityId));
  }

  /** Get all active agreements where an entity is a party */
  getActiveAgreementsForEntity(entityId: EntityId): Agreement[] {
    return this.getAgreementsForEntity(entityId)
      .filter(a => a.status === 'Active')
      .filter(a => {
        const now = Date.now();
        return a.validity.from <= now && (!a.validity.until || a.validity.until > now);
      });
  }

  /** Get all roles for an entity (derived from active agreements) */
  getRolesForEntity(entityId: EntityId): Role[] {
    return this.getActiveAgreementsForEntity(entityId)
      .flatMap(a => deriveRoles(a))
      .filter(r => r.holderId === entityId);
  }

  /** Get all permissions for an entity (derived from roles) */
  getPermissionsForEntity(entityId: EntityId): Permission[] {
    return this.getRolesForEntity(entityId)
      .flatMap(r => r.permissions);
  }

  // --- Events ---

  private recordEvent(event: AgreementEvent): void {
    this.events.push(event);
  }

  getEvents(agreementId?: AgreementId): AgreementEvent[] {
    if (agreementId) {
      return this.events.filter(e => e.agreementId === agreementId);
    }
    return [...this.events];
  }
}

// =============================================================================
// CONVENIENCE: Create standard agreement types
// =============================================================================

export const AgreementTemplates = {
  employment(
    employer: EntityId,
    employee: EntityId,
    terms: { position: string; permissions: Permission[] },
    validity: Validity,
    createdBy: EntityId
  ): Omit<Agreement, 'id' | 'status' | 'createdAt' | 'atomHash' | 'entryHash'> {
    return {
      type: 'Employment',
      name: `Employment: ${terms.position}`,
      parties: [
        { entityId: employer, role: 'Employer' },
        { entityId: employee, role: 'Employee' }
      ],
      terms: {
        permissions: terms.permissions,
        custom: { position: terms.position }
      },
      validity,
      createdBy
    };
  },

  authorization(
    grantor: EntityId,
    grantee: EntityId,
    permissions: Permission[],
    validity: Validity,
    createdBy: EntityId
  ): Omit<Agreement, 'id' | 'status' | 'createdAt' | 'atomHash' | 'entryHash'> {
    return {
      type: 'Authorization',
      name: `Authorization Grant`,
      parties: [
        { entityId: grantor, role: 'Grantor' },
        { entityId: grantee, role: 'Grantee' }
      ],
      terms: { permissions },
      validity,
      createdBy
    };
  },

  license(
    system: EntityId,
    tenant: EntityId,
    validity: Validity,
    createdBy: EntityId
  ): Omit<Agreement, 'id' | 'status' | 'createdAt' | 'atomHash' | 'entryHash'> {
    return {
      type: 'License',
      name: `Tenant License`,
      parties: [
        { entityId: system, role: 'Licensor' },
        { entityId: tenant, role: 'Licensee' }
      ],
      terms: {
        permissions: [
          { action: '*', resource: `realm:${tenant}` }  // Full access to their realm
        ]
      },
      validity,
      createdBy
    };
  }
};
