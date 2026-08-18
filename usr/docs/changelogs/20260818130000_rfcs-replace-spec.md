# RFCs replace the specification snapshot

## Participants

Andrei Makarov

## Decisions

Numbered RFCs are the Prayfile product contract. The former specification snapshot is retired. RFC prose may pass 300 lines when one concern stays coherent. Approaching 300 lines is the cognitive red zone. Hard maximum is 1000 lines. Source-file line limits for Rust, Ruby, and TypeScript stay separate.

Prayfile surface lives in RFC 0010. Prayspec and package archive live in RFC 0011. Lock and resolve live in RFC 0020. Render, markers, verify, and drift live in RFC 0030. Ownership zones live in RFC 0031. CLI, config, environment, and exit codes live in RFC 0040. Security lives in RFC 0050. Static registry lives in RFC 0060. Federation extras stay Experimental in RFC 0104. RFC 0111 records that the snapshot is gone.

## Effects

SPEC.md is deleted. Live operator docs, crate READMEs, GitHub templates, and AGENTS project notes point at rfcs/. Historical traces under usr/docs keep their original snapshot citations. JSON Schema and fixtures still win for field presence until RFC 0100 is Stable.

## Next

Apply RFC 2119 language in contract RFCs when RFC 0100 moves. Rewrite RFC 0010 examples to destination DSL when RFC 0102 is Stable. Decide schema-versus-RFC field presence in that same pass.

## Source

RFC 0001 length and lifecycle. RFC 0111. rfcs/README.md current set.
