import { Command } from 'commander';
import fs from 'node:fs';
import * as ed from '@noble/ed25519';
import { buildSigningBytes } from '../utils/signing.js';

export function commitVerifyCommand(){
  // --strict adds shape & class/delta checks
  const cmd = new Command('verify').description('Verificar assinatura e signing_bytes de um link');
  cmd.requiredOption('--file <link.json>');
  cmd.option('--strict', 'valida hex/len e coerência classe/delta')
  .option('--pretty', 'saida humana colorida')
  .action(async (opts)=>{
    const raw = JSON.parse(fs.readFileSync(opts.file, 'utf8'));
    const strict = !!opts.strict;
    const link = raw.link ?? raw; // aceitar envelope ou link direto
    let strictErrors: string[] = [];
    if (strict) {
      const v = await import('../utils/validate.js');
      strictErrors = v.basicLinkShape(link);
      const delta = BigInt(String(link.physics_delta));
      const cd = v.checkClassDelta(Number(link.intent_class), delta);
      if (cd) strictErrors.push(cd);
    }
    const sb = buildSigningBytes({
      version: Number(link.version),
      containerHex32: link.container_id,
      expectedSequence: BigInt(link.expected_sequence),
      previousHashHex32: link.previous_hash,
      atomHashHex32: link.atom_hash,
      intentClass: Number(link.intent_class),
      physicsDelta: BigInt(link.physics_delta),
    });
    const expectedSbHex = Buffer.from(sb).toString('hex');
    const providedSbHex = raw.signing_bytes_hex || '(ausente)';
    const sig = Buffer.from(link.signature.replace(/^0x/, ''), 'hex');
    const pub = Buffer.from(link.author_pubkey.replace(/^0x/, ''), 'hex');
    const ok = await ed.verifyAsync(sig, sb, pub);
    if (opts.pretty){
      const { colors } = await import('../utils/colors.js');
      const okSb  = (raw.signing_bytes_hex ? (raw.signing_bytes_hex === expectedSbHex) : null);
      const okSig = ok;
      const strictOk = strict ? (strictErrors.length === 0) : null;
      function status(b:boolean|null, label:string){
        if (b === null) return `${colors.yellow('~')} ${label}: n/d`;
        return b ? `${colors.green('✔')} ${label}: ok` : `${colors.red('✖')} ${label}: falhou`;
      }
      console.log(colors.bold('UBL commit verify'));
      console.log('  ' + status(strictOk, 'strict'));
      console.log('  ' + status(okSb, 'signing_bytes_match'));
      console.log('  ' + status(okSig, 'signature_valid'));
      if (strict && strictErrors.length){
        console.log(colors.red('  detalhes strict:'));
        for (const e of strictErrors) console.log('   - ' + e);
      }
      console.log(colors.cyan('  signing_bytes_hex: ') + expectedSbHex);
      return;
    }
    console.log(JSON.stringify({
      strict_errors: strict ? strictErrors : undefined,
      signing_bytes_match: (raw.signing_bytes_hex ? (raw.signing_bytes_hex === expectedSbHex) : null),
      signing_bytes_hex: expectedSbHex,
      signature_valid: ok
    }, null, 2));
    if(!ok) process.exitCode = 2;
  });
  return cmd;
}
