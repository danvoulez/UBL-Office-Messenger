import { Command } from 'commander';
import * as ed from '@noble/ed25519';
import { buildSigningBytes } from '../utils/signing.js';

export function linkCommands(){
  const cmd = new Command('link').description('Gerar assinatura conforme SPEC-UBL-LINK');

  cmd.command('sign')
    .requiredOption('--version <u8>')
    .requiredOption('--container <hex32>')
    .requiredOption('--sequence <u64>')
    .requiredOption('--prev <hex32>')
    .requiredOption('--atom <hex32>')
    .requiredOption('--class <u8>', '0=Observation,1=Conservation,2=Entropy,3=Evolution')
    .requiredOption('--delta <i128>')
    .requiredOption('--priv <hex>', 'chave privada Ed25519')
    .action(async (opts)=>{
      const seq = BigInt(opts.sequence);
      const delta = BigInt(opts.delta);
      const sb = buildSigningBytes({
        version: Number(opts.version),
        containerHex32: opts.container,
        expectedSequence: seq,
        previousHashHex32: opts.prev,
        atomHashHex32: opts.atom,
        intentClass: Number(opts.class),
        physicsDelta: delta,
      });
      const priv = Buffer.from(opts.priv.replace(/^0x/,''), 'hex');
      const sig = await ed.signAsync(sb, priv);
      console.log(JSON.stringify({
        signing_bytes_hex: Buffer.from(sb).toString('hex'),
        signature_hex: Buffer.from(sig).toString('hex')
      }, null, 2));
    });

  return cmd;
}
