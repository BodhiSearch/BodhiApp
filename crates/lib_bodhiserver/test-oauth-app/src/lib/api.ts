export async function requestAccess(
  bodhiServerUrl: string,
  body: {
    app_client_id: string;
    requested_role: string;
    requested?: Record<string, unknown>;
  }
) {
  const response = await fetch(`${bodhiServerUrl}/bodhi/v1/apps/request-access`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  const data = await response.json();
  if (!response.ok)
    throw new Error(data?.error?.message || data?.message || `Request failed: ${response.status}`);
  return data;
}
