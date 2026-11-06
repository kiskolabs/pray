<!-- pray:0 ignore-comments -->

# Agent context

Do not edit managed blocks in `AGENTS.md` or provisioned files under `.agents/`.
To change shared guidance, update `Prayfile` and run `pray install`.

<!-- pray:ae5d334a -->
## Additional instructions

### .agents/project.md
Repository for the pray open specification and the reference CLI.

Read `README.md` for project positioning and `rfcs/` for the normative Prayfile, prayspec, lockfile, distribution point, and CLI design (RFC 0111).

## Project intent

- Production readiness. Build the reference CLI and specification together, prioritizing validated contracts, user-facing reliability, and test coverage.
- Problem focus. Inference input is operational. Packaging shapes will keep changing. Prayfile targets reproducible composition, provenance markers, and sync of shared input libraries across repositories, not any one vendor workflow.
- What the tool must do. Resolve declared input dependencies, lock exact versions and hashes, render tool-specific files under defined contracts, cite managed blocks with compact pray markers into `Prayfile.lock`, and keep shared input pinned and updatable through manifest and lockfile semantics.
- Production focus. Prefer contract clarity, production validation, and test coverage over premature implementation.

## Rust workspace

For the pray reference implementation, run from the workspace root:

- `cargo test` for the full suite
- `cargo test -p <crate>` for a focused crate
- `cargo clippy` and `cargo fmt --check` before claiming quality checks pass

Use coverage tooling declared in this repository when validating coverage claims.

Prefer files around 150 lines or fewer when cohesion allows. Treat 300 lines as a hard upper bound for any source file unless a very small exception is clearly justified. When a file approaches that ceiling, split by semantic responsibility into separate modules, folders, or helpers rather than by arbitrary line count.

Enforce with `make loc-check` (warn >=150, fail >300 unless ratcheted in `scripts/loc-limits.allowlist`). Language linters mirror the hard ceiling: RuboCop `Pray/FileLength` (Ruby), Biome `noExcessiveLinesPerFile` (TypeScript). Clippy has no file-LOC lint yet.

Test coverage must follow `spec/README.md` guidelines.

## Pressure benchmarking binding

Consumer binding for `amkisko/pressure-benchmarking`:

- measure: `make pressure-bench` (Rust, Ruby, and npm compat harnesses)
- self-test: `ruby scripts/pressure-bench.rb --self-test` (distinct parse/resolve/render/lock p95)
- ceilings: `scripts/pressure-ceilings.json` (version 2 under `implementations`)
- load axis: package count via `PRAY_PRESSURE_LOADS` (default `1,10,100`)
- implementations: `PRAY_PRESSURE_IMPLS` (default `rust,ruby,npm`)
- samples-per-load: `PRAY_PRESSURE_SAMPLES` (use `>= 3` before gate, propose, or write)
- min episode iterations: 10 optimize/prove cycles at the stressed load
- propose-basis: timing series `max * headroom`; RSS `median * headroom`; headroom default `1.15`
- Memory probe class: `process_lifetime` (compat harness peak RSS)
- flags: `--gate`, `--propose`, `--write`, `--include-ship`

## Shared instructions
<!-- pray:ae5d334a -->

<!-- pray:9068e4a2 -->
- when fixing or refactoring code, add or update tests first to expose the current bug/regression path (or missing contract), then implement the fix, then run focused and broader checks, and do not ship behavior changes without proving before/after via specs;
- test only executable logic and user-facing behavior; tests should affect coverage metrics;
- avoid tests that only assert implementation details; avoid file/page content/ordering/regex assertions; avoid duplicating tests;
- user interface texts should never mention implementation technical details;
- prefer files around <=150 LOC when cohesion allows; split only when it improves ownership, readability, and reviewability;
- do not use abbreviations or short names unless they are very common;
- avoid explanatory comments, but allow intent comments for non-obvious constraints, invariants, concurrency edges, or external contract requirements;
- readability, structure, and clarity are product qualities;
- pull request description answers what problem is solved, why it matters, how the solution works, and relevant context; non-trivial changes include reproduction steps or a changelog entry with intent;
- pull request checklist: changelog entry with intent or reproduction steps when relevant, test coverage, and quality checks done;
- follow docs-conventions for docs timestamp-tree filenames and layout;
- report completed actions only with observed evidence; validation output must list exact commands run and observed results;
- ignore style-only dust unless it harms correctness, operability, maintainability, or auditability under realistic load;
- sibling files and executable checks beat shared defaults; mixed styles stay a split until a path boundary explains both;
- fix the cause of a race, not a retry around it; prefer positive names; compute at write when a read cannot paginate; do not change production design only so tests can reach it;
- if process state and local cache files are both cleared, name what must be rebuilt and from which durable source;
- side effects read committed state from a cursor; they are not steps of the write;
- on a store or network hot path, name the expensive unit and keep a before-and-after budget;
- after changing a published number, identifier, or contract field, search dependents and mark them stale or update them; when a published identifier shape changes, search path helpers that pass a string id, not only those that pass a record.
<!-- pray:9068e4a2 -->

