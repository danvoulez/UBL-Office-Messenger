import { Command } from 'commander';
import { spawn } from 'node:child_process';

export function s3Commands(){
  const cmd = new Command('s3').description('Atalhos para MinIO/mc');

  cmd.command('draft-push')
    .requiredOption('--src <dir>')
    .requiredOption('--dst <s3uri>', 'ex: minio512/ubl-drafts/proj/comp/sha/')
    .action((opts)=>{
      const p = spawn('mc', ['cp','--recursive', opts.src, opts.dst], { stdio: 'inherit' });
      p.on('exit', (code)=> process.exit(code ?? 1));
    });

  return cmd;
}
