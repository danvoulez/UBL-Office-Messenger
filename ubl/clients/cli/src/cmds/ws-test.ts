import { Command } from 'commander';
import { readFileSync } from 'fs';

export function wsTestCommands() {
  const cmd = new Command('ws:test')
    .description('Executa testes em workspace via Office')
    .requiredOption('--tenant <string>', 'Tenant ID')
    .requiredOption('--workspace <string>', 'Workspace ID')
    .requiredOption('--repo <string>', 'Repository name')
    .requiredOption('--sha <string>', 'Git SHA (min 8 chars)')
    .requiredOption('--suite <string>', 'Test suite name')
    .option('--limits <path>', 'Path to JSON file with limits (cpu, mem_mb, timeout_sec, net)')
    .option('--wait <true|false>', 'Wait for receipt', 'true')
    .action(async (opts) => {
      const base = process.env.OFFICE_BASE || 'http://localhost:8081';
      const body: any = {
        tenant: opts.tenant,
        workspace: opts.workspace,
        repo: opts.repo,
        sha: opts.sha,
        suite: opts.suite,
        wait: opts.wait !== 'false',
      };

      if (opts.limits) {
        try {
          body.limits = JSON.parse(readFileSync(opts.limits, 'utf8'));
        } catch (e) {
          console.error(`Failed to read limits file: ${e}`);
          process.exit(1);
        }
      }

      try {
        const res = await fetch(`${base}/office/ws/test`, {
          method: 'POST',
          headers: {
            'content-type': 'application/json',
            authorization: process.env.SID || '',
            'x-ubl-asc': process.env.ASC_ID || '',
          },
          body: JSON.stringify(body),
        });

        const json = await res.json();

        if (!res.ok) {
          console.error('ERROR', res.status, json);
          process.exit(1);
        }

        console.log(JSON.stringify(json, null, 2));
      } catch (e: any) {
        console.error('Request failed:', e.message || e);
        process.exit(1);
      }
    });

  return cmd;
}

