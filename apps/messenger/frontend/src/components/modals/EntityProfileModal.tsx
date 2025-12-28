/**
 * Entity Profile Modal
 * Shows detailed information about a user/agent
 */

import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Shield, Zap, MessageSquare, Activity, Star } from 'lucide-react';
import { Entity } from '../../types';
import { Button } from '../ui';

interface EntityProfileModalProps {
  isOpen: boolean;
  onClose: () => void;
  entity: Entity | null;
  onStartChat?: (entityId: string) => void;
}

export const EntityProfileModal: React.FC<EntityProfileModalProps> = ({
  isOpen,
  onClose,
  entity,
  onStartChat,
}) => {
  if (!isOpen || !entity) return null;

  const isAgent = entity.type === 'agent';
  
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
          className="w-full max-w-sm bg-bg-elevated border border-border-default rounded-3xl shadow-2xl overflow-hidden"
        >
          {/* Header with Avatar */}
          <div className="relative h-32 bg-gradient-to-br from-accent/20 to-accent/5">
            <button
              onClick={onClose}
              className="absolute top-4 right-4 p-2 rounded-lg bg-bg-base/50 text-text-tertiary hover:text-text-primary transition-colors"
            >
              <X className="w-5 h-5" />
            </button>
            
            <div className="absolute -bottom-12 left-1/2 -translate-x-1/2">
              <div className="relative">
                <img
                  src={entity.avatar}
                  alt={entity.name}
                  className="w-24 h-24 rounded-2xl border-4 border-bg-elevated shadow-xl"
                />
                <div className={`absolute -bottom-1 -right-1 w-6 h-6 rounded-full border-2 border-bg-elevated ${
                  entity.status === 'online' ? 'bg-success' :
                  entity.status === 'working' ? 'bg-warning' :
                  entity.status === 'away' ? 'bg-warning' : 'bg-text-tertiary'
                }`} />
              </div>
            </div>
          </div>

          {/* Content */}
          <div className="pt-14 pb-6 px-6 text-center">
            <h2 className="text-xl font-black text-text-primary mb-1">{entity.name}</h2>
            <div className="flex items-center justify-center gap-2 text-sm text-text-tertiary">
              {isAgent ? (
                <Zap className="w-4 h-4 text-accent" />
              ) : (
                <Shield className="w-4 h-4 text-info" />
              )}
              <span className="capitalize">{entity.role || entity.type}</span>
            </div>

            {entity.about && (
              <p className="mt-4 text-sm text-text-secondary leading-relaxed">
                {entity.about}
              </p>
            )}

            {/* Stats */}
            {entity.trustScore !== undefined && (
              <div className="mt-6 grid grid-cols-3 gap-4 p-4 bg-bg-surface rounded-2xl">
                <div className="text-center">
                  <div className="flex items-center justify-center gap-1 text-lg font-black text-accent">
                    <Star className="w-4 h-4" />
                    {entity.trustScore}
                  </div>
                  <div className="text-xxs text-text-tertiary uppercase tracking-wider">Trust</div>
                </div>
                <div className="text-center">
                  <div className="text-lg font-black text-text-primary">
                    {isAgent ? '24/7' : 'Active'}
                  </div>
                  <div className="text-xxs text-text-tertiary uppercase tracking-wider">Uptime</div>
                </div>
                <div className="text-center">
                  <div className="flex items-center justify-center gap-1 text-lg font-black text-success">
                    <Activity className="w-4 h-4" />
                    OK
                  </div>
                  <div className="text-xxs text-text-tertiary uppercase tracking-wider">Status</div>
                </div>
              </div>
            )}

            {/* Agent Capabilities */}
            {isAgent && entity.capabilities && (
              <div className="mt-4">
                <div className="text-xxs font-black text-text-tertiary uppercase tracking-widest mb-2">
                  Capabilities
                </div>
                <div className="flex flex-wrap justify-center gap-2">
                  {entity.capabilities.map((cap, i) => (
                    <span
                      key={i}
                      className="px-3 py-1 text-xs font-bold bg-accent/10 text-accent rounded-full"
                    >
                      {cap}
                    </span>
                  ))}
                </div>
              </div>
            )}

            {/* Actions */}
            <div className="mt-6 flex gap-3">
              {onStartChat && (
                <Button
                  onClick={() => onStartChat(entity.id)}
                  className="flex-1"
                >
                  <MessageSquare className="w-4 h-4" />
                  Message
                </Button>
              )}
              <Button
                variant="secondary"
                onClick={onClose}
                className="flex-1"
              >
                Close
              </Button>
            </div>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
};

export default EntityProfileModal;

