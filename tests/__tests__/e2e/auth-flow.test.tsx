import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { App } from '../../App';

// Mock navigator.credentials for WebAuthn
const mockCredentials = {
  create: vi.fn(),
  get: vi.fn(),
};

Object.defineProperty(navigator, 'credentials', {
  value: mockCredentials,
  writable: true,
});

describe('Authentication Flow E2E', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
  });

  it('shows login page when not authenticated', () => {
    render(<App />);
    
    expect(screen.getByText(/welcome to ubl messenger/i)).toBeInTheDocument();
  });

  it('registers with passkey', async () => {
    mockCredentials.create.mockResolvedValue({
      id: 'credential-id',
      rawId: new ArrayBuffer(32),
      response: {
        attestationObject: new ArrayBuffer(64),
        clientDataJSON: new ArrayBuffer(64),
      },
      type: 'public-key',
    });

    render(<App />);

    const usernameInput = screen.getByPlaceholderText(/username/i);
    const registerButton = screen.getByRole('button', { name: /register/i });

    fireEvent.change(usernameInput, { target: { value: 'testuser' } });
    fireEvent.click(registerButton);

    await waitFor(() => {
      expect(mockCredentials.create).toHaveBeenCalled();
    });
  });

  it('logs in with passkey', async () => {
    mockCredentials.get.mockResolvedValue({
      id: 'credential-id',
      rawId: new ArrayBuffer(32),
      response: {
        authenticatorData: new ArrayBuffer(64),
        clientDataJSON: new ArrayBuffer(64),
        signature: new ArrayBuffer(64),
        userHandle: new ArrayBuffer(16),
      },
      type:  'public-key',
    });

    render(<App />);

    const usernameInput = screen.getByPlaceholderText(/username/i);
    const loginButton = screen.getByRole('button', { name: /login with passkey/i });

    fireEvent.change(usernameInput, { target: { value: 'testuser' } });
    fireEvent.click(loginButton);

    await waitFor(() => {
      expect(mockCredentials. get).toHaveBeenCalled();
    });
  });

  it('enters demo mode', async () => {
    render(<App />);

    const demoButton = screen.getByRole('button', { name: /demo mode/i });
    fireEvent.click(demoButton);

    await waitFor(() => {
      expect(screen.getByText(/chat/i)).toBeInTheDocument();
    });
  });

  it('redirects to chat after login', async () => {
    // Pre-authenticate
    localStorage.setItem('ubl_session_token', 'mock-token');

    render(<App />);

    await waitFor(() => {
      expect(screen.queryByText(/welcome to ubl messenger/i)).not.toBeInTheDocument();
    });
  });
});