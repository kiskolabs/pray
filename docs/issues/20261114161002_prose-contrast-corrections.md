# Prose and duplication corrections

## Decisions

State the action or contract directly when an introductory denial merely anticipates a misreading. Review each match in context. Preserve negation that defines a prohibition, trust boundary, unsupported operation, condition, or evidence limit, as required by RFC 0001.

The scope is the README, maintained guides, and explanatory RFC prose. Historical investigation records retain their original evidence and qualifications. Shared provisioned instructions remain governed by Prayfile.

The follow-up audit covers duplication within and across the same eleven files. Remove passages that repeat the same information for the same reader task. Topic overlap or an explanation elsewhere in an RFC is insufficient grounds for deletion. Preserve conceptual arguments, reader-oriented summaries, recovery checklists, and worked examples when they serve distinct purposes.

## Effects

Rewrote rhetorical contrasts in the project introduction, naming explanation, distribution summary, repository layouts, admin UI guidance, and explanatory RFC paragraphs. Kept the technical contracts and their normative keywords. The README now states maintainer responsibility directly and retains the limits on inference guarantees and package-code execution.

The README scope heading changed. The follow-up removed the separate packageable-material heading together with other repeated explanations. A repository search found no references to the former headings or anchors.

The user rejected the breadth of the duplication cleanup, naming Philosophy as an example. The audit had conflated repeated topics with repeated information and overstated the case for deleting complete sections. Restored Philosophy, the operating-model diagram, the recovery checklist, the distribution API inventory, served and feedback examples, and the distribution summary's explanatory structure. The philosophical argument and the recovery questions have different reader purposes from feature and field inventories.

Validation: `cargo test --offline -p pray-core --test rfc_ids` passed all 5 tests. `cargo fmt --all --check` and `git diff --check` passed. Runtime code is unchanged; the full runtime suites and Clippy were outside this prose pass.

### Contrary pass

Checked whether removing a denial would erase a contract distinction, strengthen a conditional claim, or remove evidence qualifications. Kept signature and network-fingerprint limits, read-only command guarantees, forbidden package execution, and technical exclusions. The materialization section retains loading as a necessary condition for direct inference effects; the follow-up removed the table that repeated this condition.

The original naming explanation generalized about believers and claimed effects on perception, emotion, judgment, and state of mind without sources. Those claims were unnecessary to explain the name. The replacement frames the analogy as this project's naming intent.

This session both drafted and reviewed the changes. Parent-prompt contamination limits the independence of this contrary pass.

### Claim ledger

- README, naming explanation: claimed religious and psychological effects. Source: the original prose supplied no evidence. Outcome: unverifiable in this pass. Action: removed the empirical claims and retained the project-specific analogy.
- README, materialization and effect: referenced content must be loaded to affect inference directly. Source: the existing distinction between references and loaded input in that section. Outcome: supported as the stated project model. Action: preserved the necessary condition and the qualification about discovery effects.
- README and RFC 0060, derived metadata: annotations leave artifact hash, tree hash, and version identity unchanged. Source: RFC 0060 section 29.3. Outcome: supported as the specified contract. Action: retained all three identity fields in that RFC; the README summarizes the boundary and links to the contract.
- README, rendered output: managed spans can be traced through lock records. Source: RFC 0020 section 32.1 requires span ids, target paths, line bounds, and provenance. Outcome: supported as the specified contract. Action: replaced the blob comparison with that concrete relationship.
- README, responsibility: maintainers review instruction quality and the CLI never executes package code. Source: RFC 0002 section 5 and RFC 0050 Summary. Outcome: supported as the specified scope and prohibition. Action: preserved both responsibilities and the absence of a model-quality guarantee.

### Reader-facing assessment

Assessed the Markdown source for the README and edited guides: headings introduce the subject, paragraphs state their purpose directly, and commands remain available for the documented tasks. These prose edits add no interactive, empty, loading, or error states. Narrow rendered layouts and visual appearance still need a human preview.

### Duplication audit

Reviewed all eleven files, including their examples, reference sections, and cross-references. Consolidated these repeated facts:

