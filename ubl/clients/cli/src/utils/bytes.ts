export function hexToBytes(hex: string, expectedLen?: number): Uint8Array {
  const clean = hex.startsWith('0x') ? hex.slice(2) : hex;
  if (expectedLen !== undefined && clean.length != expectedLen*2) {
    throw new Error(`hex length ${clean.length/2} != ${expectedLen}`);
  }
  return new Uint8Array(clean.match(/.{1,2}/g)!.map(b => parseInt(b,16)));
}

export function u8(n: number): Uint8Array {
  if (n < 0 || n > 255) throw new Error('u8 out of range');
  return Uint8Array.from([n]);
}

export function u64be(n: bigint): Uint8Array {
  const out = new Uint8Array(8);
  for (let i=7;i>=0;i--){ out[i] = Number(n & 0xffn); n >>= 8n; }
  return out;
}

export function i128be(x: bigint): Uint8Array {
  const mod = 1n << 128n;
  let v = x;
  if (v < 0) v = mod + v;
  const out = new Uint8Array(16);
  for (let i=15;i>=0;i--){ out[i] = Number(v & 0xffn); v >>= 8n; }
  return out;
}

export function concatBytes(...parts: Uint8Array[]): Uint8Array {
  const len = parts.reduce((a,p)=>a+p.length,0);
  const out = new Uint8Array(len);
  let off = 0;
  for (const p of parts){ out.set(p, off); off += p.length; }
  return out;
}
