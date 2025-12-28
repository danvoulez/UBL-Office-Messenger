/**
 * UBL Messenger - Main Application
 * Beautiful Orange Theme + Properly Wired Architecture
 */

import React, { useState, useEffect, useCallback, useRef } from 'react';
import Sidebar from './components/Sidebar';
import ChatView from './components/ChatView';
import WelcomeScreen from './components/WelcomeScreen';
import BridgeConfig from './components/BridgeConfig';
import { Entity, Conversation, Message, UserSettings } from './types';
import { Icons, INITIAL_ENTITIES, INITIAL_CONVERSATIONS, INITIAL_MESSAGES, PERSONAL_AGENT_ID } from './constants';
import { ublApi } from './services/ublApi';
import { jobsApi } from './services/jobsApi';

// Check if API is configured
const isApiConfigured = () => {
  const url = localStorage.getItem('ubl_api_base_url');
  return url !== null; // null = never configured, '' = demo mode
};

const App: React.FC = () => {
  // Configuration state
  const [isConfigured, setIsConfigured] = useState(isApiConfigured());
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Session state
  const [currentUser, setCurrentUser] = useState<Entity>(INITIAL_ENTITIES[0]);
  const [entities, setEntities] = useState<Entity[]>(INITIAL_ENTITIES);
  const [conversations, setConversations] = useState<Conversation[]>(INITIAL_CONVERSATIONS);
  const [messages, setMessages] = useState<Message[]>(INITIAL_MESSAGES);
  
  // UI state
  const [activeConvId, setActiveConvId] = useState<string | null>(null);
  const [showMobileChat, setShowMobileChat] = useState(false);
  const [isTyping, setIsTyping] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [inspectEntity, setInspectEntity] = useState<{ entity: Entity; initialTab?: 'profile' | 'settings' } | null>(null);
  const [showNewEntity, setShowNewEntity] = useState(false);
  
  // Settings
  const [userSettings, setUserSettings] = useState<UserSettings>({
    theme: 'dark',
    fontSize: 'md',
    audioEnabled: true,
    notificationsEnabled: true,
    glowIntensity: 0.6
  });

  // Initialize app
  useEffect(() => {
    const init = async () => {
      if (!isConfigured) {
        setIsLoading(false);
        return;
      }

      const apiUrl = localStorage.getItem('ubl_api_base_url');
      
      // Demo mode - use initial data
      if (!apiUrl) {
        setIsLoading(false);
        return;
      }

      // Real API mode - fetch data
      try {
        const health = await ublApi.health();
        if (health.ok) {
          const bootstrap = await ublApi.bootstrap();
          setEntities(bootstrap.entities);
          setConversations(bootstrap.conversations);
          setMessages(bootstrap.messages);
        }
      } catch (err) {
        console.log('Using demo mode - API not available');
      } finally {
        setIsLoading(false);
      }
    };

    init();
  }, [isConfigured]);

  // Connect to job updates WebSocket
  useEffect(() => {
    if (!isConfigured) return;

    const unsubscribe = jobsApi.subscribe((event) => {
      console.log('[Jobs] Event:', event);
      // Handle job updates here
    });

    return () => unsubscribe();
  }, [isConfigured]);

  // Get active conversation
  const activeConversation = conversations.find(c => c.id === activeConvId);

  // Filter messages for current conversation
  const currentMessages = messages.filter(m => {
    if (!activeConversation) return false;
    if (activeConversation.isGroup) return m.to === activeConversation.id;
    const otherParticipant = activeConversation.participants.find(p => p !== currentUser.id);
    return (m.from === currentUser.id && m.to === otherParticipant) || 
           (m.from === otherParticipant && m.to === currentUser.id);
  });

  // Send message handler
  const handleSendMessage = useCallback(async (content: string, type: string = 'chat') => {
    if (!activeConversation || !content.trim()) return;

    const otherParticipant = activeConversation.isGroup 
      ? activeConversation.id 
      : activeConversation.participants.find(p => p !== currentUser.id);

    // Create new message
    const newMessage: Message = {
      id: `msg_${Date.now()}`,
      from: currentUser.id,
      to: otherParticipant,
      content,
      timestamp: new Date(),
      status: 'sent',
      hash: `0x${Math.random().toString(16).slice(2, 18).toUpperCase()}`,
      type: type as any
    };

    setMessages(prev => [...prev, newMessage]);

    // Update conversation last message
    setConversations(prev => prev.map(c => 
      c.id === activeConversation.id 
        ? { ...c, lastMessage: content, lastMessageTime: 'now' }
        : c
    ));

    // Simulate AI response if chatting with agent
    const otherEntity = entities.find(e => e.id === otherParticipant);
    if (otherEntity?.type === 'agent') {
      setIsTyping(true);
      
      // Simulate thinking time
      await new Promise(resolve => setTimeout(resolve, 1000 + Math.random() * 2000));

      const aiResponse: Message = {
        id: `msg_${Date.now()}`,
        from: otherParticipant,
        to: currentUser.id,
        content: generateAIResponse(content, otherEntity),
        timestamp: new Date(),
        status: 'sent',
        hash: `0x${Math.random().toString(16).slice(2, 18).toUpperCase()}`,
        type: 'chat'
      };

      setMessages(prev => [...prev, aiResponse]);
      setIsTyping(false);
    }
  }, [activeConversation, currentUser, entities]);

  // Simple AI response generator
  const generateAIResponse = (userMessage: string, agent: Entity): string => {
    const responses = [
      `I've analyzed your request: "${userMessage.slice(0, 50)}...". Processing via UBL Protocol.`,
      `Understood. Initiating ledger synchronization for: ${userMessage.slice(0, 30)}...`,
      `Task acknowledged. The Trinity infrastructure is handling your request.`,
      `Affirmative. I'm querying the distributed ledger for relevant context.`,
      `Processing complete. All operations have been logged to the immutable ledger.`
    ];
    return responses[Math.floor(Math.random() * responses.length)];
  };

  // Toggle status handler
  const handleToggleStatus = () => {
    setCurrentUser(prev => ({
      ...prev,
      status: prev.status === 'online' ? 'away' : 'online'
    }));
  };

  // Create new conversation
  const handleCreateConversation = () => {
    const name = prompt('Enter workstream name:');
    if (!name) return;

    const newConv: Conversation = {
      id: `conv_${Date.now()}`,
      participants: [currentUser.id],
      isGroup: true,
      name,
      unreadCount: 0,
      lastMessage: 'Workstream created'
    };

    setConversations(prev => [newConv, ...prev]);
    setActiveConvId(newConv.id);
    setShowMobileChat(true);
  };

  // Logout handler
  const handleLogout = () => {
    localStorage.removeItem('ubl_api_base_url');
    setIsConfigured(false);
  };

  // Not configured - show config screen
  if (!isConfigured) {
    return <BridgeConfig onConfigured={() => setIsConfigured(true)} />;
  }

  // Loading state
  if (isLoading) {
    return (
      <div className="h-screen w-screen bg-bg-base flex flex-col items-center justify-center p-10">
        <div className="w-12 h-12 border-4 border-accent/20 border-t-accent rounded-full animate-spin mb-6" />
        <p className="text-text-tertiary text-[10px] font-black uppercase tracking-[0.4em]">
          Initializing UBL Environment...
        </p>
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div className="h-screen w-screen bg-bg-base flex items-center justify-center p-6 text-center">
        <div className="max-w-md bg-bg-elevated border border-error/30 p-10 rounded-[40px] shadow-2xl">
          <div className="w-16 h-16 bg-error/10 text-error rounded-2xl mx-auto mb-6 flex items-center justify-center">
            <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
            </svg>
          </div>
          <h2 className="text-xl font-black text-text-primary uppercase mb-4">Infrastructure Fault</h2>
          <p className="text-text-secondary text-sm mb-8">{error}</p>
          <button 
            onClick={() => window.location.reload()} 
            className="w-full py-4 bg-accent text-bg-base font-black rounded-xl uppercase tracking-widest text-[10px]"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-screen w-screen bg-bg-base overflow-hidden font-sans antialiased text-text-primary">
      {/* Ambient Glow Effect */}
      <div 
        className="fixed inset-0 pointer-events-none transition-opacity duration-1000 z-0" 
        style={{ 
          background: `radial-gradient(circle at 50% 50%, rgba(224, 122, 95, ${(userSettings.glowIntensity || 0.6) * 0.1}), transparent 80%)`,
          opacity: userSettings.glowIntensity || 0.6 
        }} 
      />

      {/* Sidebar */}
      <div className={`${showMobileChat ? 'hidden md:flex' : 'flex w-full md:w-auto'} h-full z-10 relative`}>
        <Sidebar 
          conversations={conversations}
          activeConvId={activeConvId}
          onSelectConv={(id) => {
            setActiveConvId(id);
            setShowMobileChat(true);
          }}
          entities={entities}
          currentUser={currentUser}
          onToggleStatus={handleToggleStatus}
          onInspectEntity={(entity, tab) => setInspectEntity({ entity, initialTab: tab })}
          onNewEntity={handleCreateConversation}
          onLogout={handleLogout}
        />
      </div>

      {/* Main Content */}
      <div className={`${showMobileChat ? 'flex w-full' : 'hidden md:flex md:flex-1'} h-full z-10 relative`}>
        {activeConversation ? (
          <ChatView 
            conversation={activeConversation}
            messages={currentMessages}
            entities={entities}
            currentUser={currentUser}
            onSendMessage={handleSendMessage}
            onBack={() => setShowMobileChat(false)}
            onInspectEntity={(entity, tab) => setInspectEntity({ entity, initialTab: tab })}
            isTyping={isTyping}
          />
        ) : (
          <WelcomeScreen
            user={currentUser}
            conversations={conversations}
            entities={entities}
            onSelectConversation={(id) => {
              setActiveConvId(id);
              setShowMobileChat(true);
            }}
            onNewConversation={handleCreateConversation}
            onOpenSettings={() => setInspectEntity({ entity: currentUser, initialTab: 'settings' })}
          />
        )}
      </div>

      {/* Settings Modal */}
      {inspectEntity?.initialTab === 'settings' && (
        <div 
          className="fixed inset-0 z-[120] bg-black/80 backdrop-blur-xl flex items-center justify-center p-4 animate-fade-in" 
          onClick={() => setInspectEntity(null)}
        >
          <div 
            className="bg-bg-elevated w-full max-w-lg rounded-3xl border border-border-default shadow-2xl overflow-hidden"
            onClick={e => e.stopPropagation()}
          >
            <div className="p-6 border-b border-border-subtle flex items-center justify-between">
              <h2 className="text-lg font-black uppercase tracking-tight">Settings</h2>
              <button 
                onClick={() => setInspectEntity(null)}
                className="w-8 h-8 flex items-center justify-center text-text-tertiary hover:text-text-primary rounded-lg hover:bg-bg-hover transition-colors"
              >
                <svg className="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <line x1="18" y1="6" x2="6" y2="18"></line>
                  <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
              </button>
            </div>
            
            <div className="p-6 space-y-6">
              {/* Glow Intensity */}
              <div>
                <label className="block text-[10px] font-black text-text-tertiary uppercase tracking-wider mb-3">
                  Environmental Bloom
                </label>
                <input 
                  type="range" 
                  min="0" 
                  max="1" 
                  step="0.01" 
                  value={userSettings.glowIntensity || 0.6}
                  onChange={e => setUserSettings(prev => ({ ...prev, glowIntensity: parseFloat(e.target.value) }))}
                  className="w-full accent-accent h-2 bg-bg-hover rounded-full appearance-none cursor-pointer"
                />
              </div>

              {/* Theme */}
              <div>
                <label className="block text-[10px] font-black text-text-tertiary uppercase tracking-wider mb-3">
                  Theme
                </label>
                <div className="flex gap-2">
                  <button className="flex-1 py-3 bg-accent text-bg-base font-bold rounded-xl text-sm">
                    Dark
                  </button>
                  <button className="flex-1 py-3 bg-bg-surface text-text-tertiary font-bold rounded-xl text-sm border border-border-subtle">
                    Light
                  </button>
                </div>
              </div>

              {/* API Status */}
              <div className="p-4 bg-bg-surface rounded-xl border border-border-subtle">
                <div className="text-[10px] font-black text-text-tertiary uppercase tracking-wider mb-2">
                  API Connection
                </div>
                <div className="flex items-center gap-2">
                  <span className="w-2 h-2 bg-success rounded-full animate-pulse-soft"></span>
                  <span className="text-sm text-text-primary font-medium">
                    {localStorage.getItem('ubl_api_base_url') || 'Demo Mode'}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Profile Modal */}
      {inspectEntity && inspectEntity.initialTab !== 'settings' && (
        <div 
          className="fixed inset-0 z-[120] bg-black/80 backdrop-blur-xl flex items-center justify-center p-4 animate-fade-in" 
          onClick={() => setInspectEntity(null)}
        >
          <div 
            className="bg-bg-elevated w-full max-w-md rounded-[40px] border border-border-default p-10 shadow-2xl text-center"
            onClick={e => e.stopPropagation()}
          >
            <div className="w-24 h-24 rounded-3xl mx-auto mb-6 overflow-hidden border-2 border-accent/20 shadow-glow">
              {inspectEntity.entity.avatar ? (
                <img src={inspectEntity.entity.avatar} alt="" className="w-full h-full object-cover" />
              ) : (
                <div className="w-full h-full bg-accent/10 flex items-center justify-center text-accent">
                  <Icons.Agent />
                </div>
              )}
            </div>
            <h2 className="text-2xl font-black mb-1">{inspectEntity.entity.name}</h2>
            <div className="text-accent text-[10px] font-black uppercase tracking-[0.3em] mb-4">
              {inspectEntity.entity.role || inspectEntity.entity.type}
            </div>
            {inspectEntity.entity.about && (
              <p className="text-text-secondary text-sm mb-6">{inspectEntity.entity.about}</p>
            )}
            <button 
              onClick={() => setInspectEntity(null)}
              className="w-full py-4 bg-bg-surface text-text-secondary font-bold rounded-2xl uppercase tracking-widest text-[10px] border border-border-subtle hover:bg-bg-hover transition-colors"
            >
              Close
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

export default App;