- README purpose and input drift: introduction and Why. Removed the repeated problem section. Philosophy retains its account of language as operational input, the reasons for inspectability and reproducibility, and the distinction between reproducible input and probabilistic output. Its repeated paragraph about manual copying remains covered by Why.
- Artifact definitions: README Core model for readers and RFC 0002 section 8 for specification terminology. Removed the second RFC glossary. Retained the README operating-model diagram because it explains the processing stages.
- Lockfile contents and span provenance: README How does Prayfile keep input consistent, with the detailed contract in RFC 0020. Removed its recap list. Retained What is preserved as a recovery checklist that asks what a person should be able to reconstruct.
- Marker syntax: one README example under Render markers. Formatting rendered files links there.
- Verification findings: the README verify section defines the common findings; drift describes its additional renderer and orphan-marker findings.
- Repository shapes: docs/repository-layouts.md owns the working guide and RFC 0117 owns the contract. Merged repeated defaults and standalone-scaffold descriptions. Retained the served and feedback examples because they illustrate consumer workflows rather than repeat command definitions.
- Publishing and feedback requirements: the README owns the recommendation lists. The deployment guide links to them and retains its host, storage, and route requirements.
- Design principles: retained the README overview with a link to RFC 0002 section 2. Restored the complete goals in RFC 0002 because the shortened list obscured the stated priorities. Retained the numbered Final principle heading with a direct statement of its purpose.
- CLI commands and exit codes: RFC 0040 sections 50 and 66. The registrar and introductory text reference those sections.
- Search validation and metadata retrieval: RFC 0060 section 29. Moved the trimmed-summary, missing-metadata, and mismatched-name rules there from RFC 0040; the CLI RFC retains its output-label and concurrency prohibitions.
- Annotation production methods: RFC 0060 section 29.4. Section 29.3 retains the annotation fields and package-identity boundary.
- Workspace boundaries: RFC 0101. Removed the repeated crate inventory in RFC 0070 and replaced the old proposed crate names with the existing module boundary.
- Distribution overview: docs/p2p-s2s-summary.md retains the lifecycle, transfer guarantees, and implementation-coverage explanation, with links to RFCs 0060 and 0104. The README retains the distribution API inventory because the static-tree layout alone does not replace that capability overview.

No product contract, RFC status, RFC number, or executable behavior changed. Removed section headings had no incoming links in the tracked Markdown. A Ruby check supplied on stdin validated local links from the edited files and incoming tracked Markdown links, including fragment targets, with no failures. After consolidation, `cargo test --offline -p pray-core --test rfc_ids` again passed all 5 tests; `cargo fmt --all --check` and `git diff --check` passed.

### Review of lost meaning

The comparison with the original text found additional losses beyond Philosophy. Some came from the initial prose pass, before the duplication cleanup. Restored these meanings:

- Naming rationale: Agentfile suggests a runtime owner, Cookfile suggests execution, and Mantrafile emphasizes repetition. Their comparison explains why the project chose a broader package-lifecycle name. The prayer analogy again covers orientation and the form of language, framed as project intent. The original unsourced claims about believers' psychological responses remain unverifiable and are excluded as empirical assertions.
- Silence: deliberately omitting fragments and choosing which dependencies contribute to each target are part of composition. The shortened copy had preserved exclusion but weakened its explanatory role.
- Artifact authority: the lockfile records accepted input, the cache supports recovery, and rendered files carry influence. The restored prose names their different responsibilities explicitly.
- Feedback scope: acceptance or rejection describes use in a particular context. It does not supply benchmark results, review scores, package ratings, or inference-quality evidence. This is a trust boundary and should have survived the prose pass.
- Minimal editable packages and verification: restored the README's minimal-text package shape, unchanged identity fields, broad zero-trust treatment of metadata and feedback, method independence of annotations, separate package and rendered-byte checks, and verification-record bindings. The RFCs had retained this information; the overview had lost the explanation.
- Reader orientation: restored the consumer/publisher command grouping and the README design-principle overview. Their function differs from a command inventory or the specification's detailed statements.
- Local contract visibility: restored the explicit non-executable-host-language rule in RFC 0010, the required-prayspec label and inert-asset restrictions in RFC 0011, and the rendered-file location and commit-policy explanation in RFC 0070. These had counterparts elsewhere, but the local statements help readers interpret the surrounding rules.

