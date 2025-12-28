import { concatBytes, hexToBytes, i128be, u8, u64be } from './bytes.js';

export function buildSigningBytes(opts: {
  version: number;
  containerHex32: string;
  expectedSequence: bigint;
  previousHashHex32: string;
  atomHashHex32: string;
  intentClass: number; // 0..3
  physicsDelta: bigint; // i128
}): Uint8Array {
  return concatBytes(
    u8(opts.version),
    hexToBytes(opts.containerHex32, 32),
    u64be(opts.expectedSequence),
    hexToBytes(opts.previousHashHex32, 32),
    hexToBytes(opts.atomHashHex32, 32),
    u8(opts.intentClass),
    i128be(opts.physicsDelta),
  );
}