<!-- pray:781b7711 -->
## Credentials and Secrets

- Prefer a secret store or OS credential helper over embedding live secrets in config files, scripts, or documentation. Named managers (for example 1Password, Bitwarden, KeePassXC) are fine; the requirement is isolation, not a specific vendor.
- Config and project files may hold references (vault paths, item ids, redacted fingerprints). They must not hold live tokens, API keys, passwords, or client secrets. The same prohibition applies to skill files, prompt templates, and MCP environment settings.
- Do not pass secrets on command lines or in other process-visible arguments. Prefer secret-store lookup, short-lived credentials, or stdin/file descriptors that do not persist in shell history.
- Do not commit secrets, paste them into issues or pull requests, or write them to logs. Rotate anything that may have been exposed.
- if a live secret, credential, or confidential trace appears in this session, treat it as a security event: tell the person, do not quote the value, and do not send it to another third party; the inference provider already saw what reached this session

## Tracking and identification

- A redacted fingerprint above is a hash of a secret for config references. A device fingerprint is fields that combine to identify a person or device across sessions or observers.
- Identifiers, IP addresses, device marks, and combined attributes are personal data. They can unmask a person, a location, or a session secret. Emit them only when the feature they asked for this session needs them and they were shown that this product would.
- Silent analytics ids, leftover marks after logout, and canvas or hardware probes are security events. They can locate a person, stitch sessions, or leak a credential-shaped token.

## Ownership and destinations

- Lookups go through an ownership set; request parameters pick which row; fail closed when access cannot be proven.
- Treat user-supplied URLs as untrusted; rate-limit authentication and abuse-prone endpoints.
- An invalid or expired credential gets a protocol-level failure status. A hop's own credential is never the caller's. Modes that relax access controls must remain disabled when their configuration is missing or invalid.
- Crawl preferences in `robots.txt` and Content-Signal style directives declare intent to cooperative clients. They are not authentication, ownership proof, or a technical block. They do not replace rate limits or fail-closed access checks.

Related: `engineering-audit` security mode asks whether a parameter establishes access and whether a worker skipped policy.
<!-- pray:781b7711 -->

<!-- pray:bfe6ff38 -->
- `docs/` holds maintained explanations and working records; use stable descriptive filenames for guides; placement does not imply polish or currency;
- keep inference input (AGENTS.md, `.agents/`) separate from `docs/`; conventions that matter fail a command;
- `usr/` is the workshop for working tools and operational material;
- `usr/scripts/` holds maintainer scripts invoked from a checkout, Makefile, CI, or a runner on a named path; not paste-into-live-console support or migrate work;
- `usr/reports/` holds read-only analysis and investigation console scripts; no mutations;
- `usr/support/` holds product and customer support console scripts that mutate or write; a person pastes or runs them after review; programmable agents do not run them against a live process;
- `usr/migrate/` holds schema and data migration scripts for manual running before new code is on the process; later schema migrate is schema-only and idempotent;
- six `docs/` timestamp trees, no README index, filename `YYYYMMDDHHMMSS_<kebab-case-title>.md`: `plans` (uncertain future: design-only notes before work is committed), `issues` (certain current: live work in play; contract, findings, open next; the open queue stays here), `reviews` (one review episode: session or CI scope, coverage, recommendations, outcome), `changelogs` (factual past: what shipped), `meetings` (one sitting: who was there and what they agreed), `dependencies` (upstream defects from real work);
- issues, reviews, changelogs, and meetings make five things findable with `##` headings or equivalent: Participants (humans), Decisions (what was agreed), Effects (done, failed, recovered, rolled back), Next (todo, planned, open questions), Source (links upstream and downstream);
- name the current operator under Participants when the current session provides their display name or that person confirms it; solo sessions still get one Participants line;
- if the current operator cannot be identified without inference, omit Participants rather than guess from git configuration, commit history, filesystem paths, OS accounts, or login names;
- list other humans who decided or reported in that episode when known; do not list agents, tools, binaries, or every person who touched related code;
- omit other empty sections; git history is the edit log; add an explicit note only when a later pass changes meaning (scope cut, rollback, decision reversed);
- Problem / Fix and similar ticket outlines do not replace the five headings; map that content into Decisions and Effects; keep Participants, Next, and Source as their own headings;
- mention software, tools, agents, or binaries in a note only when that detail is needed for execution or later analysis; put it under Decisions, Effects, or Source, not under Participants;
- never put local absolute paths or private material in `docs/` or under `usr/`: no home-directory or machine-specific filesystem paths, secrets, credentials, tokens, API keys, or personal private data; prefer repository-relative paths;
- one home per fact; link, do not duplicate; goal and acceptance live apart from operating rules; when a decision changes shape, supersede it in place; name what the product does not do next to what it does;
<!-- pray:bfe6ff38 -->

