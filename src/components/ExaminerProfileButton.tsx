import { useState } from "react";
import { useExaminerProfile } from "../auth";
import { ExaminerProfileModal } from "./ExaminerProfileModal";

export function ExaminerProfileButton() {
  const { profile } = useExaminerProfile();
  const [showModal, setShowModal] = useState(false);

  const initial = profile.fullName ? profile.fullName.charAt(0).toUpperCase() : "E";

  return (
    <>
      <button
        type="button"
        className="btn btn-ghost btn-sm"
        style={{
          display: "flex",
          alignItems: "center",
          gap: 8,
          padding: "4px 10px",
          background: "var(--bg-2)",
          border: "1px solid var(--border)",
          borderRadius: 20,
          cursor: "pointer",
          transition: "all 0.15s ease",
        }}
        onClick={() => setShowModal(true)}
        title="Click to view & edit Examiner Credentials & Report Signature"
      >
        <div
          style={{
            width: 22,
            height: 22,
            borderRadius: "50%",
            background: "linear-gradient(135deg, #22c55e, #16a34a)",
            color: "#ffffff",
            fontSize: 11,
            fontWeight: 800,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          {initial}
        </div>
        <div style={{ textAlign: "left", lineHeight: 1.2 }}>
          <div style={{ fontSize: 11.5, fontWeight: 700, color: "var(--text-0)" }}>
            {profile.fullName || "Examiner"}
          </div>
          <div style={{ fontSize: 9.5, color: "var(--text-3)" }}>
            {profile.badgeNumber || profile.agency ? `${profile.badgeNumber || ""} · ${profile.agency || ""}` : "Forensic Examiner"}
          </div>
        </div>
        <span
          style={{
            fontSize: 9,
            fontWeight: 800,
            padding: "1px 5px",
            borderRadius: 3,
            background: "rgba(34, 197, 94, 0.15)",
            color: "#22c55e",
            letterSpacing: "0.03em",
          }}
        >
          ACTIVE
        </span>
      </button>

      <ExaminerProfileModal
        isOpen={showModal}
        onClose={() => setShowModal(false)}
      />
    </>
  );
}
