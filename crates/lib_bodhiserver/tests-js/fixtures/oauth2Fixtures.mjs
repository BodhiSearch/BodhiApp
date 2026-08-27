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
      createdBy: process.env.INTEG_TEST_USERNAME_ID,
      port,
      logLevel: 'debug',
    };
  }

  static getErrorTestConfig(authServerConfig, port) {
    return {
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: 'invalid-client-id',
      clientSecret: 'invalid-client-secret',
      createdBy: process.env.INTEG_TEST_USERNAME_ID,
      port,
    };
  }
}
