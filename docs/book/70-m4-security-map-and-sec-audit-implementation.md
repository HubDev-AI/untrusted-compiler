# 70 M4 Implementation: security_map and sec.audit (Current Slice)

This chapter documents the M4 implementation slice that introduced compiler-emitted security metadata and a first deterministic security audit command.

## Scope delivered
- Added security metadata generation:
  - compiler builds `SecurityMap` from AST call sites and policy context
  - output file: `build/security_map.json`
- Added initial sensitive marker/tag coverage:
  - source/gate/sink/effect/capability/middleware tags for v0 baseline intrinsics and middleware call names
- Added security audit engine:
  - deterministic finding rules (CORS, headers, CSRF/auth coupling, SSRF/internal-net, filesystem, capture/replay, secrets reveal)
  - risk scoring and severity ranking
  - text + JSON report output
- Added CLI command:
  - `ailang sec audit --path <project> [--format text|json] [--fail-on 'risk>=HIGH']`
- Added `@allow(...)` annotation pipeline:
  - parser-compatible source preprocessing strips `@allow(...)` before AST parsing
  - annotation parser validates required fields (`policy`, `bypass`, `reason`, `ticket`, `expires`)
  - expired annotations are rejected during analysis
  - parsed allowlist entries are emitted in `security_map.allows` and surfaced in `sec.audit` exceptions
- Added deterministic allowlist hygiene findings in `sec.audit`:
  - high-risk bypass findings (`SECRETS_REVEAL_ALLOWLISTED`, `INTERNAL_NET_CALL_ALLOWLISTED`)
  - expiry hygiene findings (`ALLOW_EXPIRED`, `ALLOW_EXPIRING_SOON`)
  - exception-volume hygiene finding (`ALLOW_COUNT_HIGH`)
- Added dotted stdlib call support for security/effect metadata paths:
  - parser now supports member-call expressions (`module.fn(...)`)
  - semantic intrinsic/effect/capability checks accept dotted aliases (e.g. `db.exec`, `req.json`, `secrets.reveal`)
  - `security_map` call and middleware tag extraction supports dotted call names
  - intrinsic symbol registry now includes dotted stdlib symbols alongside underscore aliases
- Added semantic sink flow enforcement for secrets/untrusted data:
  - generic wrappers `Secret<T>` and `Untrusted<T>` are recognized in the type catalog
  - log sinks reject `Secret<_>` and `Untrusted<_>` values with origin-aware diagnostics
  - JSON response sinks reject `Secret<_>` and `Untrusted<_>` values with origin-aware diagnostics
  - additional typed sinks now enforce `Secret`/`Untrusted` boundaries:
    - SQL sinks (`db.exec`, `db.queryOne`)
    - URL/network sinks (`httpClient.get`, `httpClient.getInternal`)
    - filesystem sinks (`fs.read`, `fs.write`)
    - header/cookie sinks (`res.setHeader`, `res.addCookie`)
  - error codes actively exercised in semantic fixtures: `E1002`, `E1003`, `E1004`, `E1005`
- Added first-class trust-gate semantics for request boundary APIs:
  - request sources now return explicit untrusted wrappers:
    - `req.body` -> `Untrusted<Bytes>`
    - `req.query`, `req.pathParam`, `req.header` -> `Untrusted<String>`
  - `req.json(schema)` is enforced as an explicit schema gate:
    - missing schema argument is rejected with `E4001`
    - diagnostic explains canonical gate usage and trust-boundary intent
  - semantic fixtures added for:
    - missing schema gate argument rejection
    - untrusted `req.query` flow into SQL sink rejection
- Added typed validator/sanitizer trust-gate contracts:
  - gate outputs are now typed for core safe wrappers:
    - `validate.headerValue` -> `HeaderValue`
    - `sanitize.html` -> `HtmlSafe`
    - `path.under` / `validate.pathUnder` -> `PathSafe`
    - `url.public` -> `PublicUrl`
    - `url.internal` -> `InternalUrl`
  - gates enforce canonical argument contracts:
    - string validation/sanitization gates require first argument `Untrusted<String>`
    - path gate requires `(PathSafe, Untrusted<String>)`
  - semantic fixtures cover valid gate usage and contract violations
