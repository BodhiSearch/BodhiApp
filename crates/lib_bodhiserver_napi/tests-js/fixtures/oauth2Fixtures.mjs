import { getPreConfiguredResourceClient } from '@/utils/auth-server-client.mjs';

export class OAuth2Fixtures {
  static getOAuth2ServerConfig(authServerConfig, port, appStatus = 'ready') {
    const resourceClient = getPreConfiguredResourceClient();
    return {
      appStatus,
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: resourceClient.clientId,
      clientSecret: resourceClient.clientSecret,
      port,
      logLevel: 'debug',
    };
  }

  static getOAuth2TestData() {
    return {
      serverName: 'OAuth2 Test Server Instance',
      clientName: 'OAuth2 Test App Client',
      clientDescription: 'Test app client for OAuth2 Token Exchange v2 testing',
      scopes: 'openid email profile roles',
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
