export const HELP_TEXT = `pray <command>

Commands:
  version | -V | --version
  manifest
  init [--targets tool_a,tool_b]
  prayer init
  repo init
  add <name> [constraint] [--path PATH]
  remove <name>
  update [package] [--major] [--latest] [--dry-run] [--json]
  unlock <package>
  install [--locked|--frozen|--offline]
  render [--check]
  plan
  apply
  verify [--strict]
  drift [--semantic]
  format
  package
  publish --root PATH [--server URL ...]
  login --server URL --email EMAIL
  serve [--root PATH] [--host HOST] [--port PORT]
  sync [--root PATH] [--peer URL ...]
  trust list|show|add-key|remove-key|set-signed|set-allow|check
  confess <package> | --from-lock SPAN_ID [--accepted|--rejected] [--note NOTE] [--url URL]
  list
  outdated [--remote]
  explain <package>
  vendor
  clean
  tree

TypeScript reference CLI for Prayfile. See SPEC.md for the full command surface.
`;