- Expanded `security_map` marker coverage for gate/source symbols:
  - callsite tags now include:
    - `gate.header.value`
    - `gate.sanitize.html`
    - `gate.path.under`
    - `gate.url.public`
    - `gate.url.internal`
    - source tags for `req.query`/`req.pathParam`/`req.header`/`req.body`
  - intrinsic symbol registry now includes these gate and source entries for both underscore and dotted stdlib aliases
  - `security_map` tests now assert gate/source call tagging and registry presence
- Added logging/SQL policy ingestion and deterministic audit findings:
  - `Policy` now carries parsed `[logging]` and `[sql]` posture fields
  - parser validation now enforces valid `sql.require_limit_on_select` values (`off|warn|enforce`)
  - `sec.audit` now emits policy-aware findings:
    - `LOG_STRUCTURED_ONLY_DISABLED`
    - `LOG_REMOTE_IP_ENABLED`
    - `LOG_USER_AGENT_ENABLED`
    - `SQL_RAW_ALLOWED_BY_POLICY`
    - `SQL_LIMIT_RULE_DISABLED`
  - policy and audit tests were extended for this coverage
- Added callsite-driven SQL limit hygiene detection:
  - `security_map` now tags SQL sink callsites with `sql.select_without_limit` when a query literal/template call contains `SELECT` and no `LIMIT`
  - detection currently covers direct string SQL and `sql.q("...", ...)` call forms at `db.exec` / `db.queryOne` sinks
  - `sec.audit` now emits `SQL_SELECT_WITHOUT_LIMIT` with deterministic severity mapping:
    - `MEDIUM` when `sql.require_limit_on_select = "warn"`
    - `HIGH` when `sql.require_limit_on_select = "enforce"`
  - tests cover callsite tagging and severity mapping behavior
  - keyword scanning now ignores quoted string segments and SQL comments (`-- ...`, `/* ... */`) to reduce false positives/negatives from naive text matching
- Expanded validator gate coverage beyond single-function mappings:
  - semantic trust-gate enforcement now covers core `validate.*`/`sanitize.*` string gates with required `Untrusted<String>` input contracts
  - intrinsic return typing now includes:
    - `validate.email` -> `Email`
    - `validate.uuid` -> `Uuid`
    - `validate.int64` -> `Int64`
    - `validate.nonEmpty` -> `String`
  - new semantic fixtures cover:
    - invalid `validate.email(String)` rejection
    - valid `validate.email(req.query(...))` flow
  - `security_map` marker coverage now includes validator gate tags:
    - `gate.validate.email`
    - `gate.validate.uuid`
    - `gate.validate.int64`
    - `gate.validate.non_empty`
- Added strict JSON encode schema policy coupling:
  - policy model now carries `json.require_schema_for_encode` (default `true`)
  - semantic trust-gate checks now enforce minimum schema-argument contract for `res.json` in strict mode
  - encoding without schema argument emits `E4004` with explicit strict-mode guidance
  - semantic fixtures now cover:
    - strict-mode rejection for single-argument `res.json(...)` calls
    - valid `res.json(schema, value)` path
  - policy tests now cover toggling `json.require_schema_for_encode`
- Strengthened strict JSON signature enforcement:
  - strict mode now validates full `res.json` signatures, not only minimum arity:
    - `res.json(schema, value)`
    - `res.json(status, schema, value)` where `status` is numeric
  - schema argument is now validated as non-numeric/non-boolean and not `Secret<_>`/`Untrusted<_>`
  - invalid arity, non-numeric status, and invalid schema argument each produce dedicated `E4004` diagnostics
  - JSON sink flow checks now evaluate only the value argument (last payload arg), avoiding schema/status false positives
  - semantic fixtures now cover invalid status type, invalid schema type, invalid arity, and valid status+schema+value form
- Added typed schema-value pairing checks for JSON responses:
  - introduced generic `Schema<T>` type recognition in semantic type catalog
  - when `res.json` receives a typed `Schema<T>` argument, compiler now checks payload compatibility with `T`
  - mismatches produce `E4004` (`json response value does not match schema type`) with expected/actual type notes
  - semantic fixtures now cover:
    - valid typed schema encoding (`Schema<Int>` + `Int` payload)
    - invalid typed schema mismatch (`Schema<Int>` + `String` payload)
- Added callsite argument-role metadata for audit explainability:
  - `security_map.calls[]` now optionally includes `arg_roles`
  - role labels are emitted for core sensitive APIs (for example capability/query/url/schema/value/path)
  - JSON roles adapt to signature form (`schema,value` vs `status,schema,value`)
  - tests now assert role metadata for representative callsites (`res.json`, `db.exec`)
