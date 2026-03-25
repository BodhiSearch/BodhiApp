// Vite base path — must match vite.config.ts base value and TanStack Router basepath.
// Used where the framework does NOT auto-apply the base path (e.g., window.location stripping).
export const BASE_PATH = '/ui';

export const CURRENT_CHAT_KEY = 'current-chat';
export const ROUTE_CHAT = '/chat';
export const ROUTE_LOGIN = '/login';

export const ROUTE_DEFAULT = '/chat';
export const ROUTE_RESOURCE_ADMIN = '/setup/resource-admin';
export const ROUTE_SETUP = '/setup';

export const ROUTE_SETUP_COMPLETE = '/setup/complete';
export const ROUTE_SETUP_DOWNLOAD_MODELS = '/setup/download-models';
export const ROUTE_SETUP_RESOURCE_ADMIN = '/setup/resource-admin';
export const ROUTE_SETUP_API_MODELS = '/setup/api-models';
export const ROUTE_SETUP_BROWSER_EXTENSION = '/setup/browser-extension';

// Access request routes
export const ROUTE_REQUEST_ACCESS = '/request-access';
export const ROUTE_APP_REVIEW_ACCESS = '/apps/access-requests/review';
export const ROUTE_ACCESS_REQUESTS_PENDING = '/users/pending';
export const ROUTE_ACCESS_REQUESTS_ALL = '/users/access-requests';
export const ROUTE_USERS = '/users';
export const ROUTE_UNAUTHORIZED = '/unauthorized';

// Tenant routes
export const ROUTE_SETUP_TENANTS = '/setup/tenants';
export const ROUTE_DASHBOARD_CALLBACK = '/auth/dashboard/callback';

// MCP routes
export const ROUTE_MCPS = '/mcps';
export const ROUTE_MCP_SERVERS = '/mcps/servers';
