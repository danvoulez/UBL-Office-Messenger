import { Command } from 'commander';
import { readConfig } from '../utils/config.js';

export function tailCommand(){
  const cmd = new Command('tail').description('Seguir o ledger via SSE (/tail)');
  cmd.option('--jsonl <file>', 'salvar eventos como NDJSON');
  cmd.option('--since-seq <u64>', 'cursor inicial (sequence)');
  cmd.option('--container <hex32>', 'filtrar por container_id');
  cmd.action(async (opts)=>{
    const cfg = readConfig();
    if(!cfg.server) throw new Error('config.server ausente. Rode: ubl config set server http://host:8080');
    let url = cfg.server.replace(/\/$/,'') + '/tail';
    const qs: string[] = [];
    if (opts['since-seq']) qs.push('since_seq=' + encodeURIComponent(String(opts['since-seq'])));
    if (qs.length) url += '?' + qs.join('&');
    const headers: Record<string,string> = { accept: 'text/event-stream' };
    if(cfg.token) headers['authorization'] = `Bearer ${cfg.token}`;

    const res = await fetch(url, { headers });
    if(!res.ok) throw new Error(`HTTP ${res.status}`);
    if(!res.body) throw new Error('no body');
    const reader = res.body.getReader();
    const dec = new TextDecoder();
    let buf = '';
    console.error('Conectado:', url);
    const enc = new TextEncoder();
let outFd: any = null;
if (opts.jsonl) { outFd = await (await import('node:fs/promises')).open(opts.jsonl, 'a'); }
while(true){
      const { value, done } = await reader.read();
      if(done) break;
      buf += dec.decode(value, { stream: true });
      let idx;
      while((idx = buf.indexOf('\n\n')) >= 0){
        const chunk = buf.slice(0, idx);
        buf = buf.slice(idx+2);
        for (const line of chunk.split('\n')){
          if(line.startsWith('data:')){
            const data = line.slice(5).trim();
            if(!data) continue;
            try {
              const obj = JSON.parse(data);
              if (opts.container && obj.container_id && obj.container_id !== opts.container) continue;
              if (opts.pretty && obj) {
                const seq = obj.sequence ?? obj.seq ?? '?';
                const cid = (obj.container_id ?? '').slice(0,8);
                const cls = obj.intent_class ?? obj.class ?? obj.link?.intent_class;
                const dlt = obj.physics_delta ?? obj.delta ?? obj.link?.physics_delta;
                const atom = (obj.atom_hash ?? obj.link?.atom_hash ?? '').slice(0,8);
                console.log(`[${seq}] C${cls}@Δ${dlt} atom=${atom} cid=${cid}`);
              } else {
                console.log(data);
              }
            } catch {
              console.log(data);
            }
            if(outFd){ await outFd.write(data + '\n'); }

          }
        }
      }
    }
  });
  return cmd;
}
