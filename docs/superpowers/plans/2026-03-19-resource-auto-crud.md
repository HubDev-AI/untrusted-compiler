# Resource Auto-CRUD Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

## Status Snapshot (2026-03-26)

Implemented and verified on `feature/resource-improvements`:

- `resource` parsing and semantic validation
- `@unique` and `@optional` annotations
- LASM resource CRUD dispatch
- `sec4 describe`
- `sec4 migrate`
- `sec4 openapi`

Focused follow-up that was completed during review:

- `migrate`, `describe`, and `openapi` now run semantic analysis, not parse-only AST loading
- generated DDL now respects `@unique`, `@optional`, typed defaults, and adapter-specific timestamp defaults
- OpenAPI output now matches success-envelope response shapes and PATCH-like update semantics
- describe output now includes `@unique` / `@optional`

**Goal:** Add `resource` declarations to .ut files that auto-generate 5 CRUD endpoints with typed validation, SQL generation, and deterministic response envelopes.

**Architecture:** New `resource` keyword flows through lexer -> parser -> AST -> semantic validation -> route plan extraction -> LASM runtime dispatch. The compiler validates security properties and extracts `LasmResourcePlan` metadata. A new `lasm_resource_dispatch.rs` module intercepts resource routes before existing materialization, executes type-driven validation, generates parameterized SQL, and returns standard envelopes.

**Tech Stack:** Rust (compiler: sec4-core, runtime: sec4-cli), serde_json for resource plan serialization, existing DB adapter infrastructure (Postgres/SQLite/RecordsLog).

**Spec:** `docs/superpowers/specs/2026-03-19-resource-auto-crud-design.md`

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `compiler/sec4-core/src/token.rs` | Modify | Add `Resource` keyword variant |
| `compiler/sec4-core/src/lexer.rs` | Modify | Map `"resource"` string to keyword |
| `compiler/sec4-core/src/ast.rs` | Modify | Add `ResourceDecl`, `ResourceFieldDecl`, `ResourceFieldAnnotation` types |
| `compiler/sec4-core/src/parser.rs` | Modify | Parse `resource` declarations with field annotations |
| `compiler/sec4-core/src/semantic.rs` | Modify | Validate resource fields, primary uniqueness, name collisions, generate route plans |
| `compiler/sec4-cli/src/main.rs` | Modify | Extract resource plans, inject resource routes, intercept dispatch |
| `compiler/sec4-cli/src/lasm_resource_dispatch.rs` | Create | SQL generation, field validation, CRUD operation execution, response envelope building |
| `compiler/sec4-core/tests/fixtures/semantic/valid_resource_basic.ut` | Create | Valid resource test fixture |
| `compiler/sec4-core/tests/fixtures/semantic/valid_resource_basic.golden` | Create | Expected output: OK |
| `compiler/sec4-core/tests/fixtures/semantic/invalid_resource_no_primary.ut` | Create | Invalid: no @primary |
| `compiler/sec4-core/tests/fixtures/semantic/invalid_resource_no_primary.golden` | Create | Expected error |
| `compiler/sec4-core/tests/fixtures/semantic/invalid_resource_duplicate_name.ut` | Create | Invalid: name collision with struct |
| `compiler/sec4-core/tests/fixtures/semantic/invalid_resource_duplicate_name.golden` | Create | Expected error |
| `compiler/sec4-core/tests/fixtures/parser/valid_resource_parse.ut` | Create | Parser test fixture |
| `compiler/sec4-core/tests/fixtures/parser/valid_resource_parse.golden` | Create | Expected AST output |

---

### Task 1: Add `Resource` keyword, `@` symbol to token and lexer

**Files:**
- Modify: `compiler/sec4-core/src/token.rs:6-18` (Keyword enum), `token.rs:44-74` (Symbol enum), `token.rs:76-106` (Symbol::as_str)
- Modify: `compiler/sec4-core/src/lexer.rs:227-242` (keyword match), `lexer.rs:87-130` (char dispatch)

- [ ] **Step 1: Add `Resource` keyword variant**

In `compiler/sec4-core/src/token.rs`, add `Resource` to the `Keyword` enum (line 17, after `Enum`):

```rust
    Enum,
    Resource,
}
```

Add `Resource` to `Keyword::as_str()` (line 33, after the Enum arm):

```rust
            Keyword::Enum => "enum",
            Keyword::Resource => "resource",
```

- [ ] **Step 2: Add `At` symbol variant**

In `compiler/sec4-core/src/token.rs`, add `At` to the `Symbol` enum (line 73, after `Question`):

```rust
    Question,
    At,
}
```

Add `At` to `Symbol::as_str()` (after the Question arm):

```rust
            Symbol::Question => "?",
            Symbol::At => "@",
```

- [ ] **Step 3: Add `@` character handling to lexer**

In `compiler/sec4-core/src/lexer.rs`, add `@` to the character dispatch match block (line 99, after the `'?'` arm):

```rust
                '?' => self.lex_one_symbol(Symbol::Question),
                '@' => self.lex_one_symbol(Symbol::At),
```

- [ ] **Step 4: Add `"resource"` keyword recognition**

In `compiler/sec4-core/src/lexer.rs`, add the `"resource"` mapping in the keyword match (after `"enum"` arm):

```rust
    "enum" => TokenKind::Keyword(Keyword::Enum),
    "resource" => TokenKind::Keyword(Keyword::Resource),
    "true" => TokenKind::Bool(true),
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo check -p sec4-core`
Expected: No errors. The new keyword and symbol are recognized but not yet used.

- [ ] **Step 6: Commit**

```bash
git add compiler/sec4-core/src/token.rs compiler/sec4-core/src/lexer.rs
git commit -m "feat: add Resource keyword and @ symbol to lexer"
```

---

### Task 2: Add resource AST types

**Files:**
- Modify: `compiler/sec4-core/src/ast.rs:22-67`

- [ ] **Step 1: Add resource field annotation enum**

