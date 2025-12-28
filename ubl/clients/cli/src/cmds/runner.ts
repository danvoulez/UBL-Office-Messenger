import { Command } from 'commander';
import { readConfig } from '../utils/config.js';
import { http } from '../utils/http.js';

function parseCsv(str?: string): string[] | undefined {
  if (!str) return undefined;
  return String(str).split(',').map(s => s.trim()).filter(Boolean);
}
function parseEnv(str?: string): Record<string,string> | undefined {
  if (!str) return undefined;
  const out: Record<string,string> = {};
  for (const pair of String(str).split(',')){
    const [k, v] = pair.split('=');
    if (k && v !== undefined) out[k.trim()] = v.trim();
  }
  return out;
}
function rid(): string {
  // simple random hex id
  const bytes = crypto.getRandomValues(new Uint8Array(16));
  return Buffer.from(bytes).toString('hex');
}

export function runnerCommand(){
  const cmd = new Command('runner').description('Ajuda a despachar execuções isoladas');

  cmd.command('exec')
    .requiredOption('--container <hex32>')
    .option('--type <kind>', 'cmd|image', 'cmd')
    .option('--cmd <string>', 'comando quando type=cmd')
    .option('--args <a,b,c>', 'lista de argumentos')
    .option('--env <K=V,...>', 'variáveis de ambiente')
    .option('--egress <host1,host2>', 'whitelist de egress')
    .option('--max-sec <n>', 'limite de segundos', '60')
    .option('--max-mem <mb>', 'limite de memória em MB', '256')
    .option('--tmpfs-mb <mb>', 'tamanho do tmpfs em MB', '128')
    .option('--dry-run', 'não envia; apenas imprime o payload')
    .action(async (opts)=>{
      const cfg = readConfig();
      if(!cfg.server) throw new Error('config.server ausente');
      const payload: any = {
        request_id: rid(),
        container_id: opts.container,
        job: {
          type: String(opts.type || 'cmd'),
          cmd: opts.cmd,
          args: parseCsv(opts.args),
          env: parseEnv(opts.env),
          limits: {
            seconds: Number(opts['max-sec'] || 60),
            mem_mb: Number(opts['max-mem'] || 256),
            tmpfs_mb: Number(opts['tmpfs-mb'] || 128)
          },
          egress_whitelist: parseCsv(opts.egress) || []
        }
      };
      if (opts['dry-run']) {
        console.log(JSON.stringify({ plan: payload }, null, 2));
        return;
      }
      const res = await http('POST', '/runner/exec', payload);
      console.log(JSON.stringify(res, null, 2));
    });

  return cmd;
}
