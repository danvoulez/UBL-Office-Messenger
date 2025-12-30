#!/usr/bin/env node
import { Command } from 'commander';
import { configCommands } from './cmds/config.js';
import { atomCommands } from './cmds/atom.js';
import { linkCommands } from './cmds/link.js';
import { idCommands } from './cmds/id.js';
import { s3Commands } from './cmds/s3.js';
import { commitCommands } from './cmds/commit.js';
import { commitVerifyCommand } from './cmds/commit-verify.js';
import { tailCommand } from './cmds/tail.js';
import { publishCommand } from './cmds/publish.js';
import { doctorCommand } from './cmds/doctor.js';
import { runnerCommand } from './cmds/runner.js';
import { packCommand } from './cmds/pack.js';
import { wsTestCommands } from './cmds/ws-test.js';

const program = new Command();
program
  .name('ubl')
  .description('UBL CLI — rigorosa, LLM-friendly, humana')
  .version('0.1.0');

program.addCommand(configCommands());
program.addCommand(idCommands());
program.addCommand(atomCommands());
program.addCommand(linkCommands());
program.addCommand(s3Commands());
const commit = commitCommands();
commit.addCommand(commitVerifyCommand());
program.addCommand(commit);
program.addCommand(tailCommand());
program.addCommand(publishCommand());
program.addCommand(doctorCommand());
program.addCommand(runnerCommand());
program.addCommand(packCommand());
program.addCommand(wsTestCommands());

program.parseAsync(process.argv);
