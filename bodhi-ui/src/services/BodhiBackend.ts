export interface AppInfo {
  status: 'setup' | 'ready' | 'resource-admin' | string;
}

export class BodhiBackend {
  private baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  async getAppInfo(): Promise<AppInfo> {
    const response = await fetch(`${this.baseUrl}/app/info`);
    if (!response.ok) {
      throw new Error('Network response was not ok');
    }
    return await response.json();
  }

  async setupApp(authz: boolean): Promise<AppInfo> {
    const response = await fetch(`${this.baseUrl}/app/setup`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ authz }),
    });
    if (!response.ok) {
      throw new Error('Network response was not ok');
    }
    return await response.json();
  }
}