<!-- pray:edcc5f67 -->
## Dependency issues

When work surfaces a clearly visible bug or defect in a dependency (wrong behavior, broken API contract, regression between versions, or a fix already merged upstream but not released), say so in the task output and suggest a concrete fix path: upgrade, pin, patch, vendor, workaround, or upstream report.

Store evidence under `docs/dependencies/#{YYYYMMDDHHMMSS}_<kebab-case-title>.md`; no README index in that tree. Each file should make these findable (use `##` headings or equivalent; omit empty sections): Dependency (name, version constraint, lockfile entry if any), Symptom (what breaks and where), Evidence (repro steps, logs, stack traces, links to issues or commits), Suggested fix (upgrade, pin, patch, workaround, or upstream report), Next (todo, planned, open questions), Source (links upstream: issue, PR, release note, commit, and downstream materializations in this repo). Git history is the edit log.

Do not open drive-by dependency hunts; record only issues encountered while doing the requested work and only when the defect is evident from behavior or published upstream facts, not speculation.

For proactive selection, alteration, and audit rules, use `dependency-policy` and the dependency-audit skill.
<!-- pray:edcc5f67 -->

<!-- pray:3ac5d6ce -->
## Dependency policy

Rules for adding, changing, or removing third-party packages. Apply across languages. Names vary by ecosystem; concepts do not.

Terminology:

- package manifest - declares intent (`gemspec`, `package.json`, `Cargo.toml`, `mix.exs`, etc.)
- lockfile - pins the resolved graph CI and developers install
- registry - published versions consumers resolve (`RubyGems`, `npm`, `crates.io`, `Hex`, etc.)
- hot path - code on the security, auth, crypto, IO, or request/response boundary users rely on

Stop until one of these applies before adding a dependency:

- stdlib or the framework for this tree already covers it;
- an installed transitive dependency already covers it without a second library for the same job;
- the feature needs a new package and tests will prove behavior.

Run dependency-audit for direct dependency changes, graph audits, releases with hot-path package changes, and plausible dependency vulnerability or exploitation signals found during ordinary work. Keep the evidence and assessment in the live-work or dependency record.

On resolved lockfile graph or runtime-pin upgrades (automated or human), gather before/after resource samples for install or graph storage, resident memory, CPU or a stated proxy, and shipped bundle or image size when the product ships bytes. The reviewing agent uses isolated checkouts and the same small consumer-owned measurement commands for both revisions. It reports the comparison and writes to a change request only when authorized. Cheaper or smaller without comparable samples is inference. Continuous absolute ship-path ceilings stay in `delivery-budgets`.

Related: `dependency-issues` records upstream defects found during real work; `minimal-implementation` covers YAGNI before adding deps; `engineering-audit` covers code and pipeline review; `delivery-budgets` owns continuous size and build-timing gates.
<!-- pray:3ac5d6ce -->

<!-- pray:cd3045de -->
- use Rust and Cargo features according to the versions declared in the repository;
- follow Rust API guidelines, idiomatic error handling (`Result`/`Option`), and clippy-backed conventions where the project enables them;
- prefer explicit crate boundaries; keep binaries thin and library code testable;
- test coverage must follow the conventions declared in the relevant subtree; when a project defines coverage rules in `spec/README.md` or equivalent, follow those;
- do not block the async runtime with filesystem operations or large memory copies; use blocking threads or a separate worker pool;
- keep control-plane IO on a separate execution path from large data transfers;
<!-- pray:cd3045de -->

