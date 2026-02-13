# Untrusted<T> Backend, Runtime, and ABI

## Option A - Emit C (then call `clang`)

### What it is

Your compiler generates `.c` (and maybe `.h`) files from MIR, then shells out to `clang` to produce an executable.

### Why it's great for v0.1

- **Fastest path to 'it runs'**: you avoid writing a real machine-code backend.
- You get mature optimizations/debugging 'for free' (via `clang`).
- Easy to link with C libraries for HTTP/JSON/DB (or with a Rust runtime via C ABI shims).

### What you must nail (ABI decisions)

You'll define C structs that represent Untrusted<T> runtime types:

**Strings**

- Simplest ABI: `struct { uint8_t* ptr; size_t len; }` (no ownership tracking initially).
- Better ABI: add `cap` + allocator, and define 'owned string' vs 'string slice'.

**Option/Result**

- Tag + payload:

- `Option<T> = { bool is_some; T value; }` (for trivial T)
- General: `uint8_t tag; union { T some; }`
- `Result<T,E>`: `uint8_t tag; union { T ok; E err; }`

**Enums**

- `uint32_t tag; union payload;` where payload is the max-sized variant struct.

**Async**

- Hardest part. If you want async in v0.1, you either:

  1.  **Cheat**: implement `async` as 'blocking' calls and ignore concurrency initially, or

  2.  Compile async into explicit state machines (big lift), or

  3.  Push async into a runtime (Rust/C) and make Untrusted<T> `async` sugar call runtime futures.

### HTTP/JSON bindings

With C emission, you typically:

- Link a C HTTP server library (e.g., `libuv` + http parsing, or `civetweb`, etc.)
- Or embed a Rust runtime and expose a C ABI: `serve(router_fn_ptr, port)`.

**Best v0.1 strategy with C emission**

- Keep Untrusted<T> 'handlers' as normal functions.
- Let the runtime parse HTTP/JSON and call into handler functions with pre-decoded structs.

### Debugging story

- Great: you can generate C with line directives so stack traces map back.
- Or debug at C level early.

### Downsides

- You're now maintaining a 'C backend dialect' (codegen quirks, UB pitfalls).
- Memory management can get tricky fast (strings, lists).
- Async is awkward unless you lean heavily on runtime support.

---

## Option B - Cranelift (JIT/AOT native codegen)

### What it is

You generate native code directly from MIR using **Cranelift** (a mature codegen library).

### Why it's attractive

- Still much less work than LLVM backend work.
- You get native performance and more direct control.
- ABI can be designed cleanly around your language, not C's limitations.

### What you must build

- A codegen layer: MIR
- Cranelift IR
- object/executable (AOT).
- A runtime (still needed): strings, heap alloc, collections, panic/errors, async executor.

### ABI decisions are similar, but you control more

You still need stable layouts for:

- String: pointer+len+cap and ownership model
- Enum layout: tag + union payload
- Calling convention: how functions pass/return aggregates (by value vs pointer)
- Async: likely compile to state machines or integrate a future-like runtime

### HTTP/JSON bindings

Often easiest here is:

- Implement stdlib modules in Rust (runtime crate),
- Expose them to generated code via an FFI boundary (but internal, not 'unsafe' to user),
- JSON derive can generate calls into runtime encode/decode.

### Debugging story

- Harder than C initially (unless you do DWARF, etc.).
- But you can add 'MIR interpreter' for dev/testing early.

### Downsides

- More upfront engineering than C emission.
- You'll need a runtime and object linking story earlier.

---

## How this choice changes MIR + runtime ABI

### If you emit C

**MIR should be 'C-friendly':**

- Prefer lowering complex expressions into simple three-address code.
- Avoid returning big structs by value early; use 'out pointers':

- `fn foo(...) -> T` becomes `void foo(..., T* out)`
- Make `Result`/`Option` explicit structs in MIR lowering so C code is straightforward.
- Pattern match lowering becomes `switch(tag)` in C.

**Runtime ABI tends to be 'C-ish':**

- Everything is plain structs, passed by pointer a lot.
- Ownership is manual or reference-counted early (simple but slower).

### If you use Cranelift

**MIR can be more 'SSA-native':**

- You can keep 'return by value' for small aggregates.
- You can model `Result` and `Option` as tuples `(tag, payload)` in registers.
- You can add specialized intrinsics (e.g., enum tag extraction) cleanly.

**Runtime ABI can be more 'language-first':**

- Define your own calling conventions and keep them stable.
- Easier to add escape analysis / stack allocation later.

---

## Practical recommendation for Untrusted<T> v0.1 (web backend)

If your main goal is **'get a web server + JSON endpoints working fast'**:

### Recommend: **Emit C** for v0.1

Because:

- You can stand up HTTP/JSON by linking known-good libs quickly.
- You can get something running without spending weeks on a backend.
- Most early bugs will be in parsing/type/effects anyway, not codegen speed.

**BUT**: I'd scope async down in v0.1:

- Keep `async` syntax but implement it as:
- either a compile-time error (not supported yet), or
- blocking calls with `await` allowed but meaningless (not great), or
- 'async handlers' are runtime-managed and your compiler just wires them.

Then in v0.2/v0.3, switch to Cranelift or keep C until stable.

---

## What 'tailoring MIR + ABI' means concretely

Once you pick the backend, we lock decisions like:

1.  **Value passing** - C backend: 'everything by pointer' early - Cranelift: 'small values in registers, big by pointer'

2.  **Enum/Result layout** - C: explicit tagged unions - Cranelift: tag + payload tuple or packed struct

3.  **String ownership** - For C: pick either borrowed slices everywhere + explicit `String` allocation functions - For Cranelift: you can do the same but with more safety checks in runtime

4.  **Async model** - C: runtime-managed futures, or defer async - Cranelift: state-machine lowering is more feasible

5.  **Stdlib boundary** - C: link C libs directly, or call into Rust runtime via C ABI - Cranelift: usually Rust runtime with well-defined intrinsics/FFI

---

## If you don't want to choose yet (default path)

You can design MIR so it supports both:

- Keep MIR 'structured' (blocks/terminators) and types explicit
- Add a backend interface:
- `emit_c(mir)`
- `emit_cranelift(mir)`
- Start with `emit_c`, keep MIR stable, later add `emit_cranelift`.

---

If you tell me your preference (or just 'fastest to a running web service'), I'll pin down:

- the exact ABI structs for `String`, `Option`, `Result`, `Enum`,
- the function calling convention rules,
- and a concrete mapping from MIR ops to C (or to Cranelift), plus the minimal runtime API you need for HTTP+JSON.

---

Here's a **backend-neutral MIR** design that can target **either** (A) emitted C or (B) Cranelift/native later, without redesign. The trick is to make MIR:

1.  explicit about control flow and data,

2.  explicit about memory locations vs values, and

3.  explicit about ABI decisions **only at the very last step** (backend lowering).

---

