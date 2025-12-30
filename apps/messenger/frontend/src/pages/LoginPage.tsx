/**
 * Login Page - Simplified WebAuthn Passkey Authentication
 * Clean UBL Messenger branding
 */

import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { motion } from 'framer-motion';
import { Fingerprint, Loader2, Send } from 'lucide-react';
import { useAuth } from '../hooks/useAuth';
import toast from 'react-hot-toast';

export const LoginPage: React.FC = () => {
  const navigate = useNavigate();
  const { loginWithPasskey, registerPasskey, isLoading } = useAuth();
  
  const [email, setEmail] = useState('');
  const [isRegistering, setIsRegistering] = useState(false);

  // Login: usuário que já tem passkey
  const handlePasskeyLogin = async () => {
    try {
      // WebAuthn permite login sem username - o browser mostra as passkeys disponíveis
      await loginWithPasskey('');
      toast.success('Welcome back!');
      navigate('/');
    } catch (err: any) {
      toast.error(err.message || 'Authentication failed');
    }
  };

  // Register: novo usuário digita email, cria passkey
  const handleRegister = async () => {
    if (!email.trim() || !email.includes('@')) {
      toast.error('Enter a valid email');
      return;
    }

    setIsRegistering(true);
    try {
      // Usa email como username e display name
      await registerPasskey(email, email.split('@')[0]);
      toast.success('Passkey created! Logging in...');
      // Após registro, faz login automático
      await loginWithPasskey(email);
      navigate('/');
    } catch (err: any) {
      toast.error(err.message || 'Registration failed');
    } finally {
      setIsRegistering(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && email.trim()) {
      handleRegister();
    }
  };

  return (
    <div className="min-h-screen bg-bg-base flex items-center justify-center p-6">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="w-full max-w-sm relative z-10 flex flex-col items-center"
      >
        {/* Logo */}
        <motion.img 
          src="/images/ubl-messenger-logo.png"
          alt="UBL Messenger"
          className="w-72 h-auto mb-12"
          whileHover={{ scale: 1.02 }}
          transition={{ type: 'spring', stiffness: 400 }}
        />

        {/* Sign In Button */}
        <button
          onClick={handlePasskeyLogin}
          disabled={isLoading && !isRegistering}
          className="w-full py-4 px-6 bg-accent hover:bg-accent-hover disabled:bg-accent/50 text-white font-bold text-sm uppercase tracking-wider rounded-2xl transition-all shadow-glow flex items-center justify-center gap-3"
        >
          {isLoading && !isRegistering ? (
            <Loader2 className="w-5 h-5 animate-spin" />
          ) : (
            <Fingerprint className="w-5 h-5" />
          )}
          Sign in with Passkey
        </button>

        {/* Or Register */}
        <p className="text-text-primary text-sm font-medium my-6">
          Or Register →
        </p>

        {/* Email Input + Send Button */}
        <div className="w-full flex gap-2">
          <input
            type="email"
            value={email}
            onChange={e => setEmail(e.target.value)}
            onKeyDown={handleKeyPress}
            placeholder="your@email.com"
            className="flex-1 px-4 py-3 bg-bg-elevated border border-border-default rounded-xl text-text-primary placeholder-text-tertiary outline-none focus:border-accent focus:ring-2 focus:ring-accent/20 transition-all text-sm"
            autoComplete="email"
          />
          <button
            onClick={handleRegister}
            disabled={isRegistering || !email.trim()}
            className="px-5 py-3 bg-accent hover:bg-accent-hover disabled:bg-accent/50 text-white rounded-xl transition-all flex items-center justify-center"
          >
            {isRegistering ? (
              <Loader2 className="w-5 h-5 animate-spin" />
            ) : (
              <Send className="w-5 h-5" />
            )}
          </button>
        </div>

        {/* Subtle footer */}
        <p className="text-text-tertiary text-[10px] mt-8 text-center">
          Secure authentication powered by WebAuthn
        </p>
      </motion.div>
    </div>
  );
};

export default LoginPage;

