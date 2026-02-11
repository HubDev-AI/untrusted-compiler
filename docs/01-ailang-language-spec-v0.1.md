# AILang Language Spec v0.1

Source slice: `docs/ailang-requirements.md` lines 174-1253

---
Below is a self-contained **language design spec** you can hand to an AI (or a team) to start implementing an 'AI-first' programming language for **web backends**. It includes goals, syntax, semantics, standard library expectations, tooling, and lots of examples.

---

# AILang (working name) - AI-first backend language spec v0.1

## 0. Summary

AILang is a compiled, statically typed language for web backends designed to maximize:

- **machine-checkability** (types, effects, contracts, schemas),
- **refactor safety** (explicit dependencies, explicit side effects),
- **reliable codegen by AI** (few ambiguous constructs, canonical project layout, strong defaults),
- **production practicality** (fast runtime, great interop, predictable builds, great tooling).

AILang is 'boringly explicit': it reduces 'clever' features in exchange for stronger guarantees and easier verification.

---

## 1) Design goals

### 1.1 Primary goals

1.  **Correctness by construction** - Null-safety, exhaustiveness, explicit error handling.

2.  **Explicit side effects** - Functions declare which effects they use (DB/network/time/fs/log/etc).

3.  **Contracts + invariants** - Pre/post-conditions and invariants are first-class and toolable.

4.  **Web-backend first** - HTTP routing, JSON serialization, validation, DB access patterns are standardized.

5.  **Deterministic builds** - Lockfile, reproducible compilation, canonical formatting.

6.  **AI-operable** - Canonical project structure, unambiguous import rules, minimal magic, rich compiler diagnostics.

### 1.2 Non-goals (v0.1)

- GUI / mobile UI frameworks
- Macro systems that allow arbitrary AST rewriting (too hard to reason about)
- Runtime reflection as a primary mechanism (keep it limited and explicit)
- Turing-complete type-level programming (keep types expressive but bounded)

---

## 2) Language overview

### 2.1 Key features

- Statically typed with **type inference** for locals.
- **No null**: use `Option<T>`.
- **Errors are values**: `Result<T, E>`.
- **Effects** are declared on functions.
- **Pattern matching** is exhaustive by default.
- **Immutability by default**; mutation requires `mut`.
- **Structured concurrency** (async) with explicit effects.
- **Schema-driven JSON** with compile-time derived codecs + runtime validation.

### 2.2 Core types

Built-in:

- `Bool`, `Int`, `Int64`, `Float64`, `Decimal`, `String`, `Bytes`
- `List<T>`, `Map<K,V>`, `Set<T>`
- `Option<T> = Some(T) | None`
- `Result<T,E> = Ok(T) | Err(E)`
- `Time`, `Duration`, `Uuid`

---

## 3) Syntax (informal)

### 3.1 Variables & immutability

```ailang
let x: Int = 10
let y = x + 2      // inferred
mut count = 0
count = count + 1  // only if declared `mut`
```

### 3.2 Functions

```ailang
fn add(a: Int, b: Int) -> Int { a + b }
```

### 3.3 Enums (sum types) + structs (product types)

```ailang
struct User {
  id: Uuid,
  email: Email,
  created_at: Time
}

enum AuthError {
  InvalidCredentials,
  LockedOut,
  RateLimited(retry_after: Duration)
}
```

### 3.4 Match (must be exhaustive)

```ailang
match maybe_user {
  Some(u) => u.email
  None => "unknown@example.com"
}
```

---

## 4) Type system

### 4.1 Option & Result are mandatory

No implicit exceptions. Library code can provide convenience combinators, but core is explicit.

```ailang
fn parse_uuid(s: String) -> Result<Uuid, ParseError>
```

### 4.2 Refined types (validation-backed)

Refined types are *nominal wrappers* plus validators. They compile to the underlying type, but construction requires validation.

```ailang
refined type Email = String where matches_email
refined type NonEmptyString = String where len > 0
refined type Port = Int where 1 <= value && value <= 65535
```

Construction:

```ailang
let email: Result<Email, ValidationError> = Email.try_from(input)
```

