import { useState } from "react";
import { AuthProvider } from "./auth";
import { AcquisitionProvider } from "./context/AcquisitionContext";
import { CaseListPage } from "./pages/CaseListPage";
import { CaseWorkspace } from "./pages/CaseWorkspace";

function Routes() {
  const [activeCase, setActiveCase] = useState<string | null>(null);

  if (!activeCase) return <CaseListPage onSelectCase={setActiveCase} />;
  return <CaseWorkspace caseId={activeCase} onBack={() => setActiveCase(null)} />;
}

export default function App() {
  return (
    <AuthProvider>
      <AcquisitionProvider>
        <Routes />
      </AcquisitionProvider>
    </AuthProvider>
  );
}
