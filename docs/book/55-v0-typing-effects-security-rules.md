# 55 v0 Typing and Effects Rules (Security-Critical Core)

This chapter captures the minimal formal rule set that makes AILang security checks enforceable at compile time.

## 1) Core judgments

- `Γ ⊢ e : τ` means expression `e` has type `τ`
- `Γ ⊢ e : τ ! ε` means expression `e` has type `τ` and uses effect set `ε`
- `ε_used(f)` is the set of effects used in function body `f`
- `ε_decl(f)` is the set declared in function signature `f`

Mandatory rule:
- `ε_used(f)` subset of `ε_decl(f)`

## 2) Opaque wrappers

`Untrusted<T>` and `Secret<T>` are opaque and cannot be implicitly converted.

Forbidden implicit conversions:
- `Untrusted<T> -> T`
- `Secret<T> -> T`
- `Secret<T> -> String`
- `Secret<T> -> LogValue`/JSON/string formatting sinks

## 3) Structural shape typing

### 3.1 Record typing
Record fields typecheck independently; record effects are the union of field initializer effects.

### 3.2 Field access
If `Γ ⊢ e : { ..., k: τ, ... } ! ε` then `Γ ⊢ e.k : τ ! ε`.

### 3.3 Width subtyping
A record with extra compatible fields can satisfy a narrower required shape.

### 3.4 Function-valued fields
In v0, function-typed fields inside capability objects must declare effects.

## 4) Effect propagation rules

### 4.1 Intrinsics and stdlib functions
Each intrinsic/API carries a fixed declared effect set.

### 4.2 Function calls
If `f` declares effects `εf`, call expression effects include `εf` plus argument/callee effects.

### 4.3 Capability field calls
Calling function fields on capability objects contributes those declared effects identically to regular calls.

## 5) Trust boundary rules (`Untrusted`)

### 5.1 Sources
Request/environment/raw data APIs produce `Untrusted<_>`.

### 5.2 Gates
Only explicit gate functions can convert untrusted input into trusted/safe types.

### 5.3 Schema decode as canonical gate
`req.json(ctx, req, Schema<T>)` is the primary boundary conversion for request bodies.

### 5.4 Forbidden flows
`Untrusted<_>` values cannot flow directly into typed sinks (`SqlQuery`, `HtmlSafe`, `PublicUrl`, `HeaderValue`, `PathSafe`, etc.).

## 6) Secret handling rules

### 6.1 Sources
Secret providers return `Secret<T>`.

### 6.2 Forbidden contexts
`Secret<_>` cannot be logged, encoded, interpolated, or passed into sensitive sink parameters.

### 6.3 Allowed operations
- `redact(secret)` is safe
- `reveal(secret)` requires effect + capability + policy allowance

## 7) Typed sink constraints

### 7.1 SQL
`db.exec` accepts only `SqlQuery`; SQL template parameters must be allowed scalar DB parameter types and must exclude `Untrusted<_>`/`Secret<_>`.

### 7.2 HTML
`res.html` accepts only `HtmlSafe`.

### 7.3 URLs
`httpClient.get` accepts `PublicUrl`; internal URLs require `InternalUrl` + internal capability/policy.

### 7.4 Headers/Cookies
Headers and cookies must be built from validated typed primitives.

### 7.5 Filesystem
Filesystem APIs accept only validated `PathSafe` values.

## 8) Budget rules

- JSON/body extraction must run under budget constraints.
- API design should require `Ctx`/`Budget` where body decode is used.

## 9) Policy integration

Compiler-enforced policy toggles include:
- forbidden effects
- strict encoding mode
- redirect/internal-network rules
- structured logging requirements

## 10) Minimal typechecker implementation order

1. Core types/functions/records/enums/Option/Result
2. Structural compatibility + intersections
3. Effect declarations + effect propagation
4. Opaque wrapper restrictions (`Untrusted`/`Secret`)
5. Typed sink API constraints
6. Template interpolation restrictions (`sql`/`html`)
7. Schema typing and gate validation
8. Policy enforcement and allowlist annotation checks
