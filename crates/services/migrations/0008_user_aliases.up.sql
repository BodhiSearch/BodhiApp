CREATE TABLE user_aliases (
    id TEXT PRIMARY KEY NOT NULL,
    alias TEXT NOT NULL UNIQUE,
    repo TEXT NOT NULL,
    filename TEXT NOT NULL,
    snapshot TEXT NOT NULL,
    request_params_json TEXT NOT NULL DEFAULT '{}',
    context_params_json TEXT NOT NULL DEFAULT '[]',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