<!-- pray:bf7304a6 -->
## Minimal implementation

Efficient means the smallest correct change.

Before writing code, stop at each step until one applies:
- does the feature need to exist at all (YAGNI)?
- does the language stdlib or framework for this tree already cover it?
- does an existing implementation or dependency already solve it?
- can the change be one line; if so, make it one line?
- only then write the minimum code that works.

Before generating the main implementation of an unproven method, require a small runnable check that records a feasibility number or a failure verdict.

Before adding a new library directory or first-party package, stop until one applies:
- one product owns the contract and is the only caller: keep source in that tree;
- a second in-repo caller, or no product runtime: unpublished in-repo package (own manifest, own tests, path-linked, 0.x, registry publish blocked);
- a second repository or registry consumer: extract, publish, then follow dependency-policy.

Rules:
- match the language of the directory you are changing (see Preferred stack and tools above);
- match the tree's structural height; a consistent absence of a layer is a convention; do not introduce a second pattern for the same job;
- no abstractions unless three real variations need them; drop unused public methods;
- no new dependency when stdlib, the framework for this tree, or an installed dependency suffices;
- no boilerplate the task did not ask for;
- deletion over addition; boring over clever; fewest files that stay readable;
- when a request sounds overbuilt, ask whether a simpler existing path already covers it;
- when two stdlib approaches are the same size, pick the edge-case-correct one; less code is not an excuse for a flimsier algorithm;
- document deliberate shortcuts with an intent comment: name the known ceiling (global lock, O(n^2) scan, naive heuristic) and the upgrade path when that ceiling matters.

Not optional even when minimizing scope:
- input validation at trust boundaries;
- error handling that prevents data loss;
- security and accessibility;
- calibration against real hardware and production drift when the platform ideal is not the spec;
- anything explicitly requested in the task or ticket;
- tests for non-trivial behavior per @spec/README.md; trivial one-liners need no new spec.

Related: `preferred-stack` covers native-first language choice and proximity when no tree language owns the path yet; `keep-the-work` covers the failed place after a refusal; `dependency-policy` covers third-party registry packages; `infer-conventions` encodes how this tree writes as checks.
<!-- pray:bf7304a6 -->

<!-- pray:120c3507 -->
## Finite state machines

- model lifecycles with explicit finite state machines when status, allowed transitions, and side effects matter; prefer named states and guarded transitions over scattered conditionals and implicit enums alone; when who and when matter, model the event as a record, not a boolean flag;
- finite state machines can compactly represent ordered sets or maps of strings supporting fast prefix, suffix, and fuzzy search; consider tries and automata when matching catalogs, codes, routes, or searchable vocabularies at scale;
- when digital reported state and physical process state can diverge, name both machines and the observation that couples them; occupancy listing is not the lock; a reported identity is not the person or sample at the station;
- a later artifact does not prove an earlier gate; a cache generation is not the durable commit; a lease is not ownership of the object; bound generated plans and refine after each completed step.

Related: `engineering-audit` boundary mode asks when those states disagree without an alarm; `io-simulation` injects the faults that cause the split.
<!-- pray:120c3507 -->

<!-- pray:26f3566a -->
## Branch naming

Use kebab-case after the prefix.

Prefixes:

- `feature/<title>` - new capability
- `patch/<title>` - bugfix or chore
- `trunk/<title>` - release candidate or integration work before `main`
- `plan/<title>` - exploration or ideation

Examples:

- `feature/user-access-control`
- `patch/fix-translation`
- `trunk/2026w15`
- `trunk/2026-august-pack`
- `plan/auth-redesign-notes`
- `plan/2026-q2-roadmap`

Related: `planned-release` names commitment, collection period, period label, and prepare/deliver events; its skill owns defaults, binding, and process signals. The `trunk/<title>` prefix names a candidate branch. The stable line is a separate role.
<!-- pray:26f3566a -->

<!-- pray:f528eeca -->
## Preferred stack and tools

