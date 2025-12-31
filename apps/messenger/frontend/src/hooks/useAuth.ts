/**
 * WebAuthn Authentication Hook
 * Handles passkey registration and authentication with UBL Kernel
 * 
 * Fix #2: Now includes WebAuthn PRF extension for client-side Ed25519 signing
 */

import { useState, useCallback, useEffect } from 'react';
import {
  startRegistration,
  startAuthentication,
  browserSupportsWebAuthn,
} from '@simplewebauthn/browser';
import { 
  deriveSigningKey, 
  clearSigningKey, 
  isClientSideSigningAvailable,
  getPublicKeyForRegistration,
  isPRFLikelySupported,
} from '../services/crypto';

const API_BASE = import.meta.env.VITE_API_BASE_URL || '';

interface User {
  sid: string;
  username: string;
  displayName: string;
  kind: string;
}

interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
  /** Fix #2: Whether client-side Ed25519 signing is available (PRF derived) */
  canSignClientSide: boolean;
}

// ============================================================================
// Base64url utilities for WebAuthn
// ============================================================================

function base64urlToArrayBuffer(base64url: string): ArrayBuffer {
  const padding = '='.repeat((4 - (base64url.length % 4)) % 4);
  const base64 = base64url.replace(/-/g, '+').replace(/_/g, '/') + padding;
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes.buffer;
}

function arrayBufferToBase64url(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
}

// ============================================================================
// Hook
// ============================================================================

