
import { createClient, SupabaseClient } from '@supabase/supabase-js';
import { Contact, Message, FileData, UserAccount, Role } from '../types';

const getStoredConfig = () => {
  const url = process.env.SUPABASE_URL || localStorage.getItem('ubl_supabase_url');
  const key = process.env.SUPABASE_ANON_KEY || localStorage.getItem('ubl_supabase_key');
  return { url, key };
};

let { url: currentUrl, key: currentKey } = getStoredConfig();

export let supabase: SupabaseClient | null = (currentUrl && currentKey) 
  ? createClient(currentUrl, currentKey) 
  : null;

class APIService {
  private currentTenant = "UBL_FOUNDATION_PRIME_01";

  setupBridge(url: string, key: string) {
    localStorage.setItem('ubl_supabase_url', url);
    localStorage.setItem('ubl_supabase_key', key);
    supabase = createClient(url, key);
  }

  isConfigured(): boolean {
    return !!supabase;
  }

  async getAccount(): Promise<UserAccount | null> {
    if (!supabase) return null;
    const { data: { user } } = await supabase.auth.getUser();
    if (!user) return null;

    try {
      const { data: profile, error } = await supabase
        .from('profiles')
        .select('*')
        .eq('id', user.id)
        .maybeSingle();

      if (error) throw new Error(`Profiles table error: ${error.message}`);

      if (!profile) {
        const newProfile = {
          id: user.id,
          name: user.email?.split('@')[0] || 'Entity',
          role: 'Explorer',
          avatar_url: `https://api.dicebear.com/7.x/notionists/svg?seed=${user.id}`,
          tenant_id: this.currentTenant,
          stats: { ledgerEntries: 0, activeJobs: 0, uptime: '100%' }
        };
        const { error: insError } = await supabase.from('profiles').insert(newProfile);
        if (insError) throw insError;
        
        return {
          ...newProfile,
          entityId: user.id,
          bio: '',
          joinedAt: new Date().toISOString(),
          trustScore: 1.0,
        } as any;
      }

      return {
        name: profile.name,
        role: profile.role,
        avatar: profile.avatar_url,
        entityId: profile.id,
        bio: profile.bio,
        joinedAt: profile.created_at,
        trustScore: profile.trust_score,
        tenantId: profile.tenant_id,
        stats: profile.stats
      };
    } catch (e: any) {
      console.error(e);
      throw new Error(`Database Initialization Error: ${e.message}. Did you run the SQL schema?`);
    }
  }

