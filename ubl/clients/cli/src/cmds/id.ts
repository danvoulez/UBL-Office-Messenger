import { Command } from 'commander';
import axios from 'axios';
import * as ed from '@noble/ed25519';
import { readConfig } from '../utils/config.js';

export function idCommands(){
  // extended subcommands added below
  const cmd = new Command('id').description('Identidade: pessoas (passkey), LLM e apps');

  cmd.command('agent:create')
    .requiredOption('--kind <kind>', 'llm|app')
    .requiredOption('--name <display_name>')
    .option('--pub <hex>', 'public key Ed25519; se ausente, gera par e imprime')
    .action(async (opts)=>{
      const cfg = readConfig();
      if(!cfg.server) throw new Error('config.server ausente. Rode: ubl config set server http://host:8080');
      let pub = opts.pub;
      let prvHex: string|undefined;
      if(!pub){
        const prv = ed.utils.randomPrivateKey();
        const pubb = await ed.getPublicKeyAsync(prv);
        prvHex = Buffer.from(prv).toString('hex');
        pub = Buffer.from(pubb).toString('hex');
      }
      const res = await axios.post(`${cfg.server}/id/agents`, {
        kind: opts.kind, display_name: opts.name, public_key: pub
      }, { headers: cfg.token ? { Authorization: `Bearer ${cfg.token}` } : undefined });
      console.log(JSON.stringify({ ...res.data, private_key: prvHex }, null, 2));
    });

  cmd.command('whoami')
    .action(async ()=>{
      const cfg = readConfig();
      if(!cfg.server) throw new Error('config.server ausente.');
      const res = await axios.get(`${cfg.server}/id/whoami`, { headers: cfg.token ? { Authorization: `Bearer ${cfg.token}` } : undefined });
      console.log(JSON.stringify(res.data, null, 2));
    });

  // ---- ASC Issue ----
  cmd.command('asc:issue')
    .requiredOption('--sid <sid>')
    .option('--containers <list>', 'c1,c2,...')
    .option('--classes <list>', 'ex: 0,1,2')
    .option('--max-delta <i128>')
    .option('--ttl <sec>', 'segundos')
    .option('--from-file <json>', 'arquivo com {containers, intent_classes, max_delta, ttl_seconds}')
    .action(async (opts)=>{
      const { http } = await import('../utils/http.js');
      let body;
      if (opts['from-file']) {
        const fs = await import('node:fs/promises');
        body = JSON.parse(await fs.readFile(opts['from-file'], 'utf8'));
      } else {
        body = {
          containers: String(opts.containers).split(',').map((s:string)=>s.trim()).filter(Boolean),
          intent_classes: String(opts.classes).split(',').map((s:string)=>s.trim()).filter(Boolean),
          max_delta: opts['max-delta'] ? Number(opts['max-delta']) : null,
          ttl_secs: Number(opts.ttl),
        };
      }
      const res = await http('POST', `/id/agents/${encodeURIComponent(opts.sid)}/asc`, body);
      console.log(JSON.stringify(res, null, 2));
    });

  // ---- Rotate Key ----
  cmd.command('rotate')
    .requiredOption('--sid <sid>')
    .requiredOption('--pub <hex>', 'nova chave pública ed25519')
    .action(async (opts)=>{
      const { http } = await import('../utils/http.js');
      const res = await http('POST', `/id/agents/${encodeURIComponent(opts.sid)}/rotate`, { new_public_key: opts.pub });
      console.log(JSON.stringify(res, null, 2));
    });

  // ---- ICTE (begin/finish) ----
  cmd.command('ict:begin')
    .requiredOption('--scope <container>')
    .requiredOption('--ttl <sec>')
    .action(async (opts)=>{
      const { http } = await import('../utils/http.js');
      const res = await http('POST', '/id/sessions/ict/begin', { scope: opts.scope, ttl_seconds: Number(opts.ttl) });
      console.log(JSON.stringify(res, null, 2));
    });

  cmd.command('ict:finish')
    .requiredOption('--token <ict_id>')
    .action(async (opts)=>{
      const { http } = await import('../utils/http.js');
      const res = await http('POST', '/id/sessions/ict/finish', { ict_id: opts.token });
      console.log(JSON.stringify(res, null, 2));
    });

  // ---- ASC List ----
  cmd.command('asc:list')
    .requiredOption('--sid <sid>')
    .action(async (opts)=>{
      const { http } = await import('../utils/http.js');
      const res = await http('GET', `/id/agents/${encodeURIComponent(opts.sid)}/asc`);
      console.log(JSON.stringify(res, null, 2));
    });

  // ---- ASC Revoke ----
  cmd.command('asc:revoke')
    .requiredOption('--sid <sid>')
    .requiredOption('--asc <asc_id>')
    .action(async (opts)=>{
      const { http } = await import('../utils/http.js');
      const res = await http('DELETE', `/id/agents/${encodeURIComponent(opts.sid)}/asc/${encodeURIComponent(opts.asc)}`);
      console.log(JSON.stringify(res, null, 2));
    });

  // ---- Export identity (backup) ----
  cmd.command('export')
    .requiredOption('--sid <sid>')
    .option('--out <file>', 'salvar em arquivo')
    .option('--redact', 'ofuscar campos sensíveis')
    .action(async (opts)=>{
      const { http } = await import('../utils/http.js');
      const res = await http('GET', `/id/agents/${encodeURIComponent(opts.sid)}`);
      const out = JSON.stringify(res, null, 2);
      if (opts.out) { 
        await (await import('node:fs/promises')).writeFile(opts.out, out); 
        console.log(`salvo: ${opts.out}`); 
      } else { 
        console.log(out); 
      }
    });

  return cmd;
}
