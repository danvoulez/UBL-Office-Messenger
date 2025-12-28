/**
 * Auth Context Provider
 * Global authentication state management
 */

import React, { createContext, useContext, useState, useEffect, useCallback } from 'react';
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
  avatar?: string;
}

interface AuthContextType {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  isDemoMode: boolean;
  supportsWebAuthn: boolean;
  error: string | null;
  registerPasskey: (username: string, displayName: string) => Promise<{ sid: string }>;
  loginWithPasskey: (username: string) => Promise<{ sid: string }>;
  loginDemo: () => void;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | null>(null);

export const useAuthContext = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuthContext must be used within AuthProvider');
  }
  return context;
};

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  const isDemoMode = localStorage.getItem('ubl_demo_mode') === 'true';
  const supportsWebAuthn = browserSupportsWebAuthn();

  // Check session on mount
  useEffect(() => {
    const checkSession = async () => {
      try {
        // Check demo mode
        if (isDemoMode) {
          setUser({
            sid: 'demo:user',
            username: 'Demo User',
            displayName: 'Demo User',
            kind: 'demo',
          });
          setIsLoading(false);
          return;
        }

        // Check session token
        const sessionToken = localStorage.getItem('ubl_session_token');
        if (!sessionToken) {
          setIsLoading(false);
          return;
        }

        // Validate with server
        const res = await fetch(`${API_BASE}/id/whoami`, {
          headers: { Authorization: `Bearer ${sessionToken}` },
        });

        if (res.ok) {
          const data = await res.json();
          if (data.authenticated) {
            setUser({
              sid: data.sid,
              username: data.display_name,
              displayName: data.display_name,
              kind: data.kind,
            });
          } else {
            localStorage.removeItem('ubl_session_token');
          }
        } else {
          localStorage.removeItem('ubl_session_token');
        }
      } catch (err) {
        console.error('Session check failed:', err);
      } finally {
        setIsLoading(false);
      }
    };

    checkSession();
  }, [isDemoMode]);

  // Register passkey
  const registerPasskey = useCallback(async (username: string, displayName: string) => {
    setIsLoading(true);
    setError(null);

    try {
      // Begin registration
      const beginRes = await fetch(`${API_BASE}/id/register/begin`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, display_name: displayName }),
      });

      if (!beginRes.ok) {
        throw new Error(await beginRes.text() || 'Registration failed');
      }

      const { challenge_id, options } = await beginRes.json();

      // Create credential
      const credential = await startRegistration(options);

      // Finish registration
      const finishRes = await fetch(`${API_BASE}/id/register/finish`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ challenge_id, attestation: credential }),
      });

      if (!finishRes.ok) {
        throw new Error(await finishRes.text() || 'Registration failed');
      }

      const { sid } = await finishRes.json();
      setIsLoading(false);
      return { sid };
    } catch (err: any) {
      setError(err.message);
      setIsLoading(false);
      throw err;
    }
  }, []);

  // Login with passkey
  const loginWithPasskey = useCallback(async (username: string) => {
    setIsLoading(true);
    setError(null);

    try {
      // Begin authentication
      const beginRes = await fetch(`${API_BASE}/id/login/begin`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username }),
      });

      if (!beginRes.ok) {
        throw new Error(await beginRes.text() || 'User not found');
      }

      const { challenge_id, public_key } = await beginRes.json();

      // Authenticate
      const credential = await startAuthentication(public_key);

      // Finish authentication
      const finishRes = await fetch(`${API_BASE}/id/login/finish`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ challenge_id, credential }),
      });

      if (!finishRes.ok) {
        throw new Error(await finishRes.text() || 'Authentication failed');
      }

      const { sid, session_token } = await finishRes.json();

      localStorage.setItem('ubl_session_token', session_token);
      setUser({
        sid,
        username,
        displayName: username,
        kind: 'person',
      });
      setIsLoading(false);
      return { sid };
    } catch (err: any) {
      setError(err.message);
      setIsLoading(false);
      throw err;
    }
  }, []);

  // Demo mode login
  const loginDemo = useCallback(() => {
    localStorage.setItem('ubl_demo_mode', 'true');
    setUser({
      sid: 'demo:user',
      username: 'Demo User',
      displayName: 'Demo User',
      kind: 'demo',
    });
  }, []);

  // Logout
  const logout = useCallback(() => {
    localStorage.removeItem('ubl_session_token');
    localStorage.removeItem('ubl_demo_mode');
    setUser(null);
  }, []);

  return (
    <AuthContext.Provider
      value={{
        user,
        isAuthenticated: !!user,
        isLoading,
        isDemoMode,
        supportsWebAuthn,
        error,
        registerPasskey,
        loginWithPasskey,
        loginDemo,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
};

