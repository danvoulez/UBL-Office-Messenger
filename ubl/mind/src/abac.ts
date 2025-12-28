/**
 * UBL ABAC - Agreement-Based Access Control
 * 
 * Authorization in UBL is fundamentally different from traditional RBAC:
 * 
 * Traditional RBAC:
 *   "John is Admin" → opaque, no provenance
 * 
 * UBL ABAC:
 *   "John can do X because Agreement #123 grants Employee role,
 *    which includes permission X, valid from 2024-01-01"
 *    → traceable, auditable, temporal
 * 
 * Every authorization decision answers:
 *   - WHO gave this permission? (Agreement)
 *   - WHEN did it start? (Validity)
 *   - WHEN does it end? (Validity)
 *   - IN WHAT CONTEXT? (Scope)
 */

import { 
  EntityId, 
  AgreementId, 
  Permission, 
  Role,
  AgreementStore 
} from './agreements.js';

// =============================================================================
// AUTHORIZATION TYPES
// =============================================================================

/** An authorization request */
export interface AuthorizationRequest {
  readonly actor: EntityId;
  readonly action: string;
  readonly resource: string;
  readonly context?: Record<string, unknown>;
}

/** The result of an authorization check */
export type AuthorizationDecision = 'ALLOW' | 'DENY';

/** Detailed authorization response with provenance */
export interface AuthorizationResponse {
  readonly decision: AuthorizationDecision;
  readonly request: AuthorizationRequest;
  readonly timestamp: number;
  
  // Provenance (only present if ALLOW)
  readonly grantedBy?: AgreementId;
  readonly role?: string;
  readonly permission?: Permission;
  
  // Reason (always present)
  readonly reason: string;
}

// =============================================================================
// PERMISSION MATCHING
// =============================================================================

/**
 * Check if a permission matches a request
 * Supports wildcards: '*' matches anything
 */
function permissionMatches(
  permission: Permission,
  action: string,
  resource: string
): boolean {
  const actionMatches = 
    permission.action === '*' || 
    permission.action === action ||
    (permission.action.endsWith(':*') && action.startsWith(permission.action.slice(0, -1)));
  
  const resourceMatches = 
    permission.resource === '*' ||
    permission.resource === resource ||
    (permission.resource.endsWith(':*') && resource.startsWith(permission.resource.slice(0, -1)));
  
  return actionMatches && resourceMatches;
}

// =============================================================================
// ABAC ENGINE
// =============================================================================

export class ABAC {
  private readonly store: AgreementStore;
  private readonly decisions: AuthorizationResponse[] = [];

  constructor(store: AgreementStore) {
    this.store = store;
  }

  /**
   * Authorize a request
   * 
   * The algorithm:
   * 1. Get all roles for the actor (derived from active agreements)
   * 2. For each role, check if any permission matches
   * 3. If match found → ALLOW with provenance
   * 4. If no match → DENY
   */
  authorize(request: AuthorizationRequest): AuthorizationResponse {
    const timestamp = Date.now();
    
    // Get all roles for the actor
    const roles = this.store.getRolesForEntity(request.actor);
    
    if (roles.length === 0) {
      return this.recordDecision({
        decision: 'DENY',
        request,
        timestamp,
        reason: `No active roles found for entity ${request.actor}`
      });
    }
    
    // Check each role for matching permissions
    for (const role of roles) {
      for (const permission of role.permissions) {
        if (permissionMatches(permission, request.action, request.resource)) {
          return this.recordDecision({
            decision: 'ALLOW',
            request,
            timestamp,
            grantedBy: role.grantedBy,
            role: role.role,
            permission,
            reason: `Allowed by ${role.role} role via agreement ${role.grantedBy}`
          });
        }
      }
    }
    
    // No matching permission found
    return this.recordDecision({
      decision: 'DENY',
      request,
      timestamp,
      reason: `No matching permission found. Actor has ${roles.length} role(s) but none grant ${request.action}:${request.resource}`
    });
  }

  /**
   * Check if actor can perform action (simple boolean version)
   */
  can(actor: EntityId, action: string, resource: string): boolean {
    return this.authorize({ actor, action, resource }).decision === 'ALLOW';
  }

