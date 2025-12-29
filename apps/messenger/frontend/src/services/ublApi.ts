/**
 * UBL API Service
 * Communicates with UBL Kernel's Messenger Gateway (v1) and Messenger Boundary (messenger_v1)
 */

import { api } from './apiClient';
import type { Conversation, Entity, Message, MessageType, UserSettings } from '../types';

// ============================================================================
// Types for API responses
// ============================================================================

type JsonMessage = Omit<Message, 'timestamp'> & { timestamp: string };

interface BootstrapApiResponse {
  user: {
    sid: string;
    display_name: string;
    kind: string;
  } | null;
  entities: Array<{
    id: string;
    display_name: string;
    kind: string;
    avatar_url: string | null;
    status: string | null;
  }>;
  conversations: Array<{
    id: string;
    name: string | null;
    is_group: boolean;
    participants: string[];
    last_message: string | null;
    last_message_at: string | null;
    unread_count: number;
  }>;
  messages: Array<{
    id: string;
    conversation_id: string;
    from_id: string;
    content: string;
    content_hash: string;
    message_type: string;
    timestamp: string;
  }>;
}

interface SendMessageApiResponse {
  message_id: string;
  hash: string;
  sequence: number;
}

interface CreateConversationApiResponse {
  id: string;
  hash: string;
}

interface ApprovalDecisionApiResponse {
  job_id: string;
  decision: string;
  hash: string;
}

// ============================================================================
// Mappers
// ============================================================================

function mapEntity(e: BootstrapApiResponse['entities'][0]): Entity {
  return {
    id: e.id,
    name: e.display_name,
    avatar: e.avatar_url || `https://api.dicebear.com/7.x/notionists/svg?seed=${e.id}`,
    type: e.kind === 'person' ? 'human' : e.kind === 'llm' ? 'agent' : 'system',
    status: (e.status as any) || 'online',
  };
}

function mapConversation(c: BootstrapApiResponse['conversations'][0]): Conversation {
  return {
    id: c.id,
    name: c.name || undefined,
    isGroup: c.is_group,
    participants: c.participants,
    lastMessage: c.last_message || undefined,
    lastMessageTime: c.last_message_at 
      ? formatTimeAgo(new Date(c.last_message_at)) 
      : undefined,
    unreadCount: c.unread_count,
  };
}

function mapMessage(m: BootstrapApiResponse['messages'][0]): Message {
  return {
    id: m.id,
    from: m.from_id,
    to: m.conversation_id,
    content: m.content,
    hash: m.content_hash,
    type: m.message_type as MessageType,
    timestamp: new Date(m.timestamp),
    status: 'sent',
  };
}

