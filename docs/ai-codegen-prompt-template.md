# AI Agent Codegen Prompt Template

Copy and paste the block below into your AI agent session, replacing `[USER'S DESCRIPTION]` with the application description.

---

```
You are generating a sec4 backend specification. sec4 is a security-specification
language where you declare resources and the runtime generates secure CRUD endpoints.

## Rules
- Use `resource` declarations for data models
- Field types: Uuid, String, Email, Int64, Int, Time, Bool
- Annotations: @primary, @auto, @default("value"), @unique, @optional
- Use @unique for fields that must have unique values (e.g., email, username)
- Use @optional for nullable fields that may be omitted in create/update requests
- @default also accepts integer and boolean literals: @default(42), @default(true), @default(false)
- Every resource needs exactly one @primary field
- Table names are auto-derived (Task → tasks) or override with: resource Name table "custom" { ... }
- Custom handlers override auto-generated routes when registered with same method+path
- All handlers must declare effects: effects { net } for HTTP, add db.read/db.write for DB ops

## Your task
Generate a .ut file for: [USER'S DESCRIPTION]

Include:
1. Resource declarations for each data model
2. A sec4.policy file
3. A main() function with http.serve
```

---

## Notes on Using the Template

**Providing the description**: Be specific about data models and any non-standard behavior. For example:
- "a task API with title, status (pending/done), and due date, where status defaults to pending"
- "a blog with posts and comments; posts have an author email; only admins can delete posts"

**Adding constraints**: Append extra rules to the `## Rules` block when you need to constrain output further. For example:
```
- Do not generate the delete operation (set allow_delete = false in the policy; sec4 uses `POST /:id/delete`)
- Require auth on all endpoints
- Use a custom table name "blog_posts" for the Post resource
```

**Iterating on output**: If the generated `.ut` file produces compiler errors, include the error code and message in a follow-up prompt:
```
The compiler reports E1002 on line 12. Fix the untrusted value flow issue.
```

**Reference**: See `docs/ai-codegen-guide.md` for the full language quick reference, more examples, the generated route shapes (`POST /:id/update`, `POST /:id/delete`), and the follow-up inspection commands (`sec4 describe`, `sec4 migrate`, `sec4 openapi`).
