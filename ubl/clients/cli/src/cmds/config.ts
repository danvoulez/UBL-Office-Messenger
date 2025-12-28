import { Command } from 'commander';
import { readConfig, writeConfig, UblConfig } from '../utils/config.js';

export function configCommands(){
  const cmd = new Command('config').description('Gerenciar config local (~/.ubl/config.json)');

  cmd.command('init')
    .description('Criar arquivo de config')
    .option('--server <url>', 'ex: http://10.88.0.2:8080')
    .option('--token <token>', 'Bearer (opcional)')
    .action((opts)=>{
      const cfg: UblConfig = { server: opts.server, token: opts.token };
      writeConfig(cfg);
      console.log('OK:', cfg);
    });

  cmd.command('get')
    .argument('[key]', 'chave específica')
    .action((key)=>{
      const cfg = readConfig();
      console.log(key ? (cfg as any)[key] : cfg);
    });

  cmd.command('set')
    .argument('<key>')
    .argument('<value>')
    .action((key, value)=>{
      const cfg = readConfig();
      (cfg as any)[key] = value;
      writeConfig(cfg);
      console.log('OK');
    });

  return cmd;
}
