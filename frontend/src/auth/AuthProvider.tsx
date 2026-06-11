import { useCallback, useEffect, useReducer, type ReactNode } from 'react';
import { getMe } from '../api/client';
import { AuthContext } from './auth-context';

interface AuthData {
  authenticated: boolean;
  loading: boolean;
}

function resolved(_state: AuthData, authenticated: boolean): AuthData {
  return { authenticated, loading: false };
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [{ authenticated, loading }, dispatch] = useReducer(resolved, {
    authenticated: false,
    loading: true,
  });

  const refresh = useCallback(async () => {
    const me = await getMe();
    dispatch(me.authenticated);
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return <AuthContext.Provider value={{ authenticated, loading, refresh }}>{children}</AuthContext.Provider>;
}
