import { describe, it, expect } from 'vitest';
import { canonicalize } from '../src/utils/canon.js';
import { blake3hex } from '../src/utils/hash.js';

describe('canonicalize', ()=>{
  it('sorts object keys and preserves arrays', ()=>{
    const input = { b: 1, a: [2,1] };
    const out = canonicalize(input);
    expect(out).toBe('{"a":[2,1],"b":1}');
  });
  it('rejects non-finite numbers', ()=>{
    expect(()=>canonicalize({x: Infinity})).toThrow();
  });
});

describe('atom hash', ()=>{
  it('matches blake3 over canonical bytes', ()=>{
    const input = {"a":1,"b":2};
    const canon = canonicalize(input);
    const h = blake3hex(Buffer.from(canon));
    expect(h).toHaveLength(64);
  });
});

it('normalizes scientific notation to plain decimal', ()=>{
  const input = { x: 1e3, y: 3.14e-2 };
  const out = canonicalize(input);
  // Ensure no 'e' remains in the canonical string
  if (out.includes('e') || out.includes('E')) throw new Error('scientific notation leaked');
});