- Added source-origin edge metadata for call arguments:
  - `security_map.calls[]` now optionally includes `origin_edges`
  - each edge captures `arg_index`, canonical origin label (for example `call:req.query`), and source/gate tag context
  - origin tracking currently follows local `let` bindings and propagates through:
    - member/unary wrappers
    - binary wrappers with single-origin or same-origin operands
    - `if`/`match` expressions when branch origins are consistent
    - block-tail expressions with local shadow bindings
    - interprocedural forwarding via iterative function origin summaries (`from-param` and source/gate-tagged return summaries)
  - tests now assert sink-argument origin tracing for:
    - direct `db.exec(DbCap(), raw)` flow from `req.query`
    - composite expression flows (`binary`, `if`, `match`, `block`)
    - forwarding function flows (`queryParam -> passThrough -> db.exec`)
    - deeper forwarding chains (`queryParam -> passThrough -> passthroughTwice -> wrap -> db.exec`)
- Added richer deterministic callsite evidence in `sec.audit` findings:
  - finding evidence now includes bounded `sampleCalls` arrays for representative callsites
  - each sample includes callee, location, argument roles, and available origin-edge metadata
  - enabled finding coverage currently includes:
    - `SQL_SELECT_WITHOUT_LIMIT`
    - `SECRETS_REVEAL_USED`
    - `LOG_STRUCTURED_ONLY_DISABLED`
    - `LOG_REMOTE_IP_ENABLED`
    - `SQL_RAW_ALLOWED_BY_POLICY`
    - `SQL_LIMIT_RULE_DISABLED`
    - `INTERNAL_NET_ENABLED_NO_ALLOWLIST`
    - `FS_ENABLED_NO_BASE_ALLOWLIST`
    - `PUBLIC_REDIRECTS_ENABLED_WITHOUT_REVALIDATION`
    - `CAPTURE_REDACTION_INCOMPLETE`
    - `CAPTURE_ALL_IN_PROD`
    - `REPLAY_EFFECTS_ALLOW`
  - call sampling now supports both single-tag and multi-tag families for deterministic SQL sink aggregation
  - audit tests now assert presence and shape of `sampleCalls` evidence
- Added context-first stdlib capability signature support:
  - semantic capability checks now resolve capability argument positions for both compact and context-first forms:
    - compact examples: `db.exec(cap, query)`, `httpClient.get(cap, url)`
    - context-first examples: `db.exec(ctx, cap, query)`, `httpClient.get(ctx, cap, url)`
  - sink-flow restriction offsets now adapt to call shape, so context/capability arguments are not misclassified as user payloads
  - `security_map` SQL query extraction now supports both `db.exec(cap, query)` and `db.exec(ctx, cap, query)` patterns
  - `security_map` argument-role metadata now emits context-aware roles (`context`, `capability`, `query`, etc.) for extended call forms
  - semantic and security-map fixtures now cover valid and invalid context-first capability paths
- Added typed stdlib symbol resolution for local alias/value-call paths:
  - semantic analyzer now resolves callable aliases (for example `let exec = db.exec; exec(...)`) to canonical stdlib symbols
  - capability/effect/sink checks run against resolved symbols so alias calls preserve security enforcement
  - semantic member-expression handling now avoids false `unknown identifier` diagnostics for intrinsic symbol references in alias bindings
  - `security_map` now resolves alias-invoked callsites to canonical callees and preserves tags/arg-roles/origin-edges
  - semantic and security-map tests now cover alias-invoked intrinsic calls
  - member alias paths are now canonicalized as well (for example `let repo = db; repo.exec(...)` -> `db.exec(...)`)

### Slice Explanation: Strict JSON Encode Signature Checks

#### 1) What it is
This slice adds strict signature validation for `res.json` in semantic analysis when `json.require_schema_for_encode = true`.

#### 2) Why it exists
Arity-only validation still allowed malformed response calls (wrong status type, schema position misuse). This change makes encode paths explicit and auditable at compile time.

#### 3) How it works internally
- `enforce_trust_gate_requirements` now delegates JSON encoding checks to `enforce_json_encode_signature`.
- `enforce_json_encode_signature` enforces accepted signatures and validates:
  - argument count,
  - numeric status for 3-arg form,
  - schema argument safety/type constraints.
- JSON sink flow checks now inspect only the value argument via `json_sink_value_arg_index`.