Add after the `EnumVariant` definition in `ast.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ResourceFieldAnnotation {
    Primary,
    Auto,
    Default(String),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ResourceFieldDecl {
    pub name: String,
    pub ty: TypeExpr,
    pub annotations: Vec<ResourceFieldAnnotation>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ResourceDecl {
    pub name: String,
    pub table_override: Option<String>,
    pub fields: Vec<ResourceFieldDecl>,
}
```

- [ ] **Step 2: Add Resource variant to ItemKind**

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ItemKind {
    Function(FunctionDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Resource(ResourceDecl),
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p sec4-core`
Expected: Possible warnings about non-exhaustive match in other files (parser, semantic). Note which files need updating but don't fix yet — those are Tasks 3 and 5.

- [ ] **Step 4: Commit**

```bash
git add compiler/sec4-core/src/ast.rs
git commit -m "feat: add ResourceDecl and field annotation AST types"
```

---

### Task 3: Parse `resource` declarations

**Files:**
- Modify: `compiler/sec4-core/src/parser.rs:68-82` (dispatch), add new method

- [ ] **Step 1: Write the parser test fixture**

Create `compiler/sec4-core/tests/fixtures/parser/valid_resource_parse.ut`:
```ut
resource Task {
  id: Uuid @primary,
  title: String,
  status: String @default("pending"),
  created_at: Time @auto,
}
```

Create `compiler/sec4-core/tests/fixtures/parser/valid_resource_parse.golden` with `OK` (we'll update with exact AST shape after first run if the golden test uses AST output, or just `OK` if it's parse-success-only).

- [ ] **Step 2: Run parser tests to see the fixture fail**

Run: `cargo test -p sec4-core golden_parser -- --nocapture 2>&1 | tail -20`
Expected: FAIL — `resource` is not a recognized top-level keyword.

- [ ] **Step 3: Add resource dispatch to parse_program**

In `compiler/sec4-core/src/parser.rs`, modify the top-level dispatch (around line 68-82):

```rust
let item_result = if let Some(start) = self.match_keyword(Keyword::Fn) {
    self.parse_function_item(start)
} else if let Some(start) = self.match_keyword(Keyword::Struct) {
    self.parse_struct_item(start)
} else if let Some(start) = self.match_keyword(Keyword::Enum) {
    self.parse_enum_item(start)
} else if let Some(start) = self.match_keyword(Keyword::Resource) {
    self.parse_resource_item(start)
} else {
    let token = self.current().clone();
    self.diagnostics.push(
        Diagnostic::error("P2001", "expected top-level declaration", token.span)
            .with_note("top-level items must start with `fn`, `struct`, `enum`, or `resource`"),
    );
    self.synchronize_top_level();
    continue;
};
```

- [ ] **Step 4: Implement parse_resource_item**

Add method to the parser impl block, following the `parse_struct_item` pattern (lines 246-288):

```rust
fn parse_resource_item(&mut self, start: Token) -> Result<Item, Diagnostic> {
    let (name, _name_token) =
        self.expect_identifier("P2030", "expected resource name after `resource`")?;

    // Optional table override: resource Person table "people" { ... }
    let table_override = if self.check_identifier_value("table") {
        self.advance(); // consume "table"
        let (table_name, _) =
            self.expect_string_literal("P2031", "expected table name string after `table`")?;
        Some(table_name)
    } else {
        None
    };

    self.expect_symbol(Symbol::LBrace, "P2032", "expected `{` after resource name")?;

    let mut fields = Vec::new();
    while !self.check_symbol(Symbol::RBrace) && !self.is_eof() {
        if self.should_interrupt() {
            return Err(self.interruption_diagnostic());
        }
        let (field_name, field_name_token) =
            self.expect_identifier("P2033", "expected resource field name")?;
        self.expect_symbol(Symbol::Colon, "P2034", "expected `:` after field name")?;
        let field_type = self.parse_type()?;

        // Parse field annotations: @primary, @auto, @default("value")
        let mut annotations = Vec::new();
        while self.check_symbol(Symbol::At) {
            self.advance(); // consume @
            let (ann_name, _) =
                self.expect_identifier("P2035", "expected annotation name after `@`")?;
            match ann_name.as_str() {
                "primary" => annotations.push(ResourceFieldAnnotation::Primary),
                "auto" => annotations.push(ResourceFieldAnnotation::Auto),
                "default" => {
                    self.expect_symbol(Symbol::LParen, "P2036", "expected `(` after @default")?;
                    let (value, _) = self.expect_string_literal(
                        "P2037",
                        "expected default value string in @default(...)",
                    )?;
                    self.expect_symbol(Symbol::RParen, "P2038", "expected `)` after default value")?;
                    annotations.push(ResourceFieldAnnotation::Default(value));
                }
                other => {
                    return Err(self.error_current(
                        "P2039",
                        &format!("unknown resource field annotation `@{}`", other),
                    ));
                }
            }
        }

        let field_span = join_spans(&field_name_token.span, &field_type.span);
        fields.push(ResourceFieldDecl {
            name: field_name,
            ty: field_type,
            annotations,
            span: field_span,
        });

        if self.match_symbol(Symbol::Comma).is_some() {
            continue;
        }
        if self.check_symbol(Symbol::RBrace) {
            break;
        }
        return Err(self.error_current("P2040", "expected `,` or `}` in resource declaration"));
    }

    let end = self.expect_symbol(Symbol::RBrace, "P2041", "expected `}` to close resource")?;
    let span = join_spans(&start.span, &end.span);

    Ok(Item {
        kind: ItemKind::Resource(ResourceDecl {
            name,
            table_override,
            fields,
        }),
        span,
    })
}
```

**Implementation notes:** `Symbol::At` was added in Task 1. The parser does not have `expect_string_literal` or `check_identifier_value` — you must implement these two helper methods on the parser:

```rust
/// Check if current token is an identifier with a specific value (does not consume).
fn check_identifier_value(&self, value: &str) -> bool {
    matches!(&self.current().kind, TokenKind::Identifier(name) if name == value)
}

/// Consume current token if it's a string literal, returning its value.
fn expect_string_literal(&mut self, code: &str, message: &str) -> Result<(String, Token), Diagnostic> {
    let token = self.current().clone();
    if let TokenKind::String(value) = &token.kind {
        let value = value.clone();
        self.advance();
        Ok((value, token))
    } else {
        Err(Diagnostic::error(code, message, token.span))
    }
}
```

Check the exact `TokenKind` variant for string literals — it may be `TokenKind::String(String)` or `TokenKind::StringLiteral(String)`. Match the actual enum variant name from `token.rs`.

- [ ] **Step 5: Update golden_parser.rs with Resource rendering arm**

In `compiler/sec4-core/tests/golden_parser.rs`, find the exhaustive match on `ItemKind` and add a `Resource` arm:

```rust
ItemKind::Resource(decl) => {
    format!("resource {} ({} fields)", decl.name, decl.fields.len())
}
```

- [ ] **Step 6: Run parser tests**

Run: `cargo test -p sec4-core golden_parser -- --nocapture 2>&1 | tail -20`
Expected: PASS (update golden file with actual output if format differs).

- [ ] **Step 7: Commit**

```bash
git add compiler/sec4-core/src/parser.rs compiler/sec4-core/tests/golden_parser.rs compiler/sec4-core/tests/fixtures/parser/
git commit -m "feat: parse resource declarations with field annotations"
```

---

### Task 4: Semantic validation for resources

**Files:**
- Modify: `compiler/sec4-core/src/semantic.rs:405-421` (type declarations collection)
- Create: test fixtures in `compiler/sec4-core/tests/fixtures/semantic/`

- [ ] **Step 1: Write test fixtures**

Create `compiler/sec4-core/tests/fixtures/semantic/valid_resource_basic.ut` (keep it minimal to isolate resource validation — no main() with intrinsic calls):
```ut
resource Task {
  id: Uuid @primary,
  title: String,
  status: String @default("pending"),
  created_at: Time @auto,
}
```

Create `compiler/sec4-core/tests/fixtures/semantic/valid_resource_basic.golden`:
```
OK
```

Create `compiler/sec4-core/tests/fixtures/semantic/invalid_resource_no_primary.ut`:
```ut
resource Task {
  title: String,
  status: String,
}
```

Create `compiler/sec4-core/tests/fixtures/semantic/invalid_resource_no_primary.golden`:
```
error[E5001]: resource requires exactly one @primary field
  --> invalid_resource_no_primary.ut:1:1
  note: resource `Task` has no @primary field
  note: add `@primary` annotation to the identity field
```

Create `compiler/sec4-core/tests/fixtures/semantic/invalid_resource_duplicate_name.ut`:
```ut
struct Task {
  id: Int,
}

resource Task {
  id: Uuid @primary,
  title: String,
}
```

Create `compiler/sec4-core/tests/fixtures/semantic/invalid_resource_duplicate_name.golden`:
```
error[N3002]: duplicate type declaration
  --> invalid_resource_duplicate_name.ut:5:1
  note: type `Task` is already declared
```

- [ ] **Step 2: Run semantic tests to see them fail**

Run: `cargo test -p sec4-core golden_semantic -- --nocapture 2>&1 | tail -20`
Expected: FAIL — `Resource` variant not handled in semantic analysis.

- [ ] **Step 3: Add resource validation to collect_type_declarations**

In `compiler/sec4-core/src/semantic.rs`, find the `collect_type_declarations` method (around line 405). Add a branch for `ItemKind::Resource`:

```rust
ItemKind::Resource(decl) => {
    // Name uniqueness: reject collision with struct/enum/primitive/generic
    if self.catalog.structs.contains_key(&decl.name)
        || self.catalog.enums.contains_key(&decl.name)
        || self.catalog.primitive_types.contains(&decl.name)
        || self.catalog.generic_types.contains_key(&decl.name)
        || self.catalog.resources.contains_key(&decl.name)
    {
        self.diagnostics.push(
            Diagnostic::error(
                "N3002",
                "duplicate type declaration",
                item.span.clone(),
            )
            .with_note(format!("type `{}` is already declared", decl.name)),
        );
        continue;
    }

    // Validate exactly one @primary field
    let primary_count = decl.fields.iter().filter(|f| {
        f.annotations.iter().any(|a| matches!(a, ResourceFieldAnnotation::Primary))
    }).count();

    if primary_count == 0 {
        self.diagnostics.push(
            Diagnostic::error(
                "E5001",
                "resource requires exactly one @primary field",
                item.span.clone(),
            )
            .with_note(format!("resource `{}` has no @primary field", decl.name))
            .with_note("add `@primary` annotation to the identity field"),
        );
        continue;
    }

    if primary_count > 1 {
        self.diagnostics.push(
            Diagnostic::error(
                "E5002",
                "resource cannot have multiple @primary fields",
                item.span.clone(),
            )
            .with_note(format!("resource `{}` has {} @primary fields", decl.name, primary_count)),
        );
        continue;
    }

    // Validate all field types are known built-in types
    // NOTE: TypeExpr uses TypeExprKind::Named { name, args }, not a direct .name field
    let allowed_types = ["Uuid", "String", "Email", "Int64", "Int", "Time", "Bool"];
    for field in &decl.fields {
        let type_name = match &field.ty.kind {
            sec4_core::ast::TypeExprKind::Named { name, .. } => name,
        };
        if !allowed_types.contains(&type_name.as_str()) {
            self.diagnostics.push(
                Diagnostic::error(
                    "E5003",
                    "resource field type must be a built-in type",
                    field.span.clone(),
                )
                .with_note(format!(
                    "field `{}` has type `{}` which is not supported in resource declarations",
                    field.name, type_name
                ))
                .with_note(format!("allowed types: {}", allowed_types.join(", "))),
            );
        }
    }

    // Warn if zero non-auto fields (beyond primary)
    // NOTE: Diagnostic::warning() does not exist — construct manually
    let non_auto_non_primary = decl.fields.iter().filter(|f| {
        !f.annotations.iter().any(|a| matches!(a, ResourceFieldAnnotation::Auto | ResourceFieldAnnotation::Primary))
    }).count();
    if non_auto_non_primary == 0 {
        self.diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: "W5001".to_string(),
            message: "resource has no user-editable fields".to_string(),
            span: item.span.clone(),
            notes: vec![format!("resource `{}` only has @primary and @auto fields", decl.name)],
            tags: Vec::new(),
        });
    }

    self.catalog.resources.insert(decl.name.clone(), ResourceInfo {
        fields: decl.fields.clone(),
        table_override: decl.table_override.clone(),
    });
}
```

Note: This requires adding a `resources` field to the catalog struct and a `ResourceInfo` type. Check the catalog definition and add:
```rust
pub resources: HashMap<String, ResourceInfo>,
```

And:
```rust
pub struct ResourceInfo {
    pub fields: Vec<ResourceFieldDecl>,
    pub table_override: Option<String>,
}
```

Also add `Resource` to the name-collision check in the `Struct` and `Enum` branches so they also reject names already used by resources.

- [ ] **Step 4: Handle Resource variant in any remaining match arms**

Search for `ItemKind::` in semantic.rs and ensure all match arms handle `Resource`. For most, add `ItemKind::Resource(_) => {}` (no-op) since resources don't participate in function-level analysis.

- [ ] **Step 5: Run semantic tests**

Run: `cargo test -p sec4-core golden_semantic -- --nocapture 2>&1 | tail -20`
Expected: PASS (update golden files if error format differs slightly).

- [ ] **Step 6: Commit**

```bash
git add compiler/sec4-core/src/semantic.rs compiler/sec4-core/tests/fixtures/semantic/
git commit -m "feat: semantic validation for resource declarations"
```

---

### Task 5: Generate CRUD route plans from resources

**Files:**
- Modify: `compiler/sec4-cli/src/main.rs:1790-1805` (collect_lasm_route_plans)

- [ ] **Step 1: Add LasmResourcePlan struct**

In `compiler/sec4-cli/src/main.rs`, near the `LasmRunRoutePlan` definition (line 1755), add:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct LasmResourceFieldPlan {
    name: String,
    field_type: String,
    primary: bool,
    auto_fill: bool,
    default_value: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct LasmResourcePlan {
    name: String,
    table: String,
    fields: Vec<LasmResourceFieldPlan>,
    route_prefix: String,
}
```

