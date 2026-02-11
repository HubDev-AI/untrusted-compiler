# AILang Concept Notes

# AILang Backend Requirements (Curated from Conversation)

Source: `docs/AI and Programming Languages.mhtml`

This file keeps only AILang-relevant requirement/design content and excludes generic/meta chat turns.

Picture a language that's 'boringly explicit,' strongly typed, and comes with built-in ways to *state intent* and *prove properties*. The syntax could be simple - but the real 'AI-friendliness' is in the semantics and tooling.

Here's what it might look like.

---

## 1) Core idea: code = intent + constraints + proof hooks

### Example: a function with effects + contracts

```plaintext
fn transfer(from: AccountId, to: AccountId, amount: Money)
  effects { db.read, db.write }
  requires { amount > 0, balance(from) >= amount }
  ensures  { balance(from) == old(balance(from)) - amount
             balance(to)   == old(balance(to))   + amount }
  returns Result<Unit, TransferError>
{
  // ...
}
```

Why AI likes it:

- 'effects' tells exactly what the function is allowed to do.
- requires/ensures makes the intended behavior machine-checkable.
- Result forces error-handling paths to be explicit.

---

## 2) Types that remove ambiguity (and hallucinations)

### Units + domain types by default

```plaintext
type Money = decimal<2> tagged("USD")
type Email = string refined(matches_email)
type NonEmptyString = string refined(len > 0)
```

Now the AI can't 'accidentally' pass a `string` where `Email` is required without a conversion/validation.

### No nulls (or null is explicit)

```plaintext
type Maybe<T> = Some(T) | None
```

---

## 3) Explicit data flow, minimal hidden state

### Immutable-by-default, mutation is marked

```plaintext
let user = load_user(id)
let updated = user.with_email(new_email)

mut cache.put(id, updated)   // mutation requires `mut`
```

This prevents a lot of accidental side effects and makes refactoring safer.

---

## 4) Exhaustiveness everywhere

### Pattern matching forces you to handle all cases

```plaintext
match result {
  Ok(value)      => value
  Err(NotFound)  => default
  Err(Permission)=> fail("no access")
}
```

AI-generated code becomes less 'oops we forgot the edge case.'

---

## 5) Built-in property tests and fuzzing as first-class citizens

### Properties live next to the code

```plaintext
property "transfer conserves total money" {
  for_all from, to, amount where from != to and amount > 0 {
    let before = balance(from) + balance(to)
    transfer(from, to, amount)
    let after  = balance(from) + balance(to)
    assert(after == before)
  }
}
```

Instead of 'write some tests,' the language nudges: 'state invariants.'

---

## 6) Modules with sealed boundaries + capability-based access

No 'reach into anything' by default. You import capabilities.

```plaintext
module Payments exports { transfer }

capability DbRead
capability DbWrite

fn transfer(...) requires { DbRead, DbWrite } { ... }
```

This helps AI (and humans) understand what can touch what.

---

## 7) Deterministic builds, reproducible environments, and 'compiler as referee'

An AI-first language would likely mandate:

- pinned dependencies,
- deterministic compilation,
- standardized formatter,
- a single canonical project layout.

So generated code is more likely to run the same everywhere.

---

## 8) 'AI hooks' (but still human-friendly)

Not magical natural-language inside code, but structured metadata:

```plaintext
doc {
  purpose: "Move funds between accounts"
  failure_modes: ["insufficient_funds", "account_locked", "rate_limited"]
  complexity_budget: "O(1) db ops"
}
```

This gives assistants stable anchors without turning the language into English.

---

# What would it *feel* like overall?

**A hybrid of:**

- Rust/Swift/Kotlin-level safety (types, null-safety, exhaustiveness),
- an effect system (like research languages, or functional ecosystems),
- contracts (Design by Contract / refinement types),
- and testing baked into the language.

Not 'easier syntax,' but **less room to be wrong**.

---

## The tradeoff (important)

Such a language would be:

- **slower to write initially** (more annotations),
- but **much faster to verify and maintain** (especially with AI),
  because the compiler and tools would catch nonsense immediately.
