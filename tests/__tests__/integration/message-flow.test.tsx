import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import ChatPage from '../../pages/ChatPage';
import * as ublApi from '../../__mocks__/ublApi';
import { mockSSEClient } from '../../__mocks__/sse';

vi.mock('../../services/ublApi', () => ublApi);
vi.mock('../../context/AuthContext', () => require('../../__mocks__/authContext'));

describe('Message Flow Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('completes full message send flow', async () => {
    render(
      <BrowserRouter>
        <ChatPage />
      </BrowserRouter>
    );

    // Wait for data to load
    await waitFor(() => {
      expect(screen.getByText('Test Agent')).toBeInTheDocument();
    });

    // Click on conversation
    const conversation = screen.getByText('Test Agent');
    fireEvent.click(conversation);

    // Type and send message
    const input = screen.getByPlaceholderText(/type a message/i);
    fireEvent.change(input, { target: { value: 'Hello from test!' } });

    const sendButton = screen.getByRole('button', { name: /send/i });
    fireEvent.click(sendButton);

    // Verify API call
    await waitFor(() => {
      expect(ublApi. ublApi.sendMessage).toHaveBeenCalledWith({
        conversationId: 'conv-test',
        content:  'Hello from test!',
        role: 'user',
      });
    });

    // Simulate SSE update
    mockSSEClient.simulateEvent('timeline.append', {
      conversation_id: 'conv-test',
      item: {
        item_type: 'message',
        item_data: {
          id: 'msg-new',
          conversationId: 'conv-test',
          senderId: 'user-test',
          content: 'Hello from test!',
          timestamp: new Date().toISOString(),
          role: 'user',
          status: 'sent',
        },
      },
    });

    // Verify message appears
    await waitFor(() => {
      expect(screen.getByText('Hello from test!')).toBeInTheDocument();
    });
  });
});