import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { commandHelpText, conciseHelpText, maybePrintHelp } from "./help.js";
import {
  suggestCommand,
  TOP_LEVEL_COMMANDS,
  unknownCommandMessage,
} from "./suggest.js";

describe("help", () => {
  it("includes usage synopsis and see also hint", () => {
    const text = conciseHelpText();
    assert.match(text, /Usage: pray \[OPTIONS\] <COMMAND>/);
    assert.match(text, /See 'pray help <command>'/);
    assert.match(text, /Options:/);
    assert.match(text, /--no-input/);
    assert.match(text, /upgrade/);
    assert.match(text, /outdated \[--remote\]/);
    assert.doesNotMatch(text, /Documentation:/);
    assert.doesNotMatch(text, /Exit codes:/);
  });

  it("includes offline flag for install help", () => {
    const text = commandHelpText("install");
    assert.ok(text);
    assert.match(text, /--offline/);
    assert.doesNotMatch(text, /Documentation:/);
  });

  it("documents login and upgrade", () => {
    const login = commandHelpText("login");
    assert.ok(login);
    assert.match(login, /--passkey-key|--ssh-agent/);
    const upgrade = commandHelpText("upgrade");
    assert.ok(upgrade);
    assert.match(upgrade, /npm install -g pray-cli@latest/);
  });

  it("covers listed commands with per-command help", () => {
    for (const command of [
      "remove",
      "list",
      "format",
      "fmt",
      "version",
      "sync",
    ]) {
      const text = commandHelpText(command);
      assert.ok(text, `missing help for ${command}`);
      assert.match(text, /Usage: pray/);
    }
  });

  it("detects help subcommand targets", () => {
    assert.equal(maybePrintHelp(["help", "install"]), "printed");
    assert.equal(maybePrintHelp(["install", "--help"]), "printed");
    assert.equal(maybePrintHelp(["help", "remove"]), "printed");
    assert.equal(maybePrintHelp(["install"]), "not_help");
  });
});

describe("suggest", () => {
  it("suggests install for instal typo", () => {
    assert.equal(suggestCommand("instal", TOP_LEVEL_COMMANDS), "install");
    const message = unknownCommandMessage("instal");
    assert.match(message, /Did you mean `install`\?/);
    assert.match(message, /See 'pray --help'\./);
  });
});