- native-first means: name the nearest semantic authority for the job (CPU and OS ABI, browser host, language runtime, database engine, GPU pipeline, build or cluster orchestrator, game or media engine, and similar), then prefer a language whose ordinary execution form lives there
- a portable sandbox, managed VM guest, or engine guest is not OS-native when the job is that host's contract
- match the language of the directory you are changing; follow the matching language pack when the project composes one
- when starting a new tree or an independent package with no owning language yet, choose by nearest authority and proximity band; ask only when two candidates sit in the same band and the choice changes the outcome; record language, authority, band, and why not a closer option
- house shortcuts when proximity and team practice agree: ruby for web application and API work; elixir for concurrent and distributed systems on the BEAM; rust for system programming and performance-critical native code; javascript, html, css for the browser document and style surface
- humane and accessible visual-surface principles for UI/UX, and for clear communication of intent and feedback; label icon-only controls; hide decorative duplicates from the accessibility tree
- when the change presents a screen, document, or other person-facing surface, name that surface in the work product (reply, pull request, live-work note) and record a visual assessment: hierarchy, copy, interactive states, empty/loading/error, a narrow surface, and whether a person can finish the task; record what was assessed and what remains for a human; skip and say so when there is no such surface
- pictures, previews, tickets, and chat threads are source locators for that signal; the assessment is the running product plus the written need
- product-contract and architecture work stay distinct from this visual-surface assessment
- person-facing dates and times follow the product locale form; native HTML date and datetime-local follow the browser locale and are not that form; machine APIs, data attributes, filenames, and CSV machine columns stay ISO unless the product already published another contract
- a catalog picker for growing association lists is a combobox that queries the product's live list API as the person types; each request takes a bounded page; further rows load on type or on scroll; a datalist or a select of every row is not that control
- catalog assignment starts empty when a default first row would apply a change
- a command that writes a record must look like a command

Related: the `preferred-stack` skill owns proximity bands and the selection procedure; `minimal-implementation` keeps scope small and repeats the tree-language match; language packs (`ruby-conventions`, `rust-conventions`, `elixir-conventions`, `kotlin-conventions`) own how to write after the choice; `keep-the-work` covers staying on the failed place after a refusal.
<!-- pray:f528eeca -->

<!-- pray:ca94e22d -->
## Writing and changelog prose checks

Review for marketing language, invented objections, empty contrasts, stray em dashes, and paragraph flow; keep notes and metadata honest and plain.
- docs timestamp trees, shared prayer sources (fragments, skill bodies, prompt templates, files under `prayers/`), and agent deliverable templates: plain prose readable without a rendered preview. No markdown tables, bold, italic, or other styling. Prioritize factual accuracy over presentation.
- Prefer characters typed from a normal keyboard layout. Do not use special Unicode punctuation or symbols for structure or emphasis: no Unicode arrows, em or en dashes, ellipsis characters, not-equal signs, curly quotes, section signs, or similar. Use ASCII stand-ins (`->`, `-`, `...`, `!=`, straight `"` / `'`) or plain words (`then`, `section`). Do not draw pipelines or diagrams with inaccessible glyphs; use a numbered or bulleted list, or a `text` fence with ASCII only.
- Ease, lexical diversity, coherence, mechanics, and claim integrity are separate constructs. Automated matches, readability grades, similarity, and model preference are review prompts; preserve meaning, necessary negation, scope, and uncertainty when editing.
- Keep agency on the person who acts. Tools and process nouns do mechanical work.
- Technical names, APIs, CLI verbs, RFC titles, identifiers, and UI copy use instrument and protocol words: check-in, last-seen, probe, monitor, expected tick. Body and organism metaphors such as heartbeat, pulse, and organ stay out of contracts and code. HTTP `/health` remains the liveness probe until a later RFC.
- One sentence holds one beat. Consecutive short sentences that only restated the same beat are a punchline stack.
- For material external claims, quotations, dates, or research summaries, use the claims-audit skill.
- signed comments on a public tracker speak as the account holder to the named person; chat stays in chat
- cite only what a public reader can open and check; confidential traces, unique working locators, and unverifiable session evidence stay in chat; do not expose source material that is not already public, and do not post source material without the person's consent
- greeting names the person, then a short body, a close, and the name from the tracker identity; first person when that person did the work
- one primary commit, pull request, or release link; comment when asked; do not close, assign, or reopen unless asked

Related: `security` forbids secrets in issues and owns the session notice when confidential material hits inference.
<!-- pray:ca94e22d -->

<!-- pray:d893ab3d -->
## Claims and testimony

Treat checkable facts, quotations, dates, quantities, and causal statements as claims. Treat author memory and clearly framed interpretation as testimony.

