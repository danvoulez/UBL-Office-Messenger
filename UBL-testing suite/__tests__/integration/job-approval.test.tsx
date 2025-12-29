import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import ChatPage from '../../pages/ChatPage';
import * as ublApi from '../../__mocks__/ublApi';
import { mockSSEClient } from '../../__mocks__/sse';

vi.mock('../../services/ublApi', () => ublApi);
vi.mock('../../context/AuthContext', () => require('../../__mocks__/authContext'));

describe('Job Approval Flow Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('approves a job successfully', async () => {
    render(
      <BrowserRouter>
        <ChatPage />
      </BrowserRouter>
    );

    // Wait for conversation to load
    await waitFor(() => {
      expect(screen. getByText('Test Agent')).toBeInTheDocument();
    });

    // Simulate job card appearing via SSE
    mockSSEClient.simulateEvent('timeline.append', {
      conversation_id:  'conv-test',
      item: {
        item_type: 'job_card',
        item_data:  {
          card_type: 'job. formalize',
          card_id: 'card-test',
          job_id: 'job-test',
          title: 'Test Job',
          summary: 'This is a test job',
          state: 'proposed',
          buttons: [
            {
              button_id: 'approve_btn',
              label: 'Approve',
              action: { type: 'job.approve', job_id: 'job-test' },
              style: 'primary',
              requires_input: false,
            },
          ],
        },
      },
    });

    // Wait for job card to appear
    await waitFor(() => {
      expect(screen.getByText('Test Job')).toBeInTheDocument();
    });

    // Click approve button
    const approveButton = screen.getByRole('button', { name: /approve/i });
    fireEvent.click(approveButton);

    // Verify API call
    await waitFor(() => {
      expect(ublApi.ublApi.jobActionViaGateway).toHaveBeenCalledWith('job-test', {
        action: 'job.approve',
        card_id: 'card-test',
        button_id: 'approve_btn',
      });
    });

    // Simulate job update via SSE
    mockSSEClient.simulateEvent('job.update', {
      job_id: 'job-test',
      update: {
        state: 'approved',
        updated_at: new Date().toISOString(),
      },
    });

    // Verify UI updates
    await waitFor(() => {
      expect(screen.queryByRole('button', { name: /approve/i })).not.toBeInTheDocument();
    });
  });

  it('rejects a job successfully', async () => {
    render(
      <BrowserRouter>
        <ChatPage />
      </BrowserRouter>
    );

    // Simulate job card
    mockSSEClient.simulateEvent('timeline.append', {
      conversation_id: 'conv-test',
      item: {
        item_type: 'job_card',
        item_data: {
          card_type: 'job.formalize',
          card_id: 'card-test',
          job_id: 'job-test',
          title: 'Test Job',
          buttons: [
            {
              button_id: 'reject_btn',
              label: 'Reject',
              action:  { type: 'job.reject', job_id: 'job-test' },
              style:  'danger',
              requires_input: false,
            },
          ],
        },
      },
    });

    await waitFor(() => {
      expect(screen.getByText('Test Job')).toBeInTheDocument();
    });

    // Click reject button
    const rejectButton = screen.getByRole('button', { name: /reject/i });
    fireEvent.click(rejectButton);

    // Verify API call
    await waitFor(() => {
      expect(ublApi.ublApi.jobActionViaGateway).toHaveBeenCalledWith('job-test', {
        action: 'job.reject',
        card_id: 'card-test',
        button_id: 'reject_btn',
      });
    });
  });
});