  /**
   * Get all permissions for an actor (for UI/debugging)
   */
  getEffectivePermissions(actor: EntityId): Permission[] {
    return this.store.getPermissionsForEntity(actor);
  }

  /**
   * Get all roles for an actor with provenance
   */
  getEffectiveRoles(actor: EntityId): Role[] {
    return this.store.getRolesForEntity(actor);
  }

  /**
   * Get decision audit trail
   */
  getDecisions(actor?: EntityId): AuthorizationResponse[] {
    if (actor) {
      return this.decisions.filter(d => d.request.actor === actor);
    }
    return [...this.decisions];
  }

  private recordDecision(response: AuthorizationResponse): AuthorizationResponse {
    this.decisions.push(response);
    return response;
  }
}

// =============================================================================
// POLICY ENGINE - Higher-level policies
// =============================================================================

export type PolicyEffect = 'ALLOW' | 'DENY';

export interface Policy {
  readonly id: string;
  readonly name: string;
  readonly effect: PolicyEffect;
  readonly condition: (request: AuthorizationRequest) => boolean;
  readonly priority: number;  // Higher = evaluated first
}

/**
 * Policy-based authorization layer
 * Sits on top of ABAC for additional rules
 */
export class PolicyEngine {
  private readonly abac: ABAC;
  private readonly policies: Policy[] = [];

  constructor(abac: ABAC) {
    this.abac = abac;
  }

  /**
   * Add a policy
   */
  addPolicy(policy: Policy): void {
    this.policies.push(policy);
    this.policies.sort((a, b) => b.priority - a.priority);
  }

  /**
   * Evaluate request against policies, then ABAC
   * 
   * Order:
   * 1. Check explicit DENY policies (fail-closed)
   * 2. Check explicit ALLOW policies
   * 3. Fall back to ABAC
   */
  evaluate(request: AuthorizationRequest): AuthorizationResponse {
    // Check DENY policies first (fail-closed)
    for (const policy of this.policies.filter(p => p.effect === 'DENY')) {
      if (policy.condition(request)) {
        return {
          decision: 'DENY',
          request,
          timestamp: Date.now(),
          reason: `Denied by policy: ${policy.name}`
        };
      }
    }

    // Check ALLOW policies
    for (const policy of this.policies.filter(p => p.effect === 'ALLOW')) {
      if (policy.condition(request)) {
        return {
          decision: 'ALLOW',
          request,
          timestamp: Date.now(),
          reason: `Allowed by policy: ${policy.name}`
        };
      }
    }

    // Fall back to ABAC
    return this.abac.authorize(request);
  }
}

// =============================================================================
// COMMON POLICIES
// =============================================================================

export const CommonPolicies = {
  /** Deny all requests outside business hours */
  businessHoursOnly(startHour = 9, endHour = 18): Policy {
    return {
      id: 'business-hours',
      name: 'Business Hours Only',
      effect: 'DENY',
      priority: 100,
      condition: () => {
        const hour = new Date().getHours();
        return hour < startHour || hour >= endHour;
      }
    };
  },

  /** Deny requests for specific dangerous actions */
  denyDangerousActions(actions: string[]): Policy {
    return {
      id: 'deny-dangerous',
      name: 'Deny Dangerous Actions',
      effect: 'DENY',
      priority: 200,
      condition: (req) => actions.includes(req.action)
    };
  },

  /** Allow system to do anything */
  systemBypass(systemEntityId: EntityId): Policy {
    return {
      id: 'system-bypass',
      name: 'System Bypass',
      effect: 'ALLOW',
      priority: 1000,
      condition: (req) => req.actor === systemEntityId
    };
  },

  /** Rate limit (placeholder - would need external state) */
  rateLimit(maxRequests: number, windowMs: number): Policy {
    const windows = new Map<EntityId, number[]>();
    
    return {
      id: 'rate-limit',
      name: `Rate Limit: ${maxRequests}/${windowMs}ms`,
      effect: 'DENY',
      priority: 50,
      condition: (req) => {
        const now = Date.now();
        const actorWindow = windows.get(req.actor) || [];
        
        // Clean old entries
        const validWindow = actorWindow.filter(t => t > now - windowMs);
        validWindow.push(now);
        windows.set(req.actor, validWindow);
        
        return validWindow.length > maxRequests;
      }
    };
  }
};
