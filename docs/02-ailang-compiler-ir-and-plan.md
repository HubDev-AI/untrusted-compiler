# AILang Compiler, IR, and Implementation Plan

## 1) Minimal IR design (good for v0.1, web backend focus)

A practical v0.1 pipeline is **AST - HIR (typed, resolved) - MIR (SSA-like, effect-aware) - LLVM/Cranelift/native C backend**. Keep it small:

### 1.1 HIR (High-level IR)

Purpose: after parsing + name resolution, before lowering control flow.

- Fully qualified names (modules resolved)
- Types attached to nodes (after typecheck)
- Desugared constructs:

- `if` / `match` rewritten into explicit conditional/match nodes
- `?` rewritten to `try` expression node
- method calls normalized to function calls: `x.f(a)` - `f(x, a)`
- `await` preserved as `Await(expr)`
- Effects not yet validated, but every call site has a **callee symbol**.

**HIR node essentials**

- `HirExpr { kind, ty, span }`
- `HirStmt { kind, span }`
- `HirFn { sig, body, attrs, declared_effects, contracts }`

### 1.2 MIR (Mid-level IR) - minimal SSA + blocks

Purpose: easy effect checking + codegen.  
Represent each function as:

- locals/temps typed
- basic blocks with terminators
- explicit control flow
- explicit calls with **effect tags**
- explicit error propagation (from `?`)

**Core MIR types**

```text
MirFunction {
  name: Symbol,
  params: [LocalId],
  locals: [Local { ty }],
  blocks: [BasicBlock],
  declared_effects: EffectSet,
  requires: [Predicate],   // optional lowering
  ensures:  [Predicate],
}

BasicBlock {
  stmts: [Statement],
  term: Terminator,
}

Statement =
  | Assign(LocalId, Rvalue)
  | StorageLive(LocalId) | StorageDead(LocalId)  // optional
  | Assert(Predicate, AssertKind)                // for contracts (debug builds)

Rvalue =
  | Use(Operand)
  | BinOp(op, Operand, Operand)
  | UnOp(op, Operand)
  | MakeStruct(Type, [Field: Operand])
  | MakeEnum(Type, Variant, [Operand])
  | Tuple([Operand]) | List([Operand]) | Map([(Operand,Operand)])
  | Cast(Operand, Type)
  | Call { callee: Callee, args: [Operand], effects: EffectSet, awaitable: bool }
  | DbQuery { sql: SqlTemplateId, params: [Operand] }   // optional builtin lowering
  | JsonDecode { ty: Type, bytes: Operand }             // optional builtin lowering
  | JsonEncode { value: Operand }

Operand =
  | Copy(LocalId)
  | Move(LocalId)
  | Const(ConstValue)

Terminator =
  | Return(Operand?)
  | Goto(BlockId)
  | Switch { discr: Operand, targets: [ConstValue -> BlockId], otherwise: BlockId }
  | CondBr { cond: Operand, then_bb: BlockId, else_bb: BlockId }
  | CallTerm { dest: LocalId, call: Rvalue::Call, next: BlockId, unwind: BlockId? }
  | Try { res: Operand, ok_bb: BlockId, err_bb: BlockId }  // for Result-based `?`
```

### 1.3 Effects in MIR

Every `Call` includes `effects: EffectSet` (from callee signature or builtins). Effect checking becomes:

- Gather union of effects for all reachable calls
- Validate `used_effects - declared_effects`
- Also enforce 'no blocking calls in async' by effect tags (e.g., `db.blocking`).

### 1.4 Lowering `?` (Result propagation)

In HIR: `Try(expr)` node.  
In MIR: compile into a `Try` terminator:

```text
tmp = call foo(...)
Try { res: tmp, ok_bb: bb_ok, err_bb: bb_err }

bb_err:
  Return(Err(tmp_err))  // or map error type, if From is in scope
```

### 1.5 Match lowering

`match` lowering:

- Evaluate scrutinee into a temp
- `Switch` on discriminant (enums use variant tag)
- For enums with payload, bind payload into locals on each arm entry

### 1.6 Contracts lowering (v0.1)

- In debug/test builds only: `requires` as `Assert` at function entry.
- `ensures` as `Assert` right before each return.
- `old(expr)` can be supported later; for v0.1, you can restrict `old` to *pure locals/params only* or omit it.

---

