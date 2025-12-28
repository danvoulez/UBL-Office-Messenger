import { describe, it, expect } from 'vitest';
import { i128be } from '../src/utils/bytes.js';
describe('i128be', () => {
    it('encodes max positive 2^127-1 correctly (no leading sign bit)', () => {
        const max = (1n << 127n) - 1n;
        const bytes = i128be(max);
        expect(bytes.length).toBe(16);
        expect(bytes[0] & 0x80).toBe(0); // top bit not set
    });
    it('encodes -1 as all 0xff', () => {
        const neg = -1n;
        const bytes = i128be(neg);
        expect(Array.from(bytes).every(b => b === 0xff)).toBe(true);
    });
});