- [ ] **Step 2: Write resource-to-route-plan generation**

Add a function that converts resource declarations to route plans:

```rust
fn generate_resource_route_plans(
    program: &sec4_core::ast::Program,
    explicit_routes: &[(String, String)], // (method, path) pairs from explicit registrations
) -> Vec<LasmRunRoutePlan> {
    let mut plans = Vec::new();

    for item in &program.items {
        let decl = match &item.kind {
            sec4_core::ast::ItemKind::Resource(decl) => decl,
            _ => continue,
        };

        let table = decl.table_override.clone().unwrap_or_else(|| {
            let snake = to_snake_case(&decl.name);
            format!("{}s", snake)
        });

        let prefix = format!("/{}", table);

        let resource_plan = LasmResourcePlan {
            name: decl.name.clone(),
            table: table.clone(),
            fields: decl.fields.iter().map(|f| {
                let field_type = match &f.ty.kind {
                    sec4_core::ast::TypeExprKind::Named { name, .. } => name.clone(),
                };
                LasmResourceFieldPlan {
                    name: f.name.clone(),
                    field_type,
                    primary: f.annotations.iter().any(|a| matches!(a, sec4_core::ast::ResourceFieldAnnotation::Primary)),
                    auto_fill: f.annotations.iter().any(|a| matches!(a, sec4_core::ast::ResourceFieldAnnotation::Auto)),
                    default_value: f.annotations.iter().find_map(|a| match a {
                        sec4_core::ast::ResourceFieldAnnotation::Default(v) => Some(v.clone()),
                        _ => None,
                    }),
                }
            }).collect(),
            route_prefix: prefix.clone(),
        };

        let plan_json = serde_json::to_string(&resource_plan).unwrap_or_default();

        // Generate 5 CRUD routes, skipping any that have explicit overrides
        let crud_ops = vec![
            ("GET", format!("{}", prefix), "list"),
            ("GET", format!("{}/:id", prefix), "get"),
            ("POST", format!("{}", prefix), "create"),
            ("POST", format!("{}/:id/update", prefix), "update"),
            ("POST", format!("{}/:id/delete", prefix), "delete"),
        ];

        for (method, path, op) in crud_ops {
            // Skip if developer registered an explicit handler for this method+path
            if explicit_routes.iter().any(|(m, p)| m == method && p == &path) {
                continue;
            }

            let mut headers = BTreeMap::new();
            headers.insert(
                "X-Sec4-Internal-Resource-Op".to_string(),
                op.to_string(),
            );
            headers.insert(
                "X-Sec4-Internal-Resource-Plan".to_string(),
                plan_json.clone(),
            );
            headers.insert(
                "Content-Type".to_string(),
                "application/json; charset=utf-8".to_string(),
            );

            plans.push(LasmRunRoutePlan {
                method: method.to_string(),
                path,
                status: if op == "create" { 201 } else { 200 },
                body: String::new(), // Resource dispatch handles body generation
                headers,
            });
        }
    }

    plans
}

fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_ascii_lowercase());
    }
    result
}
```

