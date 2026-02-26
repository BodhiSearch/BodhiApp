export async function requestAccess(
  bodhiServerUrl: string,
  body: {
    app_client_id: string;
    flow_type: string;
    redirect_url: string;
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
    throw new Error(data.message || data.error || `Request failed: ${response.status}`);
  return data;
}

export async function getAccessRequestStatus(bodhiServerUrl: string, id: string, clientId: string) {
  const response = await fetch(
    `${bodhiServerUrl}/bodhi/v1/apps/access-requests/${id}?app_client_id=${encodeURIComponent(clientId)}`
  );
  const data = await response.json();
  if (!response.ok)
    throw new Error(data.message || data.error || `Status check failed: ${response.status}`);
  return data;
}
