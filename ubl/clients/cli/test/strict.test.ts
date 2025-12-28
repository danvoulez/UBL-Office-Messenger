import { describe, it, expect } from 'vitest';
import { checkClassDelta } from '../src/utils/validate.js';

describe('class/delta coherence', ()=>{
  it('observation requires delta==0', ()=>{
    expect(checkClassDelta(0, 0n)).toBeNull();
    expect(checkClassDelta(0, 1n)).toBeTypeOf('string');
  });
  it('conservation requires delta!=0', ()=>{
    expect(checkClassDelta(1, 0n)).toBeTypeOf('string');
    expect(checkClassDelta(1, 5n)).toBeNull();
  });
  it('entropy requires delta!=0', ()=>{
    expect(checkClassDelta(2, 0n)).toBeTypeOf('string');
  });
  it('evolution requires delta==0', ()=>{
    expect(checkClassDelta(3, 0n)).toBeNull();
  });
});