- [ ] **Step 3: Integrate into collect_lasm_route_plans**

At the end of `collect_lasm_route_plans`, after collecting explicit routes, append resource-generated routes:

```rust
// Collect explicit (method, path) pairs for override detection
let explicit_routes: Vec<(String, String)> = plans
    .iter()
    .map(|p| (p.method.clone(), p.path.clone()))
    .collect();

// Generate resource CRUD routes, skipping those with explicit overrides
let resource_plans = generate_resource_route_plans(program, &explicit_routes);
plans.extend(resource_plans);
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check -p sec4`
Expected: No errors.

- [ ] **Step 5: Commit**

```bash
git add compiler/sec4-cli/src/main.rs
git commit -m "feat: generate CRUD route plans from resource declarations"
```

---

### Task 6: Create resource dispatch module

**Files:**
- Create: `compiler/sec4-cli/src/lasm_resource_dispatch.rs`
- Modify: `compiler/sec4-cli/src/main.rs` (add mod declaration and dispatch intercept)

**Important:** The `LasmResourcePlan` and `LasmResourceFieldPlan` structs should be defined ONLY in this module and imported by `main.rs` (not duplicated). Remove the copies from Task 5's `main.rs` additions and use `use crate::lasm_resource_dispatch::{LasmResourcePlan, LasmResourceFieldPlan};` instead.

