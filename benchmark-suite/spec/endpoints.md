# Endpoint Contract (M10)

Base URL: `http://127.0.0.1:8080`

## A. GET /ping

- Response: `200 text/plain` with body `ok`

## B. POST /decode

- Request: JSON body from `spec/payloads/user_4kb.json`
- Validation rules:
  - `email`: valid email
  - `age`: integer 0..150
  - `id`: UUID v4
  - `tags`: array length 0..16, each item length 1..32
  - `address.zip`: digits length 4..10
  - `meta.flags`: object with bool keys `a,b,c`
- Success response: `200 application/json`
  - `{"ok":true,"id":"<same id>"}`
- Failure response: `400` with standard error envelope

## C. POST /users

- Request: same as `/decode`
- Behavior: INSERT into `users`
- Success response: `201 application/json`
  - `{"ok":true,"userId":"<uuid>"}`

## D. GET /users/:id

- Validate `:id` as UUID
- Behavior: SELECT by id
- Success response: `200 application/json` user object
- Not found: `404` standard error envelope

## Optional E. GET /enrich/:id (later)

- DB read + outbound HTTP call + merged response
