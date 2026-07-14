import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { conciseHelpText, commandHelpText, maybePrintHelp } from "./help.js";
import { unknownCommandMessage, suggestCommand, TOP_LEVEL_COMMANDS } from "./suggest.js";

describe("help", () => {
  it("includes product description and help hint", () => {
    const text = conciseHelpText();
    assert.match(text, /reproducible inference input/);
    assert.match(text, /pray help/);
    assert.match(text, /--no-input/);
  });

  it("includes offline flag for install help", () => {
    const text = commandHelpText("install");
    assert.ok(text);
    assert.match(text, /--offline/);
  });

  it("detects help subcommand targets", () => {
    assert.equal(maybePrintHelp(["help", "install"]), "printed");
    assert.equal(maybePrintHelp(["install", "--help"]), "printed");
    assert.equal(maybePrintHelp(["install"]), "not_help");
  });
});

describe("suggest", () => {
  it("suggests install for instal typo", () => {
    assert.equal(suggestCommand("instal", TOP_LEVEL_COMMANDS), "install");
    assert.match(unknownCommandMessage("instal"), /Did you mean `install`\?/);
  });
});
