import { useState, useEffect, createContext, useContext, ReactNode } from "react";

export interface ExaminerProfile {
  id: string;
  fullName: string;
  title: string;
  agency: string;
  badgeNumber: string;
  email: string;
  certifications: string;
  signatureNotes?: string;
  avatarSeed?: string;
}

export const DEFAULT_EXAMINER: ExaminerProfile = {
  id: "examiner-default",
  fullName: "Lead Forensic Examiner",
  title: "Senior Digital Forensics Investigator",
  agency: "Digital Forensics & Incident Response Lab",
  badgeNumber: "DFIR-2026",
  email: "examiner@forensic.lab",
  certifications: "GCFA, EnCE, CCE",
  signatureNotes: "Certified Digital Evidence Handling & ISO 27037 Compliance",
};

interface ExaminerContextType {
  profile: ExaminerProfile;
  updateProfile: (updates: Partial<ExaminerProfile>) => void;
  resetProfile: () => void;
  // Compatibility helpers
  user: {
    id: string;
    username: string;
    fullName: string;
    agency: string;
    email: string;
    role: string;
  };
  login: (u: string, p: string) => Promise<boolean>;
  register: (data: any) => Promise<{ success: boolean; message?: string }>;
  logout: () => void;
}

const ExaminerContext = createContext<ExaminerContextType>({
  profile: DEFAULT_EXAMINER,
  updateProfile: () => {},
  resetProfile: () => {},
  user: {
    id: DEFAULT_EXAMINER.id,
    username: "examiner",
    fullName: DEFAULT_EXAMINER.fullName,
    agency: DEFAULT_EXAMINER.agency,
    email: DEFAULT_EXAMINER.email,
    role: "Lead Examiner",
  },
  login: async () => true,
  register: async () => ({ success: true }),
  logout: () => {},
});

export function AuthProvider({ children }: { children: ReactNode }) {
  const [profile, setProfile] = useState<ExaminerProfile>(() => {
    try {
      const saved = localStorage.getItem("j12_examiner_profile");
      if (saved) {
        return { ...DEFAULT_EXAMINER, ...JSON.parse(saved) };
      }
    } catch (e) {
      console.warn("Failed to load examiner profile:", e);
    }
    return DEFAULT_EXAMINER;
  });

  useEffect(() => {
    localStorage.setItem("j12_examiner_profile", JSON.stringify(profile));
  }, [profile]);

  const updateProfile = (updates: Partial<ExaminerProfile>) => {
    setProfile((prev) => {
      const next = { ...prev, ...updates };
      localStorage.setItem("j12_examiner_profile", JSON.stringify(next));
      return next;
    });
  };

  const resetProfile = () => {
    setProfile(DEFAULT_EXAMINER);
    localStorage.setItem("j12_examiner_profile", JSON.stringify(DEFAULT_EXAMINER));
  };

  const user = {
    id: profile.id,
    username: profile.badgeNumber || "examiner",
    fullName: profile.fullName,
    agency: profile.agency,
    email: profile.email,
    role: profile.title || "Lead Examiner",
  };

  const login = async () => true;
  const register = async () => ({ success: true });
  const logout = () => {};

  return (
    <ExaminerContext.Provider value={{ profile, updateProfile, resetProfile, user, login, register, logout }}>
      {children}
    </ExaminerContext.Provider>
  );
}

export function useExaminerProfile() {
  return useContext(ExaminerContext);
}

// Backward-compatible hook alias for existing components
export function useAuth() {
  return useContext(ExaminerContext);
}
