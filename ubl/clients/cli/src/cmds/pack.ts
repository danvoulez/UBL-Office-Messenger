import { Command } from 'commander';
import path from 'node:path';
import { promises as fs } from 'node:fs';
import { listFilesRecursive, sha256File } from '../utils/filehash.js';

export function packCommand(){
  const cmd = new Command('pack').description('Gera manifesto de release (arquivos, bytes, sha256)');
  cmd.requiredOption('--project <name>');
  cmd.requiredOption('--component <name>');
  cmd.requiredOption('--version <semver>');
  cmd.requiredOption('--dist <dir>');
  cmd.requiredOption('--out <file>');
  cmd.action(async (opts)=>{
    const files = await listFilesRecursive(opts.dist);
    const items: any[] = [];
    let total = 0;
    for (const f of files){
      const st = await fs.stat(f);
      const rel = path.relative(opts.dist, f);
      const sha = await sha256File(f);
      total += st.size;
      items.push({ path: rel, bytes: st.size, sha256: sha });
    }
    const manifest = {
      spec: 'UBL-RELEASE-MANIFEST/1.0',
      project: opts.project,
      component: opts.component,
      version: opts.version,
      created_at: new Date().toISOString(),
      totals: { count: items.length, bytes: total },
      files: items
    };
    await fs.writeFile(opts.out, JSON.stringify(manifest, null, 2));
    console.log(`ok: ${opts.out} (${items.length} arquivos, ${total} bytes)`);
  });
  return cmd;
}
