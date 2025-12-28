import { NetworkService } from './network';

export type ApiErrorShape = {
  error: string;
  details?: any;
};

function getBaseUrl(): string {
  // Check localStorage first (set by BridgeConfig)
  const storedUrl = localStorage.getItem('ubl_api_base_url');
  if (storedUrl) {
    return storedUrl.replace(/\/$/, '');
  }
  
  // Fall back to env variable
  const envBase = (import.meta as any).env?.VITE_API_BASE_URL as string | undefined;
  return (envBase || '').replace(/\/$/, '');
}

function getToken(): string | null {
  try {
    const raw = localStorage.getItem('ubl_session');
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    return parsed?.token || null;
  } catch {
    return null;
  }
}

async function request<T>(
  method: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE',
  path: string,
  body?: any
): Promise<T> {
  const base = getBaseUrl();
  const url = `${base}${path}`;
  const token = getToken();

  return NetworkService.execute(async () => {
    const res = await fetch(url, {
      method,
      headers: {
        'Content-Type': 'application/json',
        ...(token ? { Authorization: `Bearer ${token}` } : {})
      },
      ...(body === undefined ? {} : { body: JSON.stringify(body) })
    });

    if (!res.ok) {
      let payload: any = null;
      try {
        payload = await res.json();
      } catch {
        // ignore
      }
      const message = payload?.error || `HTTP ${res.status}`;
      const err = new Error(message);
      (err as any).details = payload?.details;
      throw err;
    }

    // 204 No Content
    if (res.status === 204) return undefined as unknown as T;
    return (await res.json()) as T;
  });
}

export const api = {
  get: <T>(path: string) => request<T>('GET', path),
  post: <T>(path: string, body?: any) => request<T>('POST', path, body),
  put: <T>(path: string, body?: any) => request<T>('PUT', path, body)
};
