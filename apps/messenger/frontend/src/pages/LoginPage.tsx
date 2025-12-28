/**
 * Login Page - WebAuthn Passkey Authentication
 * Beautiful orange theme with UBL branding
 */

import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { motion } from 'framer-motion';
import { Fingerprint, KeyRound, Loader2, ArrowRight, Shield, Zap } from 'lucide-react';
import { Button, Input } from '../components/ui';
import { useAuth } from '../hooks/useAuth';
import toast from 'react-hot-toast';

export const LoginPage: React.FC = () => {
  const navigate = useNavigate();
  const { loginWithPasskey, registerPasskey, isLoading } = useAuth();
  
  const [mode, setMode] = useState<'login' | 'register'>('login');
  const [username, setUsername] = useState('');
  const [displayName, setDisplayName] = useState('');

  const handlePasskeyLogin = async () => {
    if (!username.trim()) {
      toast.error('Enter your username');
      return;
    }

    try {
      await loginWithPasskey(username);
      toast.success('Welcome back!');
      navigate('/');
    } catch (err: any) {
      toast.error(err.message || 'Authentication failed');
    }
  };

  const handlePasskeyRegister = async () => {
    if (!username.trim()) {
      toast.error('Enter a username');
      return;
    }

    try {
      await registerPasskey(username, displayName || username);
      toast.success('Passkey created! You can now login.');
      setMode('login');
    } catch (err: any) {
      toast.error(err.message || 'Registration failed');
    }
  };

  const handleDemoMode = () => {
    localStorage.setItem('ubl_api_base_url', '');
    localStorage.setItem('ubl_demo_mode', 'true');
    navigate('/');
  };

  return (
    <div className="min-h-screen bg-bg-base flex items-center justify-center p-6 relative overflow-hidden">
      {/* Ambient Glow */}
      <div 
        className="absolute inset-0 pointer-events-none"
        style={{
          background: 'radial-gradient(circle at 50% 30%, rgba(224, 122, 95, 0.15), transparent 60%)',
        }}
      />
      
      {/* Background Pattern */}
      <div className="absolute inset-0 opacity-[0.02]">
        <div className="absolute inset-0" style={{
          backgroundImage: `url("data:image/svg+xml,%3Csvg width='60' height='60' viewBox='0 0 60 60' xmlns='http://www.w3.org/2000/svg'%3E%3Cg fill='none' fill-rule='evenodd'%3E%3Cg fill='%23ffffff' fill-opacity='1'%3E%3Cpath d='M36 34v-4h-2v4h-4v2h4v4h2v-4h4v-2h-4zm0-30V0h-2v4h-4v2h4v4h2V6h4V4h-4zM6 34v-4H4v4H0v2h4v4h2v-4h4v-2H6zM6 4V0H4v4H0v2h4v4h2V6h4V4H6z'/%3E%3C/g%3E%3C/g%3E%3C/svg%3E")`,
        }} />
      </div>

      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="w-full max-w-md relative z-10"
      >
        {/* Logo */}
        <div className="text-center mb-10">
          <motion.div 
            className="w-20 h-20 mx-auto mb-6 rounded-3xl bg-gradient-to-br from-accent to-accent-hover flex items-center justify-center shadow-glow-lg"
            whileHover={{ scale: 1.05, rotate: 5 }}
            transition={{ type: 'spring', stiffness: 400 }}
          >
            <Shield className="w-10 h-10 text-text-inverse" />
          </motion.div>
          <h1 className="text-3xl font-black tracking-tight mb-2">
            UBL <span className="text-accent">Messenger</span>
          </h1>
          <p className="text-text-tertiary text-sm">
            Secure messaging on the Universal Business Ledger
          </p>
        </div>

        {/* Card */}
        <div className="bg-bg-elevated border border-border-default rounded-4xl p-8 shadow-2xl">
          {/* Mode Tabs */}
          <div className="flex gap-2 mb-8 p-1 bg-bg-surface rounded-xl">
            <button
              onClick={() => setMode('login')}
              className={`flex-1 py-3 text-xs font-bold uppercase tracking-wider rounded-lg transition-all ${
                mode === 'login'
                  ? 'bg-accent text-text-inverse shadow-glow-sm'
                  : 'text-text-tertiary hover:text-text-primary'
              }`}
            >
              Sign In
            </button>
            <button
              onClick={() => setMode('register')}
              className={`flex-1 py-3 text-xs font-bold uppercase tracking-wider rounded-lg transition-all ${
                mode === 'register'
                  ? 'bg-accent text-text-inverse shadow-glow-sm'
                  : 'text-text-tertiary hover:text-text-primary'
              }`}
            >
              Register
            </button>
          </div>

          {/* Form */}
          <div className="space-y-5">
            <Input
              label="Username"
              placeholder="Enter your username"
              value={username}
              onChange={e => setUsername(e.target.value)}
              icon={<KeyRound className="w-4 h-4" />}
              autoComplete="username webauthn"
            />

            {mode === 'register' && (
              <motion.div
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: 'auto' }}
                exit={{ opacity: 0, height: 0 }}
              >
                <Input
                  label="Display Name"
                  placeholder="How should we call you?"
                  value={displayName}
                  onChange={e => setDisplayName(e.target.value)}
                />
              </motion.div>
            )}

            {/* Passkey Button */}
            <Button
              onClick={mode === 'login' ? handlePasskeyLogin : handlePasskeyRegister}
              disabled={isLoading || !username.trim()}
              loading={isLoading}
              className="w-full py-4 text-sm"
              size="lg"
            >
              <Fingerprint className="w-5 h-5" />
              {mode === 'login' ? 'Sign in with Passkey' : 'Create Passkey'}
            </Button>

            {/* Features */}
            <div className="grid grid-cols-2 gap-3 pt-4">
              <div className="flex items-center gap-2 text-text-tertiary text-xs">
                <Shield className="w-4 h-4 text-success" />
                <span>No passwords</span>
              </div>
              <div className="flex items-center gap-2 text-text-tertiary text-xs">
                <Zap className="w-4 h-4 text-warning" />
                <span>Instant login</span>
              </div>
            </div>
          </div>

          {/* Divider */}
          <div className="flex items-center gap-4 my-8">
            <div className="flex-1 h-px bg-border-subtle" />
            <span className="text-xxs text-text-tertiary uppercase tracking-widest">or</span>
            <div className="flex-1 h-px bg-border-subtle" />
          </div>

          {/* Demo Mode */}
          <button
            onClick={handleDemoMode}
            className="w-full flex items-center justify-center gap-2 py-3 text-text-tertiary hover:text-text-primary text-xs font-medium transition-colors"
          >
            <span>Try Demo Mode</span>
            <ArrowRight className="w-4 h-4" />
          </button>
        </div>

        {/* Footer */}
        <p className="text-center text-xxs text-text-tertiary mt-8">
          By continuing, you agree to UBL's Terms of Service
        </p>
      </motion.div>
    </div>
  );
};

export default LoginPage;

