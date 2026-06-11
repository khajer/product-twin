const API_BASE = '/api';

export interface MeResponse {
  authenticated: boolean;
}

export async function getMe(): Promise<MeResponse> {
  const res = await fetch(`${API_BASE}/me`, { credentials: 'include' });
  return (await res.json()) as MeResponse;
}

export async function login(username: string, password: string): Promise<boolean> {
  const res = await fetch(`${API_BASE}/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    credentials: 'include',
    body: JSON.stringify({ username, password }),
  });
  return res.ok;
}

export async function logout(): Promise<void> {
  await fetch(`${API_BASE}/logout`, {
    method: 'POST',
    credentials: 'include',
  });
}