### 4.3 Traits (interfaces)

- minimal, explicit

```ailang
trait ToJson {
  fn to_json(self) -> Json
}
```

No ad-hoc implicit coercions. Conversions must be explicit.

---

## 5) Effects system

### 5.1 Built-in effects

- `http.client`
- `db.read`, `db.write`, `db.tx`
- `fs.read`, `fs.write`
- `time.now`
- `random`
- `log`
- `env.read`
- `net`
- `spawn` (concurrency)
- `unsafe` (FFI / escape hatch)

### 5.2 Declaring effects

Functions must declare allowed effects:

```ailang
fn now() effects { time.now } -> Time { ... }

fn load_user(id: Uuid)
  effects { db.read }
  -> Result<User, DbError>
{ ... }
```

### 5.3 Effect subtyping

`db.write` implies `db.read` (optional rule; if enabled, must be in spec).  
Preferred: **no implicit implication** in v0.1 to keep it simple.

### 5.4 Capability passing (optional, but recommended)

Instead of global singletons, you can require capabilities:

```ailang
capability Db
capability Logger

fn handler(req: HttpRequest)
  requires { Db, Logger }
  effects { db.read, log }
  -> HttpResponse
{ ... }
```

This makes dependencies explicit and testable.

---

## 6) Contracts (Design by Contract)

### 6.1 Function contracts

```ailang
fn transfer(from: AccountId, to: AccountId, amount: Money)
  effects { db.read, db.write }
  requires { amount > Money(0), from != to }
  ensures  { balance(from) == old(balance(from)) - amount }
  -> Result<Unit, TransferError>
{ ... }
```

Rules:

- `requires` checked at runtime in debug/test builds (configurable).
- `ensures` checked at runtime in debug/test builds.
- `old(expr)` captures pre-state (only allowed for pure expressions or special 'snapshot' reads).

### 6.2 Struct invariants

```ailang
struct Account {
  id: AccountId,
  balance: Money
} invariant { balance >= Money(0) }
```

---

## 7) Concurrency & async

### 7.1 Async functions

```ailang
async fn fetch_profile(user_id: Uuid)
  effects { http.client }
  -> Result<Profile, HttpError>
{ ... }
```

### 7.2 Structured concurrency

- `spawn` returns a `Task<T>`; tasks must be awaited or joined in scope (no fire-and-forget in v0.1).

```ailang
async fn load_dashboard(id: Uuid)
  effects { db.read, http.client, spawn }
  -> Result<Dashboard, Error>
{
  let t1 = spawn load_user(id)
  let t2 = spawn fetch_profile(id)
  let user = await t1?
  let profile = await t2?
  Ok(Dashboard { user, profile })
}
```

---

## 8) Modules, packages, and project layout

### 8.1 Canonical project layout

```pgsql
my_service/
  ailang.toml
  ailang.lock
  src/
    main.ai
    api/
      routes.ai
      handlers.ai
    domain/
      user.ai
      auth.ai
    db/
      queries.ai
      migrations/
  tests/
    api_tests.ai
```

### 8.2 Imports

Absolute imports only, no wildcards:

```ailang
import domain.user.User
import api.routes.Router
```

### 8.3 Package manifest: `ailang.toml`

```toml
name = "my_service"
version = "0.1.0"
edition = "2026"

[dependencies]
stdlib = "0.1"
postgres = "0.1"
json = "0.1"

[build]
target = "native"
opt = "release"
```

Determinism rules:

- `ailang.lock` pins exact versions + hashes.
- Build is reproducible given the same lockfile.

---

## 9) Web backend standard library surface

### 9.1 HTTP server primitives

Types:

- `HttpRequest { method, path, headers, query, body }`
- `HttpResponse { status, headers, body }`
- `Router`
- `Middleware`

Handler signature:

```ailang
fn handler(req: HttpRequest)
  effects { log, db.read }
  -> Result<HttpResponse, HttpError>
```

### 9.2 Routing DSL (minimal, explicit)

```ailang
fn routes() -> Router {
  Router.new()
    .get("/health", health)
    .post("/users", create_user)
    .get("/users/:id", get_user)
}
```