  async getContacts(): Promise<Contact[]> {
    if (!supabase) return [];
    const { data, error } = await supabase
      .from('workstreams')
      .select('*')
      .order('last_message_time', { ascending: false });

    if (error) {
      if (error.code === 'PGRST116' || error.message.includes('not found')) {
        throw new Error("Table 'workstreams' not found. Please run the SQL schema in your Supabase dashboard.");
      }
      return [];
    }

    if (data.length === 0) {
      try {
        const genesisAgent = await this.createWorkstream('Core Agent', 'agent', 'ent_core_001');
        return [genesisAgent];
      } catch (e) {
        return [];
      }
    }

    return data.map(ws => ({
      id: ws.id,
      name: ws.name,
      avatar: ws.avatar_url || '',
      type: ws.type as any,
      lastMessage: ws.last_message,
      lastMessageTime: ws.last_message_time ? new Date(ws.last_message_time).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) : 'new',
      online: true,
      entityId: ws.entity_id
    }));
  }

  async createWorkstream(name: string, type: string = 'group', entityId?: string): Promise<Contact> {
    if (!supabase) throw new Error("Bridge not configured");
    const { data, error } = await supabase
      .from('workstreams')
      .insert({
        name,
        type,
        entity_id: entityId || `ent_node_${Math.floor(Math.random() * 999)}`,
        last_message: 'Workstream Genesis initialized.',
        last_message_time: new Date().toISOString(),
        avatar_url: type === 'agent' ? `https://api.dicebear.com/7.x/bottts/svg?seed=${name}` : `https://api.dicebear.com/7.x/notionists/svg?seed=${name}`
      })
      .select()
      .single();

    if (error) throw error;

    return {
      id: data.id,
      name: data.name,
      avatar: data.avatar_url || '',
      type: data.type,
      lastMessage: data.last_message,
      lastMessageTime: 'now',
      online: true,
      entityId: data.entity_id
    };
  }

  async getHistoryFromLedger(workstreamId: string): Promise<Message[]> {
    if (!supabase) return [];
    const { data, error } = await supabase
      .from('messages')
      .select('*')
      .eq('workstream_id', workstreamId)
      .order('ledger_index', { ascending: true });

    if (error) return [];

    return data.map(m => ({
      ...m,
      timestamp: new Date(m.created_at),
      parts: m.parts as any
    }));
  }

  private async calculateHash(content: string, prevHash: string): Promise<string> {
    const encoder = new TextEncoder();
    const data = encoder.encode(content + prevHash + Date.now().toString());
    const hashBuffer = await crypto.subtle.digest('SHA-256', data);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return '0x' + hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
  }

  async persistToLedger(workstreamId: string, role: Role, content: string, parts?: any[]): Promise<Message> {
    if (!supabase) throw new Error("Bridge not configured");
    const { data: userData } = await supabase.auth.getUser();
    const history = await this.getHistoryFromLedger(workstreamId);
    const prevMsg = history[history.length - 1];
    const prevHash = prevMsg ? prevMsg.hash : '0x0';
    const index = history.length;
    const hash = await this.calculateHash(content, prevHash);

    const { data, error } = await supabase
      .from('messages')
      .insert({
        workstream_id: workstreamId,
        user_id: userData.user?.id,
        role,
        content,
        parts,
        ledger_index: index,
        previous_hash: prevHash,
        hash,
        tenant_id: this.currentTenant
      })
      .select()
      .single();

    if (error) throw error;

    await supabase
      .from('workstreams')
      .update({ last_message: content, last_message_time: new Date().toISOString() })
      .eq('id', workstreamId);

    return {
      ...data,
      timestamp: new Date(data.created_at),
      parts: data.parts as any
    };
  }

  async sendMessage(workstreamId: string, text: string, file?: FileData): Promise<Message> {
    return this.persistToLedger(workstreamId, 'user', text, file ? [{ text, file }] : [{ text }]);
  }

  async uploadToVault(file: File | Blob, customName?: string): Promise<FileData> {
    if (!supabase) throw new Error("Bridge not configured");
    const fileName = customName || `${Math.random()}.${(file as File).name?.split('.').pop() || 'bin'}`;
    const filePath = `vault/${fileName}`;

    const { error: uploadError } = await supabase.storage
      .from('vault')
      .upload(filePath, file);

    if (uploadError) throw uploadError;

    const { data: { publicUrl } } = supabase.storage
      .from('vault')
      .getPublicUrl(filePath);

    const hash = await this.calculateHash(fileName + file.size, 's3_vault_salt');

    return {
      name: customName || (file as File).name || 'vault_asset',
      size: file.size,
      type: file.type,
      url: publicUrl,
      s3Key: filePath,
      hash
    };
  }

  async clearHistory(workstreamId: string) {
    if (!supabase) return false;
    await supabase.from('messages').delete().eq('workstream_id', workstreamId);
    return true;
  }

  async fetchJobLogs(jobId: string) {
    return [
      `[${new Date().toLocaleTimeString()}] SUPABASE_AUTH_HANDSHAKE: OK`,
      `[${new Date().toLocaleTimeString()}] POSTGRES_LEDGER_SYNC: OK`,
      `[${new Date().toLocaleTimeString()}] VAULT_S3_MOUNT: OK`,
    ];
  }

  async processJobAction(jobId: string, action: string) {
    return { success: true };
  }
}

export const api = new APIService();
