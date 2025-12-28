import Big from 'big.js';

function isPlainObject(x: any): x is Record<string, any> {
  return Object.prototype.toString.call(x) === '[object Object]';
}

// Serialize numbers without scientific notation; integers preserved as integers
function serializeNumber(n: number): string {
  if(!Number.isFinite(n)) throw new Error('NonFiniteNumber');
  // If integer within safe range, keep as integer
  if(Number.isInteger(n)) return String(n);
  // For decimals or exponent form, force plain string via big.js
  const s = n.toString();
  if (s.includes('e') || s.includes('E')) {
    return new Big(s).toString();
  }
  return s;
}

export function canonicalize(input: any): string {
  function walk(v: any): any {
    if (v === null) return null;
    const t = typeof v;
    if (t === 'boolean') return v;
    if (t === 'number') return JSON.parse(serializeNumber(v));
    if (t === 'string') return v.normalize('NFC');
    if (Array.isArray(v)) return v.map(walk);
    if (isPlainObject(v)) {
      const out: Record<string, any> = {};
      for (const k of Object.keys(v).sort((a,b)=>Buffer.from(a).compare(Buffer.from(b)))) {
        out[k] = walk(v[k]);
      }
      return out;
    }
    throw new Error('InvalidType');
  }
  const canon = walk(input);
  return JSON.stringify(canon);
}