**Important:** The `validate_field` function uses manual validation (no `uuid` crate dependency). UUID validation checks format with a regex or manual hex-dash check instead of `uuid::Uuid::parse_str`.

- [ ] **Step 1: Create the dispatch module**

Create `compiler/sec4-cli/src/lasm_resource_dispatch.rs`:

```rust
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// Resource plan deserialized from route headers.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LasmResourcePlan {
    pub name: String,
    pub table: String,
    pub fields: Vec<LasmResourceFieldPlan>,
    pub route_prefix: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LasmResourceFieldPlan {
    pub name: String,
    pub field_type: String,
    pub primary: bool,
    pub auto_fill: bool,
    pub default_value: Option<String>,
}

/// Check if a response contains resource dispatch headers.
pub fn is_resource_route(headers: &BTreeMap<String, String>) -> bool {
    headers.contains_key("X-Sec4-Internal-Resource-Op")
}

/// Extract and parse the resource plan from headers.
pub fn parse_resource_plan(headers: &BTreeMap<String, String>) -> Option<(String, LasmResourcePlan)> {
    let op = headers.get("X-Sec4-Internal-Resource-Op")?.clone();
    let plan_json = headers.get("X-Sec4-Internal-Resource-Plan")?;
    let plan: LasmResourcePlan = serde_json::from_str(plan_json).ok()?;
    Some((op, plan))
}

/// Generate quoted SQL identifier (works for both Postgres and SQLite).
fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// Validate a field value against its declared type.
fn validate_field(field: &LasmResourceFieldPlan, value: &Value) -> Result<(), String> {
    match field.field_type.as_str() {
        "Uuid" => {
            let s = value.as_str().ok_or("expected string for Uuid field")?;
            // Manual UUID format check (8-4-4-4-12 hex digits) — no uuid crate dependency
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() != 5
                || parts[0].len() != 8
                || parts[1].len() != 4
                || parts[2].len() != 4
                || parts[3].len() != 4
                || parts[4].len() != 12
                || !parts.iter().all(|p| p.chars().all(|c| c.is_ascii_hexdigit()))
            {
                return Err(format!("invalid UUID: {}", s));
            }
        }
        "Email" => {
            let s = value.as_str().ok_or("expected string for Email field")?;
            if !s.contains('@') || s.len() < 3 {
                return Err(format!("invalid email: {}", s));
            }
        }
        "String" => {
            let s = value.as_str().ok_or("expected string")?;
            if s.is_empty() {
                return Err("string must not be empty".to_string());
            }
        }
        "Int64" | "Int" => {
            if !value.is_i64() && !value.is_u64() {
                if let Some(s) = value.as_str() {
                    s.parse::<i64>().map_err(|_| format!("invalid integer: {}", s))?;
                } else {
                    return Err("expected integer".to_string());
                }
            }
        }
        "Bool" => {
            if !value.is_boolean() {
                return Err("expected boolean".to_string());
            }
        }
        "Time" => {
            // Accept ISO 8601 strings
            let s = value.as_str().ok_or("expected ISO 8601 string for Time field")?;
            if s.is_empty() {
                return Err("time string must not be empty".to_string());
            }
        }
        _ => {}
    }
    Ok(())
}

/// Generate SQL and params for a CREATE operation.
pub fn generate_create_sql(
    plan: &LasmResourcePlan,
    body: &Value,
) -> Result<(String, Vec<Value>), Value> {
    let mut columns = Vec::new();
    let mut placeholders = Vec::new();
    let mut params = Vec::new();
    let mut param_index = 1;

    for field in &plan.fields {
        let value = if field.auto_fill {
            // Auto fields: generate at runtime
            match field.field_type.as_str() {
                "Time" => Value::String(chrono_now_iso()),
                "Uuid" if field.primary => {
                    // Use client-provided if present, else generate
                    if let Some(v) = body.get(&field.name) {
                        if let Err(e) = validate_field(field, v) {
                            return Err(validation_error(&field.name, &e));
                        }
                        v.clone()
                    } else {
                        Value::String(generate_uuid())
                    }
                }
                "Uuid" => Value::String(generate_uuid()),
                _ => continue,
            }
        } else if field.primary && !field.auto_fill {
            // Non-auto primary: required from body or generate if Uuid
            if let Some(v) = body.get(&field.name) {
                if let Err(e) = validate_field(field, v) {
                    return Err(validation_error(&field.name, &e));
                }
                v.clone()
            } else if field.field_type == "Uuid" {
                Value::String(generate_uuid())
            } else {
                return Err(validation_error(&field.name, "required field missing"));
            }
        } else if let Some(v) = body.get(&field.name) {
            if let Err(e) = validate_field(field, v) {
                return Err(validation_error(&field.name, &e));
            }
            v.clone()
        } else if let Some(ref default) = field.default_value {
            Value::String(default.clone())
        } else {
            return Err(validation_error(&field.name, "required field missing"));
        };

        columns.push(quote_ident(&field.name));
        placeholders.push(format!("${}", param_index));
        params.push(value);
        param_index += 1;
    }

    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        quote_ident(&plan.table),
        columns.join(", "),
        placeholders.join(", "),
    );

    Ok((sql, params))
}

/// Generate SQL for a GET-one operation.
pub fn generate_get_sql(plan: &LasmResourcePlan) -> String {
    let columns: Vec<String> = plan.fields.iter().map(|f| quote_ident(&f.name)).collect();
    let primary = plan.fields.iter().find(|f| f.primary).unwrap();
    format!(
        "SELECT {} FROM {} WHERE {} = $1 LIMIT 1",
        columns.join(", "),
        quote_ident(&plan.table),
        quote_ident(&primary.name),
    )
}

/// Generate SQL for a LIST operation.
pub fn generate_list_sql(plan: &LasmResourcePlan) -> String {
    let columns: Vec<String> = plan.fields.iter().map(|f| quote_ident(&f.name)).collect();
    let primary = plan.fields.iter().find(|f| f.primary).unwrap();
    format!(
        "SELECT {} FROM {} ORDER BY {} DESC LIMIT $1 OFFSET $2",
        columns.join(", "),
        quote_ident(&plan.table),
        quote_ident(&primary.name),
    )
}

/// Generate SQL for an UPDATE operation.
pub fn generate_update_sql(
    plan: &LasmResourcePlan,
    body: &Value,
) -> Result<(String, Vec<Value>), Value> {
    let primary = plan.fields.iter().find(|f| f.primary).unwrap();
    let mut set_clauses = Vec::new();
    let mut params: Vec<Value> = Vec::new();
    // $1 is reserved for the primary key (WHERE clause)
    let mut param_index = 2;

    for field in &plan.fields {
        if field.primary || field.auto_fill {
            continue;
        }
        if let Some(v) = body.get(&field.name) {
            if let Err(e) = validate_field(field, v) {
                return Err(validation_error(&field.name, &e));
            }
            set_clauses.push(format!("{} = ${}", quote_ident(&field.name), param_index));
            params.push(v.clone());
            param_index += 1;
        }
        // PATCH semantics: omitted fields are not updated
    }

    if set_clauses.is_empty() {
        return Err(json!({
            "ok": false,
            "status": 400,
            "error": { "code": "VALIDATION.EMPTY_UPDATE", "kind": "validation", "message": "no fields to update" }
        }));
    }

    let sql = format!(
        "UPDATE {} SET {} WHERE {} = $1",
        quote_ident(&plan.table),
        set_clauses.join(", "),
        quote_ident(&primary.name),
    );

    Ok((sql, params))
}

/// Generate SQL for a DELETE operation.
pub fn generate_delete_sql(plan: &LasmResourcePlan) -> String {
    let primary = plan.fields.iter().find(|f| f.primary).unwrap();
    format!(
        "DELETE FROM {} WHERE {} = $1",
        quote_ident(&plan.table),
        quote_ident(&primary.name),
    )
}

/// Build a standard success envelope.
pub fn success_envelope(status: u16, data: Value, trace_id: &str) -> Value {
    json!({
        "ok": true,
        "status": status,
        "traceId": trace_id,
        "timeMs": epoch_ms(),
        "data": data,
    })
}

/// Build a standard error envelope.
pub fn error_envelope(status: u16, code: &str, kind: &str, message: &str, trace_id: &str) -> Value {
    json!({
        "ok": false,
        "status": status,
        "traceId": trace_id,
        "timeMs": epoch_ms(),
        "error": { "code": code, "kind": kind, "message": message },
    })
}

fn validation_error(field: &str, message: &str) -> Value {
    json!({
        "ok": false,
        "status": 400,
        "error": {
            "code": "VALIDATION.INVALID",
            "kind": "validation",
            "message": format!("field `{}`: {}", field, message),
        }
    })
}

fn chrono_now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    // Simple ISO-like timestamp without chrono dependency
    format!("{}Z", secs)
}

fn generate_uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let nanos = d.as_nanos();
    // Deterministic-enough v4-like UUID from timestamp + random-ish bits
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        (nanos >> 96) as u32,
        (nanos >> 80) as u16,
        (nanos >> 64) as u16 & 0x0fff,
        ((nanos >> 48) as u16 & 0x3fff) | 0x8000,
        nanos as u64 & 0xffffffffffff,
    )
}

fn epoch_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
```