### 9.3 JSON & schema

Derive JSON codecs:

```ailang
@derive(JsonCodec)
struct CreateUserRequest {
  email: Email,
  name: NonEmptyString
}

@derive(JsonCodec)
struct UserResponse {
  id: Uuid,
  email: Email,
  name: String
}
```

Parsing + validation:

```ailang
fn create_user(req: HttpRequest)
  effects { db.write, log }
  -> Result<HttpResponse, HttpError>
{
  let body = req.json::<CreateUserRequest>()?   // decodes + validates refined types
  let user = user_service.create(body)?         // domain logic
  Ok(HttpResponse.json(201, UserResponse.from(user)))
}
```

### 9.4 Database access (safe by default)

- No raw string SQL concatenation by default.
- Provide `sql` tagged templates or query builder that produces parameterized queries.

Example:

```ailang
fn find_user(id: Uuid)
  effects { db.read }
  -> Result<Option<User>, DbError>
{
  db.query_one<User>(
    sql"SELECT id, email, name, created_at FROM users WHERE id = ${id}"
  )
}
```

Transactions:

```ailang
fn create_user(email: Email, name: NonEmptyString)
  effects { db.tx, db.write }
  -> Result<User, DbError>
{
  db.transaction(fn () -> Result<User, DbError> {
    let id = Uuid.new()
    db.exec(sql"INSERT INTO users (id, email, name) VALUES (${id}, ${email}, ${name})")?
    Ok(User { id, email, name, created_at: time.now()? })
  })
}
```

---

## 10) Error model

### 10.1 No exceptions in user code

Errors are explicit via `Result`.

### 10.2 Error enums encouraged, with automatic HTTP mapping

```ailang
enum CreateUserError {
  EmailTaken,
  Db(DbError),
  Validation(ValidationError)
}

impl ToHttp for CreateUserError {
  fn to_http(self) -> HttpResponse {
    match self {
      EmailTaken => HttpResponse.json(409, { "error": "email_taken" })
      Db(_) => HttpResponse.json(500, { "error": "db_error" })
      Validation(v) => HttpResponse.json(400, v.to_json())
    }
  }
}
```

`?` operator:

- Works on `Result<T,E>` and returns early.
- Does not perform implicit conversions unless an explicit `From` impl exists (and that should be visible/imported).

---

## 11) Formatting and linting (mandatory)

- `ailfmt` is the only formatter; formatting is deterministic.
- `aillint` includes:

- unused imports/vars
- non-exhaustive matches (error, not warning)
- forbidden unsafe APIs in safe modules
- forbidden blocking calls inside async contexts

---

## 12) Testing (built-in)

### 12.1 Unit tests

```ailang
test "email validation" {
  assert(Email.try_from("a@b.com").is_ok())
  assert(Email.try_from("nope").is_err())
}
```

### 12.2 Property tests (first-class)

```ailang
property "json roundtrip for UserResponse" {
  for_all user in gen_user_response() {
    let json = Json.encode(user)
    let decoded = Json.decode<UserResponse>(json)?
    assert(decoded == user)
  }
}
```

### 12.3 HTTP integration tests

Standard harness:

```ailang
test "POST /users creates user" effects { db.write, db.read } {
  let app = TestApp.new(routes())
  let resp = app.post("/users", json({ "email": "x@y.com", "name": "X" }))
  assert(resp.status == 201)
}
```

---

## 13) Interop & FFI

### 13.1 Primary interop target

Pick one for v0.1 to keep it realistic:

- **C ABI** for FFI + link to existing libs.
- Optionally provide a stable interop story with **Rust** or **JVM** later.

### 13.2 Unsafe boundary is explicit

```ailang
unsafe fn sha256(bytes: Bytes) effects { unsafe } -> Bytes
```

The compiler requires an `unsafe` block to call unsafe functions:

```ailang
let h = unsafe { sha256(data) }
```

---

## 14) Compiler & tooling requirements

### 14.1 Compiler phases (implementation guidance)

1.  Parse - AST

