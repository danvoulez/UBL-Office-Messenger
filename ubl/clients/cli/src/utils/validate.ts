export function isHex(str: string): boolean {
  const s = str.startsWith('0x') ? str.slice(2) : str;
  return /^[0-9a-fA-F]+$/.test(s);
}
export function isHexLen(str: string, nBytes: number): boolean {
  const s = str.startsWith('0x') ? str.slice(2) : str;
  return isHex(str) && s.length === nBytes * 2;
}
export function toHexBare(str: string): string {
  return str.startsWith('0x') ? str.slice(2) : str;
}

export function checkClassDelta(intentClass: number, delta: bigint): string | null {
  // 0=Observation (delta==0), 1=Conservation (delta!=0), 2=Entropy (delta!=0), 3=Evolution (delta==0)
  switch(intentClass){
    case 0: return delta === 0n ? null : "Observation requires delta == 0";
    case 1: return delta !== 0n ? null : "Conservation requires delta != 0";
    case 2: return delta !== 0n ? null : "Entropy requires delta != 0";
    case 3: return delta === 0n ? null : "Evolution requires delta == 0";
    default: return "Invalid intent_class (expected 0..3)";
  }
}

export function basicLinkShape(link: any): string[] {
  const errs: string[] = [];
  const req = ["version","container_id","expected_sequence","previous_hash","atom_hash","intent_class","physics_delta","author_pubkey","signature"];
  for (const k of req){
    if(!(k in link)) errs.push(`missing field: ${k}`);
  }
  if (typeof link.version !== 'number') errs.push('version must be number');
  if (!isHexLen(link.container_id ?? "", 32)) errs.push('container_id must be hex32');
  if (!/^[0-9]+$/.test(String(link.expected_sequence ?? ''))) errs.push('expected_sequence must be u64 as string/number');
  if (!isHexLen(link.previous_hash ?? "", 32)) errs.push('previous_hash must be hex32');
  if (!isHexLen(link.atom_hash ?? "", 32)) errs.push('atom_hash must be hex32');
  if (![0,1,2,3].includes(Number(link.intent_class))) errs.push('intent_class must be 0..3');
  const d = BigInt(String(link.physics_delta ?? "0"));
  void d;
  if (!isHexLen(link.author_pubkey ?? "", 32)) errs.push('author_pubkey must be hex32');
  if (!isHexLen(link.signature ?? "", 64)) errs.push('signature must be hex64');
  return errs;
}
