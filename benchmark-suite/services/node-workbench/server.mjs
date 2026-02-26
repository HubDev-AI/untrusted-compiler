import http from 'node:http';
import { randomUUID } from 'node:crypto';
import { spawnSync } from 'node:child_process';

const PORT = Number(process.env.PORT || 18089);
const PG_DSN =
  process.env.BENCH_WORKBENCH_PG_DSN ||
  process.env.SEC4_RT_LASM_DB_POSTGRES_DSN ||
  'postgresql://127.0.0.1:5432/postgres?sslmode=disable';
const AUTH_TOKEN = process.env.BENCH_WORKBENCH_AUTH_TOKEN || 'token123';

const VALID_STATUS = new Set(['open', 'in_progress', 'done']);

class HttpError extends Error {
  constructor(code, kind, status, message) {
    super(message);
    this.code = code;
    this.kind = kind;
    this.status = status;
  }
}

function errorEnvelope(err, traceId) {
  return {
    ok: false,
    status: err.status,
    traceId,
    timeMs: Date.now(),
    error: {
      code: err.code,
      kind: err.kind,
      message: err.message,
    },
  };
}

function successEnvelope(status, traceId, data) {
  return {
    ok: true,
    status,
    traceId,
    timeMs: Date.now(),
    data,
  };
}

function writeJson(res, status, body, traceId) {
  res.writeHead(status, {
    'content-type': 'application/json; charset=utf-8',
    'x-trace-id': traceId,
  });
  res.end(JSON.stringify(body));
}

function writeText(res, status, body, traceId) {
  res.writeHead(status, {
    'content-type': 'text/plain; charset=utf-8',
    'x-trace-id': traceId,
  });
  res.end(body);
}

function sqlLiteral(value) {
  return `'${String(value).replace(/'/g, "''")}'`;
}

function parseIntParam(url, key, required = true, fallback = 0) {
  const raw = url.searchParams.get(key);
  if (raw === null || raw === '') {
    if (!required) return fallback;
    throw new HttpError(
      'VALIDATION.INVALID',
      'validation',
      400,
      `missing required query param: ${key}`,
    );
  }
  const value = Number.parseInt(raw, 10);
  if (!Number.isFinite(value)) {
    throw new HttpError(
      'VALIDATION.INVALID',
      'validation',
      400,
      `invalid integer query param: ${key}`,
    );
  }
  return value;
}

function parseTextParam(url, key, required = true, fallback = '') {
  const raw = url.searchParams.get(key);
  if (raw === null || raw === '') {
    if (!required) return fallback;
    throw new HttpError(
      'VALIDATION.INVALID',
      'validation',
      400,
      `missing required query param: ${key}`,
    );
  }
  return raw;
}

function parseJsonArrayParam(url, key, minLength) {
  const raw = url.searchParams.get(key);
  if (raw === null || raw === '') {
    return null;
  }
  let parsed;
  try {
    parsed = JSON.parse(raw);
  } catch {
    throw new HttpError(
      'VALIDATION.INVALID',
      'validation',
      400,
      `invalid JSON array query param: ${key}`,
    );
  }
  if (!Array.isArray(parsed) || parsed.length < minLength) {
    throw new HttpError(
      'VALIDATION.INVALID',
      'validation',
      400,
      `invalid JSON array query param: ${key}`,
    );
  }
  return parsed;
}

function parseIntValue(raw, key) {
  const value = Number.parseInt(String(raw), 10);
  if (!Number.isFinite(value)) {
    throw new HttpError(
      'VALIDATION.INVALID',
      'validation',
      400,
      `invalid integer query param: ${key}`,
    );
  }
  return value;
}

function parseTaskInput(url) {
  const params = parseJsonArrayParam(url, 'params', 6);
  if (params) {
    return {
      id: String(params[0]),
      title: String(params[1]),
      description: String(params[2] ?? ''),
      status: String(params[3]),
      priority: parseIntValue(params[4], 'priority'),
      createdAtMs: parseIntValue(params[5], 'created_at_ms'),
    };
  }
  return {
    id: parseTextParam(url, 'id'),
    title: parseTextParam(url, 'title'),
    description: parseTextParam(url, 'description', false, ''),
    status: parseTextParam(url, 'status'),
    priority: parseIntParam(url, 'priority'),
    createdAtMs: parseIntParam(url, 'created_at_ms'),
  };
}

