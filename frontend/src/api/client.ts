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

export interface UploadedFile {
  filename: string;
  size: number;
}

export async function uploadFile(file: File): Promise<UploadedFile | null> {
  const body = new FormData();
  body.append('file', file);
  const res = await fetch(`${API_BASE}/upload`, { method: 'POST', credentials: 'include', body });
  return res.ok ? ((await res.json()) as UploadedFile) : null;
}

export async function listUploads(): Promise<UploadedFile[]> {
  const res = await fetch(`${API_BASE}/uploads`, { credentials: 'include' });
  return res.ok ? ((await res.json()) as UploadedFile[]) : [];
}

export async function importFile(filename: string): Promise<{ imported: number } | null> {
  const res = await fetch(`${API_BASE}/import`, {
    method: 'POST',
    credentials: 'include',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ filename }),
  });
  return res.ok ? ((await res.json()) as { imported: number }) : null;
}
