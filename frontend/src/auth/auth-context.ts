import { createContext } from 'react';

export interface AuthState {
  authenticated: boolean;
  loading: boolean;
  refresh: () => Promise<void>;
}

export const AuthContext = createContext<AuthState | undefined>(undefined);