## 2) v0.1 implementation plan (milestones)

Each milestone ends with something demonstrably runnable.

### Milestone 0

- Repo + CLI skeleton
- `ailang` CLI: `build/run/test/fmt/lint` (fmt/lint can be placeholders)
- Project manifest parse (`ailang.toml`) + lockfile stub
- Basic error reporting framework (spans, colored diagnostics)

**Exit:** `ailang build` parses manifest, compiles a single file 'hello'.

---

### Milestone 1

- Tokenizer + Parser (AST)
- Tokenizer: identifiers, literals, keywords, operators, delimiters, comments
- Parser:

- items: `import`, `struct`, `enum`, `fn`, `refined type`, `capability`, `test`, `property`
- statements: `let/mut`, `return`, `if`, `match`, expr stmt
- expressions with precedence (Pratt or precedence climbing)
- AST spans retained everywhere

**Exit:** `ailang parse` prints AST; parses example web skeleton.

---

### Milestone 2

- Name resolution + module system (AST
- HIR)
- Build module graph from filesystem layout (`src/...`)
- Resolve imports to symbols
- Build symbol tables:

- types, functions, traits, enum variants, fields
- Diagnose:
- duplicate definitions
- missing imports
- unknown names

**Exit:** `ailang check` resolves names across modules.

---

### Milestone 3

- Type checker (HIR typing)

Implement minimal type system:

- primitives, structs, enums, Option/Result, generics (limited)
- type inference for locals (Hindley-ish but constrained; or simple bidirectional typing)
- method call desugaring
- pattern typing for `match`
- exhaustiveness check (start with enums only, then extend)

**Exit:** can typecheck a small service-like codebase; good diagnostics.

---

### Milestone 4

- Effects checker
- Parse `effects { ... }` clauses in fn sigs
- Compute used effects from call graph:

- for direct calls: read callee's declared effects
- for builtins: predefined effects
- Verify used - declared
- Async rules:
- `await` only allowed on `async` call results
- disallow `db.blocking` inside async (optional)

**Exit:** effect errors show missing effect names and where used.

---

### Milestone 5

- MIR lowering
- Lower typed HIR to MIR:

- blocks, temps, terminators
- desugar `if`, `match`, `?`, `await`
- Simple MIR interpreter optional (helps testing early!)

**Exit:** can emit MIR text (`--emit=mir`) for any function.

---

### Milestone 6

- Simple codegen (pick one)

Fastest realistic choices:

1.  **Cranelift** (easier than LLVM setup), or

2.  Emit **C** code and compile with system `clang` (very pragmatic v0.1).

Start with:

- Int/Bool/String (String as runtime struct)
- structs/enums (enums as tagged union)
- function calls, returns
- basic control flow
- Result/Option as tagged unions

**Exit:** compiled executable runs pure functions + simple control flow.

---

### Milestone 7

- Runtime + stdlib minimal

Add runtime support:

- String, Bytes, Vec/List
- HashMap/Map (can defer)
- Result/Option helpers
- Basic formatting/logging

**Exit:** 'hello web service' can print logs, handle CLI args.

---

### Milestone 8

- HTTP + JSON (v0.1 web backend)
- Choose an embedded runtime approach:

- If codegen target is C: link against a small C HTTP server library
- If target is native/Cranelift: you can embed a Rust runtime and expose FFI shims
- Implement stdlib modules:
- `http.server`: Router, Request/Response, `serve(port, router)`
- `json`: encode/decode derived for structs
- Derive mechanism (keep it simple):
- `@derive(JsonCodec)` expands in compiler to generated impls (no general macros)

**Exit:** A real service:

- `GET /health -> 200 ok`
- `POST /users` decodes JSON, validates refined types, responds JSON.

---

### Milestone 9

- DB (optional for v0.1+)
- Parameterized SQL template literal `sql"..."` parsed by compiler into template + params
- Provide `db.query_one<T>` with row decoding

**Exit:** CRUD endpoint with Postgres.

---

## 3) Tokenization spec

### 3.1 Character classes

- Whitespace: space, tab, newline (`\n`), carriage return ignored
- Comments:

- Line: `// ... \n`
- Block: `/* ... */` (nesting optional; if not supported, error on nested)

### 3.2 Tokens

