import React from 'react';
import { vi } from 'vitest';

export const mockUser = {
  sid: 'test-user-sid',
  displayName: 'Test User',
  avatar: 'https://api.dicebear.com/7.x/notionists/svg?seed=Test',
};

export const mockAuthContext = {
  user: mockUser,
  isAuthenticated: true,
  isLoading: false,
  isDemoMode: false,
  supportsWebAuthn: true,
  registerPasskey: vi.fn().mockResolvedValue(undefined),
  loginWithPasskey: vi.fn().mockResolvedValue(undefined),
  loginDemo: vi.fn().mockResolvedValue(undefined),
  logout: vi.fn(),
};

export const useAuthContext = vi.fn(() => mockAuthContext);

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return <>{children}</>;
};