2.  Name resolution (imports, modules)

3.  Type checking + inference

4.  Effect checking

5.  Contract lowering (optional runtime checks)

6.  IR generation

7.  Optimization

8.  Codegen (native or VM)

9.  Package build + link

### 14.2 Diagnostics are part of the spec

AILang requires:

- error messages include 'expected vs actual' types
- effect errors explain missing/extra effects
- contract violations show which predicate failed
- match exhaustiveness errors list uncovered cases

### 14.3 Canonical commands

- `ailang build`
- `ailang test`
- `ailang run`
- `ailang fmt`
- `ailang lint`

---

## 15) Full example: minimal service

### 15.1 Domain types

```ailang
refined type Email = String where matches_email
refined type NonEmptyString = String where len > 0

@derive(JsonCodec)
struct CreateUserRequest { email: Email, name: NonEmptyString }

@derive(JsonCodec)
struct UserResponse { id: Uuid, email: Email, name: String }
```

### 15.2 Handler

```ailang
fn create_user(req: HttpRequest)
  effects { db.tx, db.write, db.read, log, time.now }
  -> Result<HttpResponse, HttpError>
{
  let input = req.json::<CreateUserRequest>()?

  let user = user_repo.create(input.email, input.name)?

  Ok(HttpResponse.json(201, UserResponse {
    id: user.id,
    email: user.email,
    name: user.name.to_string()
  }))
}
```

### 15.3 Router + main

```ailang
fn routes() -> Router {
  Router.new()
    .post("/users", create_user)
    .get("/health", fn(_req) -> Result<HttpResponse, HttpError> {
      Ok(HttpResponse.text(200, "ok"))
    })
}

fn main() effects { log, env.read } -> Int {
  let port = Env.get("PORT").unwrap_or("8080").parse_int().unwrap_or(8080)
  HttpServer.serve(port, routes())
  0
}
```

---

## 16) 'AI coding contract' (guidance you can give to an AI)

When generating AILang code, the AI must:

1.  Prefer refined types for external input (request body/query params).

2.  Never use `unsafe` unless explicitly required and justified.

3.  All handlers return `Result<HttpResponse, HttpError>`.

4.  All DB ops are parameterized queries (no string concatenation).

5.  All matches must be exhaustive (compiler enforces).

6.  Every public function must declare effects and (when relevant) requires/ensures.

---

## 17) Open choices to decide early (pick defaults)

To avoid design churn, decide these up front:

1.  **Compilation target**: LLVM-native vs bytecode VM

2.  **Async runtime**: built-in vs library (I recommend built-in for web backend)

3.  **Effect granularity**: few coarse effects vs many fine effects (start coarse)

4.  **SQL approach**: tagged templates vs query builder (tagged templates are simplest)

5.  **Migration story**: built-in migrations folder + CLI integration

---

If you want, I can also produce:

- a **formal grammar** (EBNF) for the syntax,
- a **minimal IR design** for the compiler,
- and a **v0.1 implementation plan** broken into milestones (parser - type checker - effects - simple codegen - http + json).

---

