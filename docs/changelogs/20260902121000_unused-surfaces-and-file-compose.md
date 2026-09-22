# Unused specified surfaces and file-as-fragment compose

## Participants

Andrei Makarov

## Decisions

RFC 0034 strikes unused adapters, extra export kinds, extra render modes, parsed-but-ignored render fields, and origin-tagged dest delete. RFC 0108 marked compose of a UTF-8 file export, per-destination compose header, and fail-closed compose dest types. Home as --path is a normal project root. Marker dialects stay unclaimed.

## Effects

spec.adapters still parses and is not loaded. template command rule asset bundle match no destination role. render accepts only mode conflict churn header. section_markers and line_endings fail parse. RFC 0031 prune points at RFC 0033. Compose inlines a UTF-8 file export as a marked span. Exclusive file: stays unmarked. Binary file bytes are not copied as a span. Agent context banner defaults on AGENTS.md only. compose header true or false overrides. Compose of JSON binary or unknown type fails and names file: as the unmarked path. Shebang preservation and ids/0032 are not in this pass.

## Next

RFC 0108 is Stable. RFC 0033 stays Experimental until remaining Windows no-follow checks. Do not claim ids/0032 until dialects are scheduled. B5 shebang stays off. B11 reserved RFC 0110 then B12 conformance packs. B9 named slots and B10 sidecars wait until native include is tried.

## Source

RFC 0034
RFC 0108
RFC 0033
usr/docs/issues/20260902104500_home-prayfile-backlog-claims-audit.md
