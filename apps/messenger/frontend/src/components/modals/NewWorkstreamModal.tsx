/**
 * New Workstream Modal
 * Beautiful modal for creating new conversations/workstreams
 */

import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Users, User, Sparkles } from 'lucide-react';
import { Button, Input } from '../ui';
import { Entity } from '../../types';

interface NewWorkstreamModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (data: { name: string; participants: string[]; isGroup: boolean }) => void;
  entities: Entity[];
  currentUserId: string;
}

export const NewWorkstreamModal: React.FC<NewWorkstreamModalProps> = ({
  isOpen,
  onClose,
  onSubmit,
  entities,
  currentUserId,
}) => {
  const [name, setName] = useState('');
  const [selectedParticipants, setSelectedParticipants] = useState<string[]>([]);
  const [mode, setMode] = useState<'direct' | 'group'>('direct');

  const availableEntities = entities.filter(e => e.id !== currentUserId);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    if (mode === 'direct' && selectedParticipants.length === 1) {
      onSubmit({
        name: '',
        participants: [currentUserId, selectedParticipants[0]],
        isGroup: false,
      });
    } else if (mode === 'group' && name.trim()) {
      onSubmit({
        name: name.trim(),
        participants: [currentUserId, ...selectedParticipants],
        isGroup: true,
      });
    }
    
    // Reset
    setName('');
    setSelectedParticipants([]);
    onClose();
  };

  const toggleParticipant = (id: string) => {
    setSelectedParticipants(prev =>
      prev.includes(id) ? prev.filter(p => p !== id) : [...prev, id]
    );
  };

  if (!isOpen) return null;

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4"
        onClick={onClose}
      >
        <motion.div
          initial={{ scale: 0.95, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          exit={{ scale: 0.95, opacity: 0 }}
          onClick={e => e.stopPropagation()}
          className="w-full max-w-md bg-bg-elevated border border-border-default rounded-3xl shadow-2xl overflow-hidden"
        >
          {/* Header */}
          <div className="flex items-center justify-between p-6 border-b border-border-subtle">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-xl bg-accent/10 flex items-center justify-center">
                <Sparkles className="w-5 h-5 text-accent" />
              </div>
              <div>
                <h2 className="text-lg font-black text-text-primary">New Workstream</h2>
                <p className="text-xs text-text-tertiary">Start a conversation</p>
              </div>
            </div>
            <button
              onClick={onClose}
              className="p-2 rounded-lg text-text-tertiary hover:text-text-primary hover:bg-bg-surface transition-colors"
            >
              <X className="w-5 h-5" />
            </button>
          </div>

          {/* Mode Selector */}
          <div className="p-4 border-b border-border-subtle">
            <div className="flex gap-2 p-1 bg-bg-surface rounded-xl">
              <button
                onClick={() => setMode('direct')}
                className={`flex-1 flex items-center justify-center gap-2 py-3 rounded-lg text-xs font-bold uppercase tracking-wide transition-all ${
                  mode === 'direct'
                    ? 'bg-accent text-text-inverse shadow-glow-sm'
                    : 'text-text-tertiary hover:text-text-primary'
                }`}
              >
                <User className="w-4 h-4" />
                Direct
              </button>
              <button
                onClick={() => setMode('group')}
                className={`flex-1 flex items-center justify-center gap-2 py-3 rounded-lg text-xs font-bold uppercase tracking-wide transition-all ${
                  mode === 'group'
                    ? 'bg-accent text-text-inverse shadow-glow-sm'
                    : 'text-text-tertiary hover:text-text-primary'
                }`}
              >
                <Users className="w-4 h-4" />
                Group
              </button>
            </div>
          </div>

          <form onSubmit={handleSubmit}>
            {/* Group Name (only for group mode) */}
            {mode === 'group' && (
              <div className="p-4 border-b border-border-subtle">
                <Input
                  label="Workstream Name"
                  placeholder="e.g., Strategic Board"
                  value={name}
                  onChange={e => setName(e.target.value)}
                />
              </div>
            )}

            {/* Participants */}
            <div className="p-4 max-h-64 overflow-y-auto">
              <label className="block text-xxs font-black text-text-tertiary uppercase tracking-widest mb-3">
                {mode === 'direct' ? 'Select Contact' : 'Add Participants'}
              </label>
              
              <div className="space-y-2">
                {availableEntities.map(entity => (
                  <button
                    key={entity.id}
                    type="button"
                    onClick={() => {
                      if (mode === 'direct') {
                        setSelectedParticipants([entity.id]);
                      } else {
                        toggleParticipant(entity.id);
                      }
                    }}
                    className={`w-full flex items-center gap-3 p-3 rounded-xl border transition-all ${
                      selectedParticipants.includes(entity.id)
                        ? 'border-accent bg-accent/10'
                        : 'border-border-subtle hover:border-border-default bg-bg-surface'
                    }`}
                  >
                    <img
                      src={entity.avatar}
                      alt={entity.name}
                      className="w-10 h-10 rounded-full"
                    />
                    <div className="flex-1 text-left">
                      <div className="text-sm font-bold text-text-primary">{entity.name}</div>
                      <div className="text-xs text-text-tertiary capitalize">{entity.type}</div>
                    </div>
                    {selectedParticipants.includes(entity.id) && (
                      <div className="w-6 h-6 rounded-full bg-accent flex items-center justify-center">
                        <svg className="w-4 h-4 text-text-inverse" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3">
                          <polyline points="20 6 9 17 4 12" />
                        </svg>
                      </div>
                    )}
                  </button>
                ))}
              </div>
            </div>

            {/* Footer */}
            <div className="p-4 border-t border-border-subtle bg-bg-surface/50">
              <Button
                type="submit"
                disabled={
                  (mode === 'direct' && selectedParticipants.length !== 1) ||
                  (mode === 'group' && (!name.trim() || selectedParticipants.length === 0))
                }
                className="w-full"
              >
                Create Workstream
              </Button>
            </div>
          </form>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
};

export default NewWorkstreamModal;