## 0) Design principles (what keeps it portable)

### A. Two kinds of data: **Value** vs **Place**

- **Value**: something you can compute and pass around (register-like).
- **Place**: an addressable storage location (local slot, field, deref, index).

This matches both:

- C (lvalues/rvalues), and
- Cranelift (SSA values + stack slots).

### B. Calls are 'high-level', returns are abstract

MIR can say 'this function returns a `T` value', even if:

- C backend lowers it to `void f(..., T* out)`,
- Cranelift returns it in registers/stack per ABI.

So MIR doesn't commit to sret/outptr rules; the backend does.

### C. Aggregate operations are explicit

Rather than letting backends infer how to build/tear down structs/enums, MIR has:

- `Aggregate::Struct`, `Aggregate::Enum`, `ProjectField`, `Discriminant`, `DowncastVariant`, etc.

### D. Keep 'effects' and 'try/await' as first-class ops

Because they impact analysis (effects checker) and lowering (async / error propagation).

---

## 1) MIR module structure

### 1.1 Core

```text
MirModule { functions: [MirFunction], types: TypeTable, consts: ConstTable }

MirFunction {
  name: Symbol,
  sig: FnSig,                 // types + async flag
  declared_effects: EffectSet,
  locals: [LocalDecl],        // includes params as locals[0..n)
  blocks: [BasicBlock],
  debug: DebugInfo?           // spans/source maps
}
```

### 1.2 Locals and types

```text
LocalDecl { ty: TypeId, name?: Symbol, kind: Param | Temp | UserLocal }

TypeId refers into a type table (monomorphized for v0.1)
Types include:

- primitives, structs, enums, tuples
- Option/Result as normal enums
- List/Map as runtime-known structs
```

---

## 2) The big three: Place, Operand, Rvalue

### 2.1 Place (addressable storage)

A Place is a 'path' to memory:

```text
Place =
  | Local(LocalId)
  | Projection { base: Place, elem: ProjectionElem }

ProjectionElem =
  | Field(FieldIdx)                 // struct field
  | Index(Operand)                  // list/map indexing (may lower to call)
  | Deref                           // for refs/pointers (later)
  | Downcast(VariantIdx)            // enum payload view (after checking tag)
```

This maps well:

- C: `x.field`, `x[i]`, `*p`, `(payload*)`
- Cranelift: stack slot + offsets + loads/stores

### 2.2 Operand (a value you can use)

```text
Operand =
  | Copy(Place)
  | Move(Place)
  | Const(ConstValue)
```

(Yes, `Copy/Move` from a Place - again portable.)

### 2.3 Rvalue (a computation that yields a value)

```text
Rvalue =
  | Use(Operand)
  | UnaryOp(op, Operand)
  | BinaryOp(op, Operand, Operand)
  | Aggregate(AggregateKind, [Operand])
  | Cast(Operand, TypeId)

  | Discriminant(Place)                 // get enum tag
  | ProjectVariant { base: Place, variant: VariantIdx } // produces a Place (see below)

  | Call { callee: Callee, args: [Operand], ret_ty: TypeId, effects: EffectSet, async: bool }
  | Await(Operand)                       // await a Task/Future value
  | Intrinsic(Intrinsic, [Operand])      // json/db/sql/log/etc
```

**Note:** `ProjectVariant` is special because it yields a *place* (payload address). You can implement it as:

- either an Rvalue that returns a 'fat pointer' place, or
- a statement `SetDiscriminant` + `Downcast`.

I recommend: keep `Downcast` as a Place projection, and do:

- `Discriminant(place)` - tag value
- `Downcast` projection allowed only in blocks where tag is proven (match lowering ensures this).

---

## 3) Statements and Terminators

### 3.1 Statements (no control flow)

```text
Statement =
  | Assign(Place, Rvalue)
  | StorageLive(LocalId) | StorageDead(LocalId)     // optional
  | SetDiscriminant(Place, VariantIdx)              // constructing enums
  | Assert(Predicate, AssertKind)                   // contracts/debug
  | Nop
```

### 3.2 Terminators (control flow)

```text
Terminator =
  | Goto(BlockId)
  | Return(Operand?)
  | CondBr { cond: Operand, then_bb: BlockId, else_bb: BlockId }
  | Switch { discr: Operand, targets: [(ConstValue, BlockId)], otherwise: BlockId }

  | CallTerm { dest: Option<Place>, call: CallLike, next: BlockId }
  | TryTerm  { res: Operand, ok_bb: BlockId, err_bb: BlockId }   // Result-based `?`
  | Unreachable
```

Where `CallLike` is the same fields as `Rvalue::Call` but placed in terminator form to avoid 'call in the middle' complexities.

**Why both `CallTerm` and `Assign(... Call ...)`?**  
Pick one: in practice, compilers put calls in terminators because they can branch (panic/unwind/async) and it simplifies SSA later. For backend neutrality, I'd standardize on **CallTerm**.

---

## 4) Portable representation of structs/enums/Result/Option

### 4.1 Struct construction

```text
Assign(Local(x), Aggregate(Struct(User), [id, email, name, created_at]))
```

Backend lowering:

- C: literal struct init or temp + field stores
- CLIF: `stack_slot` stores or aggregate values

### 4.2 Enum construction (including Option/Result)

Enums are:

- `VariantIdx` + payload operands
- layout decided in backend (tag + union)

MIR:

```text
SetDiscriminant(place, VariantIdx::Some)
Assign(place.downcast(Some).field(0), Use(value))
```

### 4.3 Matching on enums

Lower `match e { Some(x)=>..., None=>... }` to:

- `tag = Discriminant(e_place)`
- `Switch(tag)` to arm blocks
- In arm block, bind payload via `Downcast` place projections

This is exactly what both C and CLIF want.

---

## 5) Portable calling convention (the key piece)

### 5.1 MIR rule: calls 'return a value of type T'

MIR never decides if a return is:

- in registers,
- in an out pointer,
- or indirect.

Instead, backend implements a lowering pass:

**Backend-lowering pass: `LowerCallsAndReturns`**

- If backend=C and `ret_ty` is 'non-trivial': convert
- `CallTerm { dest: Some(p), call f(args), next }`
    into:

- `CallTerm { dest: None, call f(args + [addr_of(p)]), next }`
- function signature rewritten: `void f(..., T* out)`
- If backend=Cranelift: keep as direct returns where possible.

This one pass is what allows MIR to stay stable.

### 5.2 Dest is a Place

Calls write into a `Place`:

```text
CallTerm { dest: Some(Local(tmp)), call: f(args), next: bb_next }
```

If `dest` is None, it's a call for effects only.

---

## 6) Effects + intrinsics (portable standard lib boundary)

### 6.1 Keep 'platform things' as intrinsics in MIR

Rather than hardcoding db/json/http as language syntax, you define intrinsics:

