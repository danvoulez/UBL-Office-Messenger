import { Command } from 'commander';
import { spawnSync } from 'node:child_process';
import { readConfig } from '../utils/config.js';

export function doctorCommand(){
  const cmd = new Command('doctor').description('Checagens rápidas do ambiente');
  cmd.option('--db', 'checar /healthz/db');
  cmd.option('--runner', 'provas rápidas de runner (gVisor/nsjail/iptables)');
  cmd.option('--perf', 'mede latência de /healthz e, opcionalmente, /commit e /tail');

  cmd.action(async (opts)=>{
    const cfg = readConfig();
    const out: any = { ok: true, checks: {} };

    // config
    out.checks.config = { has_server: !!cfg.server, has_s3: !!cfg.s3_alias && !!cfg.bucket_official };
    if(!out.checks.config.has_server) out.ok = false;

    // server /healthz
    try {
      const res = await fetch(cfg.server!.replace(/\/$/,'') + '/healthz');
      out.checks.healthz = res.ok;
      if(!res.ok) out.ok = false;
    } catch(e){
      out.checks.healthz = false;
      out.ok = false;
    }

    // SSE reachable (quick try)
    try {
      const ctrl = new AbortController();
      const p = fetch(cfg.server!.replace(/\/$/,'') + '/tail', { signal: ctrl.signal });
      setTimeout(()=>ctrl.abort(), 600); // quick probe
      await p;
      out.checks.tail_probe = true;
    } catch {
      out.checks.tail_probe = false;
      out.ok = false;
    }

    // mc presence
    try {
      const r = spawnSync('mc', ['--version'], { encoding: 'utf8' });
      out.checks.mc = r.status === 0;
    } catch {
      out.checks.mc = false;
    }

    if (opts.db) {
      try { const r = await fetch(cfg.server!.replace(/\/$/,'') + '/healthz/db'); out.checks.db = r.ok; if(!r.ok) out.ok=false; } catch { out.checks.db = false; out.ok=false; }
    }
    if (opts.runner){
      const { spawnSync } = await import('node:child_process');
      const outRun: any = {};
      try { outRun.runsc = spawnSync('runsc', ['--version'], { encoding: 'utf8' }).status === 0; } catch { outRun.runsc = false; }
      try { outRun.nsjail = spawnSync('nsjail', ['--version'], { encoding: 'utf8' }).status === 0; } catch { outRun.nsjail = false; }
      try { 
        const result = spawnSync('iptables', ['-S'], { encoding: 'utf8' });
        outRun.iptables = result.status !== null && [0,1].includes(result.status); 
      } catch { outRun.iptables = false; }
      out.checks.runner = outRun;
      if (!outRun.runsc && !outRun.nsjail) out.ok = false;
    }


    if (opts.perf){
      const server = cfg.server!.replace(/\/$/,'');
      const metrics: any = { healthz_ms: null, commit_ms: null, tail_eps: null };
      // 1) healthz (baseline)
      try {
        const t0 = Date.now();
        const r = await fetch(server + '/healthz');
        await r.text();
        metrics.healthz_ms = Date.now() - t0;
      } catch { metrics.healthz_ms = -1; out.ok = false; }
      // 2) optional commit loop (requires flags)
      if (process.env.UBL_PERF_COMMIT === '1'){
        try {
          const { buildSigningBytes } = await import('../utils/signing.js');
          const ed = await import('@noble/ed25519');
          const seq0 = BigInt(process.env.SEQ0 ?? '1');
          let seq = seq0;
          let prev = String(process.env.PREV ?? '0'.repeat(64));
          const container = String(process.env.CID ?? '').replace(/^0x/,''); // hex32
          const priv = Buffer.from(String(process.env.PRIV ?? '' ).replace(/^0x/,''), 'hex');
          const iter = Number(process.env.ITER ?? '3');
          const atom = '0'.repeat(63)+'1';
          const t0 = Date.now();
          for (let i=0;i<iter;i++){
            const sb = buildSigningBytes({
              version: 1,
              containerHex32: container,
              expectedSequence: seq,
              previousHashHex32: prev,
              atomHashHex32: atom,
              intentClass: 0,
              physicsDelta: 0n,
            });
            const sig = await ed.signAsync(sb, priv);
            const pub = Buffer.from(await ed.getPublicKeyAsync(priv)).toString('hex');
            const link = { version:1, container_id: container, expected_sequence: String(seq), previous_hash: prev, atom_hash: atom, intent_class:0, physics_delta:'0', pact:null, author_pubkey: pub, signature: Buffer.from(sig).toString('hex') };
            const r = await fetch(server + '/commit', { method:'POST', headers:{'content-type':'application/json'}, body: JSON.stringify(link)});
            await r.text();
            // NOTE: para medir roundtrip; atualizar seq/prev exigiria resposta do servidor
            seq += 1n;
          }
          metrics.commit_ms = Date.now() - t0;
        } catch { metrics.commit_ms = -1; }
      }
      // 3) tail throughput (seconds via env.SEC)
      try {
        const sec = Number(process.env.SEC ?? '3');
        const ac = new AbortController();
        const url = server + '/tail';
        const t0 = Date.now();
        let count = 0;
        const p = fetch(url, { signal: ac.signal }).then(async r => {
          const reader = r.body!.getReader();
          const dec = new TextDecoder();
          let buf = '';
          for(;;){
            const { value, done } = await reader.read();
            if (done) break;
            buf += dec.decode(value, { stream:true });
            for(;;){
              const i = buf.indexOf('\n\n');
              if (i < 0) break;
              const chunk = buf.slice(0, i);
              buf = buf.slice(i+2);
              if (chunk.startsWith('data:')) count++;
            }
          }
        });
        setTimeout(()=> ac.abort(), sec*1000);
        await p;
        const elapsed = (Date.now() - t0)/1000;
        metrics.tail_eps = (elapsed>0? (count/elapsed) : 0);
      } catch { metrics.tail_eps = -1; }
      out.checks.perf = metrics;
    }

    console.log(JSON.stringify(out, null, 2));
    if(!out.ok) process.exitCode = 2;
  });

  return cmd;
}