- Inventing scenes, sources, numbers, or quotations is out of scope.
- A link or citation in the text is not verification. The cited passage must support the claim's scope, date, population, and causal strength.
- If a material external claim cannot be checked in this run, mark it unverifiable rather than rounding it to certainty.
- Keep statement, domain, and quantifiers on a formal claim. Freeze quantities from the producing artifact. A later file does not prove an earlier check.
- Run the claims-audit skill when asked to verify, fact-check, or research checkable claims, or when prose under edit states material external facts, quotations, dates, or research summaries.

Related: `writing-prose` covers voice and quality constructs; `engineering-audit` covers code and pipeline behavior; `derivation-audit` covers symbolic derivation when a consumer asks for that job.
<!-- pray:d893ab3d -->

<!-- pray:b1ea9b07 -->
## RFC process

Significant user-facing contract changes start as an RFC. Skip a bugfix, typo, or refactor that leaves those contracts in place.

Claim `rfcs/ids/NNNN` before writing `rfcs/NNNN-slug.md`. Copy `rfcs/0000-template.md`. Omit unused header fields and empty sections. Implementation PRs cite `RFC-NNNN`. Numbering bands, isolation, and extra product tests live in `rfcs/README.md`. Follow the rfc-process skill. Product RFCs specify a design. Version numbers belong in changelogs. Keep existing RFC numbers. RFC titles, registrar names, and identifiers use instrument and protocol words; body and organism metaphors stay out of contracts and code.
<!-- pray:b1ea9b07 -->

<!-- pray:08c294fb -->
## Likely rejected changes

- features whose complexity outweighs user value
- giant refactors
- non-trivial changes without tests
- style-only rewrites without behavior change
- AI-generated-looking code the author does not understand
- compatibility aliases, shims, and legacy input shapes that remain accepted even though the change should remove them
<!-- pray:08c294fb -->

<!-- pray:2543c1cc -->
## Checks before publish (engineering)

- verify the change is wanted; discuss first for unconfirmed larger features
- describe what problem is solved and why it matters
- include tests
- add screenshots or screen recordings for UI changes
- keep one pull request to one concern
- understand any AI-assisted code you submit
- review generated changes before requesting revisions; write review replies from the author's own understanding
<!-- pray:2543c1cc -->

<!-- pray:48e8a6b3 -->
## Collaboration workflow

- record durable project value in the live-work queue, including improvements to shared guidance or a skill;
- keep only decision-bearing material; omit generic notes, copied chat, and filler;
- use the lightest trace that preserves context; design-only work needs no branch unless implementation starts; put that exploration under `docs/plans`;
- follow docs-conventions for `docs/` and `usr/`; when encoding how this tree writes, use infer-conventions.
<!-- pray:48e8a6b3 -->

<!-- pray:ad13bd27 -->
- test coverage must follow @spec/README.md guidelines;
- use ruby and Rails features according to the codebase versions;
- follow ruby and Rails coding conventions, principles, and best practices;
- schema DDL in `db/migrate`; row backfills in `db/data_migrations`; keep `up` and `change` as schema DDL;
- page large backfills with an id cursor or `offset`/`limit`; pass `total_records` and `processed_count`; `enqueue` the next page; when `enqueue` is absent (console `new.perform`), drain remaining pages in the same `perform`;
- log progress as `processed_count/total_records` or `print "."`;
- unpublished in-repo gem: gemspec next to the library, Gemfile `path:` link, version 0.x, metadata `allowed_push_host` empty so gem push fails;
- development and test isolation: declare the path gem in `group :development, :test` so production never requires it;
- generate the public path from the slug column when present, otherwise the primary key; lookup uuid-shaped text as primary key, then slug, then integer primary key only when the schema holds integers; do not put display names in HTTP paths;
- do not override `Model.find` or enable global FriendlyId finders; a `belongs_to` finder must exist on the relation;
- `path(record)` uses `to_param`; path of a raw id string does not;
- person-facing date and datetime fields are text (or a widget bound to the product form), not native `date` or `datetime-local`, when the product locale is not the browser locale;
- ActiveRecord `TimeZoneConverter` converts strings through `in_time_zone` before DateTime cast; a parse prepend on Date or DateTime does not see those strings unless the converter is also prepended; ISO JSON and params stay on the default caster;
- association pickers reuse the existing admin list API with a bounded page; they do not pluck the full table into the form;
- local app host for development, system and request specs, `default_url_options`, OAuth callbacks, Capybara app host, and curl against the running app: `https://<app>.rails.fi` or `http://rails.fi` (`rails.fi` and wildcard `*.rails.fi` resolve to `127.0.0.1`); prefer project `HOST` / `RAILS_HOST`;
- do not use `example.com`, `example.org`, or `example.net` in fixtures, stubs, mocks, allowlists, documentation hosts, or fake email; those names resolve on the public Internet;
- fictional remote host, external-service mock, or fake email that must not hit the local app: `<title>.<hash>` (product label plus short hex, for example `demo.7c3e9a1b` / `user@demo.7c3e9a1b`); keep house fake mail on `@rails.fi` when the address is local-app shaped;
- keep `config.hosts`, Capybara, and mailer host on `*.rails.fi`; remove leftover `example.com` names from local allowlists;

