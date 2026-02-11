# 72 AILang Editor Tooling and Zed LSP Spec (v0.1)

This chapter defines how AILang editor features should work, including diagnostics, go-to-definition, hover, autocomplete, rename, and security-focused quick fixes.

The design goal is clear:

- keep the language semantics in the compiler
- expose semantics to editors through LSP
- keep editor integration thin and replaceable

## 1. Tooling architecture

Editor support should be shipped as official toolchain packages, not embedded in language syntax/runtime:

1. `ailangc` (compiler frontend library)
2. `ailang` CLI (build/check/fmt/lint/sec audit)
3. `ailang-language-server` (LSP server, compiler-backed)
4. `zed-ailang` extension (thin integration wrapper)
5. `tree-sitter-ailang` grammar (highlighting and structure queries)

This keeps the language spec clean while still delivering a batteries-included developer experience.

## 2. Why separate packages still feel "part of the language"

The compiler already requires span-aware diagnostics and stable error codes. That is the core primitive needed for editor UX:

- red squiggles
- navigation
- code actions
- autocomplete

If the official LSP is powered by the same compiler pipeline used by `ailang check`, editor behavior and CI behavior remain consistent.

## 3. v0.1 component responsibilities

### 3.1 `ailangc`

Must provide:

- AST/HIR with stable node identity and spans
- symbol tables (definitions + references)
- type/effect/security analysis results
- structured diagnostics with optional fix edits

### 3.2 `ailang-language-server`

Must:

- reuse `ailangc` (no duplicate parser/typechecker)
- implement standard LSP requests
- support incremental updates and bounded analysis time
- never execute project code

### 3.3 `zed-ailang`

Must:

- register `.ai` language metadata
- start `ailang-language-server --stdio`
- connect language to tree-sitter grammar and LSP

### 3.4 `tree-sitter-ailang`

Must provide:

- grammar
- highlight queries
- optional outline/indent/textobject queries

## 4. LSP feature set (minimum)

Required in v0.1:

1. diagnostics (`publishDiagnostics`)
2. definition (`textDocument/definition`)
3. implementation (`textDocument/implementation`)
4. references (`textDocument/references`)
5. hover (`textDocument/hover`)
6. completion (`textDocument/completion`)
7. rename (`prepareRename` + `rename`)
8. code actions (`textDocument/codeAction`)

Optional but recommended:

- formatting (`textDocument/formatting`)
- semantic tokens (`textDocument/semanticTokens/*`)

## 5. Compiler service contract

The LSP server should call a compiler service API with these capabilities:

- open/change/close file events
- `analyze()` returning diagnostics and snapshot id
- definition/references/hover/completion/rename operations
- deterministic results per workspace snapshot

Diagnostics model requirements:

- stable code (`E1002`, `E2003`, etc.)
- severity
- exact span
- optional notes and fix edits
- tags for `security`, `taint`, `secret`, `effects`, `policy`

## 6. Security-first editor UX requirements

Because AILang is security-first, editor output should surface trust and secrecy semantics directly:

- hover on `Untrusted<T>` / `Secret<T>` explains blocked flows
- sink diagnostics include source-to-sink trace notes
- code actions suggest safe gates:
  - validate/sanitize for `Untrusted`
  - redact for `Secret`
  - typed sink alternatives for unsafe APIs

Minimum security quick-fix families:

1. insert validation gate
2. insert redaction
3. switch to typed SQL/template path
4. switch to typed header/cookie/value builder path

## 7. Incremental performance and resilience

v0.1 tooling constraints:

- incremental parse/analysis cache per file version
- module dependency invalidation for impacted files only
- bounded request budget (target ~200ms; configurable)
- bounded diagnostics per file

If budget is exceeded:

- return partial diagnostics
- include a tooling info diagnostic marking incomplete analysis

## 8. Tooling safety defaults

LSP server defaults in v0.1:

- no code execution
- no build script execution
- no network access unless explicitly enabled
- no reading outside workspace unless explicitly enabled

Recommended policy/config keys:

```toml
[tooling]
allow_network = false
allow_read_outside_workspace = false
analysis_budget_ms = 200
max_diagnostics_per_file = 200
enable_semantic_tokens = true
enable_code_actions = true
```

## 9. Zed integration requirements (v0.1)

### 9.1 Language metadata

Define language config with:

- `name = "AILang"`
- `grammar = "ailang"`
- `path_suffixes = ["ai"]`
- comments and bracket rules

### 9.2 Extension manifest

Register:

- grammar source (`tree-sitter-ailang`)
- language server entry tied to `AILang`

### 9.3 Extension runtime hook

Start LSP via:

- `ailang-language-server --stdio`

No implicit dependency download in v0.1.

## 10. Acceptance tests (editor tooling)

Compiler/LSP acceptance tests:

1. diagnostics include correct span + stable code
2. go-to-definition resolves call to declaration
3. references returns full set in workspace snapshot
4. rename only succeeds for fully-resolved symbols
5. security sink violation returns tagged security diagnostic with origin note
6. analysis budget enforcement returns bounded response

Zed smoke tests:

1. opening `.ai` starts server and diagnostics flow
2. definition navigation works
3. completion latency is acceptable on typical files
4. hover shows types/effects/security wrappers
5. security code actions are offered when applicable

## 11. Rollout plan

Recommended sequencing:

1. compiler service API stabilization
2. LSP diagnostics + definition + hover
3. completion + references + rename
4. security code actions
5. Zed extension and tree-sitter packaging
6. performance tuning and budget hardening

This sequence keeps tooling consistent with core compiler maturity and avoids building editor UX on unstable semantic outputs.