#### 4) Inputs/outputs and constraints
- Input: semantic call expression for `res.json(...)`.
- Output: either accepted call shape or `E4004` diagnostics.
- Constraints:
  - strict mode controlled by `json.require_schema_for_encode`,
  - accepted signatures are limited to 2-arg and 3-arg forms.

#### 5) Failure modes and diagnostics
- Missing schema argument: `E4004 json response encoding requires explicit schema argument`.
- Invalid argument count: `E4004 json response encoding has invalid argument count`.
- Non-numeric status: `E4004 json response status must be numeric`.
- Invalid schema argument type/taint: `E4004 json response schema argument is invalid`.

#### 6) Example usage
- Valid:
  - `res.json("UserSchema", value)`
  - `res.json(201, "UserSchema", value)`
- Invalid:
  - `res.json(value)` (strict mode)
  - `res.json("200", "UserSchema", value)`
  - `res.json(200, 123, value)`

#### 7) Tradeoffs and next steps
- Current schema argument validation is structural/heuristic, not true schema-type pairing.
- Typed schema pairing now works for `Schema<T>` arguments, but untyped schema descriptors (for example string placeholders) still rely on structural checks.
- Next step is to move from descriptor-style schema arguments to richer typed schema symbols across stdlib APIs.

### Slice Explanation: `security_map` Source-Origin Edge Metadata

#### 1) What it is
This slice adds argument-level origin traces to `security_map` call records via `origin_edges`.

#### 2) Why it exists
Call-level tags alone do not show which argument carried untrusted or gate-derived data into a sink. Origin edges provide deterministic evidence for audit findings and future diagnostics.

#### 3) How it works internally
- `collect_block` now tracks local binding origins in a scoped map.
- `collect_expr` emits `origin_edges` for call arguments by inspecting inferred origins.
- `infer_expr_origin` resolves origins from:
  - tracked identifiers,
  - source/gate-tagged calls (for example `req.query`, `validate.*`, `sanitize.*`),
  - wrapper/control-flow forms (`member`, `unary`, `binary`, `if`, `match`, block tails) when origin can be resolved deterministically.

#### 4) Inputs/outputs and constraints
- Input: function body expressions and local `let` bindings.
- Output: optional `origin_edges` array on each `security_map.calls[]` entry.
- Constraints:
  - flow tracking is structural with summary-based interprocedural forwarding,
  - only source/gate-tagged origins are attached.

#### 5) Failure modes and diagnostics
- This metadata path does not emit compile errors directly.
- If no origin can be inferred for an argument, no edge is emitted for that argument.

#### 6) Example usage
- Source:
  - `let raw = req.query("q");`
  - `db.exec(DbCap(), raw);`
- Emitted edge (conceptual):
  - `{ arg_index: 1, origin: "call:req.query", tags: ["source.http.query"] }`

#### 7) Tradeoffs and next steps
- Current origin inference is intentionally conservative and tag-oriented (origin chain details are flattened in summaries).
- Next step is to extend provenance detail beyond summary tags (for example branch-sensitive call-chain traces).

### Slice Explanation: `sec.audit` Callsite Evidence

#### 1) What it is
This slice enriches audit findings with deterministic `sampleCalls` evidence extracted from `security_map.calls`.

#### 2) Why it exists
Count-only findings hide which callsites triggered risk. Sample call evidence makes findings actionable in CI and review without requiring a separate metadata inspection step.

#### 3) How it works internally
- `run_security_audit` now attaches `sampleCalls` for selected findings.
- `call_samples_for_tag` and `call_samples_for_tags` filter tagged call records and emit bounded deterministic lists.
- each sample includes:
  - callee,
  - source location,
  - optional `argRoles`,
  - optional `originEdges` (when available from security-map origin tracking).

#### 4) Inputs/outputs and constraints
- Input: `SecurityMap.calls[]` plus finding tag id.
- Output: JSON evidence field `sampleCalls`.
- Constraints:
  - sample set is bounded (currently top 5 callsites per finding),
  - ordering follows deterministic AST traversal order.

#### 5) Failure modes and diagnostics
- If no tagged callsites exist, `sampleCalls` is empty or absent based on finding trigger conditions.
- No new compile diagnostics are introduced by this slice.

#### 6) Example usage
- `SQL_SELECT_WITHOUT_LIMIT` evidence now includes callsite objects like:
  - `{ callee: "db.exec", location: { file, line, column }, argRoles: [...] }`
