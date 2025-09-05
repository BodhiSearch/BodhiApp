/**
 * Get auth server configuration (no fallbacks - fail if not provided)
 */
export function getAuthServerConfig() {
  const config = {
    authUrl: process.env.INTEG_TEST_MAIN_AUTH_URL,
    authRealm: process.env.INTEG_TEST_AUTH_REALM,
    devConsoleClientSecret: process.env.INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET,
  };

  // Validate required environment variables
  const requiredVars = ['authUrl', 'authRealm', 'devConsoleClientSecret'];
  for (const varName of requiredVars) {
    if (!config[varName]) {
      throw new Error(`Required environment variable missing: ${varName.toUpperCase()}`);
    }
  }

  return config;
}

/**
 * Get test user credentials
 */
export function getTestCredentials() {
  const config = {
    username: process.env.INTEG_TEST_USERNAME,
    password: process.env.INTEG_TEST_PASSWORD,
  };

  // Validate required environment variables
  const requiredVars = ['username', 'password'];
  for (const varName of requiredVars) {
    if (!config[varName]) {
      throw new Error(`Required environment variable missing: ${varName.toUpperCase()}`);
    }
  }

  return config;
}

/**
 * AuthServerTestClient - Handles OAuth2 Token Exchange v2 operations
 */
export class AuthServerTestClient {
  constructor(config) {
    this.authUrl = config.authUrl;
    this.authRealm = config.authRealm;
    this.devConsoleClientId = 'client-bodhi-dev-console';
    this.devConsoleClientSecret = config.devConsoleClientSecret;
  }

  /**
   * Get dev console user token using direct access grant
   * @param {string} username - Username for authentication
   * @param {string} password - Password for authentication
   * @returns {Promise<string>} Access token for dev console operations
   */
  async getDevConsoleToken(username, password) {
    const tokenUrl = `${this.authUrl}/realms/${this.authRealm}/protocol/openid-connect/token`;

    const params = new URLSearchParams({
      grant_type: 'password',
      client_id: this.devConsoleClientId,
      client_secret: this.devConsoleClientSecret,
      username: username,
      password: password,
      scope: 'openid email profile roles',
    });

    const response = await fetch(tokenUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: params.toString(),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.log('Token request failed:', response.status, response.statusText);
      console.log('Error response body:', errorText);
      throw new Error(
        `Failed to get dev console token: ${response.status} ${response.statusText} - ${errorText}`
      );
    }

    const data = await response.json();
    return data.access_token;
  }

  /**
   * Create an app client (public, no secrets)
   * @param {string} devConsoleToken - Dev console access token
   * @param {number} appUrl - App URL for redirect URI
   * @param {string} name - Client name
   * @param {string} description - Client description
   * @param {string[]} customRedirectUris - Custom redirect URIs (optional)
   * @returns {Promise<Object>} Created app client info
   */
  async createAppClient(
    devConsoleToken,
    appUrl,
    name = 'Test App Client',
    description = 'Test app client for Playwright tests',
    customRedirectUris = null
  ) {
    const appsUrl = `${this.authUrl}/realms/${this.authRealm}/bodhi/apps`;

    // Use custom redirect URIs if provided, otherwise use default ones
    const redirectUris = customRedirectUris || [`${appUrl}/ui/auth/callback`];

    const response = await fetch(appsUrl, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${devConsoleToken}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        name,
        description,
        redirect_uris: redirectUris,
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.log('Create app client failed:', response.status, response.statusText);
      console.log('Error response body:', errorText);
      throw new Error(`Failed to create app client: ${response.status} ${response.statusText}`);
    }

    const data = await response.json();
    return {
      clientId: data.client_id,
    };
  }

  /**
   * Create a resource client (confidential, with secrets)
   * @param {number} serverUrl - Server URL for redirect URI
   * @param {string} name - Client name
   * @param {string} description - Client description
   * @returns {Promise<Object>} Created resource client info
   */
  async createResourceClient(
    serverUrl,
    name = 'Test Resource Client',
    description = 'Test resource client for Playwright tests'
  ) {
    const resourcesUrl = `${this.authUrl}/realms/${this.authRealm}/bodhi/resources`;
    const response = await fetch(resourcesUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        name,
        description,
        redirect_uris: [`${serverUrl}/ui/auth/callback`],
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.log('Create resource client failed:', response.status, response.statusText);
      console.log('Error response body:', errorText);
      throw new Error(
        `Failed to create resource client: ${response.status} ${response.statusText}`
      );
    }

    const data = await response.json();
    return {
      clientId: data.client_id,
      clientSecret: data.client_secret,
      scope: data.scope,
    };
  }

