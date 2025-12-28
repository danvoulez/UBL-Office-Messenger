import { readConfig } from './config.js';

export async function http(method: 'GET'|'POST'|'PUT'|'DELETE', path: string, body?: any){
  const cfg = readConfig();
  if(!cfg.server) throw new Error('config.server ausente. Rode: ubl config set server http://host:8080');
  const url = cfg.server.replace(/\/$/,'') + path;
  const headers: Record<string,string> = { 'content-type': 'application/json' };
  
  // Priorizar SID, depois token
  if(cfg.sid) {
    headers['authorization'] = `Bearer ${cfg.sid}`;
  } else if(cfg.token) {
    headers['authorization'] = `Bearer ${cfg.token}`;
  }
  
  const res = await fetch(url, {
    method, headers,
    body: body !== undefined ? JSON.stringify(body) : undefined
  });
  if(!res.ok){
    const txt = await res.text().catch(()=>'');
    throw new Error(`HTTP ${res.status}: ${txt}`);
  }
  const ct = res.headers.get('content-type') || '';
  if (ct.includes('application/json')) return res.json();
  return res.text();
}