- `Intrinsic::JsonDecode(TypeId)`
- `Intrinsic::JsonEncode`
- `Intrinsic::HttpServe`
- `Intrinsic::DbQuery`
- `Intrinsic::Log`

MIR:

```text
CallTerm { dest: Some(Local(req_struct)),
  call: Intrinsic(JsonDecode<T>)(body_bytes),
  next: ok_bb }
```

Backend lowering:

- C backend maps intrinsics to runtime C functions.
- CLIF backend maps intrinsics to runtime symbols too.

This is hugely stabilizing for v0.1.

### 6.2 Effects live on calls and intrinsics

Every `CallLike` has `effects: EffectSet`.  
Effects checker uses this, independent of backend.

---

## 7) Async without committing early (but still supported)

### 7.1 MIR supports `Await(Operand)`

MIR has `Await(x)` and `Call.async = true` flag. That's enough to:

- build the type checker rules,
- effect-check `spawn`/`await`,
- and later choose an async strategy.

### 7.2 Two backend strategies later

- **Strategy 1 (runtime futures)**: `async fn` compiles to normal function returning `Task<T>`. `Await` lowers to runtime `task_await`.
- **Strategy 2 (state machine)**: lower MIR to a coroutine/state machine IR. This is a later milestone.

For v0.1 'supports both backends', keep async as runtime futures (easiest).

---

## 8) Minimal 'backend interface' over MIR

To keep codebase clean:

```text
trait Backend {
  fn lower_abi(&mut self, module: &mut MirModule);     // sret/outptr rewriting etc.
  fn emit(&mut self, module: MirModule) -> Artifact;   // .c files or object files
}
```

Backends:

- `CBackend`: `lower_abi` is heavy (outptr rewriting), `emit` outputs C
- `ClifBackend`: `lower_abi` is light, `emit` builds object/exe

---

## 9) Concrete example (Result + ? + match) in portable MIR

Source:

```ut
fn get_user(id: Uuid) effects { db.read } -> Result<User, DbError> {
  let u = repo.find(id)? 
  Ok(u)
}
```

MIR sketch:

```text
bb0:
  CallTerm dest=Some(tmp0) call=repo.find(args=[id]) effects={db.read} next=bb1

bb1:
  TryTerm res=Copy(Local(tmp0)) ok_bb=bb2 err_bb=bb_err

bb2:
  Assign(Local(ret0), Aggregate(Enum(Result), [Variant::Ok, Copy(Local(tmp0_ok_payload))]))
  Return(Copy(Local(ret0)))

bb_err:
  // return Err(e)
  Assign(Local(ret1), Aggregate(Enum(Result), [Variant::Err, Copy(Local(tmp0_err_payload))]))
  Return(Copy(Local(ret1)))
```

A lowering pass will decide how `Result<User,DbError>` is returned (direct vs outptr) per backend.

---

## 10) What you gain with this design

- One MIR supports:

- emit-C backend now,
- Cranelift backend later,
- and even a MIR interpreter for tests.
- ABI decisions isolated in a single lowering pass per backend.
- Enums/Result/Option are uniform and easy to lower.
- Effects and contracts are backend-independent.

---

If you want, next I can pin down:

1.  a **precise type layout contract** at the MIR level (what counts as 'trivial' vs 'non-trivial'),

2.  the **runtime C ABI** signatures for intrinsics (JSON/HTTP/DB), and

3.  the **exact `lower_abi` transformations** (return rewriting, aggregate passing rules) so both backends behave identically.

---

## 1) MIR-level layout contract (portable 'data model')

MIR needs a notion of **representation class** so both backends make consistent ABI decisions.

### 1.1 Representation classes

Define these categories for every `TypeId`:

**A. Scalar**

- `Bool`, `Int32`, `Int64`, `Float64`, `Ptr`, `FnPtr`
- ABI: passed/returned in registers (or C scalar)

**B. ThinRef**

- `&T` / `*T` (if you add references/pointers)
- ABI: pointer-sized scalar

**C. AggregateTrivial**

- Small fixed-size structs/tuples containing only Scalar/ThinRef and no drop logic
- ABI: can be passed/returned by value *if backend supports easily* (Cranelift yes, C 'maybe')

**D. AggregateNonTrivial**

- Anything that:

- is bigger than a threshold (e.g., \> 16 bytes),
- contains heap-owned fields (like `String`, `List<T>`),
- is an enum with nontrivial payload,
- requires drop/cleanup.
- ABI: passed/returned indirectly (out pointer / by reference) in C backend; Cranelift may still return by value but we'll force consistent rule for v0.1.

**E. RuntimeManaged**

- `String`, `Bytes`, `List<T>`, `Map<K,V>`, `Json`, `Task<T>`, etc.
- Always treated as NonTrivial; operations go through runtime ABI.

### 1.2 'Trivial vs NonTrivial' rule (v0.1, deterministic)

To keep both backends aligned, choose a single rule:

> **Rule:** only Scalars and ThinRef are 'direct' returns/args. Everything else is indirect.

This is conservative but makes the C backend trivial and keeps Cranelift identical. Later you can optimize.

So in v0.1:

- Any function returning `T` where `T` is not Scalar/ThinRef becomes `void f(..., T* out)`
- Any parameter of non-scalar type becomes passed as `T*` (const pointer for immut).

### 1.3 Memory model for RuntimeManaged (handles)

For v0.1 portability, treat runtime-managed types as **opaque handles**:

- In MIR they are normal types (`String`, `List<T>`)
- In ABI they are represented as:
- either `struct { void* ptr; }` handle, or
- `void*` directly.

I recommend a **typed handle struct** in C to preserve debugging clarity:

```c
typedef struct { void* _p; } ai_string;
typedef struct { void* _p; } ai_json;
typedef struct { void* _p; } ai_task;
```

Then operations are runtime calls.

---

## 2) Runtime C ABI signatures for intrinsics (HTTP + JSON + core)

Below is a minimal, stable ABI you can implement in C **or** in Rust with `extern "C"`.

### 2.1 Core types (C side)

```c
#include <stdint.h>
#include <stddef.h>

typedef struct { void* _p; } ai_string;
typedef struct { void* _p; } ai_bytes;
typedef struct { void* _p; } ai_json;
typedef struct { void* _p; } ai_router;
typedef struct { void* _p; } ai_request;
typedef struct { void* _p; } ai_response;
typedef struct { void* _p; } ai_task;

typedef struct { uint8_t* ptr; size_t len; } ai_slice_u8;     // borrowed
typedef struct { const char* ptr; size_t len; } ai_slice_str; // borrowed
```

### 2.2 Result encoding at ABI boundary

Use a uniform 'tag + payload out parameters' model, so C and CLIF can share it.

For any `Result<T,E>`:

- runtime functions return `int32_t` status where 0=Ok, nonzero=Err code
- Ok/Err payload written to out pointers

Example pattern:

```c
int32_t ai_json_decode_user(ai_json json, /*out*/ void* out_user, /*out*/ ai_string* out_err_msg);
```