function formatTimeAgo(date: Date): string {
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const minutes = Math.floor(diff / 60000);
  
  if (minutes < 1) return 'now';
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h`;
  const days = Math.floor(hours / 24);
  return `${days}d`;
}

// ============================================================================
// API Client
// ============================================================================

export const ublApi = {
  /**
   * Health check - tests connectivity
   */
  async health(): Promise<{ ok: true }> {
    return api.get<{ ok: true }>(`/health`);
  },

  /**
   * Bootstrap - get initial state (user, entities, conversations, messages)
   * Endpoint: GET /messenger/bootstrap
   */
  async bootstrap(): Promise<{ 
    user: Entity | null;
    entities: Entity[]; 
    conversations: Conversation[]; 
    messages: Message[];
  }> {
    const res = await api.get<BootstrapApiResponse>(`/messenger/bootstrap`);
    
    return {
      user: res.user ? {
        id: res.user.sid,
        name: res.user.display_name,
        avatar: `https://api.dicebear.com/7.x/notionists/svg?seed=${res.user.sid}`,
        type: res.user.kind === 'person' ? 'human' : 'agent',
        status: 'online',
      } : null,
      entities: res.entities.map(mapEntity),
      conversations: res.conversations.map(mapConversation),
      messages: res.messages.map(mapMessage),
    };
  },

  /**
   * List entities
   * Endpoint: GET /messenger/entities
   */
  async listEntities(): Promise<Entity[]> {
    const res = await api.get<BootstrapApiResponse['entities']>(`/messenger/entities`);
    return res.map(mapEntity);
  },

  /**
   * List conversations
   * Endpoint: GET /messenger/conversations
   */
  async listConversations(): Promise<Conversation[]> {
    const res = await api.get<BootstrapApiResponse['conversations']>(`/messenger/conversations`);
    return res.map(mapConversation);
  },

  /**
   * Create conversation/workstream
   * Endpoint: POST /messenger/conversations
   */
  async createConversation(input: { 
    participants: string[]; 
    name?: string; 
    isGroup?: boolean;
  }): Promise<{ id: string; hash: string }> {
    const res = await api.post<CreateConversationApiResponse>(`/messenger/conversations`, {
      participants: input.participants,
      name: input.name,
      is_group: input.isGroup,
    });
    return { id: res.id, hash: res.hash };
  },

  /**
   * Send message
   * Endpoint: POST /messenger/messages
   */
  async sendMessage(input: { 
    conversationId: string; 
    content: string; 
    type?: MessageType;
  }): Promise<{ messageId: string; hash: string; sequence: number }> {
    const res = await api.post<SendMessageApiResponse>(`/messenger/messages`, {
      conversation_id: input.conversationId,
      content: input.content,
      message_type: input.type || 'text',
    });
    return { 
      messageId: res.message_id, 
      hash: res.hash, 
      sequence: res.sequence,
    };
  },

  /**
   * Approve a job
   * Endpoint: POST /messenger/jobs/:id/approve
   */
  async approveJob(jobId: string, reason?: string): Promise<ApprovalDecisionApiResponse> {
    return api.post<ApprovalDecisionApiResponse>(`/messenger/jobs/${jobId}/approve`, { reason });
  },

  /**
   * Reject a job
   * Endpoint: POST /messenger/jobs/:id/reject
   */
  async rejectJob(jobId: string, reason?: string): Promise<ApprovalDecisionApiResponse> {
    return api.post<ApprovalDecisionApiResponse>(`/messenger/jobs/${jobId}/reject`, { reason });
  },

  /**
   * Get user info (from identity API)
   * Endpoint: GET /id/whoami
   */
  async getMe(): Promise<{ sid: string; displayName: string; kind: string; authenticated: boolean }> {
    const res = await api.get<{
      sid: string | null;
      kind: string | null;
      display_name: string | null;
      authenticated: boolean;
    }>(`/id/whoami`);
    
    return {
      sid: res.sid || '',
      displayName: res.display_name || '',
      kind: res.kind || '',
      authenticated: res.authenticated,
    };
  },

  /**
   * Get user settings (placeholder - to be implemented)
   */
  async getSettings(): Promise<UserSettings> {
    // TODO: Implement when settings endpoint exists
    return {
      theme: 'dark',
      fontSize: 'md',
      audioEnabled: true,
      notificationsEnabled: true,
    };
  },

  /**
   * Update user settings (placeholder - to be implemented)
   */
  async updateSettings(patch: Partial<UserSettings>): Promise<UserSettings> {
    // TODO: Implement when settings endpoint exists
    console.log('Settings update:', patch);
    return this.getSettings();
  },

  // ============================================================================
  // Gateway API (v1)
  // ============================================================================

  /**
   * Send message via Gateway
   * Endpoint: POST /v1/conversations/:id/messages
   */
  async sendMessageViaGateway(input: {
    conversationId: string;
    content: string;
    messageType?: MessageType;
    idempotencyKey?: string;
  }): Promise<{ messageId: string; hash: string; sequence: number; action: string }> {
    const res = await api.post<{
      message_id: string;
      hash: string;
      sequence: number;
      action: string;
    }>(`/v1/conversations/${input.conversationId}/messages`, {
      content: input.content,
      message_type: input.messageType || 'text',
      idempotency_key: input.idempotencyKey,
    });
    return {
      messageId: res.message_id,
      hash: res.hash,
      sequence: res.sequence,
      action: res.action,
    };
  },

  /**
   * Handle job action via Gateway
   * Endpoint: POST /v1/jobs/:id/actions
   */
  async jobActionViaGateway(input: {
    jobId: string;
    actionType: string;
    buttonId: string;
    cardId: string;
    inputData?: any;
    idempotencyKey?: string;
  }): Promise<{ success: boolean; eventIds: string[] }> {
    const res = await api.post<{
      success: boolean;
      event_ids: string[];
    }>(`/v1/jobs/${input.jobId}/actions`, {
      action_type: input.actionType,
      button_id: input.buttonId,
      card_id: input.cardId,
      input_data: input.inputData,
      idempotency_key: input.idempotencyKey,
    });
    return {
      success: res.success,
      eventIds: res.event_ids,
    };
  },

  /**
   * Get conversation timeline
   * Endpoint: GET /v1/conversations/:id/timeline
   */
  async getTimeline(input: {
    conversationId: string;
    tenantId?: string;
    cursor?: string;
    limit?: number;
  }): Promise<{ items: any[]; cursor: string }> {
    const params = new URLSearchParams();
    if (input.tenantId) params.set('tenant_id', input.tenantId);
    if (input.cursor) params.set('cursor', input.cursor);
    if (input.limit) params.set('limit', input.limit.toString());

    const res = await api.get<{
      items: any[];
      cursor: string;
    }>(`/v1/conversations/${input.conversationId}/timeline?${params.toString()}`);
    return res;
  },

  /**
   * Get job details for drawer
   * Endpoint: GET /v1/jobs/:id
   */
  async getJob(input: {
    jobId: string;
    tenantId?: string;
  }): Promise<{
    jobId: string;
    title: string;
    goal: string;
    state: string;
    owner: any;
    availableActions: any[];
    timeline: any[];
    artifacts: any[];
  }> {
    const params = new URLSearchParams();
    if (input.tenantId) params.set('tenant_id', input.tenantId);

    const res = await api.get<{
      job_id: string;
      title: string;
      goal: string;
      state: string;
      owner: any;
      available_actions: any[];
      timeline: any[];
      artifacts: any[];
    }>(`/v1/jobs/${input.jobId}?${params.toString()}`);
    
    return {
      jobId: res.job_id,
      title: res.title,
      goal: res.goal,
      state: res.state,
      owner: res.owner,
      availableActions: res.available_actions,
      timeline: res.timeline,
      artifacts: res.artifacts,
    };
  },
};

export default ublApi;