function parseTaskWithCommentInput(url) {
  const taskParams = parseJsonArrayParam(url, 'task_params', 6);
  const commentParams = parseJsonArrayParam(url, 'comment_params', 4);
  if (taskParams && commentParams) {
    return {
      taskId: String(taskParams[0]),
      title: String(taskParams[1]),
      description: String(taskParams[2] ?? ''),
      status: String(taskParams[3]),
      priority: parseIntValue(taskParams[4], 'priority'),
      createdAtMs: parseIntValue(taskParams[5], 'created_at_ms'),
      commentId: String(commentParams[0]),
      commentTaskId: String(commentParams[1]),
      commentBody: String(commentParams[2]),
      commentCreatedAtMs: parseIntValue(commentParams[3], 'comment_created_at_ms'),
    };
  }
  return {
    taskId: parseTextParam(url, 'id'),
    title: parseTextParam(url, 'title'),
    description: parseTextParam(url, 'description', false, ''),
    status: parseTextParam(url, 'status'),
    priority: parseIntParam(url, 'priority'),
    createdAtMs: parseIntParam(url, 'created_at_ms'),
    commentId: parseTextParam(url, 'comment_id'),
    commentTaskId: '',
    commentBody: parseTextParam(url, 'comment_body'),
    commentCreatedAtMs: parseIntParam(url, 'comment_created_at_ms'),
  };
}

function parseCommentInput(url, taskId) {
  const params = parseJsonArrayParam(url, 'params', 4);
  if (params) {
    const paramsTaskId = String(params[1]);
    if (paramsTaskId !== taskId) {
      throw new HttpError(
        'VALIDATION.INVALID',
        'validation',
        400,
        'comment task id must match route task id',
      );
    }
    return {
      id: String(params[0]),
      body: String(params[2]),
      createdAtMs: parseIntValue(params[3], 'comment_created_at_ms'),
    };
  }
  return {
    id: parseTextParam(url, 'comment_id'),
    body: parseTextParam(url, 'comment_body'),
    createdAtMs: parseIntParam(url, 'comment_created_at_ms'),
  };
}

function parseListInput(url) {
  const params = parseJsonArrayParam(url, 'params', 3);
  if (params) {
    return {
      status: String(params[0]),
      limit: parseIntValue(params[1], 'limit'),
      offset: parseIntValue(params[2], 'offset'),
    };
  }
  return {
    status: parseTextParam(url, 'status', false, ''),
    limit: parseIntParam(url, 'limit', false, 20),
    offset: parseIntParam(url, 'offset', false, 0),
  };
}

function requireAuth(req) {
  const auth = req.headers.authorization || '';
  if (auth !== `Bearer ${AUTH_TOKEN}`) {
    throw new HttpError('AUTH.REQUIRED', 'auth', 401, 'authorization token is required');
  }
}

function runPsql(sql) {
  const result = spawnSync(
    'psql',
    [PG_DSN, '-t', '-A', '-v', 'ON_ERROR_STOP=1', '-c', sql],
    { encoding: 'utf8' },
  );
  if (result.status !== 0) {
    const stderr = (result.stderr || '').trim();
    throw new HttpError(
      'DB.QUERY_FAILED',
      'internal',
      500,
      stderr || 'postgres command failed',
    );
  }
  return (result.stdout || '').trim();
}

function runPsqlTx(statements) {
  runPsql('BEGIN;');
  try {
    for (const sql of statements) runPsql(sql);
    runPsql('COMMIT;');
  } catch (err) {
    try {
      runPsql('ROLLBACK;');
    } catch {
      // ignore rollback errors to preserve primary failure
    }
    throw err;
  }
}

function setupSchema() {
  runPsql(
    'create table if not exists wb_tasks (id text primary key, title text not null, description text not null default \'\', status text not null, priority integer not null, created_at_ms bigint not null);',
  );
  runPsql(
    'create table if not exists wb_comments (id text primary key, task_id text not null, body text not null, created_at_ms bigint not null);',
  );
  runPsql(
    'create table if not exists wb_labels (task_id text not null, name text not null, primary key(task_id, name));',
  );
}

function insertTask(url) {
  const { id, title, description, status, priority, createdAtMs } = parseTaskInput(url);
  if (!VALID_STATUS.has(status)) {
    throw new HttpError(
      'VALIDATION.INVALID',
      'validation',
      400,
      'status must be one of: open, in_progress, done',
    );
  }
  if (priority < 1 || priority > 5) {
    throw new HttpError('VALIDATION.INVALID', 'validation', 400, 'priority must be in range 1..5');
  }
  runPsql(
    `insert into wb_tasks (id, title, description, status, priority, created_at_ms) values (${sqlLiteral(id)}, ${sqlLiteral(title)}, ${sqlLiteral(description)}, ${sqlLiteral(status)}, ${priority}, ${createdAtMs});`,
  );
  return { id };
}

function insertComment(taskId, url) {
  const { id, body, createdAtMs } = parseCommentInput(url, taskId);
  runPsql(
    `insert into wb_comments (id, task_id, body, created_at_ms) values (${sqlLiteral(id)}, ${sqlLiteral(taskId)}, ${sqlLiteral(body)}, ${createdAtMs});`,
  );
  return { id };
}

