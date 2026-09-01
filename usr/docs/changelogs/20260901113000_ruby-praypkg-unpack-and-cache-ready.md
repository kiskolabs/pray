# Ruby praypkg unpack and empty registry cache

## Participants

Andrei Makarov

## Decisions

Ruby pray-cli stops setting Encoding.default_internal in bin/pray. Archive pack and unpack clear Encoding.default_internal around Open3 tar and zstd calls so binary stdin and stdout are not transcoded when a caller still sets a UTF-8 default internal encoding. force_encoding BINARY alone does not fix Open3.capture2; the pipe still transcodes when default_internal is UTF-8.

Registry.cache_ready? rescues Pray::Error as well as filesystem errors. An empty or corrupt .pray/cache/registry directory is treated as not ready so the next install unpacks again instead of reporting no prayspec.

Ruby and TypeScript unpack list tar members first, reject path escape and non-file types, enforce archive size ceilings shared with Rust resource limits, then extract. Registry install unpacks under a .staging directory and renames into the final cache path on success.

Ruby resolve implies source: from a matching package namespace or the sole declared source, matching Rust and TypeScript.

## Effects

pray install against a git distribution that serves .praypkg artifacts no longer fails with zstd unexpected end of file or ASCII-8BIT to UTF-8 conversion on 0xB5. A failed unpack that left an empty cache directory no longer blocks the retry with a resolution error. Hostile ../ members are refused before extract. Format that omits source: for a matching namespace resolves on Ruby as well.

## Next

Publish 1.9.2 after review. Tag v1.9.2 and cut a GitHub Release so upgrade notices resolve. Publish order for crates.io: pray-core, then pray-transport, then pray-cli.

## Source

rubygems/pray-cli/bin/pray
rubygems/pray-cli/lib/pray/archive.rb
rubygems/pray-cli/lib/pray/archive_unpack.rb
rubygems/pray-cli/lib/pray/registry.rb
rubygems/pray-cli/lib/pray/registry_install.rb
rubygems/pray-cli/lib/pray/resolve_source.rb
npmjs/pray-cli/src/archive/praypkg.ts
npmjs/pray-cli/src/registry/install.ts
CHANGELOG.md 1.9.2
usr/docs/issues/20260901133800_audit-ruby-praypkg-unpack-cache.md