- `SECRETS_REVEAL_USED` evidence now includes reveal callsite samples for triage.

#### 7) Tradeoffs and next steps
- Current sampling is per-tag and static; it does not yet group by module or severity hot spots.
- Next step is to expand sample evidence coverage to additional high-signal findings (for example CORS/security-headers and auth/CSRF coupling posture families).

### Slice Explanation: Context-First Stdlib Capability Signatures

#### 1) What it is
This slice extends capability/effect enforcement and sink indexing to handle both compact and context-first stdlib call signatures.

#### 2) Why it exists
The security stdlib contract models handlers around `Ctx` plus capabilities. Previous capability checks assumed the first argument was always the capability, which misaligned with `ctx, cap, ...` forms.

#### 3) How it works internally
- semantic analyzer now computes capability argument index per intrinsic family and argument count.
- sink-flow start offsets are computed with the same call-shape logic.
- security-map SQL query argument extraction now branches by call form for `db.exec`/`db.queryOne`.
- security-map role mapping now emits context-aware role labels for these forms.

#### 4) Inputs/outputs and constraints
- Input: intrinsic call name + argument list length.
- Output:
  - capability diagnostics pinned to the correct argument index,
  - sink checks applied to payload arguments only,
  - security-map roles/query extraction aligned to call shape.
- Constraints:
  - shape inference is currently arity-based,
  - only known intrinsic families are covered.

#### 5) Failure modes and diagnostics
- missing capability argument still emits `E2003`.
- wrong capability type emits `E2004` and now reports the exact argument position.

#### 6) Example usage
- valid:
  - `db.exec(dbCap, "SELECT ...")`
  - `db.exec(ctx, dbCap, "SELECT ...")`
- invalid:
  - `db.exec(ctx, netCap, "SELECT ...")` -> `E2004` at argument 2.

#### 7) Tradeoffs and next steps
- current mapping is intrinsic-name based and arity-driven.
- next step is to lift this into richer typed stdlib symbol metadata so alias/value-call paths can share the same enforcement.

### Slice Explanation: Typed Symbol Resolution for Alias/Value Calls

#### 1) What it is
This slice adds local callable alias resolution so stdlib intrinsic semantics apply even when calls go through bound values.

#### 2) Why it exists
Without alias resolution, code like `let exec = db.exec; exec(...)` bypasses canonical callee matching, weakening capability/effect/sink checks and audit metadata.

#### 3) How it works internally
- semantic scope now tracks `callable_aliases` alongside type bindings.
- when a `let` binds a callable intrinsic/function symbol, an alias entry is recorded.
- call analysis resolves callee name through alias chains before:
  - intrinsic capability/effect enforcement,
  - sink-flow restriction checks.
- `security_map` mirrors this with alias-aware call collection and origin inference.

#### 4) Inputs/outputs and constraints
- Input: local `let` bindings that reference callable symbols.
- Output: canonical callee resolution for analysis and emitted metadata.
- Constraints:
  - alias resolution is currently local-scope only,
  - resolution is limited to symbols known as intrinsic/function call targets.

#### 5) Failure modes and diagnostics
- capability mismatches still emit `E2004`, now anchored to resolved canonical intrinsic signatures.
- unresolved alias chains fall back to existing unknown-call diagnostics behavior.

#### 6) Example usage
- source:
  - `let exec = db.exec;`
  - `exec(dbCap, "SELECT ...");`
- result:
  - semantic checks treat call as `db.exec(...)`,
  - `security_map` records callee `db.exec` with sink/capability tags.

#### 7) Tradeoffs and next steps
- current alias support is lexical and local, without cross-function symbol-table propagation.
- next step is to extend typed symbol metadata through deeper interprocedural value-call forwarding.

### Slice Explanation: Typed Symbol Resolution for Member Alias Value Calls

#### 1) What it is
This slice extends alias-aware call resolution from direct callable aliases to member value-call aliases such as `let repo = db; repo.exec(...)`.

#### 2) Why it exists
Without member alias canonicalization, capability/effect/sink checks could miss real stdlib sink calls when the callee is reached through a value alias plus member access.

#### 3) How it works internally
- alias inference now accepts typed stdlib namespace stems as alias targets (for example `db`, `fs`, `secrets`).
- callee resolution now supports dotted prefix alias rewrites:
  - if a direct alias exists, it is resolved first.
  - otherwise, for dotted names, the head segment is alias-resolved and the full callee is rebuilt (for example `repo.exec` -> `db.exec`).