  /**
   * Get resource client service account token
   * @param {string} clientId - Resource client ID
   * @param {string} clientSecret - Resource client secret
   * @returns {Promise<string>} Service account access token
   */
  async getResourceServiceAccountToken(clientId, clientSecret) {
    const tokenUrl = `${this.authUrl}/realms/${this.authRealm}/protocol/openid-connect/token`;

    const params = new URLSearchParams({
      grant_type: 'client_credentials',
      scope: 'service_account',
    });

    const credentials = Buffer.from(`${clientId}:${clientSecret}`).toString('base64');

    const response = await fetch(tokenUrl, {
      method: 'POST',
      headers: {
        Authorization: `Basic ${credentials}`,
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: params.toString(),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.log('Get service account token failed:', response.status, response.statusText);
      console.log('Error response body:', errorText);
      throw new Error(
        `Failed to get service account token: ${response.status} ${response.statusText}`
      );
    }

    const data = await response.json();
    return data.access_token;
  }

  /**
   * Request audience access for resource client (dynamic audience management)
   * @param {string} resourceToken - Resource client service account token
   * @param {string} appClientId - App client ID to request access for
   * @returns {Promise<string>} Resource scope name
   */
  async requestAudienceAccess(resourceToken, appClientId) {
    const requestAccessUrl = `${this.authUrl}/realms/${this.authRealm}/bodhi/resources/request-access`;

    const response = await fetch(requestAccessUrl, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${resourceToken}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        app_client_id: appClientId,
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.log('Request audience access failed:', response.status, response.statusText);
      console.log('Error response body:', errorText);
      throw new Error(
        `Failed to request audience access: ${response.status} ${response.statusText}`
      );
    }

    const data = await response.json();
    return data.scope;
  }

  async makeResourceAdmin(resourceClientId, resourceClientSecret, username) {
    const resourceAdminUrl = `${this.authUrl}/realms/${this.authRealm}/bodhi/resources/make-resource-admin`;
    const resourceToken = await this.getResourceServiceAccountToken(
      resourceClientId,
      resourceClientSecret
    );
    const response = await fetch(resourceAdminUrl, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${resourceToken}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        username: username,
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.log('Make resource admin failed:', response.status, response.statusText);
      console.log('Error response body:', errorText);
      throw new Error(`Failed to make resource admin: ${response.status} ${response.statusText}`);
    }
  }

  /**
   * Perform OAuth2 Token Exchange v2
   * @param {string} resourceClientId - Resource client ID
   * @param {string} resourceClientSecret - Resource client secret
   * @param {string} subjectToken - App user token to exchange
   * @returns {Promise<string>} Exchanged access token
   */
  async exchangeToken(resourceClientId, resourceClientSecret, subjectToken) {
    const tokenUrl = `${this.authUrl}/realms/${this.authRealm}/protocol/openid-connect/token`;

    const params = new URLSearchParams({
      subject_token: subjectToken,
      scope: 'openid email profile roles scope_user_user',
      grant_type: 'urn:ietf:params:oauth:grant-type:token-exchange',
      subject_token_type: 'urn:ietf:params:oauth:token-type:access_token',
      requested_token_type: 'urn:ietf:params:oauth:token-type:access_token',
    });

    const credentials = Buffer.from(`${resourceClientId}:${resourceClientSecret}`).toString(
      'base64'
    );

    const response = await fetch(tokenUrl, {
      method: 'POST',
      headers: {
        Authorization: `Basic ${credentials}`,
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: params.toString(),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.log('Token exchange failed:', response.status, response.statusText);
      console.log('Error response body:', errorText);
      throw new Error(`Failed to exchange token: ${response.status} ${response.statusText}`);
    }

    const data = await response.json();
    return data.access_token;
  }
}

/**
 * Create an auth server test client
 * @param {Object} config - Configuration object with authUrl, authRealm, devConsoleClientSecret
 * @returns {AuthServerTestClient} New auth server test client instance
 */
export function createAuthServerTestClient(config) {
  return new AuthServerTestClient(config);
}
