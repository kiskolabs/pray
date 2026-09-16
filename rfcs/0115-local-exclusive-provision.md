# RFC 0115: Local exclusive provision

- Feature Name: local-exclusive-provision
- Type: Standards Track
- Status: Rejected
- Created: 2026-09-16
- Author: Andrei Makarov
- Relates: RFC 0010, RFC 0033, RFC 0108
- Requires: RFC 0010, RFC 0033

## Summary

This RFC proposed exclusive copy of a project-local file or directory to another path in the same repository, with a lock ledger for those dest files. That proposal is not adopted.

RFC 0010 remains the parse contract. Compose still embeds local files as fragments. Package `file:` and `tree` still provision package exports.

## Motivation

The proposal copied RFC 0033 exclusive dest ownership onto a path that already lives in the project. `pray ".agents/zshrc", file: ".zshrc"` would write unmarked bytes at `.zshrc` from `.agents/zshrc`. A local directory inside `tree` would do the same for a folder. The lock ledger would record those dest files.

Those forms keep two copies of the same bytes. Operators already have the source file. Dest should not duplicate it.

## Guide-level explanation

The proposed Prayfile forms were:

```manifest
pray ".agents/zshrc", file: ".zshrc"
file ".gitignore" do
  pray ".agents/gitignore"
end
tree ".agents/skills" do
  pray ".agents/local-skills"
end
```

They are not in the language. RFC 0010 refuses a local path with `file:`, a local path inside `tree`, and a `file` block whose pray token is a local path. Bare `pray "relative/or/./path"` remains valid only inside `compose`.

Forms that stay, specified by RFC 0010 and RFC 0033:

```manifest
compose "AGENTS.md" do
  pray ".agents/project.md"
  pray "amkisko/working-rules", "~> 2.0"
end
tree ".agents/skills" do
  pray "amkisko/engineering-audit", "~> 2.0"
end
pray "amkisko/community-security", "~> 1.0", file: "SECURITY.md"
```

## Reference-level explanation

This RFC does not specify a contract.

The withdrawn design would have treated a local path with `file:` as an exclusive dest copy, treated a local path inside `tree` as an exclusive directory copy, and written `[[provisioned]]` records with `package` `local`. None of those forms or records ship.

Parse refusals for those forms live in RFC 0010. Exclusive ownership of package `file:` and `tree` leaves lives in RFC 0033.

## Drawbacks

Of not adopting: a project that wants an unmarked dest file from local bytes must keep that dest as the only copy, or put the bytes in a package export.

## Rationale and alternatives

The proposal used the provisioned ledger as the reason to duplicate project files. Compose embed is a different job: it patches one destination from ordered fragments. Package `file:` and `tree` pull exports from a package, not from another path in the same project.

A path package with a folder or file export already copies into dest from a package tree.

## Prior art

RFC 0033 exclusive leaves for package exports. Stow-style copy of a local tree onto another path.

## Unresolved questions

None.

## Future possibilities

Path-fork material in a path package tree remains RFC 0116.
