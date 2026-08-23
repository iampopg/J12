import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Entity {
  id: string;
  email_address: string;
  display_name: string | null;
  first_seen: string | null;
  last_seen: string | null;
  sent_count: number;
  received_count: number;
  role: string;
}

interface EntityDetail {
  email: string;
  display_name: string | null;
  first_seen: string | null;
  last_seen: string | null;
  sent_count: number;
  received_count: number;
  sent_to: [string, number][];
  received_from: [string, number][];
}

interface Props {
  caseId: string;
}

export function EntityDiveView({ caseId }: Props) {
  const [entities, setEntities] = useState<Entity[]>([]);
  const [selectedEntity, setSelectedEntity] = useState<EntityDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [diveLoading, setDiveLoading] = useState(false);
  const [searchTerm, setSearchTerm] = useState("");

  useEffect(() => {
    loadEntities();
  }, [caseId]);

  const loadEntities = async () => {
    setLoading(true);
    try {
      const data = await invoke<Entity[]>("entity_list", { input: { case_id: caseId } });
      setEntities(data);
      // Auto-select first entity
      if (data.length > 0) {
        loadEntityDive(data[0].email_address);
      }
    } catch (e) {
      console.error("Failed to load entities:", e);
    }
    setLoading(false);
  };

  const loadEntityDive = async (email: string) => {
    setDiveLoading(true);
    try {
      const data = await invoke<EntityDetail>("entity_dive", {
        input: { case_id: caseId, email_address: email }
      });
      setSelectedEntity(data);
    } catch (e) {
      console.error("Failed to load entity dive:", e);
    }
    setDiveLoading(false);
  };

  const filteredEntities = entities.filter(e =>
    e.email_address.toLowerCase().includes(searchTerm.toLowerCase()) ||
    (e.display_name || "").toLowerCase().includes(searchTerm.toLowerCase())
  );

  if (loading) return <div className="empty">Loading entities...</div>;

  if (entities.length === 0) {
    return (
      <div>
        <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)", marginBottom: 16 }}>Entity Profiles</h2>
        <div className="card" style={{ textAlign: "center", padding: "60px 40px" }}>
          <div style={{ fontSize: 48, marginBottom: 16 }}>👥</div>
          <h3 style={{ fontSize: 18, marginBottom: 8, color: "var(--text-0)" }}>No Entities Extracted</h3>
          <p className="muted">Run entity extraction from email data to see entity profiles.</p>
          <button className="btn btn-primary mt-4" onClick={loadEntities}>Extract Entities</button>
        </div>
      </div>
    );
  }

  return (
    <div>
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>Entity Profiles</h2>
          <p className="muted">{entities.length} entities detected in this case</p>
        </div>
        <button className="btn btn-ghost btn-sm" onClick={loadEntities}>↻ Refresh</button>
      </div>

      <div className="grid-2" style={{ gap: 16 }}>
        {/* Entity List */}
        <div className="card" style={{ maxHeight: "70vh", overflowY: "auto" }}>
          <input
            className="input mb-4"
            placeholder="Search entities..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {filteredEntities.map((e) => (
              <div
                key={e.id}
                className={`row between tr-click`}
                style={{
                  padding: "10px 12px",
                  borderRadius: "var(--r-sm)",
                  background: selectedEntity?.email === e.email_address ? "var(--accent-subtle)" : "transparent",
                  border: selectedEntity?.email === e.email_address ? "1px solid var(--accent)" : "1px solid transparent",
                }}
                onClick={() => loadEntityDive(e.email_address)}
              >
                <div>
                  <div style={{ fontSize: 12, color: "var(--text-1)", fontWeight: 500 }}>
                    {e.display_name || e.email_address}
                  </div>
                  {e.display_name && (
                    <div style={{ fontSize: 11, color: "var(--accent)", fontFamily: "var(--mono)" }}>
                      {e.email_address}
                    </div>
                  )}
                </div>
                <div style={{ textAlign: "right" }}>
                  <div style={{ fontSize: 12, fontWeight: 600, color: "var(--text-0)" }}>
                    {e.sent_count + e.received_count}
                  </div>
                  <div style={{ fontSize: 10, color: "var(--text-3)" }}>emails</div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Entity Detail */}
        <div className="card" style={{ maxHeight: "70vh", overflowY: "auto" }}>
          {diveLoading && <div className="empty">Loading profile...</div>}
          {!diveLoading && selectedEntity && (
            <div>
              {/* Header */}
              <div style={{ marginBottom: 20 }}>
                <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 12 }}>
                  <div style={{ width: 48, height: 48, borderRadius: "50%", background: "linear-gradient(135deg, #3b82f6, #6366f1)", display: "flex", alignItems: "center", justifyContent: "center", fontSize: 20, color: "#fff", fontWeight: 700 }}>
                    {(selectedEntity.display_name || selectedEntity.email).charAt(0).toUpperCase()}
                  </div>
                  <div>
                    <h3 style={{ fontSize: 18, fontWeight: 700, color: "var(--text-0)" }}>
                      {selectedEntity.display_name || selectedEntity.email}
                    </h3>
                    <p style={{ fontSize: 13, color: "var(--accent)", fontFamily: "var(--mono)" }}>
                      {selectedEntity.email}
                    </p>
                  </div>
                </div>

                {/* Stats */}
                <div className="row gap-4" style={{ marginBottom: 20 }}>
                  <div style={{ flex: 1, padding: 12, background: "var(--bg-3)", borderRadius: "var(--r-sm)", textAlign: "center" }}>
                    <div style={{ fontSize: 20, fontWeight: 700, color: "var(--text-0)" }}>{selectedEntity.sent_count}</div>
                    <div style={{ fontSize: 10, color: "var(--text-3)" }}>SENT</div>
                  </div>
                  <div style={{ flex: 1, padding: 12, background: "var(--bg-3)", borderRadius: "var(--r-sm)", textAlign: "center" }}>
                    <div style={{ fontSize: 20, fontWeight: 700, color: "var(--text-0)" }}>{selectedEntity.received_count}</div>
                    <div style={{ fontSize: 10, color: "var(--text-3)" }}>RECEIVED</div>
                  </div>
                </div>

                {/* Date Range */}
                <div style={{ padding: 12, background: "var(--bg-3)", borderRadius: "var(--r-sm)", marginBottom: 20 }}>
                  <div className="row between">
                    <div>
                      <div style={{ fontSize: 10, color: "var(--text-3)" }}>FIRST SEEN</div>
                      <div style={{ fontSize: 13, color: "var(--text-1)" }}>
                        {selectedEntity.first_seen ? new Date(selectedEntity.first_seen).toLocaleDateString() : "—"}
                      </div>
                    </div>
                    <div style={{ textAlign: "right" }}>
                      <div style={{ fontSize: 10, color: "var(--text-3)" }}>LAST SEEN</div>
                      <div style={{ fontSize: 13, color: "var(--text-1)" }}>
                        {selectedEntity.last_seen ? new Date(selectedEntity.last_seen).toLocaleDateString() : "—"}
                      </div>
                    </div>
                  </div>
                </div>

                {/* Top Partners */}
                <div style={{ marginBottom: 20 }}>
                  <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 12 }}>Top Sent To</h4>
                  {selectedEntity.sent_to.length > 0 ? (
                    selectedEntity.sent_to.map(([email, count], i) => (
                      <div key={i} className="row between" style={{ padding: "6px 0", borderBottom: "1px solid var(--border)" }}>
                        <span style={{ fontSize: 12, fontFamily: "var(--mono)", color: "var(--text-1)" }}>{email}</span>
                        <span className="badge badge-blue">{count}</span>
                      </div>
                    ))
                  ) : <div className="muted text-sm">No data</div>}
                </div>

                 <div>
                   <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 12 }}>Top Received From</h4>
                   {selectedEntity.received_from.length > 0 ? (
                     selectedEntity.received_from.map(([email, count], i) => (
                       <div key={i} className="row between" style={{ padding: "6px 0", borderBottom: "1px solid var(--border)" }}>
                         <span style={{ fontSize: 12, fontFamily: "var(--mono)", color: "var(--text-1)" }}>{email}</span>
                         <span className="badge badge-gray">{count}</span>
                       </div>
                     ))
                   ) : <div className="muted text-sm">No data</div>}
                 </div>

                 {/* Communication Heatmap */}
                 <CommunicationHeatmap email={selectedEntity.email} />
               </div>
             </div>
           )}
         </div>
       </div>
    </div>
  );
}

