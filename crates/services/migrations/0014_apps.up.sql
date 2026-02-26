CREATE TABLE IF NOT EXISTS apps (
  client_id TEXT PRIMARY KEY,
  encrypted_client_secret TEXT NOT NULL,
  salt_client_secret TEXT NOT NULL,
  nonce_client_secret TEXT NOT NULL,
  app_status TEXT NOT NULL DEFAULT 'setup',
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
