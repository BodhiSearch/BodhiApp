export class OAuth2Fixtures {
  static getOAuth2ServerConfig(authServerConfig, port, appStatus = 'setup') {
    return {
      appStatus,
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      port,
      logLevel: 'debug',
    };
  }

  static getOAuth2TestData() {
    return {
      serverName: 'OAuth2 Test Server Instance',
      clientName: 'OAuth2 Test App Client',
      clientDescription: 'Test app client for OAuth2 Token Exchange v2 testing',
      scopes: 'openid email profile roles scope_user_user',
    };
  }

  static getErrorTestConfig(authServerConfig, port) {
    return {
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: 'invalid-client-id',
      clientSecret: 'invalid-client-secret',
      port,
    };
  }
}