export function useAuth() {
  const [state, setState] = useState<AuthState>({
    user: null,
    isAuthenticated: false,
    isLoading: true,
    error: null,
    canSignClientSide: false,
  });

  // Check if WebAuthn is supported
  const supportsWebAuthn = browserSupportsWebAuthn();
  const supportsPRF = isPRFLikelySupported();

  // Load session on mount
  useEffect(() => {
    const checkSession = async () => {
      try {
        const sessionToken = localStorage.getItem('ubl_session_token');
        if (!sessionToken) {
          setState(s => ({ ...s, isLoading: false }));
          return;
        }

        const res = await fetch(`${API_BASE}/id/whoami`, {
          headers: {
            Authorization: `Bearer ${sessionToken}`,
          },
        });

        if (res.ok) {
          const data = await res.json();
          if (data.authenticated) {
            // Note: On session restore, we don't have the signing key
            // User needs to re-authenticate with PRF to enable client-side signing
            setState({
              user: {
                sid: data.sid,
                username: data.display_name,
                displayName: data.display_name,
                kind: data.kind,
              },
              isAuthenticated: true,
              isLoading: false,
              error: null,
              canSignClientSide: isClientSideSigningAvailable(),
            });
            return;
          }
        }

        // Invalid session
        localStorage.removeItem('ubl_session_token');
        setState(s => ({ ...s, isLoading: false }));
      } catch (err) {
        setState(s => ({ ...s, isLoading: false }));
      }
    };

    checkSession();
  }, []);

  // Register new passkey
  const registerPasskey = useCallback(async (username: string, displayName: string) => {
    setState(s => ({ ...s, isLoading: true, error: null }));

    try {
      // 1. Begin registration - get challenge from server
      const beginRes = await fetch(`${API_BASE}/id/register/begin`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, display_name: displayName }),
      });

      if (!beginRes.ok) {
        const err = await beginRes.text();
        throw new Error(err || 'Failed to start registration');
      }

      const { challenge_id, options } = await beginRes.json();

      // 2. Create credential with browser
      const credential = await startRegistration(options);

      // 3. Finish registration - send credential to server
      const finishRes = await fetch(`${API_BASE}/id/register/finish`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          challenge_id,
          attestation: credential,
        }),
      });

      if (!finishRes.ok) {
        const err = await finishRes.text();
        throw new Error(err || 'Failed to complete registration');
      }

      const { sid } = await finishRes.json();
      
      setState(s => ({ ...s, isLoading: false }));
      return { sid, username };
    } catch (err: any) {
      const message = err.message || 'Registration failed';
      setState(s => ({ ...s, isLoading: false, error: message }));
      throw new Error(message);
    }
  }, []);

  // Login with passkey
  const loginWithPasskey = useCallback(async (username: string) => {
    setState(s => ({ ...s, isLoading: true, error: null }));

    try {
      // 1. Begin authentication - get challenge from server
      const beginRes = await fetch(`${API_BASE}/id/login/begin`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username }),
      });

      if (!beginRes.ok) {
        const err = await beginRes.text();
        throw new Error(err || 'User not found');
      }

      const { challenge_id, public_key } = await beginRes.json();

      // 2. Authenticate with browser + PRF extension (Fix #2)
      // We use raw WebAuthn API to include PRF extension
      const prfSalt = new TextEncoder().encode('ubl-ed25519-signing-v1');
      
      let credential: PublicKeyCredential | null = null;
      let prfResult: ArrayBuffer | null = null;
      
      try {
        // Try with PRF extension first
        const publicKeyOptions: PublicKeyCredentialRequestOptions = {
          ...public_key,
          challenge: base64urlToArrayBuffer(public_key.challenge),
          allowCredentials: public_key.allowCredentials?.map((c: any) => ({
            ...c,
            id: base64urlToArrayBuffer(c.id),
          })),
          extensions: {
            // @ts-ignore - PRF extension
            prf: {
              eval: {
                first: prfSalt,
              },
            },
          },
        };
        
        credential = await navigator.credentials.get({
          publicKey: publicKeyOptions,
        }) as PublicKeyCredential;
        
        // Check for PRF results
        const extensions = credential.getClientExtensionResults() as any;
        if (extensions.prf?.results?.first) {
          prfResult = extensions.prf.results.first;
          console.log('PRF extension successful - client-side signing enabled');
        } else {
          console.log('PRF not supported by authenticator - using server-side signing');
        }
      } catch (prfError) {
        // Fallback to standard authentication without PRF
        console.warn('PRF extension failed, using standard auth:', prfError);
        credential = await startAuthentication(public_key) as unknown as PublicKeyCredential;
      }

      if (!credential) {
        throw new Error('Authentication cancelled');
      }

      // 3. If we got PRF material, derive signing key
      let canSign = false;
      if (prfResult) {
        try {
          await deriveSigningKey(new Uint8Array(prfResult));
          canSign = true;
          console.log('Ed25519 signing key derived from PRF');
        } catch (deriveError) {
          console.error('Failed to derive signing key:', deriveError);
        }
      }

      // 4. Convert credential to format expected by server
      const response = credential.response as AuthenticatorAssertionResponse;
      const credentialForServer = {
        id: credential.id,
        rawId: arrayBufferToBase64url(credential.rawId),
        type: credential.type,
        response: {
          authenticatorData: arrayBufferToBase64url(response.authenticatorData),
          clientDataJSON: arrayBufferToBase64url(response.clientDataJSON),
          signature: arrayBufferToBase64url(response.signature),
          userHandle: response.userHandle ? arrayBufferToBase64url(response.userHandle) : null,
        },
        clientExtensionResults: credential.getClientExtensionResults(),
      };

      // 5. Finish authentication - verify with server
      const finishRes = await fetch(`${API_BASE}/id/login/finish`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          challenge_id,
          credential: credentialForServer,
        }),
      });

      if (!finishRes.ok) {
        const err = await finishRes.text();
        throw new Error(err || 'Authentication failed');
      }

      const { sid, session_token } = await finishRes.json();

      // Save session
      localStorage.setItem('ubl_session_token', session_token);

      setState({
        user: {
          sid,
          username,
          displayName: username,
          kind: 'person',
        },
        isAuthenticated: true,
        isLoading: false,
        error: null,
        canSignClientSide: canSign,
      });

      return { sid, username, canSignClientSide: canSign };
    } catch (err: any) {
      const message = err.message || 'Login failed';
      setState(s => ({ ...s, isLoading: false, error: message }));
      throw new Error(message);
    }
  }, []);

  // Logout
  const logout = useCallback(() => {
    localStorage.removeItem('ubl_session_token');
    localStorage.removeItem('ubl_demo_mode');
    // Fix #2: Clear signing key on logout
    clearSigningKey();
    setState({
      user: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,
      canSignClientSide: false,
    });
  }, []);

  // Demo mode login
  const loginDemo = useCallback(() => {
    localStorage.setItem('ubl_demo_mode', 'true');
    setState({
      user: {
        sid: 'demo:user',
        username: 'Demo User',
        displayName: 'Demo User',
        kind: 'demo',
      },
      isAuthenticated: true,
      isLoading: false,
      error: null,
      canSignClientSide: false, // Demo mode doesn't have real signing
    });
  }, []);

  return {
    ...state,
    supportsWebAuthn,
    supportsPRF,
    registerPasskey,
    loginWithPasskey,
    loginDemo,
    logout,
    // Fix #2: Expose signing utilities
    getPublicKeyForRegistration,
  };
}
