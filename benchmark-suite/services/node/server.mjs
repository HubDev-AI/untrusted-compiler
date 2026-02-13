import http from 'node:http';
import { randomUUID } from 'node:crypto';

const PORT = Number(process.env.PORT || 8080);
const users = new Map();

const UUID_V4_RE =
  /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
const ZIP_RE = /^\d{4,10}$/;

function jsonResponse(res, status, traceId, payload) {
  res.writeHead(status, {
    'content-type': 'application/json; charset=utf-8',
    'x-trace-id': traceId,
  });
  res.end(JSON.stringify(payload));
}

function textResponse(res, status, traceId, body) {
  res.writeHead(status, {
    'content-type': 'text/plain; charset=utf-8',
    'x-trace-id': traceId,
  });
  res.end(body);
}

function errorEnvelope(code, message, status, traceId) {
  return {
    error: {
      code,
      kind: status >= 500 ? 'internal' : status === 404 ? 'not_found' : 'validation',
      message,
      status,
      traceId,
      timeMs: Date.now(),
    },
  };
}

function collectBody(req) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    req.on('data', (chunk) => chunks.push(chunk));
    req.on('end', () => resolve(Buffer.concat(chunks).toString('utf8')));
    req.on('error', reject);
  });
}

function validateUserPayload(body) {
  if (body === null || typeof body !== 'object') {
    return 'body must be an object';
  }
  if (typeof body.id !== 'string' || !UUID_V4_RE.test(body.id)) {
    return 'id must be a UUID v4 string';
  }
  if (typeof body.email !== 'string' || !EMAIL_RE.test(body.email)) {
    return 'email must be a valid email string';
  }
  if (!Number.isInteger(body.age) || body.age < 0 || body.age > 150) {
    return 'age must be an integer between 0 and 150';
  }
  if (!Array.isArray(body.tags) || body.tags.length > 16) {
    return 'tags must be an array of length <= 16';
  }
  for (const tag of body.tags) {
    if (typeof tag !== 'string' || tag.length < 1 || tag.length > 32) {
      return 'tags must contain strings of length 1..32';
    }
  }
  if (
    body.address === null ||
    typeof body.address !== 'object' ||
    typeof body.address.zip !== 'string' ||
    !ZIP_RE.test(body.address.zip)
  ) {
    return 'address.zip must be a digit string of length 4..10';
  }
  if (
    body.meta === null ||
    typeof body.meta !== 'object' ||
    body.meta.flags === null ||
    typeof body.meta.flags !== 'object'
  ) {
    return 'meta.flags must be an object';
  }
  for (const key of ['a', 'b', 'c']) {
    if (typeof body.meta.flags[key] !== 'boolean') {
      return `meta.flags.${key} must be boolean`;
    }
  }
  return null;
}

const server = http.createServer(async (req, res) => {
  const traceId = randomUUID();
  try {
    if (!req.url || !req.method) {
      jsonResponse(res, 400, traceId, errorEnvelope('HTTP.BAD_REQUEST', 'missing url or method', 400, traceId));
      return;
    }

    const url = new URL(req.url, `http://${req.headers.host || '127.0.0.1'}`);
    const { pathname } = url;

    if (req.method === 'GET' && pathname === '/ping') {
      textResponse(res, 200, traceId, 'ok');
      return;
    }

    if (req.method === 'POST' && pathname === '/decode') {
      const raw = await collectBody(req);
      let body;
      try {
        body = JSON.parse(raw);
      } catch {
        jsonResponse(
          res,
          400,
          traceId,
          errorEnvelope('JSON.INVALID_SYNTAX', 'invalid JSON payload', 400, traceId),
        );
        return;
      }

      const validationError = validateUserPayload(body);
      if (validationError) {
        jsonResponse(
          res,
          400,
          traceId,
          errorEnvelope('VALIDATION.INVALID', validationError, 400, traceId),
        );
        return;
      }

      jsonResponse(res, 200, traceId, { ok: true, id: body.id });
      return;
    }

    if (req.method === 'POST' && pathname === '/users') {
      const raw = await collectBody(req);
      let body;
      try {
        body = JSON.parse(raw);
      } catch {
        jsonResponse(
          res,
          400,
          traceId,
          errorEnvelope('JSON.INVALID_SYNTAX', 'invalid JSON payload', 400, traceId),
        );
        return;
      }

      const validationError = validateUserPayload(body);
      if (validationError) {
        jsonResponse(
          res,
          400,
          traceId,
          errorEnvelope('VALIDATION.INVALID', validationError, 400, traceId),
        );
        return;
      }

      users.set(body.id, body);
      jsonResponse(res, 201, traceId, { ok: true, userId: body.id });
      return;
    }

    if (req.method === 'GET' && pathname.startsWith('/users/')) {
      const id = pathname.slice('/users/'.length);
      if (!UUID_V4_RE.test(id)) {
        jsonResponse(
          res,
          400,
          traceId,
          errorEnvelope('VALIDATION.UUID_INVALID', 'id must be UUID v4', 400, traceId),
        );
        return;
      }

      const user = users.get(id);
      if (!user) {
        jsonResponse(
          res,
          404,
          traceId,
          errorEnvelope('HTTP.NOT_FOUND', 'user not found', 404, traceId),
        );
        return;
      }

      jsonResponse(res, 200, traceId, user);
      return;
    }

    jsonResponse(res, 404, traceId, errorEnvelope('HTTP.NOT_FOUND', 'route not found', 404, traceId));
  } catch (error) {
    jsonResponse(
      res,
      500,
      traceId,
      errorEnvelope('HTTP.INTERNAL', 'internal error', 500, traceId),
    );
    console.error(error);
  }
});

server.listen(PORT, () => {
  console.log(`node benchmark service listening on :${PORT}`);
});