The publishing protection list remains complete in the README and the deployment guide links to it. Search rules removed from RFC 0040 remain in RFC 0060, with a reference from RFC 0040. Exit codes remain complete in RFC 0040 section 66. These are relocations or exact consolidations, rather than losses of their rules.

The original overview also named passkeys and other configured keys as possible package-signing methods. The retained RFC 0050 describes ed25519 signing and reserved future forms; it does not establish those other methods as current package-signing capabilities. This wording needs a separate design-versus-implementation clarification before being restored as a capability claim.

### Markdown and diff review

Coverage: all eleven changed source documents and this record. Applied the change-review criteria to the working-tree diff and parsed the current Markdown with the already installed Commonmarker library. No dependency was added. Markdown structure, rendered HTML, local links, and the diff were assessed; a browser preview on a narrow screen remains unassessed.

The contrary pass found two meaning changes introduced during consolidation. Moving a lockfile reference from the confession requirements into the signed-payload field list implied new signing coverage. Adding local overrides to the list of recorded lockfile contents turned a recovery goal into a storage claim. Restored both original scopes. The earlier statement that contracts had been preserved throughout the cleanup was too strong.

Also restored RFC 0010's local explanation of canonical meaning and whitespace-only changes. Restored RFC 0002 sections 3 and 81, retaining their established numbered headings. The existing README glossary had equated a managed span with its lockfile record; it now distinguishes the rendered region from the record that tracks it.

Formatting changes:

- Labeled 25 previously unlabeled fences as text, manifest, or shell examples. All 86 pre-existing fenced bodies remained byte-identical to the snapshot at the start of this audit.
- Retained the README's two tables and aligned their source columns. Removed unnecessary bold emphasis without removing the terms.
- Marked technical filename patterns as code and replaced decorative Unicode punctuation in prose with ASCII equivalents. Fenced example contents were preserved.
- Raised the three RFC 0060 headings numbered 29.3, 29.4, and 29.6 to siblings of the other reference sections. Their titles and fragment identifiers remain unchanged.
- Replaced the repeated short sentences about input categories with connected prose. The declarative, procedural, stylistic, and inherited categories remain present.

Validation after the formatting pass: `cargo test --offline -p pray-core --test rfc_ids` passed 5 tests, `cargo fmt --all --check` passed, and `git diff --check` passed. Ruby checks supplied on stdin parsed 12 Markdown files, verified 50 local links and fragments, and parsed all 4 fenced JSON examples. A separate comparison confirmed that the signed-payload coverage sentence and recorded-lockfile scope match the committed original after removing Markdown emphasis. These structural checks do not establish every factual claim in the documents.

### Recovery

The committed baseline is 967ec912f1fa7b7d2ec477d71e65a085eec14400. All eleven original source documents were read from that revision and verified byte-for-byte against the recovery snapshot. For example, `git show 967ec912f1fa7b7d2ec477d71e65a085eec14400:README.md` retrieves the original README. These edits have not removed that Git object or rewritten history.

The current edits are uncommitted, and this record is untracked. Git cannot recover an exact intermediate working-tree version unless that version was separately stored. Earlier tool calls describe edits but are not a durable Git version history. A temporary recovery archive captures the start of this audit, the committed originals, and the tracked diff; it also includes this record as a file. Temporary storage needs a separate retained copy for long-term recovery.

No committed source text is irrecoverable because of this change. The original unsupported psychological and signing-method claims also remain available in the baseline, even though the edited prose qualifies or omits them. This review cannot establish whether older, unstored edits existed before the session.

## Next

- Clarify the design-versus-implementation status of the original passkey package-signing sentence.
- Review the Markdown in a browser at a narrow width.

## Source

- [RFC 0001 prose rules](../../rfcs/0001-rfc-process.md#prose)
- [README](../../README.md)
- [Distribution synchronization summary](../p2p-s2s-summary.md)
- [Repository layouts](../repository-layouts.md)
- [Server deployment guide](../serve-platforms.md)
- [RFC 0002](../../rfcs/0002-problem-and-positioning.md)
- [RFC 0020](../../rfcs/0020-resolve-and-lock.md)
- [RFC 0050](../../rfcs/0050-security-and-trust.md)
- [RFC 0060](../../rfcs/0060-distribution.md)
- [RFC 0117](../../rfcs/0117-local-prayers.md)
