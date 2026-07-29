# pray CLI exit codes

The reference CLI (`pray`) uses stable numeric exit codes so scripts and CI can branch on failure class. Errors go to stderr; primary command output goes to stdout.

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error (I/O, missing or invalid manifest context) |
| 2 | Parse error or usage/CLI argument error |
| 3 | Resolution error |
| 4 | Integrity error |
| 5 | Render/check failed |
| 6 | Verify failed (also when `pray drift` finds drift) |
| 7 | Network/fetch error |
| 8 | Unsupported feature |

Examples:

- Unknown command or bad flags → exit 2
- Missing `Prayfile` → exit 1
- Hash or signature mismatch → exit 4
- `pray verify --strict` with findings → exit 6
- Unreachable registry host → exit 7

Normative mapping: SPEC.md section 66. Man page: `docs/man/pray.1` (EXIT STATUS). Concise `pray --help` does not list these codes; use this page or the man page.