But to preserve typed errors later, you can have:

- error code + optional string.

For v0.1 web backend, simplest:

- use `ai_string` error message.

### 2.3 JSON ABI

Your compiler generates per-type encode/decode functions (or calls runtime generic functions if you implement reflection-like schema tables).

**Option 1 (simplest v0.1): compiler-generated codecs call runtime building blocks**  
Runtime primitives:

```c
ai_json ai_json_parse(ai_slice_u8 bytes, /*out*/ ai_string* out_err);     // returns ai_json handle or null handle
ai_slice_u8 ai_json_stringify(ai_json j);                                 // borrowed view; or return ai_bytes
void ai_json_free(ai_json j);

int32_t ai_json_get_field(ai_json obj, ai_slice_str key, /*out*/ ai_json* out_val);
int32_t ai_json_as_string(ai_json v, /*out*/ ai_string* out);
int32_t ai_json_as_i64(ai_json v, /*out*/ int64_t* out);
int32_t ai_json_as_bool(ai_json v, /*out*/ uint8_t* out);
```

Compiler-generated `Decode<T>` becomes a series of `get_field` + conversions + refined validation.

Encoding:

```c
ai_json ai_json_object_new(void);
void ai_json_object_set(ai_json obj, ai_slice_str key, ai_json val);
ai_json ai_json_string(ai_slice_str s);
ai_json ai_json_i64(int64_t x);
ai_json ai_json_bool(uint8_t b);
```

**Option 2 (faster later): schema tables**  
Compiler emits a schema table; runtime has `ai_json_decode(schema_id, json)`.

### 2.4 HTTP ABI

Core runtime functions:

```c
ai_router ai_router_new(void);
void ai_router_get(ai_router r, ai_slice_str path, void (*handler)(ai_request, /*out*/ ai_response*));
void ai_router_post(ai_router r, ai_slice_str path, void (*handler)(ai_request, /*out*/ ai_response*));

ai_slice_u8 ai_request_body(ai_request req);
ai_slice_str ai_request_path_param(ai_request req, ai_slice_str name); // or returns error
ai_slice_str ai_request_query(ai_request req, ai_slice_str name);      // optional

ai_response ai_response_text(int32_t status, ai_slice_str body);
ai_response ai_response_json(int32_t status, ai_json body);

int32_t ai_http_serve(int32_t port, ai_router router);
```

This design assumes:

- handlers are plain functions (sync for v0.1)
- async can be added by allowing handler to return a task handle.

### 2.5 Logging + time (effects)

```c
void ai_log_info(ai_slice_str msg);
int64_t ai_time_now_unix_ms(void);
```

### 2.6 Database (optional v0.1+)

Keep DB behind handles and parameterized query objects:

```c
typedef struct { void* _p; } ai_db;

int32_t ai_db_query_one(ai_db db, ai_slice_str sql, ai_json params, /*out*/ ai_json* out_row, /*out*/ ai_string* out_err);
```

---

## 3) Exact `lower_abi` transformations (backend-neutral MIR

- backend MIR)

### 3.1 Classification function

Given `TypeId`, compute:

- `AbiClass::Direct` for Scalar/ThinRef
- `AbiClass::Indirect` for everything else (v0.1 rule)

### 3.2 Function signature lowering

For each `fn f(p1:T1, ..., pn:Tn) -> R`:

**Parameters**

- If `Ti` is Indirect, rewrite param to `*const Ti` (pointer) in ABI signature
- In MIR, keep function body expecting a local of type `Ti` (value). Insert loads at entry:

- create local `pi_val: Ti`
- `pi_val = Load(*pi_ptr)` (or treat as 'borrowed place')

(Alternative: represent params as `Place` directly to avoid copying. For v0.1, easiest is to treat indirect params as `Place` borrowing.)

**Return**

- If `R` is Indirect:

- rewrite signature to `-> Unit` (or `void`)
- add an extra param: `out_ret: *mut R`
- rewrite each `Return(v)` to:
- `Store(out_ret, v)` then `Return()`

**C backend additionally** may want to:

- ensure all aggregates are passed as pointers even if 'small' (already true by rule)

### 3.3 Call lowering

For each `CallTerm { dest: Some(p), call: f(args), next }`:

If callee returns Indirect:

- Allocate a temp local `tmp: R` or use existing `dest` place
- Pass `&dest` (address) as extra final arg
- Set `dest=None` (void call)

If an argument is Indirect:

- Pass address of the argument place rather than its value
- If you only have an Operand (value), you must:
- materialize into a temp local (spill) then pass its address

**This is why having Place vs Operand matters.**

### 3.4 Aggregate passing rule

Because v0.1 says 'everything non-scalar indirect':

- `Aggregate` Rvalues writing to a Place are always ok.
- If you ever need an aggregate as an Operand, you must materialize it in a temp.

Backend-neutral MIR style guideline:

> Prefer 'construct into a Place' rather than 'aggregate as value'.

### 3.5 Enum/Result layout is *not* decided here

`lower_abi` does not pick tag sizes or payload offsets. It only decides:

- by-value vs by-pointer
  Actual layout is:

- C backend: emit a C struct/union layout in generated headers
- Cranelift backend: choose consistent in-memory layout and use loads/stores accordingly

To keep both identical, define an explicit layout policy next.

---

## 4) Cross-backend consistent layout policy (enums/structs)

If you want emitted C and Cranelift code to interop with the same runtime, you must match layouts.

### 4.1 Struct layout policy (v0.1)

- Fields laid out in order
- Align each field to its alignment
- Overall alignment = max field alignment
- Size rounded up to alignment
  This matches C 'natural' layout if you avoid bitfields and enforce `#pragma pack` off.

### 4.2 Enum layout policy (v0.1)

Use:

```c
struct EnumX {
  uint32_t tag;
  union { Variant0 v0; Variant1 v1; ... } payload;
};
```
- Tag is `uint32_t` always (simple)
- Payload union alignment = max alignment of variants
- Payload size = max size of variants
- Overall alignment = max(align(tag), align(payload))

Option/Result are just enums:

- `Option<T>` has variants `None`, `Some(T)`
- `Result<T,E>` has variants `Ok(T)`, `Err(E)`

This makes C and Cranelift layouts easy to match.

### 4.3 String/list handles

Opaque handles are pointer-sized; layout matches trivially.

---

## 5) Minimal runtime API the compiler assumes (stdlib boundary)

To keep codegen simple, the compiler assumes these functions exist:

### Core

- `ai_log_info`
- `ai_time_now_unix_ms`

### JSON primitives

- parse/stringify, object set/get, primitives conversion

### HTTP

- router register, request body, response constructors, serve

Everything else can be added later.

---

## 6) How both backends use the same MIR + ABI

### Emitted C backend

- Run `lower_abi` to pointer-ize aggregates.
- Emit:

