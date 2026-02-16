import { getPreConfiguredResourceClient } from '@/utils/auth-server-client.mjs';

export class PublicHostFixtures {
  static getPublicHostConfig(port, overrides = {}) {
    return {
      envVars: {
        BODHI_PUBLIC_HOST: 'localhost',
        BODHI_PUBLIC_SCHEME: 'http',
        BODHI_PUBLIC_PORT: port.toString(),
        ...overrides.envVars,
      },
      host: '0.0.0.0',
      port: port.toString(),
      ...overrides,
    };
  }

  static getServerManagerConfig(authServerConfig, port) {
    const resourceClient = getPreConfiguredResourceClient();
    return {
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: resourceClient.clientId,
      clientSecret: resourceClient.clientSecret,
      ...PublicHostFixtures.getPublicHostConfig(port),
    };
  }
}
