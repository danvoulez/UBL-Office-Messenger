import { Command } from 'commander';
import fs from 'node:fs';
import path from 'node:path';
import * as ed from '@noble/ed25519';
import { canonicalize } from '../utils/canon.js';
import { blake3hex } from '../utils/hash.js';
import { buildSigningBytes } from '../utils/signing.js';
import { http } from '../utils/http.js';
import { readConfig } from '../utils/config.js';
import { spawn } from 'node:child_process';

export function publishCommand(){
  const cmd = new Command('publish').description('Publicar um release: commit Observation + upload MinIO official');

  cmd
   .requiredOption('--project <name>')
   .requiredOption('--component <name>')
   .requiredOption('--version <semver>')
   .requiredOption('--manifest <file>', 'JSON do release (será canonicalizado)')
   .requiredOption('--dist <dir>', 'pasta com artefatos já buildados')
   .requiredOption('--container <hex32>')
   .requiredOption('--sequence <u64>')
   .requiredOption('--prev <hex32>')
   .requiredOption('--priv <hex>', 'chave privada Ed25519 para assinar o link')
   .option('--class <u8>', 'classe física (default Observation=0)', '0')
   .option('--delta <i128>', 'delta físico (default 0)', '0')
   .option('--verify', 'verifica assinatura local antes de enviar')
   .option('--dry-run', 'não comita nem sobe S3; apenas imprime plano')
   .option('--plan', 'imprime plano detalhado de publicação (arquivos, tamanhos, hashes)')
   .action(async (opts)=>{
     const cfg = readConfig();
     if(!cfg.server) throw new Error('config.server ausente.');
     if(!cfg.s3_alias) throw new Error('config.s3_alias ausente. Rode: ubl config set s3_alias minio512');
     if(!cfg.bucket_official) throw new Error('config.bucket_official ausente. Rode: ubl config set bucket_official ubl-official');

     // 1) canonicalizar manifesto e hashear
     const manifestRaw = JSON.parse(fs.readFileSync(opts.manifest, 'utf8'));
     const canon = canonicalize(manifestRaw);
     const atomHash = blake3hex(Buffer.from(canon, 'utf8'));

     // 2) montar e assinar link (Observation por padrão)
     const sb = buildSigningBytes({
       version: 1,
       containerHex32: opts.container,
       expectedSequence: BigInt(opts.sequence),
       previousHashHex32: opts.prev,
       atomHashHex32: atomHash,
       intentClass: Number(opts.class),
       physicsDelta: BigInt(opts.delta),
     });
     const priv = Buffer.from(opts.priv.replace(/^0x/, ''), 'hex');
     const sig = await ed.signAsync(sb, priv);
     const pub = Buffer.from(await ed.getPublicKeyAsync(priv)).toString('hex');
     const link = {
       version: 1,
       container_id: opts.container,
       expected_sequence: String(opts.sequence),
       previous_hash: opts.prev,
       atom_hash: atomHash,
       intent_class: Number(opts.class),
       physics_delta: String(opts.delta),
       pact: null,
       author_pubkey: pub,
       signature: Buffer.from(sig).toString('hex')
     };

     // 3) commit
     if (opts['verify']) {
       const ok = await ed.verifyAsync(sig, sb, Buffer.from(pub, 'hex'));
       if (!ok) throw new Error('Assinatura inválida no publish --verify');
       console.log('VERIFY OK: assinatura válida');
     }
     if (!opts['dry-run']) { const commitRes = await http('POST', '/commit', link); console.log('Commit OK:', JSON.stringify(commitRes)); } else { console.log('DRY-RUN: pular /commit'); }

     // 4) upload para MinIO official
     const dst = `${cfg.s3_alias}/${cfg.bucket_official}/${opts.project}/${opts.component}/${opts.version}/`;
     console.log('Upload →', dst);
     // Build plan (file list + sizes + sha256)
     if (opts['plan']) {
       const { listFilesRecursive, sha256File } = await import('../utils/filehash.js');
       const { stat } = await import('node:fs/promises');
       const files = await listFilesRecursive(opts.dist);
       const items: any[] = [];
       for (const f of files){
         const s = await stat(f);
         const h = await sha256File(f);
         items.push({ path: f, bytes: s.size, sha256: h });
       }
       const plan = {
         destination: dst,
         manifest: opts.manifest,
         manifest_hash: atomHash,
         count: items.length,
         total_bytes: items.reduce((a,b)=>a+b.bytes,0),
         files: items
       };
       console.log(JSON.stringify({ plan }, null, 2));
     }

     if (opts['dry-run']) { console.log('DRY-RUN: pular upload'); }
     else await new Promise((resolve, reject)=>{
       const p1 = spawn('mc', ['cp', '--recursive', opts.dist, dst], { stdio: 'inherit' });
       p1.on('exit', (code)=> code===0 ? resolve(0) : reject(new Error('mc cp dist failed')));
     }).catch(()=>{ throw new Error('Falha ao subir dist/'); });

     await new Promise((resolve, reject)=>{
       const p2 = spawn('mc', ['cp', opts.manifest, dst], { stdio: 'inherit' });
       p2.on('exit', (code)=> code===0 ? resolve(0) : reject(new Error('mc cp manifest failed')));
     }).catch(()=>{ throw new Error('Falha ao subir manifest'); });

     console.log('Publicação concluída.');
   });

  return cmd;
}
