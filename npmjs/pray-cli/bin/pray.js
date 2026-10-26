#!/usr/bin/env node
import { runCli } from "../dist/cli/main.js";

const exitCode = await runCli(process.argv.slice(2));
process.exit(exitCode);
