import { Command } from 'commander';
import * as ed from '@noble/ed25519';
import fs from 'node:fs';
import { buildSigningBytes } from '../utils/signing.js';
import { http } from '../utils/http.js';

export function commitCommands(){
  // includes make/send/verify
  const cmd = new Command('commit').description('Criar e enviar commits (ubl-link)');

  cmd.command('make')
    .requiredOption('--ver <u8>', 'version do protocolo')
    .requiredOption('--container <hex32>')
    .requiredOption('--sequence <u64>')
    .requiredOption('--prev <hex32>')
    .requiredOption('--atom <hex32>')
    .requiredOption('--class <u8>', '0=Observation,1=Conservation,2=Entropy,3=Evolution')
    .requiredOption('--delta <i128>')
    .requiredOption('--priv <hex>', 'chave privada Ed25519')
    .option('--pact-file <json>', 'incluir PactProof no envelope (não entra nos signing_bytes)')
    .option('--strict', 'bloqueia envelope inválido (hex32/64, classe↔delta)')
    .option('--out <file>', 'salvar em arquivo (opcional)')
    .action(async (opts)=>{
            // STRICT validation (shape + class/delta)
      if (opts.strict){
        const v = await import('../utils/validate.js');
        const errs = v.basicLinkShape({
          version: Number(opts.ver),
          container_id: opts.container,
          expected_sequence: String(opts.sequence),
          previous_hash: opts.prev,
          atom_hash: opts.atom,
          intent_class: Number(opts.class),
          physics_delta: String(opts.delta),
          author_pubkey: '0'.repeat(64), // placeholder to satisfy shape
          signature: '0'.repeat(128) // placeholder
        });
        const cd = v.checkClassDelta(Number(opts.class), BigInt(String(opts.delta)));
        if (cd) errs.push(cd);
        if (errs.length){
          console.error(JSON.stringify({ error: 'strict_validation_failed', details: errs }, null, 2));
          process.exit(2);
        }
      }
      const sb = buildSigningBytes({
        version: Number(opts.ver),
        containerHex32: opts.container,
        expectedSequence: BigInt(opts.sequence),
        previousHashHex32: opts.prev === '0x00' ? '0'.repeat(64) : opts.prev,
        atomHashHex32: opts.atom,
        intentClass: Number(opts.class),
        physicsDelta: BigInt(opts.delta),
      });
      const priv = Buffer.from(opts.priv.replace(/^0x/,''), 'hex');
      const sig = await ed.signAsync(sb, priv);
      const pub = Buffer.from(await ed.getPublicKeyAsync(priv)).toString('hex');
      let pact = null as any;
      if (opts['pact-file']) {
        pact = JSON.parse(require('node:fs').readFileSync(String(opts['pact-file']), 'utf8'));
      }
      
      // Normalize genesis previousHash
      const normalizedPrev = opts.prev === '0x00' ? '0'.repeat(64) : opts.prev;
      
      // Map intent_class number to string
      const classNames = ['Observation', 'Conservation', 'Entropy', 'Evolution'];
      const intentClassName = classNames[Number(opts.class)] || 'Observation';
      
      const link = {
        version: Number(opts.ver),
        container_id: opts.container,
        expected_sequence: Number(opts.sequence),
        previous_hash: normalizedPrev,
        atom_hash: opts.atom,
        intent_class: intentClassName,
        physics_delta: String(opts.delta), // i128 as string
        pact,
        author_pubkey: pub,
        signature: Buffer.from(sig).toString('hex')
      };
      const out = JSON.stringify({ link, signing_bytes_hex: Buffer.from(sb).toString('hex') }, null, 2);
      if(opts.out){ fs.writeFileSync(opts.out, out); console.log(`salvo: ${opts.out}`); }
      else console.log(out);
    });

  cmd.command('send')
    .requiredOption('--file <link.json>', 'arquivo gerado por commit make')
    .action(async (opts)=>{
      const raw = JSON.parse(fs.readFileSync(opts.file, 'utf8'));
      const res = await http('POST', '/link/commit', raw.link);
      console.log(JSON.stringify(res, null, 2));
    });

  return cmd;
}
