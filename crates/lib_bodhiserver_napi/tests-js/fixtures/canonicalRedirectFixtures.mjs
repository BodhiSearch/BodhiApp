export class CanonicalRedirectFixtures {
  static getCanonicalRedirectConfig(port, enabled = true, overrides = {}) {
    return {
      envVars: {
        BODHI_PUBLIC_HOST: 'localhost',
        BODHI_PUBLIC_SCHEME: 'http',
        BODHI_PUBLIC_PORT: port.toString(),
        BODHI_CANONICAL_REDIRECT: enabled.toString(),
        ...overrides.envVars,
      },
      host: '0.0.0.0', // Bind to all interfaces to allow 127.0.0.1 access
      port: port.toString(),
      ...overrides,
    };
  }

  static getServerManagerConfig(authServerConfig, port, canonicalRedirectEnabled = true) {
    return {
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: 'dummy-client-id', // Dummy credentials since no auth needed
      clientSecret: 'dummy-client-secret',
      ...this.getCanonicalRedirectConfig(port, canonicalRedirectEnabled),
    };
  }
}
