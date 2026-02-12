CREATE TABLE IF NOT EXISTS app_access_requests (
    id TEXT PRIMARY KEY,                    -- UUID (access_request_id)
    app_client_id TEXT NOT NULL,
    app_name TEXT,                          -- App display name from KC
    app_description TEXT,                   -- App description from KC
    flow_type TEXT NOT NULL,                -- 'redirect' | 'popup'
    redirect_uri TEXT,                      -- Required for redirect flow
    status TEXT NOT NULL DEFAULT 'draft',   -- 'draft' | 'approved' | 'denied' | 'failed'
    requested TEXT NOT NULL,                -- JSON: {"toolset_types": [{"tool_type": "builtin-exa-search"}]}
    approved TEXT,                          -- JSON: {"toolset_types": [{"tool_type":"...", "status":"approved", "instance_id":"..."}]}
    user_id TEXT,                           -- NULL until user approves/denies
    resource_scope TEXT,                    -- KC-returned "scope_resource-xyz" (set after KC call)
    access_request_scope TEXT,              -- KC-returned "scope_access_request:<uuid>" (set after user approval, NULL for auto-approve)
    error_message TEXT,                     -- Error details when status='failed'
    expires_at INTEGER NOT NULL,            -- Unix timestamp, draft TTL = 10 minutes
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX idx_app_access_requests_status ON app_access_requests(status);
CREATE INDEX idx_app_access_requests_app_client ON app_access_requests(app_client_id);
