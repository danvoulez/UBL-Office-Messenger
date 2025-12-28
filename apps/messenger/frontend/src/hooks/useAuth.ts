/**
 * WebAuthn Authentication Hook
 * Handles passkey registration and authentication with UBL Kernel
 */

import { useState, useCallback, useEffect } from 'react';
import {
  startRegistration,
  startAuthentication,
  browserSupportsWebAuthn,
} from '@simplewebauthn/browser';

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
}

export function useAuth() {
  const [state, setState] = useState<AuthState>({
    user: null,
    isAuthenticated: false,
    isLoading: true,
    error: null,
  });

  // Check if WebAuthn is supported
  const supportsWebAuthn = browserSupportsWebAuthn();

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

      // 2. Authenticate with browser
      const credential = await startAuthentication(public_key);

      // 3. Finish authentication - verify with server
      const finishRes = await fetch(`${API_BASE}/id/login/finish`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          challenge_id,
          credential,
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
      });

      return { sid, username };
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
    setState({
      user: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,
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
    });
  }, []);

  return {
    ...state,
    supportsWebAuthn,
    registerPasskey,
    loginWithPasskey,
    loginDemo,
    logout,
  };
}

