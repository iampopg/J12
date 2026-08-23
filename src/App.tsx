import { useState } from "react";
import { AuthProvider, useAuth } from "./auth";
import { LoginPage } from "./pages/LoginPage";
import { CaseListPage } from "./pages/CaseListPage";
import { CaseWorkspace } from "./pages/CaseWorkspace";

function Routes() {
  const { user } = useAuth();
  const [activeCase, setActiveCase] = useState<string | null>(null);

  if (!user) return <LoginPage />;
  if (!activeCase) return <CaseListPage onSelectCase={setActiveCase} />;
  return <CaseWorkspace caseId={activeCase} onBack={() => setActiveCase(null)} />;
}

export default function App() {
  return (
    <AuthProvider>
      <Routes />
    </AuthProvider>
  );
}
