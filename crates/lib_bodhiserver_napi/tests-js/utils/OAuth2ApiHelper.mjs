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

  async requestAudienceAccess(appClientId) {
    const response = await fetch(`${this.baseUrl}/bodhi/v1/apps/request-access`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ app_client_id: appClientId }),
    });

    if (response.status !== 200) {
      throw new Error(`Failed to request access: ${response.status}, ${await response.text()}`);
    }

    return await response.json();
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