**Keywords (reserved)**  
`import module struct enum trait impl for refined type where invariant capability fn async effects requires ensures let mut const return if else while for in match break continue await unsafe test property true false null`

**Identifiers**

- `Ident`: `[A-Za-z_][A-Za-z0-9_]*`
- Convention:

- `UpperIdent` starts uppercase (types/variants), but token is same; parser/semantic decides.

**Literals**

- Int: `123`
- Int64 suffix: `123i64`
- Float: `123.45` with optional exponent
- String: `"..."` with escapes
- Bytes: `b"..."` (treat as bytes literal)
- Specialized:

- `uuid"..."`, `time"..."`, `dur"..."` can lex as `IDENT + STRING` or single literal token (your call; single token simplifies parsing)

**Delimiters**  
`( ) { } [ ] , : ; . -> =>`

**Operators**

- Arithmetic: `+
- * / %`
- Comparison: `== != < <= > >=`
- Boolean: `&& || !`
- Try: `?`
- Assignment: `=`
- Pipe for patterns: `|`
- Optional: `+` unary, `-` unary

### 3.3 Newline/semicolon rules

Pick one for v0.1 (simplest):

**Rule A (recommended): semicolons optional, newline acts as separator**

- The lexer emits `NEWLINE` tokens.
- The parser treats `;` and `NEWLINE` as `Terminator`, but:

- Newlines are ignored inside `() [] {}` nesting.
- Newline is ignored after tokens that obviously continue an expression:
    `.` `,` `:` `->` `=>` binary operators `(` `[` `{` - Newline is ignored before tokens that obviously continue:  
    `)` `]` `}` `,` `.` `?`

**Rule B: semicolons required**

- Much simpler parsing. Many people dislike it.

If you want 'AI-friendly,' Rule A is fine **as long as it's deterministic**.

---

## 4) Fully unambiguous grammar (LR/LL friendly) with precedence encoded

Instead of the earlier EBNF with `Expr = ...` recursion, use precedence levels:

```ebnf
(* Expression grammar with precedence encoded *)

Expr            = LambdaExpr | OrExpr ;

LambdaExpr      = "fn" , WS , "(" , WS , [ ParamList ] , WS , ")" , WS , "=>" , WS , Expr
                | Ident , WS , "=>" , WS , Expr ;

OrExpr          = AndExpr , { WS , "||" , WS , AndExpr } ;
AndExpr         = EqExpr  , { WS , "&&" , WS , EqExpr } ;
EqExpr          = RelExpr , { WS , ( "==" | "!=" ) , WS , RelExpr } ;
RelExpr         = AddExpr , { WS , ( "<" | "<=" | ">" | ">=" ) , WS , AddExpr } ;
AddExpr         = MulExpr , { WS , ( "+" | "-" ) , WS , MulExpr } ;
MulExpr         = PrefixExpr , { WS , ( "*" | "/" | "%" ) , WS , PrefixExpr } ;

PrefixExpr      = { PrefixOp , WS } , PostfixExpr ;
PrefixOp        = "!" | "-" | "+" | "await" ;

PostfixExpr     = PrimaryExpr , { WS? , PostfixOp } ;
PostfixOp       = CallOp | MemberOp | IndexOp | TryOp ;
CallOp          = "(" , WS , [ ArgList ] , WS , ")" ;
MemberOp        = "." , Ident ;
IndexOp         = "[" , WS , Expr , WS , "]" ;
TryOp           = "?" ;

PrimaryExpr     = Literal
                | IdentPath
                | "(" , WS , Expr , WS , ")"
                | Block
                | IfExpr
                | MatchExpr
                | StructInit
                | ListLit
                | MapLit
                | SetLit ;

IfExpr          = "if" , WS , Expr , WS , Block ,
                  { WS , "else" , WS , "if" , WS , Expr , WS , Block } ,
                  [ WS , "else" , WS , Block ] ;

MatchExpr       = "match" , WS , Expr , WS , "{" , WS ,
                  MatchArm , { WS , MatchArm } ,
                  WS , "}" ;

StructInit      = IdentPath , WS , "{" , WS , [ FieldInitList ] , WS , "}" ;
```

This is LR-friendly and easy to implement as Pratt too.

---

## 5) Reference parser skeleton (Rust) matching the EBNF style

This is a minimal, workable scaffold: lexer - Pratt expr parser - item parsing. (It's not the whole compiler, but it's a correct foundation.)