- C type definitions for structs/enums per layout policy
- C functions with the lowered signatures
- Calls to runtime functions for intrinsics
- Compile with `clang` and link against `libsec4_runtime.a`

### Cranelift backend

- Optionally run the same `lower_abi` (so signatures match runtime exactly).
- Emit machine code that:

- uses the same struct/enum memory layouts
- calls the same runtime symbols (`ai_*`)
- Link with the same runtime library

This gives you backend switching with the same program behavior.

---

If you want, next I can provide:

1.  a small '**ABI header**' (`sec4_runtime.h`) template that your runtime exports,

2.  the **intrinsic list** as an enum and how the compiler lowers high-level stdlib calls into intrinsics, and

3.  an example end-to-end lowering of a simple `/health` handler into MIR, then into emitted C.

---

Below are the three concrete deliverables:

1.  a minimal **`sec4_runtime.h`** ABI header template,

2.  a **compiler intrinsic list** + how high-level stdlib calls lower into intrinsics, and

3.  an **end-to-end example**: `/health` handler - backend-neutral MIR - emitted C (after `lower_abi`).

---

## 1) `sec4_runtime.h` (v0.1 ABI template)

```c
#ifndef SEC4_RUNTIME_H
#define SEC4_RUNTIME_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// --------------------
// Opaque runtime handles (pointer-sized)
// --------------------
typedef struct { void* _p; } ai_string;
typedef struct { void* _p; } ai_bytes;
typedef struct { void* _p; } ai_json;

typedef struct { void* _p; } ai_router;
typedef struct { void* _p; } ai_request;
typedef struct { void* _p; } ai_response;

// Optional future
typedef struct { void* _p; } ai_task;

// --------------------
// Borrowed slices (non-owning views)
// --------------------
typedef struct { const uint8_t* ptr; size_t len; } ai_slice_u8;
typedef struct { const char*    ptr; size_t len; } ai_slice_str;

// --------------------
// Utility constructors for slices (optional convenience)
// --------------------
static inline ai_slice_str ai_str(const char* p, size_t n) { ai_slice_str s = { p, n }; return s; }
static inline ai_slice_u8  ai_u8 (const uint8_t* p, size_t n) { ai_slice_u8 s = { p, n }; return s; }

// --------------------
// Logging / time
// --------------------
void   ai_log_info(ai_slice_str msg);
void   ai_log_warn(ai_slice_str msg);
void   ai_log_error(ai_slice_str msg);

int64_t ai_time_now_unix_ms(void);

// --------------------
// String (minimal; you can expand later)
// --------------------
// Create an owned runtime string from borrowed bytes/utf8.
ai_string ai_string_from_utf8(ai_slice_u8 bytes, /*out*/ int32_t* out_ok);
// Get a borrowed view (valid until next runtime call on same thread OR until freed; define your rule)
ai_slice_str ai_string_as_slice(ai_string s);
// Free owned string
void ai_string_free(ai_string s);

// --------------------
// JSON (minimal building blocks)
// --------------------
int32_t ai_json_parse(ai_slice_u8 bytes, /*out*/ ai_json* out_json, /*out*/ ai_string* out_err);
int32_t ai_json_stringify(ai_json j, /*out*/ ai_bytes* out_bytes, /*out*/ ai_string* out_err);

void    ai_json_free(ai_json j);
void    ai_bytes_free(ai_bytes b);
ai_slice_u8 ai_bytes_as_slice(ai_bytes b);

// JSON constructors
ai_json ai_json_object_new(void);
ai_json ai_json_array_new(void);
ai_json ai_json_string(ai_slice_str s);
ai_json ai_json_i64(int64_t x);
ai_json ai_json_bool(uint8_t b);
ai_json ai_json_null(void);

// JSON object/array ops (return 0=ok, nonzero=err)
int32_t ai_json_object_set(ai_json obj, ai_slice_str key, ai_json val);
int32_t ai_json_object_get(ai_json obj, ai_slice_str key, /*out*/ ai_json* out_val);

// JSON conversions
int32_t ai_json_as_string(ai_json v, /*out*/ ai_string* out);
int32_t ai_json_as_i64(ai_json v, /*out*/ int64_t* out);
int32_t ai_json_as_bool(ai_json v, /*out*/ uint8_t* out);

// --------------------
// HTTP server
// --------------------
ai_router ai_router_new(void);
void      ai_router_free(ai_router r);

// Handlers: sync v0.1 shape (req in, response out)
typedef void (*ai_handler_fn)(ai_request req, /*out*/ ai_response* out_resp);

void ai_router_get(ai_router r, ai_slice_str path, ai_handler_fn h);
void ai_router_post(ai_router r, ai_slice_str path, ai_handler_fn h);

// Request accessors
ai_slice_u8  ai_request_body(ai_request req);
ai_slice_str ai_request_path_param(ai_request req, ai_slice_str name); // define behavior if missing
ai_slice_str ai_request_query(ai_request req, ai_slice_str name);      // define behavior if missing

// Response constructors
ai_response ai_response_text(int32_t status, ai_slice_str body);
ai_response ai_response_json(int32_t status, ai_json body);

// Serve (blocking)
int32_t ai_http_serve(int32_t port, ai_router router);

// --------------------
// Error helpers (optional)
// --------------------
ai_response ai_http_error_json(int32_t status, ai_slice_str code, ai_slice_str message);

#ifdef __cplusplus
}
#endif

#endif // SEC4_RUNTIME_H
```

**Notes for the compiler/runtime contract**

- All `int32_t` status codes: `0 = ok`, nonzero = error. Keep it consistent everywhere.
- Owned handles (`ai_string`, `ai_bytes`, `ai_json`, `ai_router`) must be freed by caller unless documented otherwise.

---

## 2) Intrinsics list + lowering rules

### 2.1 Intrinsic enum (compiler-internal)

These are MIR-level operations that the backend maps to runtime symbols (same for C and Cranelift).

```text
enum Intrinsic {
  // logging/time
  LogInfo, LogWarn, LogError,
  TimeNowUnixMs,

  // string/bytes
  StringFromUtf8,     // (slice_u8) -> Result<String, ErrMsg>
  StringAsSlice,      // (String) -> slice_str
  StringFree,
  BytesAsSlice,
  BytesFree,

  // json
  JsonParse,          // (slice_u8) -> Result<Json, ErrMsg>
  JsonStringify,      // (Json) -> Result<Bytes, ErrMsg>
  JsonFree,
  JsonObjectNew,
  JsonArrayNew,
  JsonString, JsonI64, JsonBool, JsonNull,
  JsonObjectSet, JsonObjectGet,
  JsonAsString, JsonAsI64, JsonAsBool,

  // http
  RouterNew, RouterFree,
  RouterGet, RouterPost,
  RequestBody, RequestPathParam, RequestQuery,
  ResponseText, ResponseJson,
  HttpServe,

  // (optional later) db, spawn/await, env, fs, random...
}
```

### 2.2 How source-level stdlib lowers to intrinsics