- [ ] **Step 2: Add mod declaration in main.rs**

Near the top of `compiler/sec4-cli/src/main.rs`, add:
```rust
mod lasm_resource_dispatch;
```

- [ ] **Step 3: Add resource dispatch intercept**

In the main request handling loop in `main.rs` (around line 10314-10339), before calling `apply_lasm_dynamic_response_materialization`, add an intercept:

```rust
// Resource dispatch intercept — before normal materialization
if lasm_resource_dispatch::is_resource_route(&response.headers) {
    // Resource routes are handled by the resource dispatch module.
    // This function reads the request body, validates fields, generates SQL,
    // executes via the DB adapter, and writes the response directly.
    // It returns true if the route was handled.
    // For now, this is a stub that will be wired in the next task.
}
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check -p sec4`
Expected: No errors (warnings about unused code in the new module are OK for now).

- [ ] **Step 5: Commit**

```bash
git add compiler/sec4-cli/src/lasm_resource_dispatch.rs compiler/sec4-cli/src/main.rs
git commit -m "feat: add resource dispatch module with SQL generation and validation"
```

---

### Task 7: Wire resource dispatch to DB adapter execution

**Files:**
- Modify: `compiler/sec4-cli/src/main.rs` (wire dispatch intercept to DB execution)
- Modify: `compiler/sec4-cli/src/lasm_resource_dispatch.rs` (add dispatch entry point)

