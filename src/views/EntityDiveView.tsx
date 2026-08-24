import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";

function cleanDisplayName(name: string | null): string {
  if (!name) return "";
  let cleaned = name
    .replace(/@ENRON.*$/g, "")
    .replace(/IMCEANOTES-[^<]*/g, "")
    .replace(/<[^>]*>/g, "")
    .replace(/"/g, "")
    .replace(/\s+/g, " ")
    .trim();
  if (cleaned.includes("@")) {
    return cleaned.split("@")[0].trim() || cleaned;
  }
  return cleaned;
}

interface Entity {
  id: string;
  email_address: string;
  display_name: string | null;
  first_seen: string | null;
  last_seen: string | null;
  sent_count: number;
  received_count: number;
  role: string;
  aliases?: string | null;
}

interface EntityDetail {
  email: string;
  display_name: string | null;
  first_seen: string | null;
  last_seen: string | null;
  sent_count: number;
  received_count: number;
  deleted_count: number;
  flagged_count: number;
  total_count: number;
  aliases: string[];
  sent_to: [string, number][];
  received_from: [string, number][];
  top_subjects: [string, number][];
}

interface EntityEmail {
  id: string;
  evidence_id: string;
  from_addr: string;
  from_display: string | null;
  to_addrs: string;
  cc_addrs: string;
  subject: string | null;
  date_sent: string | null;
  date_sent_utc: string;
  risk_score: number;
  folder_category: string;
  is_deleted: boolean;
  deleted_recovered: boolean;
  body_text: string | null;
  headers_raw: string | null;
}

type TabType = "all" | "sent" | "received" | "deleted" | "flagged" | "partners";
type EntityTier = "key" | "internal" | "all";

interface Props {
  caseId: string;
  onSelectEmail?: (id: string) => void;
}

export function EntityDiveView({ caseId }: Props) {
  const [entities, setEntities] = useState<Entity[]>([]);
  const [selectedEntity, setSelectedEntity] = useState<EntityDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [diveLoading, setDiveLoading] = useState(false);
  const [searchTerm, setSearchTerm] = useState("");
  const [entityTier, setEntityTier] = useState<EntityTier>("key");
  const [sortOption, setSortOption] = useState<"total" | "sent" | "received" | "name">("total");

  // Tab & Filter states
  const [activeTab, setActiveTab] = useState<TabType>("all");
  const [partnerFilter, setPartnerFilter] = useState<string>("");
  const [emailSearch, setEmailSearch] = useState("");
  const [dateFrom, setDateFrom] = useState("");
  const [dateTo, setDateTo] = useState("");
  const [hasAttachment, setHasAttachment] = useState(false);

  // Email messages state
  const [emails, setEmails] = useState<EntityEmail[]>([]);
  const [emailsLoading, setEmailsLoading] = useState(false);
  const [selectedEmail, setSelectedEmail] = useState<EntityEmail | null>(null);
  const [settingTarget, setSettingTarget] = useState(false);

  useEffect(() => {
    loadEntities();
  }, [caseId]);

  const loadEntities = async () => {
    setLoading(true);
    try {
      let data = await invoke<Entity[]>("entity_list", { input: { case_id: caseId } });
      if (data.length === 0) {
        await invoke<number>("extract_entities", { caseId });
        data = await invoke<Entity[]>("entity_list", { input: { case_id: caseId } });
      }
      setEntities(data);
      if (data.length > 0) {
        loadEntityDive(data[0].email_address);
      }
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  const loadEntityDive = async (email: string) => {
    setDiveLoading(true);
    setPartnerFilter("");
    setEmailSearch("");
    setDateFrom("");
    setDateTo("");
    setHasAttachment(false);
    setSelectedEmail(null);

    try {
      const data = await invoke<EntityDetail>("entity_dive", {
        input: { case_id: caseId, email_address: email },
      });
      setSelectedEntity(data);
      // Load initial emails for this entity
      loadEmailsForEntity(email, "all", "", "", "", false, "");
    } catch (e) {
      console.error(e);
    } finally {
      setDiveLoading(false);
    }
  };

  const loadEmailsForEntity = async (
    email: string,
    tab: TabType,
    partner: string,
    from: string,
    to: string,
    hasAtt: boolean,
    query: string
  ) => {
    setEmailsLoading(true);
    try {
      const data = await invoke<EntityEmail[]>("entity_emails", {
        input: {
          case_id: caseId,
          email,
          filter_type: tab === "partners" ? "all" : tab,
          partner_email: partner,
          q: query,
          date_from: from,
          date_to: to,
          has_attachment: hasAtt,
        },
      });
      setEmails(data);
    } catch (e) {
      console.error(e);
      setEmails([]);
    } finally {
      setEmailsLoading(false);
    }
  };

  const handleFilterChange = (
    tab: TabType,
    partner: string,
    from: string,
    to: string,
    hasAtt: boolean,
    query: string
  ) => {
    if (!selectedEntity) return;
    loadEmailsForEntity(selectedEntity.email, tab, partner, from, to, hasAtt, query);
  };

  const handleTabSelect = (tab: TabType) => {
    setActiveTab(tab);
    handleFilterChange(tab, partnerFilter, dateFrom, dateTo, hasAttachment, emailSearch);
  };

  const handlePartnerSelect = (partnerEmail: string) => {
    if (partnerFilter === partnerEmail) {
      setPartnerFilter("");
      handleFilterChange(activeTab, "", dateFrom, dateTo, hasAttachment, emailSearch);
    } else {
      setPartnerFilter(partnerEmail);
      handleFilterChange(activeTab, partnerEmail, dateFrom, dateTo, hasAttachment, emailSearch);
    }
  };

  const handleSetAsTarget = async () => {
    if (!selectedEntity) return;
    setSettingTarget(true);
    try {
      await invoke("case_update", {
        input: {
          id: caseId,
          target_email: selectedEntity.email,
          target_name: selectedEntity.display_name,
        },
      });
      alert(`🎯 ${selectedEntity.display_name || selectedEntity.email} has been set as the primary target for this case!`);
    } catch (e: any) {
      alert(`Failed to set target: ${e}`);
    } finally {
      setSettingTarget(false);
    }
  };

  const handleReExtract = async () => {
    setLoading(true);
    try {
      await invoke<number>("extract_entities", { caseId });
      await loadEntities();
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  // Filter and sort entities list
  const filteredEntities = useMemo(() => {
    let result = entities.filter((e) => {
      const matchSearch =
        e.email_address.toLowerCase().includes(searchTerm.toLowerCase()) ||
        (e.display_name || "").toLowerCase().includes(searchTerm.toLowerCase()) ||
        (e.aliases || "").toLowerCase().includes(searchTerm.toLowerCase());
      if (!matchSearch) return false;

      const total = e.sent_count + e.received_count;
      if (entityTier === "key") {
        // Key people: either >= 5 communications or has known full human name with space
        return total >= 5 || (e.display_name && e.display_name.includes(" "));
      } else if (entityTier === "internal") {
        return e.email_address.endsWith("@enron.com");
      }
      return true;
    });

    result.sort((a, b) => {
      switch (sortOption) {
        case "sent":
          return b.sent_count - a.sent_count;
        case "received":
          return b.received_count - a.received_count;
        case "name":
          return (a.display_name || a.email_address).localeCompare(
            b.display_name || b.email_address
          );
        default:
          return b.sent_count + b.received_count - (a.sent_count + a.received_count);
      }
    });

    return result;
  }, [entities, searchTerm, entityTier, sortOption]);

  if (loading) return <div className="empty">Loading unified entity profiles...</div>;

  if (entities.length === 0) {
    return (
      <div>
        <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)", marginBottom: 16 }}>
          Entity Profiles
        </h2>
        <div className="card empty">No entities found. Upload and parse email data first.</div>
      </div>
    );
  }

  return (
    <div>
      {/* Top Header */}
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>
            Entity Profiles & Person Resolution
          </h2>
          <p className="muted" style={{ fontSize: 12 }}>
            Unified person profiles merging Exchange accounts, aliases, and corporate addresses into single individuals.
          </p>
        </div>
        <div className="row gap-2">
          <button
            className="btn btn-primary btn-sm"
            onClick={handleReExtract}
            title="Re-extract, resolve, and unify all entities and aliases"
          >
            ⚡ Re-Extract & Unify
          </button>
          <button className="btn btn-ghost btn-sm" onClick={loadEntities}>
            ↻ Refresh
          </button>
        </div>
      </div>

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "350px 1fr",
          gap: 16,
          alignItems: "start",
        }}
      >
        {/* Left Column: Entity Directory */}
        <div
          className="card"
          style={{
            padding: 14,
            maxHeight: "82vh",
            display: "flex",
            flexDirection: "column",
            marginBottom: 0,
          }}
        >
          {/* Entity Tier Tabs */}
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "1fr 1fr 1fr",
              gap: 4,
              background: "var(--bg-3)",
              padding: 3,
              borderRadius: "var(--r-sm)",
              marginBottom: 10,
            }}
          >
            <button
              type="button"
              style={{
                padding: "4px 6px",
                fontSize: 11,
                fontWeight: 600,
                border: "none",
                borderRadius: "var(--r-xs)",
                background: entityTier === "key" ? "var(--accent)" : "transparent",
                color: entityTier === "key" ? "#fff" : "var(--text-2)",
                cursor: "pointer",
              }}
              onClick={() => setEntityTier("key")}
            >
              Key People
            </button>
            <button
              type="button"
              style={{
                padding: "4px 6px",
                fontSize: 11,
                fontWeight: 600,
                border: "none",
                borderRadius: "var(--r-xs)",
                background: entityTier === "internal" ? "var(--accent)" : "transparent",
                color: entityTier === "internal" ? "#fff" : "var(--text-2)",
                cursor: "pointer",
              }}
              onClick={() => setEntityTier("internal")}
            >
              Internal Org
            </button>
            <button
              type="button"
              style={{
                padding: "4px 6px",
                fontSize: 11,
                fontWeight: 600,
                border: "none",
                borderRadius: "var(--r-xs)",
                background: entityTier === "all" ? "var(--accent)" : "transparent",
                color: entityTier === "all" ? "#fff" : "var(--text-2)",
                cursor: "pointer",
              }}
              onClick={() => setEntityTier("all")}
            >
              All ({entities.length})
            </button>
          </div>

          {/* Search and Sort */}
          <div className="mb-2">
            <input
              className="input mb-2"
              style={{ fontSize: 12, padding: "6px 10px" }}
              placeholder="Search name, email, alias..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
            />
            <select
              className="select input"
              style={{ fontSize: 11, padding: "5px 8px" }}
              value={sortOption}
              onChange={(e) => setSortOption(e.target.value as any)}
            >
              <option value="total">Sort: Total Messages (High → Low)</option>
              <option value="sent">Sort: Sent Count (High → Low)</option>
              <option value="received">Sort: Received Count (High → Low)</option>
              <option value="name">Sort: Name (A → Z)</option>
            </select>
          </div>

          <div style={{ fontSize: 11, color: "var(--text-3)", marginBottom: 8, paddingLeft: 4 }}>
            Showing <strong>{filteredEntities.length}</strong> {entityTier === "key" ? "key participants" : "entities"}
          </div>

          {/* List */}
          <div
            style={{
              flex: 1,
              overflowY: "auto",
              display: "flex",
              flexDirection: "column",
              gap: 4,
              paddingRight: 4,
            }}
          >
            {filteredEntities.map((e) => {
              const isSelected = selectedEntity?.email === e.email_address;
              const total = e.sent_count + e.received_count;
              const initial = (e.display_name || e.email_address).charAt(0).toUpperCase();

              return (
                <div
                  key={e.id}
                  className="tr-click"
                  style={{
                    padding: "9px 10px",
                    borderRadius: "var(--r-md)",
                    background: isSelected ? "var(--accent-subtle)" : "var(--bg-3)",
                    border: isSelected ? "1px solid var(--accent)" : "1px solid transparent",
                    display: "flex",
                    alignItems: "center",
                    gap: 10,
                    transition: "all 0.15s",
                  }}
                  onClick={() => loadEntityDive(e.email_address)}
                >
                  <div
                    style={{
                      width: 32,
                      height: 32,
                      borderRadius: "50%",
                      background: isSelected
                        ? "var(--accent)"
                        : "linear-gradient(135deg, #3b82f6, #6366f1)",
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "center",
                      fontSize: 13,
                      color: "#fff",
                      fontWeight: 700,
                      flexShrink: 0,
                    }}
                  >
                    {initial}
                  </div>

                  <div style={{ flex: 1, minWidth: 0 }}>
                    <div
                      style={{
                        fontSize: 12,
                        fontWeight: 600,
                        color: isSelected ? "var(--accent)" : "var(--text-0)",
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                      }}
                    >
                      {cleanDisplayName(e.display_name) || e.email_address}
                    </div>
                    <div
                      style={{
                        fontSize: 11,
                        color: "var(--text-3)",
                        fontFamily: "var(--mono)",
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                      }}
                    >
                      {e.email_address}
                    </div>
                  </div>

                  <div style={{ textAlign: "right", flexShrink: 0 }}>
                    <span
                      className="badge"
                      style={{
                        background: isSelected ? "var(--accent)" : "var(--bg-4)",
                        color: isSelected ? "#fff" : "var(--text-1)",
                        fontSize: 10,
                        fontWeight: 600,
                      }}
                    >
                      {total.toLocaleString()}
                    </span>
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Right Column: Selected Entity Investigation Hub */}
        <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
          {diveLoading ? (
            <div className="card" style={{ padding: 48, textAlign: "center" }}>
              <div className="empty">Loading unified person profile...</div>
            </div>
          ) : selectedEntity ? (
            <>
              {/* Profile Card */}
              <div
                className="card mb-0"
                style={{
                  padding: 20,
                  borderLeft: "4px solid var(--accent)",
                  background: "var(--bg-2)",
                }}
              >
                <div className="row between" style={{ alignItems: "flex-start" }}>
                  <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
                    <div
                      style={{
                        width: 56,
                        height: 56,
                        borderRadius: "50%",
                        background: "linear-gradient(135deg, #3b82f6, #6366f1)",
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "center",
                        fontSize: 24,
                        color: "#fff",
                        fontWeight: 700,
                        boxShadow: "0 4px 12px rgba(59,130,246,0.3)",
                      }}
                    >
                      {(selectedEntity.display_name || selectedEntity.email)
                        .charAt(0)
                        .toUpperCase()}
                    </div>
                    <div>
                      <h3 style={{ fontSize: 20, fontWeight: 700, color: "var(--text-0)" }}>
                        {cleanDisplayName(selectedEntity.display_name) || selectedEntity.email}
                      </h3>
                      <p
                        style={{
                          fontSize: 13,
                          color: "var(--accent)",
                          fontFamily: "var(--mono)",
                          marginBottom: 4,
                        }}
                      >
                        {selectedEntity.email}
                      </p>

                      {/* Merged Aliases List */}
                      {selectedEntity.aliases && selectedEntity.aliases.length > 0 && (
                        <div className="row gap-1 mb-2" style={{ flexWrap: "wrap" }}>
                          <span style={{ fontSize: 10, color: "var(--text-3)", fontWeight: 600 }}>
                            🔗 Unified Aliases:
                          </span>
                          {selectedEntity.aliases.map((alias) => (
                            <span
                              key={alias}
                              className="badge"
                              style={{
                                fontSize: 10,
                                fontFamily: "var(--mono)",
                                background: "var(--bg-4)",
                                color: "var(--text-2)",
                              }}
                            >
                              {alias}
                            </span>
                          ))}
                        </div>
                      )}

                      <div className="row gap-3" style={{ fontSize: 11, color: "var(--text-3)" }}>
                        <span>
                          📅 First Seen:{" "}
                          <strong>
                            {selectedEntity.first_seen
                              ? new Date(selectedEntity.first_seen).toLocaleDateString()
                              : "—"}
                          </strong>
                        </span>
                        <span>·</span>
                        <span>
                          📅 Last Seen:{" "}
                          <strong>
                            {selectedEntity.last_seen
                              ? new Date(selectedEntity.last_seen).toLocaleDateString()
                              : "—"}
                          </strong>
                        </span>
                      </div>
                    </div>
                  </div>

                  <div className="row gap-2">
                    <button
                      className="btn btn-primary btn-sm"
                      onClick={handleSetAsTarget}
                      disabled={settingTarget}
                      title="Set this person as the primary target for this case"
                    >
                      🎯 {settingTarget ? "Setting..." : "Set as Target Profile"}
                    </button>
                  </div>
                </div>
              </div>

              {/* Category Filter Tabs */}
              <div
                style={{
                  display: "grid",
                  gridTemplateColumns: "repeat(5, 1fr)",
                  gap: 8,
                }}
              >
                <div
                  className="tr-click"
                  style={{
                    padding: "12px 14px",
                    background: activeTab === "all" ? "var(--accent-subtle)" : "var(--bg-2)",
                    border: activeTab === "all" ? "1px solid var(--accent)" : "1px solid var(--border)",
                    borderRadius: "var(--r-md)",
                    textAlign: "center",
                  }}
                  onClick={() => handleTabSelect("all")}
                >
                  <div style={{ fontSize: 18, fontWeight: 700, color: "var(--text-0)" }}>
                    {selectedEntity.total_count}
                  </div>
                  <div style={{ fontSize: 10, color: "var(--text-3)", fontWeight: 600, marginTop: 2 }}>
                    ALL MESSAGES
                  </div>
                </div>

                <div
                  className="tr-click"
                  style={{
                    padding: "12px 14px",
                    background: activeTab === "sent" ? "rgba(59,130,246,0.15)" : "var(--bg-2)",
                    border: activeTab === "sent" ? "1px solid #3b82f6" : "1px solid var(--border)",
                    borderRadius: "var(--r-md)",
                    textAlign: "center",
                  }}
                  onClick={() => handleTabSelect("sent")}
                >
                  <div style={{ fontSize: 18, fontWeight: 700, color: "#3b82f6" }}>
                    {selectedEntity.sent_count}
                  </div>
                  <div style={{ fontSize: 10, color: "var(--text-3)", fontWeight: 600, marginTop: 2 }}>
                    SENT BY THIS PERSON
                  </div>
                </div>

                <div
                  className="tr-click"
                  style={{
                    padding: "12px 14px",
                    background: activeTab === "received" ? "rgba(34,197,94,0.15)" : "var(--bg-2)",
                    border: activeTab === "received" ? "1px solid #22c55e" : "1px solid var(--border)",
                    borderRadius: "var(--r-md)",
                    textAlign: "center",
                  }}
                  onClick={() => handleTabSelect("received")}
                >
                  <div style={{ fontSize: 18, fontWeight: 700, color: "#22c55e" }}>
                    {selectedEntity.received_count}
                  </div>
                  <div style={{ fontSize: 10, color: "var(--text-3)", fontWeight: 600, marginTop: 2 }}>
                    RECEIVED (TO / CC)
                  </div>
                </div>

                <div
                  className="tr-click"
                  style={{
                    padding: "12px 14px",
                    background: activeTab === "deleted" ? "rgba(239,68,68,0.15)" : "var(--bg-2)",
                    border: activeTab === "deleted" ? "1px solid #ef4444" : "1px solid var(--border)",
                    borderRadius: "var(--r-md)",
                    textAlign: "center",
                  }}
                  onClick={() => handleTabSelect("deleted")}
                >
                  <div style={{ fontSize: 18, fontWeight: 700, color: "#ef4444" }}>
                    {selectedEntity.deleted_count}
                  </div>
                  <div style={{ fontSize: 10, color: "var(--text-3)", fontWeight: 600, marginTop: 2 }}>
                    DELETED / RECOVERED
                  </div>
                </div>

                <div
                  className="tr-click"
                  style={{
                    padding: "12px 14px",
                    background: activeTab === "flagged" ? "rgba(234,179,8,0.15)" : "var(--bg-2)",
                    border: activeTab === "flagged" ? "1px solid #eab308" : "1px solid var(--border)",
                    borderRadius: "var(--r-md)",
                    textAlign: "center",
                  }}
                  onClick={() => handleTabSelect("flagged")}
                >
                  <div style={{ fontSize: 18, fontWeight: 700, color: "#eab308" }}>
                    {selectedEntity.flagged_count}
                  </div>
                  <div style={{ fontSize: 10, color: "var(--text-3)", fontWeight: 600, marginTop: 2 }}>
                    FLAGGED / HIGH RISK
                  </div>
                </div>
              </div>

              {/* Communication Partners & Top Subjects Row */}
              <div className="grid-2 mb-0" style={{ gap: 16 }}>
                {/* Top Sent To */}
                <div className="card mb-0" style={{ padding: 16 }}>
                  <div className="row between mb-2">
                    <strong style={{ fontSize: 12, color: "var(--text-0)" }}>
                      📤 Communicated / Sent To (Click to Filter)
                    </strong>
                  </div>
                  {selectedEntity.sent_to.length > 0 ? (
                    <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                      {selectedEntity.sent_to.map(([email, count]) => {
                        const isPartnerSelected = partnerFilter === email;
                        return (
                          <div
                            key={email}
                            className="row between tr-click"
                            style={{
                              padding: "6px 8px",
                              borderRadius: "var(--r-sm)",
                              background: isPartnerSelected
                                ? "var(--accent-subtle)"
                                : "var(--bg-3)",
                              border: isPartnerSelected
                                ? "1px solid var(--accent)"
                                : "1px solid transparent",
                            }}
                            onClick={() => handlePartnerSelect(email)}
                          >
                            <span
                              style={{
                                fontSize: 11,
                                fontFamily: "var(--mono)",
                                color: isPartnerSelected ? "var(--accent)" : "var(--text-1)",
                                overflow: "hidden",
                                textOverflow: "ellipsis",
                                whiteSpace: "nowrap",
                              }}
                            >
                              {email}
                            </span>
                            <span className="badge badge-blue">{count}</span>
                          </div>
                        );
                      })}
                    </div>
                  ) : (
                    <div className="muted text-sm">No sent communications</div>
                  )}
                </div>

                {/* Top Received From */}
                <div className="card mb-0" style={{ padding: 16 }}>
                  <div className="row between mb-2">
                    <strong style={{ fontSize: 12, color: "var(--text-0)" }}>
                      📥 Received From (Click to Filter)
                    </strong>
                  </div>
                  {selectedEntity.received_from.length > 0 ? (
                    <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                      {selectedEntity.received_from.map(([email, count]) => {
                        const isPartnerSelected = partnerFilter === email;
                        return (
                          <div
                            key={email}
                            className="row between tr-click"
                            style={{
                              padding: "6px 8px",
                              borderRadius: "var(--r-sm)",
                              background: isPartnerSelected
                                ? "var(--accent-subtle)"
                                : "var(--bg-3)",
                              border: isPartnerSelected
                                ? "1px solid var(--accent)"
                                : "1px solid transparent",
                            }}
                            onClick={() => handlePartnerSelect(email)}
                          >
                            <span
                              style={{
                                fontSize: 11,
                                fontFamily: "var(--mono)",
                                color: isPartnerSelected ? "var(--accent)" : "var(--text-1)",
                                overflow: "hidden",
                                textOverflow: "ellipsis",
                                whiteSpace: "nowrap",
                              }}
                            >
                              {email}
                            </span>
                            <span className="badge badge-gray">{count}</span>
                          </div>
                        );
                      })}
                    </div>
                  ) : (
                    <div className="muted text-sm">No received communications</div>
                  )}
                </div>
              </div>

              {/* Email Messages Explorer */}
              <div className="card mb-0" style={{ padding: 16 }}>
                <div className="row between mb-3">
                  <div className="row gap-2">
                    <strong style={{ fontSize: 13, color: "var(--text-0)" }}>
                      📧 Messages (
                      {activeTab === "sent"
                        ? "Sent"
                        : activeTab === "received"
                        ? "Received"
                        : activeTab === "deleted"
                        ? "Deleted"
                        : activeTab === "flagged"
                        ? "Flagged"
                        : "All"}
                      : {emails.length})
                    </strong>
                    {partnerFilter && (
                      <span
                        className="badge badge-blue"
                        style={{ cursor: "pointer" }}
                        onClick={() => handlePartnerSelect("")}
                        title="Click to clear partner filter"
                      >
                        Thread with {partnerFilter} ✕
                      </span>
                    )}
                  </div>

                  <div className="row gap-2">
                    <label className="row gap-1" style={{ fontSize: 11, color: "var(--text-2)", cursor: "pointer" }}>
                      <input
                        type="checkbox"
                        checked={hasAttachment}
                        onChange={(e) => {
                          setHasAttachment(e.target.checked);
                          handleFilterChange(
                            activeTab,
                            partnerFilter,
                            dateFrom,
                            dateTo,
                            e.target.checked,
                            emailSearch
                          );
                        }}
                      />
                      Has Attachments
                    </label>
                  </div>
                </div>

                {/* Filter Controls Bar */}
                <div className="row gap-2 mb-3" style={{ flexWrap: "wrap" }}>
                  <input
                    className="input"
                    style={{ flex: 1, minWidth: 200, fontSize: 12, padding: "6px 10px" }}
                    placeholder="Search subject or body text..."
                    value={emailSearch}
                    onChange={(e) => {
                      setEmailSearch(e.target.value);
                      handleFilterChange(
                        activeTab,
                        partnerFilter,
                        dateFrom,
                        dateTo,
                        hasAttachment,
                        e.target.value
                      );
                    }}
                  />
                  <div className="row gap-1">
                    <input
                      type="date"
                      className="input"
                      style={{ width: 140, fontSize: 11, padding: "5px 8px" }}
                      value={dateFrom}
                      onChange={(e) => {
                        setDateFrom(e.target.value);
                        handleFilterChange(
                          activeTab,
                          partnerFilter,
                          e.target.value,
                          dateTo,
                          hasAttachment,
                          emailSearch
                        );
                      }}
                      title="Date from"
                    />
                    <input
                      type="date"
                      className="input"
                      style={{ width: 140, fontSize: 11, padding: "5px 8px" }}
                      value={dateTo}
                      onChange={(e) => {
                        setDateTo(e.target.value);
                        handleFilterChange(
                          activeTab,
                          partnerFilter,
                          dateFrom,
                          e.target.value,
                          hasAttachment,
                          emailSearch
                        );
                      }}
                      title="Date to"
                    />
                  </div>
                </div>

                {/* Messages List Table */}
                {emailsLoading ? (
                  <div className="empty">Loading emails...</div>
                ) : emails.length === 0 ? (
                  <div className="empty">No emails match the selected filters</div>
                ) : (
                  <div style={{ maxHeight: 380, overflowY: "auto", border: "1px solid var(--border)", borderRadius: "var(--r-md)" }}>
                    <div
                      style={{
                        display: "grid",
                        gridTemplateColumns: "160px 1fr 100px 70px",
                        padding: "8px 12px",
                        background: "var(--bg-1)",
                        borderBottom: "1px solid var(--border)",
                        fontSize: 10,
                        fontWeight: 700,
                        textTransform: "uppercase",
                        letterSpacing: "0.06em",
                        color: "var(--text-3)",
                      }}
                    >
                      <div>From</div>
                      <div>Subject</div>
                      <div style={{ textAlign: "right" }}>Date</div>
                      <div style={{ textAlign: "center" }}>Risk</div>
                    </div>

                    {emails.map((em) => {
                      const isEmailActive = selectedEmail?.id === em.id;
                      return (
                        <div
                          key={em.id}
                          className="tr-click"
                          style={{
                            display: "grid",
                            gridTemplateColumns: "160px 1fr 100px 70px",
                            alignItems: "center",
                            padding: "8px 12px",
                            borderBottom: "1px solid var(--border)",
                            background: isEmailActive ? "var(--accent-subtle)" : "transparent",
                            fontSize: 12,
                          }}
                          onClick={() => setSelectedEmail(isEmailActive ? null : em)}
                        >
                          <div
                            style={{
                              overflow: "hidden",
                              textOverflow: "ellipsis",
                              whiteSpace: "nowrap",
                              color: "var(--text-1)",
                            }}
                            title={em.from_addr}
                          >
                            {cleanDisplayName(em.from_display) || em.from_addr}
                          </div>
                          <div
                            style={{
                              overflow: "hidden",
                              textOverflow: "ellipsis",
                              whiteSpace: "nowrap",
                              color: "var(--text-0)",
                              fontWeight: 500,
                            }}
                          >
                            {em.subject || <span className="muted">(no subject)</span>}
                            {em.deleted_recovered && (
                              <span
                                className="badge badge-red"
                                style={{ fontSize: 9, marginLeft: 6 }}
                              >
                                DELETED
                              </span>
                            )}
                          </div>
                          <div style={{ textAlign: "right", fontSize: 11, color: "var(--text-3)" }}>
                            {em.date_sent_utc
                              ? new Date(em.date_sent_utc).toLocaleDateString()
                              : "—"}
                          </div>
                          <div style={{ textAlign: "center" }}>
                            <span
                              className={`badge ${
                                em.risk_score >= 50
                                  ? "badge-red"
                                  : em.risk_score >= 25
                                  ? "badge-orange"
                                  : "badge-green"
                              }`}
                              style={{ fontSize: 9 }}
                            >
                              {em.risk_score}
                            </span>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                )}

                {/* Inline Message Preview if Selected */}
                {selectedEmail && (
                  <div
                    style={{
                      marginTop: 16,
                      padding: 16,
                      background: "var(--bg-1)",
                      border: "1px solid var(--border)",
                      borderRadius: "var(--r-md)",
                    }}
                  >
                    <div className="row between mb-2">
                      <strong style={{ fontSize: 14, color: "var(--text-0)" }}>
                        {selectedEmail.subject || "(no subject)"}
                      </strong>
                      <button
                        className="btn btn-ghost btn-sm"
                        style={{ padding: "2px 6px", fontSize: 11 }}
                        onClick={() => setSelectedEmail(null)}
                      >
                        ✕ Close Preview
                      </button>
                    </div>
                    <div className="grid-2 mb-3" style={{ fontSize: 12 }}>
                      <div>
                        <span className="muted">From: </span>
                        <strong>{selectedEmail.from_addr}</strong>
                      </div>
                      <div>
                        <span className="muted">Date: </span>
                        {selectedEmail.date_sent_utc
                          ? new Date(selectedEmail.date_sent_utc).toLocaleString()
                          : "—"}
                      </div>
                    </div>
                    <div style={{ fontSize: 12, marginBottom: 8 }}>
                      <span className="muted">To: </span>
                      <span className="mono">{selectedEmail.to_addrs}</span>
                    </div>
                    {selectedEmail.body_text && (
                      <pre
                        style={{
                          background: "var(--bg-0)",
                          border: "1px solid var(--border)",
                          borderRadius: "var(--r-md)",
                          padding: 12,
                          fontSize: 12,
                          maxHeight: 200,
                          overflow: "auto",
                          whiteSpace: "pre-wrap",
                        }}
                      >
                        {selectedEmail.body_text}
                      </pre>
                    )}
                  </div>
                )}
              </div>
            </>
          ) : (
            <div className="card empty">Select an entity from the list to explore</div>
          )}
        </div>
      </div>
    </div>
  );
}
