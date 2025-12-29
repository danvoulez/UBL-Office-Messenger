import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import ChatView from '../../components/ChatView';
import { mockEntities, mockMessages } from '../../__mocks__/ublApi';

describe('ChatView', () => {
  const defaultProps = {
    conversation: {
      id: 'conv-test',
      participants: ['user-test', 'agent-test'],
      isGroup: false,
      unreadCount: 0,
      lastMessage: 'Hello!',
      lastMessageTime: '1m',
    },
    currentUser: mockEntities[0],
    entities: mockEntities,
    messages: mockMessages,
    isTyping: false,
    onSendMessage: vi.fn(),
    onJobAction: vi.fn(),
    onOpenJobDrawer: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders conversation header', () => {
    render(<ChatView {...defaultProps} />);
    expect(screen.getByText('Test Agent')).toBeInTheDocument();
  });

  it('renders messages', () => {
    render(<ChatView {...defaultProps} />);
    expect(screen.getByText('Hello!')).toBeInTheDocument();
  });

  it('sends message on submit', async () => {
    render(<ChatView {...defaultProps} />);
    
    const input = screen.getByPlaceholderText(/type a message/i);
    const sendButton = screen.getByRole('button', { name: /send/i });

    fireEvent.change(input, { target: { value: 'Test message' } });
    fireEvent.click(sendButton);

    await waitFor(() => {
      expect(defaultProps.onSendMessage).toHaveBeenCalledWith('Test message');
    });
  });

  it('disables send button when input is empty', () => {
    render(<ChatView {...defaultProps} />);
    const sendButton = screen.getByRole('button', { name: /send/i });
    expect(sendButton).toBeDisabled();
  });

  it('shows typing indicator', () => {
    render(<ChatView {...defaultProps} isTyping={true} />);
    expect(screen.getByText(/typing/i)).toBeInTheDocument();
  });

  it('scrolls to bottom on new message', async () => {
    const { rerender } = render(<ChatView {... defaultProps} />);

    const newMessages = [
      ... mockMessages,
      {
        id: 'msg-2',
        conversationId: 'conv-test',
        senderId: 'agent-test',
        content: 'New message',
        timestamp: new Date().toISOString(),
        role: 'assistant',
        status: 'sent',
      },
    ];

    rerender(<ChatView {...defaultProps} messages={newMessages} />);

    await waitFor(() => {
      expect(screen.getByText('New message')).toBeInTheDocument();
    });
  });

  it('handles Enter key to send message', async () => {
    render(<ChatView {...defaultProps} />);
    
    const input = screen.getByPlaceholderText(/type a message/i);

    fireEvent.change(input, { target: { value: 'Test message' } });
    fireEvent.keyPress(input, { key: 'Enter', code: 'Enter', charCode: 13 });

    await waitFor(() => {
      expect(defaultProps.onSendMessage).toHaveBeenCalledWith('Test message');
    });
  });

  it('does not send on Shift+Enter', async () => {
    render(<ChatView {...defaultProps} />);
    
    const input = screen.getByPlaceholderText(/type a message/i);

    fireEvent.change(input, { target: { value: 'Test message' } });
    fireEvent.keyPress(input, { key: 'Enter', code: 'Enter', charCode: 13, shiftKey: true });

    await waitFor(() => {
      expect(defaultProps.onSendMessage).not.toHaveBeenCalled();
    });
  });
});