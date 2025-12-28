import { createHash } from 'node:crypto';
import { promises as fs } from 'node:fs';
import { join } from 'node:path';

export async function sha256File(path: string): Promise<string> {
  const buf = await fs.readFile(path);
  const h = createHash('sha256'); h.update(buf);
  return h.digest('hex');
}

export async function listFilesRecursive(dir: string): Promise<string[]> {
  const res: string[] = [];
  async function walk(d: string){
    const items = await fs.readdir(d, { withFileTypes: true });
    for (const it of items){
      const full = join(d, it.name);
      if (it.isDirectory()) await walk(full);
      else if (it.isFile()) res.push(full);
    }
  }
  await walk(dir);
  return res;
}