This is the integration task that connects the resource dispatch module to the existing LASM DB adapter infrastructure. The dispatch entry point function should:

1. Detect resource route via `is_resource_route()`
2. Parse the `LasmResourcePlan` from headers
3. Based on op (create/get/list/update/delete):
   - Parse request body (for create/update)
   - Validate path params (for get/update/delete)
   - Generate SQL via the module's SQL generation functions
   - Execute via existing DB adapter path (reuse `handle_lasm_internal_db_exec_operation` or its underlying adapter calls)
   - Build and set the response envelope

- [ ] **Step 1: Add the dispatch entry point function**

In `lasm_resource_dispatch.rs`, add a top-level dispatch function that takes the same parameters as `apply_lasm_dynamic_response_materialization`:

```rust
/// Dispatch a resource CRUD operation. Returns true if the route was handled.
pub fn try_dispatch_resource_operation(
    response: &mut sec4_core::HttpResponse,
    request_body: &[u8],
    path_params: &BTreeMap<String, String>,
    route_headers: &BTreeMap<String, String>,
    trace_id: &str,
    // DB adapter context — match the types used by existing LASM DB dispatch
    // This will be refined when wiring to actual adapter calls
) -> bool {
    let (op, plan) = match parse_resource_plan(route_headers) {
        Some(pair) => pair,
        None => return false,
    };

    let result = match op.as_str() {
        "create" => dispatch_create(response, request_body, &plan, trace_id),
        "get" => dispatch_get(response, path_params, &plan, trace_id),
        "list" => dispatch_list(response, request_body, &plan, trace_id),
        "update" => dispatch_update(response, request_body, path_params, &plan, trace_id),
        "delete" => dispatch_delete(response, path_params, &plan, trace_id),
        _ => return false,
    };

    if let Err(error_envelope) = result {
        set_json_response(response, 400, &error_envelope);
    }

    true
}
```

Then implement each `dispatch_*` function stub that generates SQL, calls a placeholder for DB execution, and sets the response. The actual DB adapter wiring depends on the existing LASM dispatch infrastructure — study `handle_lasm_internal_db_exec_operation` in `lasm_db_runtime_dispatch.rs` and replicate the adapter call pattern.

- [ ] **Step 2: Wire the intercept in main.rs**

Replace the stub intercept from Task 6 Step 3 with an actual call:

```rust
if lasm_resource_dispatch::is_resource_route(&plan.headers) {
    let handled = lasm_resource_dispatch::try_dispatch_resource_operation(
        &mut response,
        &request.body,
        &exchange.path_params,
        &plan.headers,
        trace_id.as_str(),
    );
    if handled {
        // Resource dispatch handled the response — skip normal materialization
        // Continue to response writing
    }
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p sec4`
Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add compiler/sec4-cli/src/lasm_resource_dispatch.rs compiler/sec4-cli/src/main.rs
git commit -m "feat: wire resource dispatch to request handling loop"
```

---

### Task 8: Integration test — resource CRUD with SQLite

**Files:**
- Create: `examples/resource-basic/sec4.toml`
- Create: `examples/resource-basic/sec4.policy`
- Create: `examples/resource-basic/src/main.ut`
- Create: `scripts/smoke-resource-crud.sh`

- [ ] **Step 1: Create the example project**

Create `examples/resource-basic/sec4.toml`:
```toml
[package]
name = "resource-basic"
version = "0.1.0"
edition = "2026"

[build]
entry = "src/main.ut"
```

Create `examples/resource-basic/sec4.policy`:
```toml
[policy]
name = "resource-basic-dev"
mode = "enforce"

[http]
max_body_bytes = 65536

[resource]
max_list_limit = 100
default_list_limit = 20
allow_delete = true
require_auth = false

[auth]
mode = "none"
```

Create `examples/resource-basic/src/main.ut`:
```ut
resource Task {
  id: Uuid @primary,
  title: String,
  status: String @default("pending"),
  created_at: Time @auto,
}

fn health() effects { net } -> Int {
  res.text(200, "ok");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", health);
  http.serve(8080, router);
  0
}
```

- [ ] **Step 2: Create the smoke test script**

Create `scripts/smoke-resource-crud.sh`:
```bash
#!/usr/bin/env bash
set -euo pipefail

PORT=${1:-18099}
DB_BASE="/tmp/sec4-resource-smoke-$$"
mkdir -p "$DB_BASE"

echo "=== Building ==="
cargo build -p sec4

echo "=== Starting server (SQLite, port $PORT) ==="
target/debug/sec4 run \
  --path examples/resource-basic \
  --backend lasm \
  --db-adapter sqlite \
  --db-base "$DB_BASE" \
  --port "$PORT" &
SERVER_PID=$!
sleep 2

cleanup() { kill "$SERVER_PID" 2>/dev/null || true; rm -rf "$DB_BASE"; }
trap cleanup EXIT

BASE="http://127.0.0.1:$PORT"

echo "=== Health check ==="
curl -sf "$BASE/health" | grep -q "ok" && echo "PASS: health" || { echo "FAIL: health"; exit 1; }

echo "=== CREATE task ==="
CREATE_RESP=$(curl -sf -X POST "$BASE/tasks" \
  -H "Content-Type: application/json" \
  -d '{"title":"Test task","status":"active"}')
echo "$CREATE_RESP"
echo "$CREATE_RESP" | grep -q '"ok":true' && echo "PASS: create" || { echo "FAIL: create"; exit 1; }