Related: `docs-conventions` names `usr/scripts` for maintainer scripts run from a checkout or CI, `usr/reports` for read-only investigation scripts, `usr/support` for support console scripts that mutate, and `usr/migrate` for schema and data migration scripts for manual running.
<!-- pray:ad13bd27 -->

<!-- pray:6978f200 -->
## Keep the work

- after a refusal, stay on the place they used until they choose the next step: URL, document, screen, command, or query
- the refusal they see must match what the platform reports; HTTP status when the place is a URL
- an error document is internal; show the error on the requested place
- unknown identifiers keep a real refusal; the status that leaves must match
- leave that place only when it is no longer the identifier of the failed attempt: a moved resource (301/308 to the replacement), an auth gate with a return, a successful submit that earned a new place, or a canonical host or scheme
- a successful GET with an alternate identifier for the same record is a moved resource; redirect show to the canonical identifier; leave POST, PATCH, and live frames on the requested place
- keep the failed command or query visible
- redisplay a failed form on the same place; keep passing and failing answers
- a typed value the widget rewrote is a failed answer; redisplay the string the person typed
- a confirm that cannot run is not consent; nested destroy must ask on that place, then stay on that place after cancel or success unless the person earned a new place
- a catalog picker that failed to load still shows the query the person typed
- on an acknowledged forbidden, stay and offer the next human step the product supports; rewrite to sign-in only when credentials are missing
- warn before a session timeout and give time to extend; restore in-progress answers after re-auth
- restore in-progress answers after a process restart the same way as after re-auth
- a live refresh or reconnect must not wipe the work still on the place

Related: `preferred-stack` states the humane design preference; `engineering-audit` asks whether a person can still operate on the place they used after a refusal.
<!-- pray:6978f200 -->

<!-- pray:c3a7a60e -->
## Delivery budgets

Every product that ships a byte-bearing artifact keeps a named size budget and a sibling build-timing signal for the same ship path. Size is delivery burden. Timing is delivery-path cost. Do not mix them in one metric series.

- name each shipped artifact (store upload, container image, path inside an image, gem, wasm, static bundle);
- fail CI when size exceeds an absolute ceiling, or (when a baseline exists) grows beyond the allowed growth over that baseline;
- gate the metric the recipient pays for (installed or on-disk bytes versus transfer or compressed bytes); keep those as distinct series;
- measure wall-clock build or push time for that same ship path; keep timings in a separate report kind;
- every size or timing sample records when and where so samples compare across runners: start and finish timestamps, repository, run identity or URL, workflow, job, step, and runner class;
- name the metric and tool explicitly (file bytes, local image size, registry or manifest compressed size, path disk usage, wall seconds);
- update the baseline in the same change when growth is intentional; a missing baseline may skip the growth check until the first real sample, but the absolute ceiling still applies;
- prefer one gate tool and one budget file shape across artifacts in a repository;
- ingest (dashboard or webhook) is optional infrastructure; the local CI gate is not optional when the product ships.

Skip only for trees that never become shipped bytes (pure specification or prose), and state that reason.

Related: `engineering-audit` resource and budget mode audits utilization during a review; this fragment requires continuous CI budgets on the ship path. `dependency-policy` upgrade resource compare produces before/after deltas on dependency upgrades and may cite the same artifact series. `pressure-benchmarking` owns load ladders, ceiling ratchet, and pressure-cooker optimize/prove episodes. `ci-review` studies run health after the fact. `publish-checks` covers review hygiene, not byte ceilings.
<!-- pray:c3a7a60e -->
