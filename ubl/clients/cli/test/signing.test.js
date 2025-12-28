import { describe, it, expect } from 'vitest';
import { buildSigningBytes } from '../src/utils/signing.js';
describe('signing_bytes', () => {
    it('encodes in strict big-endian', () => {
        const sb = buildSigningBytes({
            version: 1,
            containerHex32: 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
            expectedSequence: 1n,
            previousHashHex32: 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
            atomHashHex32: 'cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc',
            intentClass: 2,
            physicsDelta: -1n,
        });
        const hex = Buffer.from(sb).toString('hex');
        expect(hex.startsWith('01')).toBe(true);
        expect(hex.includes('02')).toBe(true);
        expect(hex.endsWith('ff'.repeat(16))).toBe(true);
    });
});
