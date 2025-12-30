import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import Sidebar from '../../components/Sidebar';
import { mockEntities, mockConversations } from '../../__mocks__/ublApi';

const mockNavigate = vi.fn();

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => mockNavigate,
    useParams: () => ({ conversationId: 'conv-test' }),
  };
});

describe('Sidebar', () => {
  const defaultProps = {
    currentUser: mockEntities[0],
    entities: mockEntities,
    conversations: mockConversations,
    onNewWorkstream: vi.fn(),
    onInspectEntity: vi.fn(),
    onLogout: vi.fn(),
  };

  const renderSidebar = (props = {}) => {
    return render(
      <BrowserRouter>
        <Sidebar {...defaultProps} {... props} />
      </BrowserRouter>
    );
  };

  it('renders current user info', () => {
    renderSidebar();
    expect(screen.getByText('Test User')).toBeInTheDocument();
  });

  it('renders conversation list', () => {
    renderSidebar();
    expect(screen.getByText('Test Agent')).toBeInTheDocument();
  });

  it('calls onNewWorkstream when button clicked', () => {
    renderSidebar();
    const newButton = screen.getByRole('button', { name: /new/i });
    fireEvent.click(newButton);
    expect(defaultProps.onNewWorkstream).toHaveBeenCalled();
  });

  it('navigates to conversation on click', () => {
    renderSidebar();
    const conversation = screen.getByText('Test Agent').closest('button');
    fireEvent.click(conversation! );
    expect(mockNavigate).toHaveBeenCalledWith('/chat/conv-test');
  });

  it('highlights active conversation', () => {
    renderSidebar();
    const activeConv = screen.getByText('Test Agent').closest('button');
    expect(activeConv).toHaveClass('bg-bg-active');
  });

  it('shows presence indicators', () => {
    renderSidebar();
    const onlineIndicator = screen.getByTitle(/online/i);
    expect(onlineIndicator).toBeInTheDocument();
  });

  it('calls onLogout when logout clicked', () => {
    renderSidebar();
    const logoutButton = screen.getByRole('button', { name: /logout/i });
    fireEvent.click(logoutButton);
    expect(defaultProps.onLogout).toHaveBeenCalled();
  });
});