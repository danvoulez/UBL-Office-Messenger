import { describe, it, expect } from 'vitest';
import { LedgerService, LEDGER_CONFIG } from '../../services/ledger';

describe('LedgerService', () => {
  describe('generateHashSync', () => {
    it('generates a hash with correct format', () => {
      const hash = LedgerService.generateHashSync('test content');
      expect(hash).toMatch(/^0X[0-9A-F]+$/);
    });

    it('generates unique hashes', () => {
      const hash1 = LedgerService. generateHashSync('test');
      const hash2 = LedgerService.generateHashSync('test');
      expect(hash1).not.toBe(hash2);
    });
  });

  describe('generateHash', () => {
    it('generates async hash with correct format', async () => {
      const hash = await LedgerService.generateHash('test content');
      expect(hash).toMatch(/^0X[0-9A-F]{32}$/);
    });
  });

  describe('calculateExecutionCost', () => {
    it('calculates cost for user', () => {
      const cost = LedgerService.calculateExecutionCost('hello', false);
      expect(cost).toBeGreaterThan(0);
      expect(cost).toBeLessThan(0.01);
    });

    it('calculates higher cost for agent', () => {
      const userCost = LedgerService. calculateExecutionCost('hello', false);
      const agentCost = LedgerService.calculateExecutionCost('hello', true);
      expect(agentCost).toBeGreaterThan(userCost);
    });

    it('scales with content length', () => {
      const shortCost = LedgerService.calculateExecutionCost('hi', false);
      const longCost = LedgerService.calculateExecutionCost('hello world this is a longer message', false);
      expect(longCost).toBeGreaterThan(shortCost);
    });
  });

  describe('verifyIntegrity', () => {
    it('validates correct hash format', () => {
      expect(LedgerService.verifyIntegrity('0X1234567890ABCDEF1234567890ABCDEF')).toBe(true);
    });

    it('rejects invalid format', () => {
      expect(LedgerService.verifyIntegrity('invalid')).toBe(false);
      expect(LedgerService.verifyIntegrity('0x123')).toBe(false);
      expect(LedgerService.verifyIntegrity('1234567890')).toBe(false);
    });
  });

  describe('LEDGER_CONFIG', () => {
    it('has correct constants', () => {
      expect(LEDGER_CONFIG.GAS_PRICE).toBe(0.000012);
      expect(LEDGER_CONFIG. NETWORK_ID).toBe('ubl-mainnet-v1');
      expect(LEDGER_CONFIG. PRECISION).toBe(8);
    });
  });
});