Define stdlib functions in Untrusted<T> as 'known' (either:

- compiler recognizes `http.Router.new()` directly, or
- stdlib is written in Untrusted<T> but marked `@intrinsic("RouterNew")`).

**Example mapping (source

- MIR intrinsic):**
- `Router.new()`
- `Intrinsic::RouterNew()`
- `router.get("/health", health)`
- `Intrinsic::RouterGet(router, "/health", &health_fnptr)`
- `req.body()`
- `Intrinsic::RequestBody(req)`
- `HttpResponse.text(200, "ok")`
- `Intrinsic::ResponseText(200, "ok")`
- `HttpServer.serve(port, router)`
- `Intrinsic::HttpServe(port, router)`
- `Json.parse(bytes)`
- `Intrinsic::JsonParse(bytes)` returning `Result<ai_json, ai_string>`

### 2.3 Effect tags associated with intrinsics (used by effect checker)

- `Log*`
- `log`
- `TimeNowUnixMs`
- `time.now`
- `Json*`
- (no effect or `cpu` only; you can treat as pure or `alloc`)
- `HttpServe`, `RouterGet/Post`
- `net` (and maybe `spawn`), plus `log` if server logs internally
- `RequestBody`
- pure (reads request object, not system effect)

Keep it simple v0.1:

- treat JSON construction/parsing as `alloc` (optional) but not required in effect system
- only enforce 'real-world effects': `log`, `time.now`, `db.*`, `net`, `fs.*`, `env.read`, `random`, `spawn`

---

## 3) Example end-to-end: `/health` handler

### 3.1 Source

```ut
fn health(_req: HttpRequest)
  effects { }
  -> Result<HttpResponse, HttpError>
{
  Ok(HttpResponse.text(200, "ok"))
}

fn main() effects { log, env.read, net } -> Int {
  let r = Router.new()
  r.get("/health", health)
  HttpServer.serve(8080, r)
  0
}
```

For v0.1 runtime ABI above, handlers are sync and don't return `Result`. So you have two choices:

- **Choice A (recommended v0.1):** compiler lowers `Result<HttpResponse, HttpError>` handler into a C ABI handler that always produces a response, mapping `Err` to a standard error response.
- **Choice B:** require handlers to be `fn(req, out_resp)` already in v0.1 source.

Below I'll show **Choice A** lowering (more ergonomic language, simpler runtime).

---

### 3.2 Backend-neutral MIR (before ABI lowering)

#### Types (conceptual)