function CommunicationHeatmap({ email }: { email: string }) {
  const [data, setData] = useState<{ date: string; count: number }[]>([]);

  useEffect(() => {
    invoke<any>("entity_heatmap", { input: { email_address: email } })
      .then(d => setData(d.data || []))
      .catch(() => setData([]));
  }, [email]);

  if (data.length === 0) return null;

  const maxCount = Math.max(...data.map(d => d.count), 1);

  return (
    <div style={{ marginTop: 20 }}>
      <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 12 }}>Activity Heatmap</h4>
      <div style={{ display: "flex", flexWrap: "wrap", gap: 2 }}>
        {data.map((d, i) => {
          const intensity = d.count / maxCount;
          const bg = d.count === 0 ? "var(--bg-3)" : `rgba(59, 130, 246, ${0.2 + intensity * 0.8})`;
          return (
            <div
              key={i}
              title={`${d.date}: ${d.count} emails`}
              style={{
                width: 12,
                height: 12,
                borderRadius: 2,
                background: bg,
                cursor: "pointer",
              }}
            />
          );
        })}
      </div>
      <div className="row between" style={{ marginTop: 8 }}>
        <span style={{ fontSize: 10, color: "var(--text-3)" }}>{data[0]?.date}</span>
        <span style={{ fontSize: 10, color: "var(--text-3)" }}>Less</span>
        <div style={{ display: "flex", gap: 2 }}>
          {[0.2, 0.4, 0.6, 0.8, 1.0].map((v, i) => (
            <div key={i} style={{ width: 10, height: 10, borderRadius: 2, background: `rgba(59, 130, 246, ${v})` }} />
          ))}
        </div>
        <span style={{ fontSize: 10, color: "var(--text-3)" }}>More</span>
        <span style={{ fontSize: 10, color: "var(--text-3)" }}>{data[data.length - 1]?.date}</span>
      </div>
    </div>
  );
}