```ebnf
(* AILang v0.1 ' Formal Grammar (EBNF)
   Notes:

- This is a syntax grammar (parsing). Most semantic rules (indentation, types, effects,
     trait bounds, exhaustiveness, refined predicates, etc.) are enforced in later phases. - Whitespace/newlines are generally insignificant except where noted; a semicolon may be
     used as a statement separator, but newlines can also separate statements. - Keywords are reserved.
*)

(****************************************************************)
(* 1. LEXICAL                                                  *)
(****************************************************************)

letter        = "A"'"Z" | "a"'"z" | "_" ;
digit         = "0"'"9" ;
hexDigit      = digit | "A"'"F" | "a"'"f" ;

identStart    = letter ;
identCont     = letter | digit ;
Ident         = identStart , { identCont } ;

UpperIdent    = ("A"'"Z") , { identCont } ;
LowerIdent    = ("a"'"z" | "_") , { identCont } ;

IntLit        = digit , { digit } ;
Int64Lit      = IntLit , "i64" ;
FloatLit      = digit , { digit } , "." , digit , { digit } , [ FloatExp ] ;
FloatExp      = ("e" | "E") , [ "+" | "-" ] , digit , { digit } ;

StringChar    = ? any char except " and \ and newline ? | EscapeSeq ;
EscapeSeq     = "\" , ( "\" | "\"" | "n" | "r" | "t" | "0"
                      | "x" , hexDigit , hexDigit
                      | "u" , hexDigit , hexDigit , hexDigit , hexDigit ) ;
StringLit     = "\"" , { StringChar } , "\"" ;

BytesLit      = "b" , "\"" , { ? byte chars with escapes ? } , "\"" ;

UuidLit       = "uuid" , "\"" , hexDigit,hexDigit,hexDigit,hexDigit,hexDigit,hexDigit,hexDigit,hexDigit,
                          "-", hexDigit,hexDigit,hexDigit,hexDigit,
                          "-", hexDigit,hexDigit,hexDigit,hexDigit,
                          "-", hexDigit,hexDigit,hexDigit,hexDigit,
                          "-", hexDigit,hexDigit,hexDigit,hexDigit,hexDigit,hexDigit,hexDigit,hexDigit,hexDigit,hexDigit,hexDigit,hexDigit , "\"" ;

TimeLit       = "time" , "\"" , { ? RFC3339-like timestamp chars ? } , "\"" ;
DurationLit   = "dur" , "\"" , digit , { digit } , ( "ms" | "s" | "m" | "h" | "d" ) , "\"" ;

BoolLit       = "true" | "false" ;
NullLit       = "null" ;   (* parsed but rejected in semantic phase; prefer Option *)

CommentLine   = "//" , { ? not newline ? } ;
CommentBlock  = "/*" , { ? any, including newlines, but not closing */ ? } , "*/" ;

WS            = { " " | "\t" | "\r" | "\n" | CommentLine | CommentBlock } ;

(****************************************************************)
(* 2. PROGRAM STRUCTURE                                        *)
(****************************************************************)

Program       = WS , { Item , WS } ;

Item          = ImportDecl
              | ModuleDecl
              | StructDecl
              | EnumDecl
              | TraitDecl
              | ImplDecl
              | CapabilityDecl
              | RefinedTypeDecl
              | ConstDecl
              | FnDecl
              | TestDecl
              | PropertyDecl
              ;

(****************************************************************)
(* 3. MODULES & IMPORTS                                        *)
(****************************************************************)

ImportDecl    = "import" , WS , ImportPath , [ WS , "as" , WS , Ident ] , Terminator ;

ImportPath    = Ident , { "." , Ident } ;

ModuleDecl    = "module" , WS , Ident , WS , "{" , WS , { Item , WS } , "}" ;

(****************************************************************)
(* 4. TYPES                                                    *)
(****************************************************************)

Type          = FuncType
              | TypeAtom
              ;

FuncType      = "fn" , WS , "(" , [ WS , TypeList , WS ] , ")" ,
                WS , "->" , WS , Type ;

TypeList      = Type , { WS , "," , WS , Type } ;

TypeAtom      = TypeName , [ WS , TypeArgs ] , [ WS , TypeRefinement ] ;

TypeName      = Ident , { "." , Ident } ;

TypeArgs      = "<" , WS , Type , { WS , "," , WS , Type } , WS , ">" ;

(* Optional syntactic sugar for refined types in annotations; semantic phase validates. *)
TypeRefinement = "where" , WS , PredicateExpr ;

(****************************************************************)
(* 5. DECLARATIONS                                             *)
(****************************************************************)

ConstDecl     = "const" , WS , Ident , WS , ":" , WS , Type ,
                WS , "=" , WS , Expr , Terminator ;

RefinedTypeDecl
             = "refined" , WS , "type" , WS , Ident , WS , "=" , WS , Type ,
               WS , "where" , WS , PredicateExpr , Terminator ;

CapabilityDecl
             = "capability" , WS , Ident , Terminator ;

StructDecl    = [ Attributes ] , "struct" , WS , Ident , WS ,
                "{" , WS , [ FieldList ] , WS , "}" ,
                [ WS , InvariantClause ] , Terminator? ;

FieldList     = Field , { WS , "," , WS , Field } , [ WS , "," ] ;

Field         = Ident , WS , ":" , WS , Type ;

InvariantClause
             = "invariant" , WS , BlockPredicate ;

EnumDecl      = [ Attributes ] , "enum" , WS , Ident , WS ,
                "{" , WS , [ VariantList ] , WS , "}" , Terminator? ;

VariantList   = Variant , { WS , "," , WS , Variant } , [ WS , "," ] ;

Variant       = Ident , [ WS , VariantPayload ] ;

VariantPayload
             = "(" , WS , ParamList , WS , ")" ;

TraitDecl     = "trait" , WS , Ident , [ WS , GenericParams ] , WS ,
                "{" , WS , { TraitMember , WS } , "}" ;

TraitMember   = FnSig , Terminator ;

ImplDecl      = "impl" , WS , [ GenericParams , WS ] ,
                TypeName , WS ,
                ( "for" , WS , TypeName , WS )? ,
                "{" , WS , { ImplMember , WS } , "}" ;

ImplMember    = FnDecl
              | ConstDecl
              ;

(****************************************************************)
(* 6. FUNCTIONS                                                *)
(****************************************************************)

FnDecl        = [ Attributes ] ,
                [ "async" , WS ] ,
                "fn" , WS , Ident ,
                [ WS , GenericParams ] ,
                WS , "(" , WS , [ ParamList ] , WS , ")" ,
                [ WS , FnClauses ] ,
                WS , "->" , WS , Type ,
                WS , Block ;

FnSig         = [ "async" , WS ] ,
                "fn" , WS , Ident ,
                [ WS , GenericParams ] ,
                WS , "(" , WS , [ ParamList ] , WS , ")" ,
                [ WS , FnClauses ] ,
                WS , "->" , WS , Type ;

ParamList     = Param , { WS , "," , WS , Param } , [ WS , "," ] ;

Param         = Ident , WS , ":" , WS , Type ;

GenericParams = "<" , WS , GenericParam , { WS , "," , WS , GenericParam } , WS , ">" ;

GenericParam  = Ident , [ WS , ":" , WS , TraitBounds ] ;

TraitBounds   = TypeName , { WS , "+" , WS , TypeName } ;

FnClauses     = { WS , FnClause } ;

FnClause      = EffectsClause
              | RequiresClause
              | EnsuresClause
              | RequiresCapsClause
              ;

EffectsClause = "effects" , WS , "{" , WS , EffectList? , WS , "}" ;
EffectList    = Effect , { WS , "," , WS , Effect } , [ WS , "," ] ;
Effect        = Ident , { "." , Ident } ;

RequiresClause
             = "requires" , WS , BlockPredicate ;

EnsuresClause = "ensures" , WS , BlockPredicate ;

RequiresCapsClause
             = "requires" , WS , "{" , WS , CapList? , WS , "}" ;

CapList       = Ident , { WS , "," , WS , Ident } , [ WS , "," ] ;

BlockPredicate
             = "{" , WS , PredicateExpr , WS , "}" ;

(****************************************************************)
(* 7. ATTRIBUTES (METADATA / DERIVES)                           *)
(****************************************************************)

Attributes    = { Attribute , WS } ;

Attribute     = "@" , Ident , [ WS , "(" , WS , AttributeArgs? , WS , ")" ] ;

AttributeArgs = AttributeArg , { WS , "," , WS , AttributeArg } , [ WS , "," ] ;
AttributeArg  = Ident , [ WS , "=" , WS , Literal ] | Literal ;

(****************************************************************)
(* 8. TESTS                                                    *)
(****************************************************************)

TestDecl      = "test" , WS , StringLit ,
                [ WS , EffectsClause ] ,
                WS , Block ;

PropertyDecl  = "property" , WS , StringLit ,
                [ WS , EffectsClause ] ,
                WS , Block ;

(****************************************************************)
(* 9. STATEMENTS & BLOCKS                                      *)
(****************************************************************)

Block         = "{" , WS , { Stmt , WS } , "}" ;

Stmt          = LetStmt
              | AssignStmt
              | ReturnStmt
              | IfStmt
              | WhileStmt
              | ForStmt
              | MatchStmt
              | ExprStmt
              | BreakStmt
              | ContinueStmt
              ;

Terminator    = WS , ( ";" | "\n" ) ;

LetStmt       = ( "let" | "mut" ) , WS , Ident ,
                [ WS , ":" , WS , Type ] ,
                [ WS , "=" , WS , Expr ] ,
                Terminator ;

AssignStmt    = LValue , WS , "=" , WS , Expr , Terminator ;

LValue        = Ident , { WS? , "." , WS? , Ident | WS? , "[" , WS , Expr , WS , "]" } ;

ReturnStmt    = "return" , [ WS , Expr ] , Terminator ;

BreakStmt     = "break" , Terminator ;
ContinueStmt  = "continue" , Terminator ;

ExprStmt      = Expr , Terminator ;

(****************************************************************)
(* 10. CONTROL FLOW                                            *)
(****************************************************************)

IfStmt        = "if" , WS , Expr , WS , Block ,
                { WS , "else" , WS , "if" , WS , Expr , WS , Block } ,
                [ WS , "else" , WS , Block ] ;

WhileStmt     = "while" , WS , Expr , WS , Block ;

ForStmt       = "for" , WS , Ident , WS , "in" , WS , Expr , WS , Block ;

MatchStmt     = "match" , WS , Expr , WS , "{" , WS ,
                MatchArm , { WS , MatchArm } ,
                WS , "}" ;

MatchArm      = Pattern , WS , "=>" , WS , ( Block | Expr ) , Terminator? ;

(****************************************************************)
(* 11. EXPRESSIONS                                             *)
(****************************************************************)

Expr          = LambdaExpr ;

LambdaExpr    = [ "fn" , WS ] ,
                ( "(" , WS , [ ParamList ] , WS , ")" | Ident ) ,
                WS , "=>" , WS , Expr
              | OrExpr ;

OrExpr        = AndExpr , { WS , "||" , WS , AndExpr } ;
AndExpr       = EqExpr  , { WS , "&&" , WS , EqExpr } ;

EqExpr        = RelExpr , { WS , ( "==" | "!=" ) , WS , RelExpr } ;

RelExpr       = AddExpr , { WS , ( "<" | "<=" | ">" | ">=" ) , WS , AddExpr } ;

AddExpr       = MulExpr , { WS , ( "+" | "-" ) , WS , MulExpr } ;

MulExpr       = UnaryExpr , { WS , ( "*" | "/" | "%" ) , WS , UnaryExpr } ;

UnaryExpr     = { ( "!" | "-" | "+" | "await" ) , WS } , PostfixExpr ;

PostfixExpr   = PrimaryExpr , { WS? , PostfixOp } ;

PostfixOp     = CallOp
              | MemberOp
              | IndexOp
              | TryOp
              ;

CallOp        = "(" , WS , [ ArgList ] , WS , ")" ;
ArgList       = Arg , { WS , "," , WS , Arg } , [ WS , "," ] ;
Arg           = [ Ident , WS , ":" , WS ] , Expr ;

MemberOp      = "." , Ident ;

IndexOp       = "[" , WS , Expr , WS , "]" ;

TryOp         = "?" ;

PrimaryExpr   = Literal
              | IdentPath
              | "(" , WS , Expr , WS , ")"
              | BlockExpr
              | IfExpr
              | MatchExpr
              | StructInit
              | ListLit
              | MapLit
              | SetLit
              ;

IdentPath     = Ident , { "." , Ident } ;

BlockExpr     = Block ;

IfExpr        = "if" , WS , Expr , WS , Block ,
                { WS , "else" , WS , "if" , WS , Expr , WS , Block } ,
                [ WS , "else" , WS , Block ] ;

MatchExpr     = "match" , WS , Expr , WS , "{" , WS ,
                MatchArm , { WS , MatchArm } ,
                WS , "}" ;

StructInit    = IdentPath , WS , "{" , WS , [ FieldInitList ] , WS , "}" ;
FieldInitList = FieldInit , { WS , "," , WS , FieldInit } , [ WS , "," ] ;
FieldInit     = Ident , WS , ":" , WS , Expr ;

ListLit       = "[" , WS , [ ExprList ] , WS , "]" ;
ExprList      = Expr , { WS , "," , WS , Expr } , [ WS , "," ] ;

MapLit        = "map" , WS , "{" , WS , [ MapEntryList ] , WS , "}" ;
MapEntryList  = MapEntry , { WS , "," , WS , MapEntry } , [ WS , "," ] ;
MapEntry      = Expr , WS , ":" , WS , Expr ;

SetLit        = "set" , WS , "{" , WS , [ ExprList ] , WS , "}" ;

(****************************************************************)
(* 12. LITERALS                                                *)
(****************************************************************)

Literal       = Int64Lit
              | IntLit
              | FloatLit
              | StringLit
              | BytesLit
              | BoolLit
              | UuidLit
              | TimeLit
              | DurationLit
              | NullLit
              ;

(****************************************************************)
(* 13. PATTERNS (MATCH)                                        *)
(****************************************************************)

Pattern       = WildcardPat
              | LiteralPat
              | IdentPat
              | QualifiedPat
              | TuplePat
              | StructPat
              | EnumPat
              | OrPat
              ;

WildcardPat   = "_" ;
LiteralPat    = Literal ;

IdentPat      = Ident ;

QualifiedPat  = IdentPath ;   (* for enum variants without payload *)

TuplePat      = "(" , WS , Pattern , { WS , "," , WS , Pattern } , WS , ")" ;

StructPat     = IdentPath , WS , "{" , WS , [ PatFieldList ] , WS , "}" ;
PatFieldList  = PatField , { WS , "," , WS , PatField } , [ WS , "," ] ;
PatField      = Ident , [ WS , ":" , WS , Pattern ] ;

EnumPat       = IdentPath , WS , "(" , WS , [ PatternList ] , WS , ")" ;
PatternList   = Pattern , { WS , "," , WS , Pattern } , [ WS , "," ] ;

OrPat         = PatternNoOr , { WS , "|" , WS , PatternNoOr } ;
PatternNoOr   = WildcardPat | LiteralPat | IdentPat | QualifiedPat | TuplePat | StructPat | EnumPat ;

(****************************************************************)
(* 14. PREDICATES (CONTRACTS / REFINED TYPES)                   *)
(****************************************************************)

PredicateExpr = PredOr ;
PredOr        = PredAnd , { WS , "||" , WS , PredAnd } ;
PredAnd       = PredNot , { WS , "&&" , WS , PredNot } ;
PredNot       = [ "!" , WS ] , PredAtom ;

PredAtom      = "(" , WS , PredicateExpr , WS , ")"
              | PredCompare
              | PredCall
              | PredIdent
              | PredLiteral
              ;

PredCompare   = PredValue , WS , ( "==" | "!=" | "<" | "<=" | ">" | ">=" ) , WS , PredValue ;
PredValue     = Expr ;   (* reuse expression grammar; semantic phase enforces purity where needed *)

PredCall      = IdentPath , WS , "(" , WS , [ ArgList ] , WS , ")" ;
PredIdent     = IdentPath ;
PredLiteral   = Literal ;

(****************************************************************)
(* 15. KEYWORDS (RESERVED)                                     *)
(****************************************************************)

(* Reserved keywords include:
   import, module, struct, enum, trait, impl, for,
   refined, type, where, invariant, capability,
   fn, async, effects, requires, ensures,
   let, mut, return, if, else, while, for, in, match,
   break, continue, await, unsafe,
   test, property,
   true, false, null, const
*)
```

If you want, I can also provide:

- a **tokenization spec** (operators, precedence table, newline/semicolon rules),
- a **fully unambiguous** grammar (LL(1)/LR-friendly) with precedence encoded,
- or a **reference parser skeleton** (e.g., in Rust/Go) that matches this EBNF.

---
