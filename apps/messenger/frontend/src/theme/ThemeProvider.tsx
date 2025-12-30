import React, { createContext, useContext, useEffect, useMemo, useState } from 'react';

type Role = 'user' | 'admin';

type ThemeCtx = {
  role: Role;
  setRole: (r: Role) => void;
};

const ThemeContext = createContext<ThemeCtx | null>(null);

function setHtmlRole(role: Role) {
  const html = document.documentElement;
  html.setAttribute('data-role', role === 'admin' ? 'admin' : 'user');
}

function setMetaThemeColor(role: Role) {
  const color = role === 'admin' ? '#1479ff' : '#ff6a2b';
  let el = document.querySelector('meta[name="theme-color"]') as HTMLMetaElement | null;
  if (!el) {
    el = document.createElement('meta');
    el.name = 'theme-color';
    document.head.appendChild(el);
  }
  el.content = color;
}

export const ThemeProvider: React.FC<React.PropsWithChildren<{ initialRole?: Role }>> = ({
  children,
  initialRole = 'user',
}) => {
  const [role, setRole] = useState<Role>(() => {
    const fromStorage = (localStorage.getItem('ubl.role') as Role | null) || initialRole;
    return fromStorage === 'admin' ? 'admin' : 'user';
  });

  useEffect(() => {
    localStorage.setItem('ubl.role', role);
    setHtmlRole(role);
    setMetaThemeColor(role);
  }, [role]);

  const value = useMemo(() => ({ role, setRole }), [role]);
  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
};

export function useTheme() {
  const ctx = useContext(ThemeContext);
  if (!ctx) throw new Error('useTheme must be used within ThemeProvider');
  return ctx;
}

/**
 * Convenience helper: call this with your bootstrap payload to flip the theme on the fly.
 * It expects something like { user: { role: 'admin' | 'user' } }.
 */
export function applyBootstrapRole(bootstrap: any, setter: (role: Role) => void) {
  try {
    const role: Role = (bootstrap?.user?.role === 'admin' ? 'admin' : 'user') as Role;
    setter(role);
  } catch {
    // ignore and keep current role
  }
}
