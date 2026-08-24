import { useState, useEffect, createContext, useContext, ReactNode } from "react";

export interface User {
  id: string;
  username: string;
  fullName?: string;
  agency?: string;
  email?: string;
  role: string;
}

export interface StoredAccount extends User {
  passwordHash: string;
  createdAt: string;
}

interface AuthState {
  user: User | null;
  login: (username: string, password: string) => Promise<boolean>;
  register: (data: { username: string; password: string; fullName: string; agency: string; email?: string }) => Promise<{ success: boolean; message?: string }>;
  logout: () => void;
}

const DEFAULT_ACCOUNTS: StoredAccount[] = [
  {
    id: "admin-1",
    username: "admin",
    fullName: "Lead Forensic Examiner",
    agency: "Digital Forensics & Incident Response",
    email: "admin@forensic.local",
    role: "admin",
    passwordHash: "admin123",
    createdAt: new Date().toISOString(),
  },
];

const AuthContext = createContext<AuthState>({
  user: null,
  login: async () => false,
  register: async () => ({ success: false }),
  logout: () => {},
});

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(() => {
    const saved = localStorage.getItem("j12_current_user");
    if (saved) {
      try { return JSON.parse(saved); } catch { return null; }
    }
    return null;
  });

  const getAccounts = (): StoredAccount[] => {
    const raw = localStorage.getItem("j12_accounts");
    if (!raw) {
      localStorage.setItem("j12_accounts", JSON.stringify(DEFAULT_ACCOUNTS));
      return DEFAULT_ACCOUNTS;
    }
    try {
      return JSON.parse(raw);
    } catch {
      return DEFAULT_ACCOUNTS;
    }
  };

  const login = async (username: string, password: string): Promise<boolean> => {
    const cleanUser = username.trim().toLowerCase();
    const cleanPass = password.trim();

    const accounts = getAccounts();
    const match = accounts.find(
      (a) => a.username.toLowerCase() === cleanUser && a.passwordHash === cleanPass
    );

    if (match) {
      const authUser: User = {
        id: match.id,
        username: match.username,
        fullName: match.fullName || match.username,
        agency: match.agency,
        email: match.email,
        role: match.role,
      };
      setUser(authUser);
      localStorage.setItem("j12_current_user", JSON.stringify(authUser));
      return true;
    }

    return false;
  };

  const register = async (data: {
    username: string;
    password: string;
    fullName: string;
    agency: string;
    email?: string;
  }): Promise<{ success: boolean; message?: string }> => {
    const cleanUser = data.username.trim();
    const cleanPass = data.password.trim();

    if (!cleanUser || cleanUser.length < 3) {
      return { success: false, message: "Username must be at least 3 characters." };
    }
    if (!cleanPass || cleanPass.length < 4) {
      return { success: false, message: "Password must be at least 4 characters." };
    }

    const accounts = getAccounts();
    if (accounts.some((a) => a.username.toLowerCase() === cleanUser.toLowerCase())) {
      return { success: false, message: "Username is already registered. Please sign in or use another username." };
    }

    const newAccount: StoredAccount = {
      id: "usr-" + Date.now(),
      username: cleanUser,
      fullName: data.fullName.trim() || cleanUser,
      agency: data.agency.trim() || "Independent Digital Forensic Lab",
      email: data.email?.trim() || "",
      role: "examiner",
      passwordHash: cleanPass,
      createdAt: new Date().toISOString(),
    };

    const updated = [...accounts, newAccount];
    localStorage.setItem("j12_accounts", JSON.stringify(updated));

    const authUser: User = {
      id: newAccount.id,
      username: newAccount.username,
      fullName: newAccount.fullName,
      agency: newAccount.agency,
      email: newAccount.email,
      role: newAccount.role,
    };
    setUser(authUser);
    localStorage.setItem("j12_current_user", JSON.stringify(authUser));

    return { success: true };
  };

  const logout = () => {
    setUser(null);
    localStorage.removeItem("j12_current_user");
  };

  return (
    <AuthContext.Provider value={{ user, login, register, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  return useContext(AuthContext);
}
