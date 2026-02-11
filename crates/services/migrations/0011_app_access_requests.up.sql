CREATE TABLE IF NOT EXISTS app_access_requests (
    id TEXT PRIMARY KEY,                    -- UUID (access_request_id)
    app_client_id TEXT NOT NULL,
    flow_type TEXT NOT NULL,                -- 'redirect' | 'popup'
    redirect_uri TEXT,                      -- Required for redirect flow
    status TEXT NOT NULL DEFAULT 'draft',   -- 'draft' | 'approved' | 'denied' | 'failed'
    tools_requested TEXT NOT NULL,          -- JSON: [{"tool_type": "builtin-exa-search"}]
    tools_approved TEXT,                    -- JSON: [{"tool_type":"...", "status":"approved", "toolset_id":"..."}]
    user_id TEXT,                           -- NULL until user approves/denies
    resource_scope TEXT,                    -- KC-returned "scope_resource-xyz" (set after KC call)
    access_request_scope TEXT,              -- KC-returned "scope_access_request:<uuid>" (set after KC call)
    error_message TEXT,                     -- Error details when status='failed'
    expires_at INTEGER NOT NULL,            -- Unix timestamp, draft TTL = 10 minutes
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX idx_app_access_requests_status ON app_access_requests(status);
CREATE INDEX idx_app_access_requests_app_client ON app_access_requests(app_client_id);
