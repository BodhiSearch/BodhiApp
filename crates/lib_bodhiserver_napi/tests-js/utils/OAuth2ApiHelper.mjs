export class OAuth2ApiHelper {
  constructor(baseUrl, authClient) {
    this.baseUrl = baseUrl;
    this.authClient = authClient;
  }

  async getDevConsoleToken(username, password) {
    return await this.authClient.getDevConsoleToken(username, password);
  }

  async createAppClient(devConsoleToken, port, clientName, description, redirectUris) {
    return await this.authClient.createAppClient(
      devConsoleToken,
      port,
      clientName,
      description,
      redirectUris
    );
  }

  async testApiWithToken(accessToken) {
    const response = await fetch(`${this.baseUrl}/bodhi/v1/user`, {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
      },
    });

    return {
      status: response.status,
      data: await response.json(),
    };
  }

  async testUnauthenticatedApi() {
    const response = await fetch(`${this.baseUrl}/bodhi/v1/user`, {
      headers: { 'Content-Type': 'application/json' },
    });

    return {
      status: response.status,
      data: await response.json(),
    };
  }
}