```rust
// Cargo.toml suggestion:
// [dependencies]
// thiserror = "1"

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Import, Module, Struct, Enum, Fn, Async, Let, Mut, Return, If, Else, Match,
    Refined, Type, Where, Capability, Effects, Requires, Ensures,
    Test, Property, Trait, Impl, For, Const, Invariant,
    Await, Unsafe,
    True, False, Null,

    // Ident & literals
    Ident(String),
    Int(String),
    Float(String),
    Str(String),

    // Punct / ops
    LParen, RParen, LBrace, RBrace, LBrack, RBrack,
    Comma, Colon, Semi, Dot,
    Arrow, FatArrow, Assign,
    Plus, Minus, Star, Slash, Percent,
    EqEq, Ne, Lt, Le, Gt, Ge,
    AndAnd, OrOr, Bang,
    Question,
    Pipe,

    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: (usize, usize),
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("unexpected token {found:?} at {span:?}, expected {expected}")]
    Unexpected { found: TokenKind, span: (usize, usize), expected: &'static str },
    #[error("unexpected EOF")]
    Eof,
}

// ---------- AST (minimal) ----------
#[derive(Debug)]
pub enum Item {
    Fn(FnDecl),
    Struct(StructDecl),
    Import(ImportDecl),
    // ...
}

#[derive(Debug)]
pub struct ImportDecl {
    pub path: Vec<String>,
    pub alias: Option<String>,
}

#[derive(Debug)]
pub struct StructDecl {
    pub name: String,
    pub fields: Vec<(String, TypeAst)>,
}

#[derive(Debug)]
pub struct FnDecl {
    pub is_async: bool,
    pub name: String,
    pub params: Vec<(String, TypeAst)>,
    pub ret: TypeAst,
    pub body: Block,
}

#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub enum Stmt {
    Let { is_mut: bool, name: String, ty: Option<TypeAst>, init: Option<Expr> },
    Return(Option<Expr>),
    Expr(Expr),
    // if/match as stmt if you want
}

#[derive(Debug)]
pub enum Expr {
    Ident(Vec<String>), // path a.b.c
    Int(i64),
    Str(String),
    Call { callee: Box<Expr>, args: Vec<Expr> },
    Member { base: Box<Expr>, name: String },
    Index { base: Box<Expr>, idx: Box<Expr> },
    Unary { op: UnaryOp, rhs: Box<Expr> },
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr> },
    Try(Box<Expr>),
    If { cond: Box<Expr>, then_blk: Block, else_blk: Option<Block> },
    Match { scrut: Box<Expr>, arms: Vec<(Pattern, ExprOrBlock)> },
    Block(Block),
    StructInit { path: Vec<String>, fields: Vec<(String, Expr)> },
    // ...
}

#[derive(Debug)]
pub enum ExprOrBlock { Expr(Expr), Block(Block) }

#[derive(Debug)]
pub enum Pattern {
    Wildcard,
    Ident(String),
    // literal, enum payload, struct pattern...
}

#[derive(Debug)]
pub enum UnaryOp { Not, Neg, Pos, Await }

#[derive(Debug)]
pub enum BinOp {
    Add, Sub, Mul, Div, Rem,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
}

#[derive(Debug)]
pub enum TypeAst {
    Path(Vec<String>),
    Generic { base: Vec<String>, args: Vec<TypeAst> },
    Func { params: Vec<TypeAst>, ret: Box<TypeAst> },
}

// ---------- Pratt precedence ----------
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
enum Prec {
    None = 0,
    Or = 1,
    And = 2,
    Eq = 3,
    Rel = 4,
    Add = 5,
    Mul = 6,
    Prefix = 7,
    Postfix = 8,
}

fn infix_prec(tok: &TokenKind) -> Option<(Prec, BinOp)> {
    use TokenKind::*;
    Some(match tok {
        OrOr => (Prec::Or, BinOp::Or),
        AndAnd => (Prec::And, BinOp::And),
        EqEq => (Prec::Eq, BinOp::Eq),
        Ne => (Prec::Eq, BinOp::Ne),
        Lt => (Prec::Rel, BinOp::Lt),
        Le => (Prec::Rel, BinOp::Le),
        Gt => (Prec::Rel, BinOp::Gt),
        Ge => (Prec::Rel, BinOp::Ge),
        Plus => (Prec::Add, BinOp::Add),
        Minus => (Prec::Add, BinOp::Sub),
        Star => (Prec::Mul, BinOp::Mul),
        Slash => (Prec::Mul, BinOp::Div),
        Percent => (Prec::Mul, BinOp::Rem),
        _ => return None,
    })
}

// ---------- Parser ----------
pub struct Parser {
    tokens: Vec<Token>,
    i: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { Self { tokens, i: 0 } }

    fn peek(&self) -> &Token { self.tokens.get(self.i).unwrap() }
    fn at(&self, kind: &TokenKind) -> bool { &self.peek().kind == kind }

    fn bump(&mut self) -> Token {
        let t = self.tokens[self.i].clone();
        self.i += 1;
        t
    }

    fn expect(&mut self, expected: &'static str, pred: impl Fn(&TokenKind)->bool) -> Result<Token, ParseError> {
        if pred(&self.peek().kind) { Ok(self.bump()) }
        else {
            let t = self.peek().clone();
            Err(ParseError::Unexpected { found: t.kind, span: t.span, expected })
        }
    }

    fn eat_newlines(&mut self) {
        while matches!(self.peek().kind, TokenKind::Newline) { self.bump(); }
    }

    // ---- Top-level ----
    pub fn parse_program(&mut self) -> Result<Vec<Item>, ParseError> {
        let mut items = vec![];
        self.eat_newlines();
        while !matches!(self.peek().kind, TokenKind::Eof) {
            items.push(self.parse_item()?);
            self.eat_newlines();
        }
        Ok(items)
    }

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        use TokenKind::*;
        match &self.peek().kind {
            Import => Ok(Item::Import(self.parse_import()?)),
            Struct => Ok(Item::Struct(self.parse_struct()?)),
            Fn | Async => Ok(Item::Fn(self.parse_fn()?)),
            _ => {
                let t = self.peek().clone();
                Err(ParseError::Unexpected { found: t.kind, span: t.span, expected: "item" })
            }
        }
    }

    fn parse_import(&mut self) -> Result<ImportDecl, ParseError> {
        self.expect("import", |k| matches!(k, TokenKind::Import))?;
        let mut path = vec![];
        // path = Ident { "." Ident }
        let first = self.expect("identifier", |k| matches!(k, TokenKind::Ident(_)))?;
        if let TokenKind::Ident(s) = first.kind { path.push(s); }

        while matches!(self.peek().kind, TokenKind::Dot) {
            self.bump();
            let t = self.expect("identifier", |k| matches!(k, TokenKind::Ident(_)))?;
            if let TokenKind::Ident(s) = t.kind { path.push(s); }
        }

        let alias = if matches!(self.peek().kind, TokenKind::Ident(_)) {
            // optionally support: import a.b as C
            None
        } else { None };

        self.consume_terminator()?;
        Ok(ImportDecl { path, alias })
    }

    fn parse_struct(&mut self) -> Result<StructDecl, ParseError> {
        self.expect("struct", |k| matches!(k, TokenKind::Struct))?;
        let name = self.take_ident("struct name")?;
        self.expect("{", |k| matches!(k, TokenKind::LBrace))?;
        self.eat_newlines();
        let mut fields = vec![];
        while !matches!(self.peek().kind, TokenKind::RBrace) {
            let fname = self.take_ident("field name")?;
            self.expect(":", |k| matches!(k, TokenKind::Colon))?;
            let fty = self.parse_type()?;
            fields.push((fname, fty));
            if matches!(self.peek().kind, TokenKind::Comma) { self.bump(); }
            self.eat_newlines();
        }
        self.expect("}", |k| matches!(k, TokenKind::RBrace))?;
        self.maybe_terminator()?;
        Ok(StructDecl { name, fields })
    }

    fn parse_fn(&mut self) -> Result<FnDecl, ParseError> {
        let is_async = if matches!(self.peek().kind, TokenKind::Async) { self.bump(); true } else { false };
        self.expect("fn", |k| matches!(k, TokenKind::Fn))?;
        let name = self.take_ident("function name")?;
        self.expect("(", |k| matches!(k, TokenKind::LParen))?;
        let params = self.parse_params()?;
        self.expect(")", |k| matches!(k, TokenKind::RParen))?;

        // ignore clauses for now; you can parse effects/requires/ensures here

        self.expect("->", |k| matches!(k, TokenKind::Arrow))?;
        let ret = self.parse_type()?;
        let body = self.parse_block()?;
        Ok(FnDecl { is_async, name, params, ret, body })
    }

    fn parse_params(&mut self) -> Result<Vec<(String, TypeAst)>, ParseError> {
        let mut params = vec![];
        self.eat_newlines();
        if matches!(self.peek().kind, TokenKind::RParen) { return Ok(params); }
        loop {
            let name = self.take_ident("param name")?;
            self.expect(":", |k| matches!(k, TokenKind::Colon))?;
            let ty = self.parse_type()?;
            params.push((name, ty));
            if matches!(self.peek().kind, TokenKind::Comma) { self.bump(); self.eat_newlines(); continue; }
            break;
        }
        Ok(params)
    }

    fn parse_type(&mut self) -> Result<TypeAst, ParseError> {
        // Minimal: parse a path with optional generics: Foo<Bar,Baz>
        // Extend later with fn types.
        let mut path = vec![self.take_ident("type")?];
        while matches!(self.peek().kind, TokenKind::Dot) {
            self.bump();
            path.push(self.take_ident("type path segment")?);
        }
        if matches!(self.peek().kind, TokenKind::Lt) {
            // you'd need tokens for < >
            unimplemented!("generic args tokenization not shown");
        }
        Ok(TypeAst::Path(path))
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        self.expect("{", |k| matches!(k, TokenKind::LBrace))?;
        self.eat_newlines();
        let mut stmts = vec![];
        while !matches!(self.peek().kind, TokenKind::RBrace) {
            stmts.push(self.parse_stmt()?);
            self.eat_newlines();
        }
        self.expect("}", |k| matches!(k, TokenKind::RBrace))?;
        Ok(Block { stmts })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        use TokenKind::*;
        match &self.peek().kind {
            Let => {
                self.bump();
                let name = self.take_ident("let name")?;
                let ty = if matches!(self.peek().kind, Colon) {
                    self.bump();
                    Some(self.parse_type()?)
                } else { None };
                let init = if matches!(self.peek().kind, Assign) {
                    self.bump();
                    Some(self.parse_expr(Prec::None)?)
                } else { None };
                self.consume_terminator()?;
                Ok(Stmt::Let { is_mut: false, name, ty, init })
            }
            Mut => {
                self.bump();
                let name = self.take_ident("mut name")?;
                let ty = if matches!(self.peek().kind, Colon) { self.bump(); Some(self.parse_type()?) } else { None };
                self.expect("=", |k| matches!(k, Assign))?;
                let init = Some(self.parse_expr(Prec::None)?);
                self.consume_terminator()?;
                Ok(Stmt::Let { is_mut: true, name, ty, init })
            }
            Return => {
                self.bump();
                let expr = if matches!(self.peek().kind, Semi | Newline) { None } else { Some(self.parse_expr(Prec::None)?) };
                self.consume_terminator()?;
                Ok(Stmt::Return(expr))
            }
            _ => {
                let e = self.parse_expr(Prec::None)?;
                self.consume_terminator()?;
                Ok(Stmt::Expr(e))
            }
        }
    }

    // Pratt parser
    fn parse_expr(&mut self, min_prec: Prec) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_prefix()?;

        loop {
            // postfix ops (call/member/index/try)
            lhs = match &self.peek().kind {
                TokenKind::LParen => self.parse_call(lhs)?,
                TokenKind::Dot => {
                    self.bump();
                    let name = self.take_ident("member name")?;
                    Expr::Member { base: Box::new(lhs), name }
                }
                TokenKind::LBrack => {
                    self.bump();
                    let idx = self.parse_expr(Prec::None)?;
                    self.expect("]", |k| matches!(k, TokenKind::RBrack))?;
                    Expr::Index { base: Box::new(lhs), idx: Box::new(idx) }
                }
                TokenKind::Question => { self.bump(); Expr::Try(Box::new(lhs)) }
                _ => break,
            };

            // infix ops
            if let Some((prec, op)) = infix_prec(&self.peek().kind) {
                if prec < min_prec { break; }
                let _tok = self.bump();
                // left-associative: next min_prec is prec+1
                let rhs = self.parse_expr((prec as u8 + 1).into_prec())?;
                lhs = Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
                continue;
            }
            break;
        }
        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        use TokenKind::*;
        match &self.peek().kind {
            Bang => { self.bump(); Ok(Expr::Unary { op: UnaryOp::Not, rhs: Box::new(self.parse_expr(Prec::Prefix)?) }) }
            Minus => { self.bump(); Ok(Expr::Unary { op: UnaryOp::Neg, rhs: Box::new(self.parse_expr(Prec::Prefix)?) }) }
            Plus => { self.bump(); Ok(Expr::Unary { op: UnaryOp::Pos, rhs: Box::new(self.parse_expr(Prec::Prefix)?) }) }
            Await => { self.bump(); Ok(Expr::Unary { op: UnaryOp::Await, rhs: Box::new(self.parse_expr(Prec::Prefix)?) }) }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        use TokenKind::*;
        match self.bump().kind {
            Ident(s) => {
                let mut path = vec![s];
                while matches!(self.peek().kind, Dot) {
                    self.bump();
                    path.push(self.take_ident("path segment")?);
                }
                Ok(Expr::Ident(path))
            }
            Int(s) => Ok(Expr::Int(s.parse().unwrap())),
            Str(s) => Ok(Expr::Str(s)),
            LParen => {
                let e = self.parse_expr(Prec::None)?;
                self.expect(")", |k| matches!(k, RParen))?;
                Ok(e)
            }
            LBrace => {
                // Put back '{' handling: easiest is to not bump and call parse_block that expects '{'
                self.i -= 1;
                Ok(Expr::Block(self.parse_block()?))
            }
            If => {
                let cond = self.parse_expr(Prec::None)?;
                let then_blk = self.parse_block()?;
                let else_blk = if matches!(self.peek().kind, Else) {
                    self.bump();
                    Some(self.parse_block()?)
                } else { None };
                Ok(Expr::If { cond: Box::new(cond), then_blk, else_blk })
            }
            other => {
                Err(ParseError::Unexpected { found: other, span: self.peek().span, expected: "expression" })
            }
        }
    }

    fn parse_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        self.expect("(", |k| matches!(k, TokenKind::LParen))?;
        self.eat_newlines();
        let mut args = vec![];
        if !matches!(self.peek().kind, TokenKind::RParen) {
            loop {
                args.push(self.parse_expr(Prec::None)?);
                if matches!(self.peek().kind, TokenKind::Comma) { self.bump(); self.eat_newlines(); continue; }
                break;
            }
        }
        self.expect(")", |k| matches!(k, TokenKind::RParen))?;
        Ok(Expr::Call { callee: Box::new(callee), args })
    }

    fn take_ident(&mut self, expected: &'static str) -> Result<String, ParseError> {
        let t = self.expect(expected, |k| matches!(k, TokenKind::Ident(_)))?;
        if let TokenKind::Ident(s) = t.kind { Ok(s) } else { unreachable!() }
    }

    fn consume_terminator(&mut self) -> Result<(), ParseError> {
        // Accept ; or Newline (and eat multiple newlines)
        match &self.peek().kind {
            TokenKind::Semi => { self.bump(); Ok(()) }
            TokenKind::Newline => { self.eat_newlines(); Ok(()) }
            TokenKind::RBrace => Ok(()), // allow implicit terminator before }
            _ => {
                let t = self.peek().clone();
                Err(ParseError::Unexpected { found: t.kind, span: t.span, expected: "statement terminator" })
            }
        }
    }

    fn maybe_terminator(&mut self) -> Result<(), ParseError> {
        if matches!(self.peek().kind, TokenKind::Semi) { self.bump(); }
        self.eat_newlines();
        Ok(())
    }
}

// Small helper: convert u8 to Prec safely
trait IntoPrec { fn into_prec(self) -> Prec; }
impl IntoPrec for u8 {
    fn into_prec(self) -> Prec {
        match self {
            1 => Prec::Or, 2 => Prec::And, 3 => Prec::Eq, 4 => Prec::Rel,
            5 => Prec::Add, 6 => Prec::Mul, 7 => Prec::Prefix, _ => Prec::None
        }
    }
}
```

This skeleton intentionally:

- uses Pratt parsing for expressions (easiest to keep precedence correct),
- leaves generics (`<...>`) and some tokens as TODO,
- is still aligned with the earlier EBNF and the precedence-encoded grammar.

---
