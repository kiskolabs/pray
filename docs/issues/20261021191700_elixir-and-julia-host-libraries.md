# Elixir and Julia host libraries

## Participants

Andrei Makarov

## Decisions

Treat host libraries as three layers: full CLI port (resolve, lock, render), RFC 0106 lock adapter (read Prayfile.lock and check dest spans without resolve), and runtime composition for inference input. Keep resolve and render in the Rust reference CLI or pray_core::embed. Reject a fourth full resolver for Elixir or Julia.

Elixir stays on the existing house path from RFC 0106 and the 20260915130000 / 20260915133000 notes: Mix or Hex only after a real Phoenix or Elixir in-repo caller exists. Prefer spawn of pray verify or pray drift first, then an optional thin Hex package that binds inspect_locked_destinations. Do not open hex/pray-cli as a parity CLI with Ruby and TypeScript.

Julia is not in the preferred stack and has no prior RFC or live-work commitment. Do not add a General-registry or in-tree Julia package until a named consumer and an explicit follow-on decision exist. If that later happens, use the same thin path as Elixir (spawn first, lock inspect second), not a full CLI port.

Python remains in the same host-adapter queue as Elixir (pytest plugin or spawn), ahead of any Julia work when host languages are prioritized.

## Effects

Research pass 2026-10-21. No code, Hex tree, or Julia package was added.

Elixir LLM host demand is real on Hex (langchain and related agent packages) but is not the same as Prayfile packaging demand. Rustler exists for a later NIF bind of the lock-adapter subset only.

Julia has PromptingTools.jl and smaller agent packages; the leading toolkit positions itself as REPL-oriented rather than large-scale host infrastructure. jlrs or ccall can bind Rust later; that path is heavier than Rustler and is not justified without a caller.

Claim outcomes from the pass: project rejection of a fourth resolver and the Elixir Mix-when-Phoenix-asks gate are supported by RFC 0106 and the September host-adapter issues. Elixir Hex LLM activity is supported by hex.pm package pages checked that day. Julia download volume for PromptingTools was not frozen (package stats URL returned 404). Equating Elixir and Julia as the same library job is unsupported.

## Next

Do nothing until a Phoenix or Elixir repo asks for Mix integration. When it does: Mix task that shells out to pray, then optionally a Hex lock-adapter against RFC 0100 span fixtures. Keep PyO3 after RFC 0109 field freeze as the next named bind after Mix if Python callers appear first.

Record a new issue before any Julia tree. Do not start a fourth CLI port for either language.

## Source

rfcs/0002-problem-and-positioning.md
rfcs/0106-host-language-lock-adapter.md
rfcs/0109-embedder-api.md
docs/issues/20260915130000_host-language-lock-adapter.md
docs/issues/20260915133000_render-fixtures-and-lock-adapter-ports.md
README.md (RubyGems install section)
https://hex.pm/packages/langchain
https://github.com/brainlid/langchain
https://github.com/svilupp/PromptingTools.jl
https://julialang.org/assets/2025-julia-user-developer-survey.pdf
https://github.com/JuliaRegistries/General (LLM registration guidelines)
