# 54 v0 Security Stdlib Surface

This chapter defines an implementable v0 standard-library API surface for security-first backend development.

## 1) Core wrappers and safe sink types

```ailang
type Untrusted<T> = { __untrusted: T }
type Secret<T> = { __secret: T }

type SqlQuery = opaque
type HtmlSafe = opaque
type PublicUrl = opaque
type InternalUrl = opaque
type PathSafe = opaque
type HeaderName = opaque
type HeaderValue = opaque
type Cookie = opaque
type LogValue = opaque
```

## 2) Budget and context

```ailang
type Budget = {
  maxBodyBytes: Int64
  maxJsonBytes: Int64
  maxJsonDepth: Int
  deadlineMs: Int64
  maxNetBytes?: Int64
  maxDbRows?: Int64
}

type Ctx = {
  log: Log
  budget: Budget
  caps: Caps
  traceId: String
}
```

## 3) Capabilities

```ailang
type Caps = {
  db?: DbCap
  net?: NetCap
  fs?: FsCap
  secrets?: SecretsCap
  internalNet?: InternalNetCap
}

type DbCap = opaque
type TxCap = opaque
type NetCap = opaque
type InternalNetCap = opaque
type FsCap = opaque
type SecretsCap = opaque
```

## 4) Logging interface

```ailang
type Log = {
  info: fn(LogValue) effects { log } -> Unit
  warn: fn(LogValue) effects { log } -> Unit
  error: fn(LogValue) effects { log } -> Unit
}

module log {
  fn event(name: String) -> LogValue
  fn field(k: String, v: LogValue) -> LogValue
  fn obj(fields: List<{k:String, v:LogValue}>) -> LogValue

  fn str(s: String) -> LogValue
  fn i64(x: Int64) -> LogValue
  fn bool(b: Bool) -> LogValue

  fn redacted(label: String) -> LogValue
}
```

## 5) HTTP server and request API

```ailang
type Router = opaque
type Request = opaque
type Response = opaque

type Handler = fn(Ctx, Request) -> Result<Response, HttpError>

module http {
  fn router() -> Router
  fn get(r: Router, path: String, h: Handler) -> Router
  fn post(r: Router, path: String, h: Handler) -> Router
  fn serve(port: Int, r: Router) effects { net } -> Result<Unit, HttpError>
}

module req {
  fn body(ctx: Ctx, r: Request) effects { net } -> Result<Untrusted<Bytes>, HttpError>
  fn query(r: Request, name: String) -> Untrusted<Option<String>>
  fn pathParam(r: Request, name: String) -> Untrusted<String>
  fn header(r: Request, name: String) -> Untrusted<Option<String>>

  fn json<T>(ctx: Ctx, r: Request, s: Schema<T>)
    effects { net }
    -> Result<T, HttpError>
}
```

## 6) Response and sink APIs

```ailang
module res {
  fn text(status: Int, body: String) -> Response
  fn json<T>(status: Int, s: Schema<T>, value: T) -> Result<Response, HttpError>
  fn html(status: Int, body: HtmlSafe) -> Response

  fn setHeader(resp: Response, name: HeaderName, value: HeaderValue) -> Response
  fn addCookie(resp: Response, c: Cookie) -> Response
}
```

## 7) Validation and sanitization gates

```ailang
module validate {
  fn email(u: Untrusted<String>) -> Result<Email, ValidationError>
  fn nonEmpty(u: Untrusted<String>) -> Result<NonEmptyString, ValidationError>
  fn int64(u: Untrusted<String>) -> Result<Int64, ValidationError>
  fn uuid(u: Untrusted<String>) -> Result<Uuid, ValidationError>

  fn headerName(u: Untrusted<String>) -> Result<HeaderName, ValidationError>
  fn headerValue(u: Untrusted<String>) -> Result<HeaderValue, ValidationError>
}

module sanitize {
  fn html(u: Untrusted<String>) -> HtmlSafe
}
```

## 8) URL and SSRF-safe client surface

```ailang
module url {
  fn public(u: Untrusted<String>, ctx: Ctx)
    effects { net }
    -> Result<PublicUrl, ValidationError>

  fn internal(u: Untrusted<String>, ctx: Ctx, cap: InternalNetCap)
    effects { net }
    -> Result<InternalUrl, ValidationError>
}

module httpClient {
  fn get(ctx: Ctx, cap: NetCap, url: PublicUrl)
    effects { net }
    -> Result<Bytes, NetError>

  fn getInternal(ctx: Ctx, cap: InternalNetCap, url: InternalUrl)
    effects { net }
    -> Result<Bytes, NetError>
}
```

## 9) Path safety and filesystem

```ailang
module path {
  fn base(p: String) -> PathSafe
  fn under(base: PathSafe, u: Untrusted<String>) -> Result<PathSafe, ValidationError>
}

module fs {
  fn read(ctx: Ctx, cap: FsCap, p: PathSafe) effects { fs.read } -> Result<Bytes, FsError>
  fn write(ctx: Ctx, cap: FsCap, p: PathSafe, b: Bytes) effects { fs.write } -> Result<Unit, FsError>
}
```

## 10) SQL and DB surface

```ailang
module sql {
  fn q(template: String, params: List<DbParam>) -> SqlQuery
}

type DbParam = Int64 | Bool | String | Uuid | Time | Decimal | Bytes

module db {
  fn exec(ctx: Ctx, cap: DbCap, q: SqlQuery)
    effects { db.write }
    -> Result<Unit, DbError>

  fn queryOne<T>(ctx: Ctx, cap: DbCap, q: SqlQuery, row: RowSchema<T>)
    effects { db.read }
    -> Result<T, DbError>

  fn tx(ctx: Ctx, cap: DbCap)
    effects { db.tx }
    -> Result<TxCap, DbError>

  fn execTx(ctx: Ctx, tx: TxCap, q: SqlQuery)
    effects { db.write }
    -> Result<Unit, DbError>
}
```

## 11) Secrets and schema APIs

```ailang
module secrets {
  fn get(ctx: Ctx, cap: SecretsCap, name: String)
    effects { secrets.read }
    -> Result<Secret<String>, SecretError>

  fn redact(s: Secret<String>) -> String

  fn reveal<T>(ctx: Ctx, cap: SecretsCap, s: Secret<T>)
    effects { secrets.reveal }
    -> T
}

type Schema<T> = opaque

module json {
  fn decode<T>(ctx: Ctx, s: Schema<T>, raw: Untrusted<Bytes>) -> Result<T, JsonError>
  fn encode<T>(s: Schema<T>, value: T) -> Json
}
```

## 12) Canonical handler pattern

```ailang
schema CreateUserRequest {
  email: Email = validate.email
}

function createUser(ctx: Ctx, r: Request)
  effects { net, db.write, log }
  -> Result<Response, HttpError>
{
  const cap = ctx.caps.db? else return Err(HttpError.forbidden())

  const body = req.json(ctx, r, CreateUserRequest)?
  db.exec(ctx, cap, sql`INSERT INTO users(email) VALUES(${body.email})`)?

  ctx.log.info(log.obj([
    {k:"event", v: log.str("user_created")},
    {k:"email", v: log.str(body.email.toString())},
  ]))

  return res.text(201, "ok")
}
```
