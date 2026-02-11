# 32 Language Syntax by Example

Examples below map directly to the current M1 parser behavior.

## Function with explicit return type

```ailang
fn main() -> Int {
  0
}
```

## Struct and enum declarations

```ailang
struct User {
  id: Int,
  email: String,
}

enum MaybeUser {
  Some(user: User),
  None,
}
```

## Option/Result generic types

```ailang
fn resolve_user(found: Bool) -> Result<Option<User>, String> {
  let selected: Option<User> = match found {
    true => Some(User()),
    false => None,
  };
  selected
}
```

## Optional type sugar (`T?`)

```ailang
fn parse_age(raw: String) -> Int? {
  return None;
}
```

In M1 AST, `Int?` is normalized to `Option<Int>`.

## `check --emit ast`

```bash
cargo run -p ailang -- check --path examples/hello --emit ast
```

This prints pretty JSON AST with source spans.