- `HttpRequest` represented as runtime handle `ai_request`
- `HttpResponse` represented as runtime handle `ai_response`
- `Result<HttpResponse, HttpError>` is an enum (but we'll erase it during handler lowering)

#### Function `health` MIR (high-level)

```text
fn health(req: HttpRequest) -> Result<HttpResponse, HttpError> effects {}

locals:
  l0 = param req
  l1 = tmp resp: HttpResponse
  l2 = tmp ret: Result<HttpResponse,HttpError>

bb0:
  // resp = HttpResponse.text(200, "ok")
  CallTerm dest=Some(Local(l1))
    call=Intrinsic(ResponseText)(Const(200), Const("ok"))
    effects={}
    next=bb1

bb1:
  // ret = Ok(resp)
  SetDiscriminant(Local(l2), Variant::Ok)
  Assign(Local(l2).Downcast(Ok).Field(0), Use(Move(Local(l1))))
  Return(Copy(Local(l2)))
```

#### Function `main` MIR (high-level)

```text
fn main() -> Int effects {net, log, env.read}

locals:
  l0 = tmp router: Router
  l1 = tmp status: Int32
  l2 = tmp ret: Int

bb0:
  CallTerm dest=Some(Local(l0)) call=Intrinsic(RouterNew)() effects={net} next=bb1

bb1:
  CallTerm dest=None call=Intrinsic(RouterGet)(Copy(Local(l0)), Const("/health"), FnPtr(health)) effects={net} next=bb2

bb2:
  CallTerm dest=Some(Local(l1)) call=Intrinsic(HttpServe)(Const(8080), Move(Local(l0))) effects={net} next=bb3

bb3:
  Assign(Local(l2), Const(0))
  Return(Copy(Local(l2)))
```

---

### 3.3 ABI lowering + handler lowering (what happens before emitting C)

#### Step 1: ABI rule (v0.1) 'non-scalars indirect'

- `ai_router`, `ai_request`, `ai_response` are opaque handles (scalar-like pointer). You can treat them as **Direct**.
- `Result<...>` is aggregate
- **Indirect**.
- So `health(req) -> Result<HttpResponse,HttpError>` would become `void health(req, out_ret*)` **if we kept it**.

#### Step 2: Special-case: 'exported HTTP handler lowering'

We lower Untrusted<T> handler into runtime handler shape:

```c
void health__handler(ai_request req, ai_response* out_resp)
```

and implement:

- call original `health(req)` (or inline it),
- if `Ok(r)` write `*out_resp = r`,
- if `Err(e)` write `*out_resp = ai_http_error_json(500, ...)` (or 400 based on error type).

For v0.1, easiest: *don't even materialize `Result`*; just generate the handler body directly:

**Lowered handler MIR (post-transform)**

```text
fn health__handler(req: ai_request, out_resp: *mut ai_response) -> Unit effects {}

locals:
  l0 = param req
  l1 = param out_resp
  l2 = tmp resp: ai_response

bb0:
  CallTerm dest=Some(Local(l2)) call=Intrinsic(ResponseText)(Const(200), Const("ok")) effects={} next=bb1
bb1:
  Assign(Place(Deref(Local(l1))), Use(Move(Local(l2))))
  Return()
```

Now it's trivially emit-able to C and Clif.

---

### 3.4 Emitted C (what C backend would output)

#### Generated C types (for opaque handles we rely on runtime header)

The generated file includes:

```c
#include "sec4_runtime.h"
```

#### Generated handler

```c
// Untrusted<T>: health__handler
void health__handler(ai_request req, /*out*/ ai_response* out_resp) {
    // resp = ai_response_text(200, "ok")
    const char* lit = "ok";
    ai_slice_str s = ai_str(lit, 2);
    ai_response resp = ai_response_text(200, s);

    *out_resp = resp;
}
```

#### Generated main

```c
int32_t main(void) {
    ai_router r = ai_router_new();

    // register GET /health
    ai_router_get(r, ai_str("/health", 7), &health__handler);

    // serve 8080 (blocking)
    (void)ai_http_serve(8080, r);

    // cleanup (optional if serve never returns; but keep it correct)
    ai_router_free(r);
    return 0;
}
```

That's it - this is why the backend-neutral MIR + ABI lowering is so effective: once you reduce everything to (1) calls to runtime intrinsics and (2) simple stores, both C and Cranelift become straightforward.

---

## What I can provide next (if you want to continue)

- A complete **`Intrinsic -> runtime symbol` mapping table** (name mangling rules too),
- The **monomorphization strategy** (how generics become concrete MIR types for v0.1),
- A minimal **JSON derive algorithm**: how `@derive(JsonCodec)` becomes decode/encode MIR using only the JSON intrinsics above.

---

Great - here are the next three pieces:

1.  an **Intrinsic - runtime symbol mapping table** (plus name mangling),

2.  a **v0.1 monomorphization strategy** that keeps the compiler simple, and

3.  a concrete **`@derive(JsonCodec)` algorithm** that lowers to MIR using only the JSON intrinsics from the runtime ABI.

---

## 1) Intrinsic

- runtime symbol mapping + name mangling

### 1.1 Runtime symbol mapping table (v0.1)

This is the canonical mapping both backends use (C emitter just prints these calls; Cranelift calls the same symbols).

| MIR Intrinsic | Runtime symbol | Signature shape |
|----|----|----|
| `LogInfo` | `ai_log_info` | `(ai_slice_str msg) -> void` |
| `LogWarn` | `ai_log_warn` | `(ai_slice_str) -> void` |
| `LogError` | `ai_log_error` | `(ai_slice_str) -> void` |
| `TimeNowUnixMs` | `ai_time_now_unix_ms` | `() -> int64_t` |
| `StringFromUtf8` | `ai_string_from_utf8` | `(ai_slice_u8, out_ok*) -> ai_string` |
| `StringAsSlice` | `ai_string_as_slice` | `(ai_string) -> ai_slice_str` |
| `StringFree` | `ai_string_free` | `(ai_string) -> void` |
| `BytesAsSlice` | `ai_bytes_as_slice` | `(ai_bytes) -> ai_slice_u8` |
| `BytesFree` | `ai_bytes_free` | `(ai_bytes) -> void` |
| `JsonParse` | `ai_json_parse` | `(ai_slice_u8, out_json*, out_err*) -> int32` |
| `JsonStringify` | `ai_json_stringify` | `(ai_json, out_bytes*, out_err*) -> int32` |
| `JsonFree` | `ai_json_free` | `(ai_json) -> void` |
| `JsonObjectNew` | `ai_json_object_new` | `() -> ai_json` |
| `JsonArrayNew` | `ai_json_array_new` | `() -> ai_json` |
| `JsonString` | `ai_json_string` | `(ai_slice_str) -> ai_json` |
| `JsonI64` | `ai_json_i64` | `(int64) -> ai_json` |
| `JsonBool` | `ai_json_bool` | `(uint8) -> ai_json` |
| `JsonNull` | `ai_json_null` | `() -> ai_json` |
| `JsonObjectSet` | `ai_json_object_set` | `(ai_json obj, ai_slice_str key, ai_json val) -> int32` |
| `JsonObjectGet` | `ai_json_object_get` | `(ai_json obj, ai_slice_str key, out_val*) -> int32` |
| `JsonAsString` | `ai_json_as_string` | `(ai_json v, out_str*) -> int32` |
| `JsonAsI64` | `ai_json_as_i64` | `(ai_json v, out_i64*) -> int32` |
| `JsonAsBool` | `ai_json_as_bool` | `(ai_json v, out_u8*) -> int32` |
| `RouterNew` | `ai_router_new` | `() -> ai_router` |
| `RouterFree` | `ai_router_free` | `(ai_router) -> void` |
| `RouterGet` | `ai_router_get` | `(ai_router, ai_slice_str path, ai_handler_fn) -> void` |
| `RouterPost` | `ai_router_post` | `(ai_router, ai_slice_str, ai_handler_fn) -> void` |
| `RequestBody` | `ai_request_body` | `(ai_request) -> ai_slice_u8` |
| `RequestPathParam` | `ai_request_path_param` | `(ai_request, ai_slice_str name) -> ai_slice_str` |
| `RequestQuery` | `ai_request_query` | `(ai_request, ai_slice_str name) -> ai_slice_str` |
| `ResponseText` | `ai_response_text` | `(int32 status, ai_slice_str body) -> ai_response` |
| `ResponseJson` | `ai_response_json` | `(int32, ai_json) -> ai_response` |
| `HttpServe` | `ai_http_serve` | `(int32 port, ai_router router) -> int32` |
| (optional) `HttpErrorJson` | `ai_http_error_json` | `(int32, ai_slice_str code, ai_slice_str msg) -> ai_response` |

**Guideline:** runtime functions should avoid returning aggregates (besides opaque handles) to keep ABI uniform across C + Cranelift.

---

### 1.2 Name mangling rules (v0.1)

You need two naming layers:

1.  **Runtime symbols** (fixed names above, never mangled)

2.  **User-defined functions/types** (mangled to avoid collisions, encode modules + generics)

#### Recommended v0.1 scheme: stable, readable, ASCII-only

- Prefix every compiled symbol with `aiu__` ('ai user')
- Use module path separators as `__`
- Encode special chars with `_XX` hex (rare)
- Encode monomorphized type params in a suffix

**Function:**

```text
aiu__<module_path>__<fn_name>__<monomorph_suffix>
```

Example:

- `api.handlers.health` - `aiu__api__handlers__health`
- `domain.user.parse<T>` specialized for `T=Int64` - `aiu__domain__user__parse__T$Int64`
- Multiple params: `__T$Int64__U$String`

**Type names** (for generated codec fns etc.) use the same path basis:

- `JsonCodec::decode(domain.user.CreateUserRequest)`:
- `aiu__domain__user__CreateUserRequest__json_decode`
- `aiu__domain__user__CreateUserRequest__json_encode`

This scheme is:

- deterministic
- easy to debug
- good enough for v0.1
  Later you can switch to a hash-based suffix for shorter names without changing semantics.

---

## 2) v0.1 Monomorphization strategy (keep it simple)

Goal: support generics enough for stdlib containers and `Result/Option`, without building a super-advanced type system.

### 2.1 Core approach: 'Whole-program monomorphization after typecheck'

Pipeline:

1.  Parse - AST

2.  Resolve names - HIR

3.  Typecheck + infer - **Typed HIR**

4.  Build **Instantiation Set** (all concrete uses of generic functions/types)

5.  Clone generic bodies for each concrete instantiation - **Monomorphized HIR**

6.  Lower to MIR

### 2.2 Instantiation Set algorithm (practical)

- Start from entry points:

- `main`
- exported HTTP handlers (or router-registered handlers)
- tests/properties when running `sec4 test`
- Traverse call graph:
- for each call `f<...>(...)`, record the concrete type args
- for each struct/enum generic used concretely, record type args
- Use a worklist until no new instantiations appear.

**Important v0.1 constraint (recommended):**

- No higher-ranked generics.
- No generic recursion that produces infinite specializations (compiler detects and errors).
- No generic trait specialization (keep traits minimal).

### 2.3 Representation of generic types in v0.1

- `Option<T>` and `Result<T,E>` are *built-in* generic enums, but they are just regular generics to the monomorphizer.
- `List<T>`, `Map<K,V>` can be runtime-managed handles and don't need layout specialization:

- represent them as `ai_list` handle regardless of `T` (type erased at runtime)
- the type parameter only matters to the typechecker, not codegen.
    This massively reduces specialization explosion.

So you can pick:

- **Monomorphize value types** (structs/enums/functions)
- **Erase runtime container params** (List/Map/Json/Task) to handles

### 2.4 Function cloning rules

When specializing `fn f<T,U>(...) -> ...` for concrete types, you:

- substitute `T,U` everywhere in the body and signature
- give it a mangled symbol name with concrete types encoded
- cache it so you don't duplicate

### 2.5 ABI lowering happens after monomorphization

Because ABI decisions need concrete sizes/classes. Even if v0.1 uses 'indirect for all non-scalars,' it's still simpler post-mono.

---

## 3) `@derive(JsonCodec)`

- concrete algorithm + MIR lowering

This is where 'AI-first' really shines: the compiler generates predictable decode/encode code that is easy to verify.

### 3.1 What `@derive(JsonCodec)` generates

For each annotated struct `S`:

- `fn S__json_decode(j: Json) -> Result<S, JsonError>`
- `fn S__json_encode(s: S) -> Json`

Where:

- `Json` is runtime handle `ai_json`
- `JsonError` in v0.1 can be a simple struct:

```ut
struct JsonError { message: String, path: String }
```

  (or just `String` initially)

### 3.2 Decode rules by field type (v0.1)

Let field name be `f` and field type be `T`.

1.  **Primitive** - `String`: require `json_as_string` - `Int64`: require `json_as_i64` - `Bool`: require `json_as_bool`

2.  **Refined type** `Email = String where matches_email` - Decode underlying type first (String) - Call generated validator: `Email.try_from(str)` (this is normal Untrusted<T> code) - On fail: produce error `"invalid Email"`

3.  **Option\<T\>** - Try `object_get`; if missing key - `None` - If present but `null` - `None` (optional policy) - Else decode T - `Some(value)`

4.  **Nested struct with JsonCodec** - Call its `__json_decode`

5.  **List\<T\> / Map\<K,V\>** - If you want in v0.1: keep very limited (or omit) - Otherwise:

- require array/object
- loop decode each element (needs iteration intrinsics; you can defer)

**Recommended v0.1 scope:** structs only, primitives + refined + nested structs + Option. Add arrays/maps in v0.2.

### 3.3 Encode rules

- Create `obj = json_object_new()`
- For each field:

- encode value to `ai_json`:
- primitive: `JsonString/JsonI64/JsonBool`
- refined: unwrap to underlying, then encode
- option: if None, omit key (or set null; pick one policy)
- nested struct: call its encode
- `json_object_set(obj, "field", val)`
- return `obj`

### 3.4 Concrete example: generated decode/encode (pseudo Untrusted<T>)

Source:

```ut
@derive(JsonCodec)
struct CreateUserRequest {
  email: Email,
  name: NonEmptyString,
  age: Option<Int64>
}
```

Generated:

```ut
fn CreateUserRequest__json_decode(j: Json) -> Result<CreateUserRequest, JsonError> { ... }
fn CreateUserRequest__json_encode(x: CreateUserRequest) -> Json { ... }
```

### 3.5 MIR-level lowering using intrinsics only (decode)

High-level structure:

1.  For each field:

- `json_object_get(j, "email") -> json_val`
- convert json_val to primitive (string/i64/bool) using `JsonAs*`
- validate refined types using regular calls

2.  Construct struct aggregate and return Ok

Key detail: the runtime JSON API returns status codes; so decode is naturally a chain of checks.

#### MIR sketch (decode email + name + optional age)

```text
locals:
  j: ai_json (param)
  v_email: ai_json
  s_email: ai_string
  email: Email
  v_name: ai_json
  s_name: ai_string
  name: NonEmptyString
  age_opt: Option<Int64>
  tmp_i64: int64
  v_age: ai_json
  out: CreateUserRequest
  err: JsonError

bb0:
  // get email field
  CallTerm dest=Some(tmp_status)
    call=Intrinsic(JsonObjectGet)(j, "email", &v_email)
    next=bb1

bb1:
  if tmp_status != 0 goto bb_err_missing_email else bb2

bb2:
  CallTerm dest=Some(tmp_status2)
    call=Intrinsic(JsonAsString)(v_email, &s_email)
    next=bb3
bb3:
  if tmp_status2 != 0 goto bb_err_type_email else bb4

bb4:
  // email = Email.try_from(s_email)  (regular user-level function, not intrinsic)
  CallTerm dest=Some(tmp_res_email)
    call=Email__try_from(s_email)
    next=bb5
bb5:
  TryTerm res=tmp_res_email ok_bb=bb6 err_bb=bb_err_refine_email

bb6:
  // similarly name...
  ...

  // age optional:
  CallTerm dest=Some(st_age)
    call=Intrinsic(JsonObjectGet)(j, "age", &v_age)
    next=bb_age1
bb_age1:
  if st_age != 0 goto bb_age_none else bb_age_some

bb_age_none:
  SetDiscriminant(age_opt, None)
  goto bb_done

bb_age_some:
  CallTerm dest=Some(st_age2) call=Intrinsic(JsonAsI64)(v_age, &tmp_i64) next=bb_age2
bb_age2:
  if st_age2 != 0 goto bb_err_type_age else bb_age3
bb_age3:
  SetDiscriminant(age_opt, Some)
  Assign(age_opt.downcast(Some).field0, Use(tmp_i64))
  goto bb_done

bb_done:
  Assign(out, Aggregate(Struct(CreateUserRequest), [email, name, age_opt]))
  SetDiscriminant(ret, Ok)
  Assign(ret.ok_payload, Move(out))
  Return(ret)

bb_err_missing_email:
  ... construct JsonError("missing field", "email") return Err
```

This is fully backend-neutral: only intrinsics + normal calls + enum/struct ops.

### 3.6 Encoder MIR sketch

```text
obj = Intrinsic(JsonObjectNew)()
val_email = Intrinsic(JsonString)(StringAsSlice(unwrap(email)))
Intrinsic(JsonObjectSet)(obj, "email", val_email)
...
Return(obj)
```

### 3.7 Where 'String literals' become slices

In emitted C you can produce `ai_slice_str` directly.  
In MIR, treat `"ok"` as a `Const(StringLit)` and backend lowers:

- C: `ai_str("ok", 2)`
- Clif: create a rodata global and pass pointer+len

---

## 4) Recommended v0.1 policies (so codegen is deterministic)

Pick these now to avoid churn:

1.  **Missing optional field** - `None`

2.  **Missing required field** - decode error

3.  **Null for non-Option field** - decode error

4.  **Option field present but null** - `None` (optional; consistent with many APIs)

5.  **Encode None** - omit field (or set null). Omit is common for backends; pick one.