- the same resolution path is implemented in both semantic analysis and `security_map` collection/origin inference.

#### 4) Inputs/outputs and constraints
- Input: local value aliases whose callsites are member expressions.
- Output: canonical callee names for enforcement and metadata (`db.exec` instead of `repo.exec`).
- Constraints:
  - resolution remains lexical and local-scope,
  - namespace matching is currently stdlib-family based.

#### 5) Failure modes and diagnostics
- capability mismatches continue to emit `E2004`, now also for member alias forms.
- unresolved member alias paths fall back to existing unknown-call behavior (no false positive sink tagging).

#### 6) Example usage
- valid:
  - `let repo = db;`
  - `repo.exec(dbCap, "SELECT id FROM users");`
- invalid:
  - `repo.exec(netCap, "SELECT id FROM users");` -> `E2004` (expects `DbCap`).

#### 7) Tradeoffs and next steps
- current namespace stem detection is explicit and conservative.
- next step is deeper value-call forwarding across function boundaries and richer callable-value shapes.

### Slice Explanation: Iterative Interprocedural Origin Summaries

#### 1) What it is
This slice upgrades function-origin summarization from a single-pass snapshot to an iterative fixed-point computation so deeper forwarding chains preserve source tags.

#### 2) Why it exists
Single-pass summaries only captured direct/one-hop forwarding. Multi-hop helper chains could drop source-origin tags before reaching sinks, weakening `security_map` evidence.

#### 3) How it works internally
- `build_function_origin_summaries` now:
  - gathers all functions,
  - recomputes summaries iteratively using currently known summaries,
  - stops when no summary changes or when bounded rounds are exhausted.
- summary inference still uses deterministic forms:
  - `FromParam { index }` for pure forwarders,
  - `Tagged { tags }` for source/gate-returning functions.
- call argument origin inference consumes these converged summaries, so sink edges retain source tags across deeper helper stacks.

#### 4) Inputs/outputs and constraints
- Input: full program function set.
- Output: converged summary map used by origin inference.
- Constraints:
  - round count is bounded by function count (deterministic runtime),
  - summaries intentionally carry tags/param indices, not full path traces.

#### 5) Failure modes and diagnostics
- this path does not add compile diagnostics directly.
- if no stable informative summary is inferable, function summary is omitted and downstream edges may be absent.

#### 6) Example usage
- flow:
  - `queryParam()` -> `passThrough()` -> `passthroughTwice()` -> `wrap()` -> `db.exec(...)`
- result:
  - sink argument edge keeps `source.http.query` tag instead of losing origin through helper depth.

#### 7) Tradeoffs and next steps
- summaries remain intentionally compact and avoid path explosion.
- next step is richer call-chain provenance output while keeping deterministic bounded analysis.

## Core architecture
- `policy` remains source of truth for effective security posture and validation.
- `security_map` is generated statically from parsed program calls + policy-derived middleware attrs.
- `sec.audit` consumes policy + `security_map` and computes findings via a deterministic rule table.

## Files added/updated
- `compiler/ailang-core/src/security_map.rs`
- `compiler/ailang-core/src/audit.rs`
- `compiler/ailang-core/src/policy.rs` (expanded policy fields + validation for security_headers/csrf/auth coupling)
- `compiler/ailang-core/src/lib.rs` (module exports + `write_security_map`)
- `compiler/ailang-cli/src/main.rs` (new `sec audit` command)

## Tests added
- `compiler/ailang-core/tests/security_map.rs`
- `compiler/ailang-core/tests/sec_audit.rs`
- extended `compiler/ailang-core/tests/policy.rs` for csrf/auth/cors validations

## Current limitations
- middleware detection currently relies on known callable names (dynamic dispatch and indirect call targets are not yet mapped).
- finding set is intentionally baseline-focused and will expand in M4/M8.

## Next implementation steps
1. Expand typed stdlib symbol metadata from local/member alias calls into deeper interprocedural forwarding paths.
2. Add richer SQL hygiene parsing (full query normalization/AST) for robust handling beyond keyword heuristics.
3. Extend typed schema enforcement beyond `res.json` into broader encode/decode stdlib paths.
4. Extend source-origin tracing from converged tag summaries into richer call-chain provenance details.
5. Expand `sec.audit` sample-call evidence coverage across more finding families.
