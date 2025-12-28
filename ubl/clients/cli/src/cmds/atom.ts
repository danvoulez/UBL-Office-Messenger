import fs from 'node:fs';
import { Command } from 'commander';
import { canonicalize } from '../utils/canon.js';
import { blake3hex } from '../utils/hash.js';

export function atomCommands(){
  const cmd = new Command('atom').description('JSON✯Atomic — canonicalize + hash');

  cmd.command('canonicalize')
    .argument('<file>', 'arquivo JSON de entrada')
    .action((file)=>{
      const input = JSON.parse(fs.readFileSync(file,'utf8'));
      const out = canonicalize(input);
      console.log(out);
    });

  cmd.command('hash')
    .argument('<file>')
    .action((file)=>{
      const input = JSON.parse(fs.readFileSync(file,'utf8'));
      const canon = canonicalize(input);
      const h = blake3hex(Buffer.from(canon, 'utf8'));
      console.log(h);
    });

  return cmd;
}