function createTaskWithComment(url) {
  const {
    taskId,
    title,
    description,
    status,
    priority,
    createdAtMs,
    commentId,
    commentTaskId,
    commentBody,
    commentCreatedAtMs,
  } = parseTaskWithCommentInput(url);
  if (commentTaskId !== '' && commentTaskId !== taskId) {
    throw new HttpError(
      'VALIDATION.INVALID',
      'validation',
      400,
      'comment task id must match task id',
    );
  }

  if (!VALID_STATUS.has(status)) {
    throw new HttpError(
      'VALIDATION.INVALID',
      'validation',
      400,
      'status must be one of: open, in_progress, done',
    );
  }
  if (priority < 1 || priority > 5) {
    throw new HttpError('VALIDATION.INVALID', 'validation', 400, 'priority must be in range 1..5');
  }

  runPsqlTx([
    `insert into wb_tasks (id, title, description, status, priority, created_at_ms) values (${sqlLiteral(taskId)}, ${sqlLiteral(title)}, ${sqlLiteral(description)}, ${sqlLiteral(status)}, ${priority}, ${createdAtMs});`,
    `insert into wb_comments (id, task_id, body, created_at_ms) values (${sqlLiteral(commentId)}, ${sqlLiteral(taskId)}, ${sqlLiteral(commentBody)}, ${commentCreatedAtMs});`,
  ]);

  return { taskId, commentId };
}

function getTask(taskId) {
  const rowJson = runPsql(
    `select row_to_json(t)::text from (select id, title, description, status, priority, created_at_ms, (select count(*)::int from wb_comments c where c.task_id = wb_tasks.id) as comments_count from wb_tasks where id = ${sqlLiteral(taskId)} limit 1) t;`,
  );
  if (!rowJson) {
    throw new HttpError('TASK.NOT_FOUND', 'missing_dependency', 404, 'task not found');
  }
  return JSON.parse(rowJson);
}

function listTasks(url) {
  const { status, limit, offset } = parseListInput(url);
  const boundedLimit = Math.min(Math.max(limit, 1), 100);
  const boundedOffset = Math.max(offset, 0);
  const whereSql =
    status === '' ? 'true' : `status = ${sqlLiteral(status)}`;

  const itemsJson = runPsql(
    `select coalesce(json_agg(t), '[]'::json)::text from (select id, title, description, status, priority, created_at_ms from wb_tasks where ${whereSql} order by created_at_ms desc, id desc limit ${boundedLimit} offset ${boundedOffset}) t;`,
  );
  const countRaw = runPsql(
    `select count(*)::text from wb_tasks where ${whereSql};`,
  );

  return {
    items: itemsJson ? JSON.parse(itemsJson) : [],
    count: Number.parseInt(countRaw || '0', 10) || 0,
    limit: boundedLimit,
    offset: boundedOffset,
  };
}

const server = http.createServer((req, res) => {
  const traceId = randomUUID();
  try {
    if (!req.url || !req.method) {
      throw new HttpError('HTTP.BAD_REQUEST', 'validation', 400, 'missing url or method');
    }
    const url = new URL(req.url, `http://${req.headers.host || '127.0.0.1'}`);
    const { pathname } = url;

    if (req.method === 'GET' && pathname === '/health') {
      writeText(res, 200, 'ok', traceId);
      return;
    }

    if (req.method === 'POST' && pathname === '/wb/setup') {
      requireAuth(req);
      setupSchema();
      writeJson(res, 200, successEnvelope(200, traceId, { setup: true }), traceId);
      return;
    }

    if (req.method === 'POST' && pathname === '/wb/tasks') {
      requireAuth(req);
      const data = insertTask(url);
      writeJson(res, 201, successEnvelope(201, traceId, data), traceId);
      return;
    }

    if (req.method === 'POST' && pathname === '/wb/tasks/with-comment') {
      requireAuth(req);
      const data = createTaskWithComment(url);
      writeJson(res, 201, successEnvelope(201, traceId, data), traceId);
      return;
    }

    if (req.method === 'POST' && pathname.startsWith('/wb/tasks/') && pathname.endsWith('/comments')) {
      requireAuth(req);
      const taskId = pathname.slice('/wb/tasks/'.length, -'/comments'.length);
      const data = insertComment(taskId, url);
      writeJson(res, 201, successEnvelope(201, traceId, data), traceId);
      return;
    }

    if (req.method === 'GET' && pathname.startsWith('/wb/tasks/')) {
      const taskId = pathname.slice('/wb/tasks/'.length);
      const data = getTask(taskId);
      writeJson(res, 200, successEnvelope(200, traceId, data), traceId);
      return;
    }

    if (req.method === 'GET' && pathname === '/wb/tasks') {
      const data = listTasks(url);
      writeJson(res, 200, successEnvelope(200, traceId, data), traceId);
      return;
    }

    throw new HttpError('HTTP.NOT_FOUND', 'missing_dependency', 404, 'route not found');
  } catch (err) {
    if (err instanceof HttpError) {
      writeJson(res, err.status, errorEnvelope(err, traceId), traceId);
      return;
    }
    const internal = new HttpError('HTTP.INTERNAL', 'internal', 500, 'internal error');
    writeJson(res, 500, errorEnvelope(internal, traceId), traceId);
    console.error(err);
  }
});

server.listen(PORT, () => {
  console.log(`node-workbench service listening on :${PORT}`);
});
