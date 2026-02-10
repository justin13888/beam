import { createContext, useContext, useState, useEffect, type ReactNode } from "react";
import { env } from "@/env";

export interface User {
  id: string;
  username: string;
  email: string;
}

export interface AuthResponse {
  token: string;
  session_id: string;
  user: User;
}

export interface AuthContextType {
  user: User | null;
  token: string | null;
  sessionId: string | null;
  isAuthenticated: boolean;
  login: (data: AuthResponse) => void;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [sessionId, setSessionId] = useState<string | null>(null);

  useEffect(() => {
    // Initialize from localStorage
    const storedToken = localStorage.getItem("token");
    const storedSessionId = localStorage.getItem("session_id");
    const storedUser = localStorage.getItem("user");

    if (storedToken && storedSessionId && storedUser) {
      setToken(storedToken);
      setSessionId(storedSessionId);
      setUser(JSON.parse(storedUser));
    }
  }, []);

  const login = (data: AuthResponse) => {
    setToken(data.token);
    setSessionId(data.session_id);
    setUser(data.user);

    localStorage.setItem("token", data.token);
    localStorage.setItem("session_id", data.session_id);
    localStorage.setItem("user", JSON.stringify(data.user));
  };

  const logout = () => {
    // Ideally call API to revoke session here
    if (sessionId) {
      fetch(`${env.C_STREAM_SERVER_URL}/auth/logout`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ session_id: sessionId }),
      }).catch(console.error);
    }

    setToken(null);
    setSessionId(null);
    setUser(null);

    localStorage.removeItem("token");
    localStorage.removeItem("session_id");
    localStorage.removeItem("user");
  };

  const isAuthenticated = !!token;

  return (
    <AuthContext.Provider value={{ user, token, sessionId, isAuthenticated, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
}
