import { useState, useEffect, createContext, useContext, ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";

interface User {
  id: string;
  username: string;
  role: string;
}

interface AuthState {
  user: User | null;
  login: (username: string, password: string) => Promise<boolean>;
  logout: () => void;
}

const AuthContext = createContext<AuthState>({
  user: null,
  login: async () => false,
  logout: () => {},
});

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);

  const login = async (username: string, password: string): Promise<boolean> => {
    // Phase 0: simple auth. Phase N: real auth with hashing
    if (username === "admin" && password === "admin123") {
      setUser({ id: "1", username: "admin", role: "admin" });
      return true;
    }
    return false;
  };

  const logout = () => setUser(null);

  return (
    <AuthContext.Provider value={{ user, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  return useContext(AuthContext);
}
