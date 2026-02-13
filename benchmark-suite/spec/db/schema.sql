CREATE TABLE IF NOT EXISTS users (
  id UUID PRIMARY KEY,
  email TEXT NOT NULL,
  age INT NOT NULL,
  name TEXT NOT NULL,
  tags TEXT[] NOT NULL,
  address_line1 TEXT NOT NULL,
  address_city TEXT NOT NULL,
  address_zip TEXT NOT NULL,
  address_country TEXT NOT NULL,
  meta_flags_a BOOLEAN NOT NULL,
  meta_flags_b BOOLEAN NOT NULL,
  meta_flags_c BOOLEAN NOT NULL,
  meta_notes TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
