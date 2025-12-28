import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

export type UblConfig = {
  server?: string;
  token?: string;      // bearer token OU SID (ubl:sid:...)
  sid?: string;        // SID do agente (usado no Authorization header)
  s3_alias?: string;        // ex: minio512
  bucket_drafts?: string;   // ex: ubl-drafts
  bucket_official?: string; // ex: ubl-official
};

export const configPath = path.join(os.homedir(), '.ubl', 'config.json');

export function ensureConfigDir(){
  const dir = path.dirname(configPath);
  if(!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true, mode: 0o700 });
}

export function readConfig(): UblConfig {
  try {
    const raw = fs.readFileSync(configPath, 'utf8');
    return JSON.parse(raw);
  } catch {
    return {};
  }
}

export function writeConfig(cfg: UblConfig){
  ensureConfigDir();
  fs.writeFileSync(configPath, JSON.stringify(cfg, null, 2), { mode: 0o600 });
}