# Extract id from response
TASK_ID=$(echo "$CREATE_RESP" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
echo "Created task: $TASK_ID"

echo "=== GET task ==="
GET_RESP=$(curl -sf "$BASE/tasks/$TASK_ID")
echo "$GET_RESP"
echo "$GET_RESP" | grep -q '"ok":true' && echo "PASS: get" || { echo "FAIL: get"; exit 1; }

echo "=== LIST tasks ==="
LIST_RESP=$(curl -sf "$BASE/tasks?limit=10&offset=0")
echo "$LIST_RESP"
echo "$LIST_RESP" | grep -q '"items"' && echo "PASS: list" || { echo "FAIL: list"; exit 1; }

echo "=== UPDATE task ==="
UPDATE_RESP=$(curl -sf -X POST "$BASE/tasks/$TASK_ID/update" \
  -H "Content-Type: application/json" \
  -d '{"title":"Updated task"}')
echo "$UPDATE_RESP"
echo "$UPDATE_RESP" | grep -q '"ok":true' && echo "PASS: update" || { echo "FAIL: update"; exit 1; }

echo "=== DELETE task ==="
DELETE_RESP=$(curl -sf -X POST "$BASE/tasks/$TASK_ID/delete")
echo "$DELETE_RESP"
echo "$DELETE_RESP" | grep -q '"ok":true' && echo "PASS: delete" || { echo "FAIL: delete"; exit 1; }

echo ""
echo "=== ALL RESOURCE CRUD TESTS PASSED ==="
```

```bash
chmod +x scripts/smoke-resource-crud.sh
```

- [ ] **Step 3: Run the smoke test (expect failures initially)**

Run: `bash scripts/smoke-resource-crud.sh`
Expected: Will fail on CREATE — this verifies the test harness works and pinpoints where runtime wiring needs fixing.

- [ ] **Step 4: Iterate on dispatch wiring until all 5 operations pass**

Debug cycle: fix dispatch → rebuild → rerun smoke → repeat.

Key areas to wire:
- **Table setup**: The spec says "tables must pre-exist" (no DDL generation). The smoke test script must create the table before starting the server. Add a setup step: `sqlite3 "$DB_BASE/sec4.db" "CREATE TABLE IF NOT EXISTS tasks (id TEXT PRIMARY KEY, title TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'pending', created_at TEXT NOT NULL);"` — this runs OUTSIDE sec4, matching the spec's "DDL is not the language's job" contract.
- Request body parsing: deserialize JSON body from `request.body`
- DB adapter calls: call through existing SQLite adapter path
- Response envelope: build standard `ok/status/data` shape

- [ ] **Step 5: Commit when all tests pass**

```bash
git add examples/resource-basic/ scripts/smoke-resource-crud.sh
git commit -m "feat: resource CRUD integration test with SQLite"
```

---

### Task 9: Policy integration for resources

**Files:**
- Modify: `compiler/sec4-cli/src/main.rs` (read resource policy)
- Modify: `compiler/sec4-cli/src/lasm_resource_dispatch.rs` (apply policy limits)

- [ ] **Step 1: Parse resource policy section**

Find where sec4.policy is parsed in main.rs. Add parsing for the `[resource]` section:

```rust
struct ResourcePolicy {
    max_list_limit: u32,
    default_list_limit: u32,
    allow_delete: bool,
    require_auth: bool,
}
```

Default: `max_list_limit=100, default_list_limit=20, allow_delete=true, require_auth=true`.

- [ ] **Step 2: Apply pagination limits in list dispatch**

In `dispatch_list`, clamp the `limit` query param to `[1, max_list_limit]` and default to `default_list_limit`.

- [ ] **Step 3: Suppress delete route when policy disables it**

In `generate_resource_route_plans`, skip the delete route when `allow_delete=false`.

- [ ] **Step 4: Run smoke test**

Run: `bash scripts/smoke-resource-crud.sh`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add compiler/sec4-cli/src/main.rs compiler/sec4-cli/src/lasm_resource_dispatch.rs
git commit -m "feat: resource policy integration (pagination, delete, auth)"
```

---

### Task 10: Custom handler override detection

**Files:**
- Modify: `compiler/sec4-cli/src/main.rs` (override detection in route plan generation)
- Create: test fixture

- [ ] **Step 1: Write test fixture**

Create `compiler/sec4-core/tests/fixtures/semantic/valid_resource_with_override.ut`:
```ut
resource Task {
  id: Uuid @primary,
  title: String,
}

fn createTask() effects { net, db.write } -> Int {
  let db = DbCap();
  db.exec(db, sql.q("INSERT INTO tasks (id, title) VALUES ($1, $2)", 2));
  res.json(201, "TaskResponse", 0);
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.post(router, "/tasks", createTask);
  http.serve(8080, router);
  0
}
```

Create `compiler/sec4-core/tests/fixtures/semantic/valid_resource_with_override.golden`:
```
OK
```

- [ ] **Step 2: Verify the override detection works**

The explicit `http.post(router, "/tasks", createTask)` should prevent the auto-generated POST /tasks route. The other 4 CRUD routes (GET /tasks, GET /tasks/:id, POST /tasks/:id/update, POST /tasks/:id/delete) should still be auto-generated.

Run: `cargo test -p sec4-core golden_semantic -- --nocapture 2>&1 | tail -20`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add compiler/sec4-core/tests/fixtures/semantic/
git commit -m "feat: resource custom handler override detection"
```

---

### Task 11: Final integration and cleanup

**Files:**
- All previously modified files

- [ ] **Step 1: Run full test suite**

Run: `cargo test -p sec4-core 2>&1 | tail -20`
Expected: All existing tests PASS, new resource tests PASS.

- [ ] **Step 2: Run clippy**

Run: `cargo clippy -p sec4-core -p sec4 -- -D warnings 2>&1 | tail -20`
Expected: No warnings.

- [ ] **Step 3: Run the smoke test one more time**

Run: `bash scripts/smoke-resource-crud.sh`
Expected: ALL RESOURCE CRUD TESTS PASSED.

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat: resource auto-CRUD — smart runtime for Path B

Adds resource declarations to .ut files that auto-generate 5 CRUD
endpoints (create, get, list, update, delete) with type-driven
validation, quoted SQL generation, and deterministic response
envelopes. Includes policy integration, custom handler override,
and SQLite integration test."
```
