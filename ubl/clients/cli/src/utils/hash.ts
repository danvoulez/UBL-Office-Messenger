import { blake3 } from '@noble/hashes/blake3';

export function blake3hex(bytes: Uint8Array): string {
  const h = blake3(bytes);
  return Buffer.from(h).toString('hex');
}
