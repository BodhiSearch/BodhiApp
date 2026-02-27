/**
 * Database configuration mapping for dual-DB E2E testing.
 *
 * Maps Playwright project names to server ports and database URLs.
 * SQLite is the default (no DB URLs needed), PostgreSQL requires
 * the test dependency containers from docker-compose-test-deps.yml.
 */

const DB_CONFIGS = {
  sqlite: {
    port: 51135,
    // No DB URLs needed - SQLite is the default
  },
  postgres: {
    port: 41135,
    appDbUrl:
      process.env.E2E_PG_APP_DB_URL || 'postgres://bodhi_test:bodhi_test@localhost:64320/bodhi_app',
    sessionDbUrl:
      process.env.E2E_PG_SESSION_DB_URL ||
      'postgres://bodhi_test:bodhi_test@localhost:54320/bodhi_sessions',
  },
};

/**
 * Get database configuration for a Playwright project.
 * Strips browser suffix if present (e.g., "sqlite-chromium" -> "sqlite").
 * @param {string} projectName - Playwright project name
 * @returns {Object} Database configuration with port and optional DB URLs
 */
export function getDbConfig(projectName) {
  const dbType = projectName?.split('-')[0] || 'sqlite';
  return DB_CONFIGS[dbType] || DB_CONFIGS.sqlite;
}

/**
 * Get the server URL for a Playwright project.
 * @param {string} projectName - Playwright project name
 * @returns {string} Server URL (e.g., "http://localhost:51135")
 */
export function getServerUrl(projectName) {
  const config = getDbConfig(projectName);
  return `http://localhost:${config.port}`;
}
