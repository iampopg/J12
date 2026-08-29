import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { EmailDetailModal, EmailModalData } from "../components/EmailDetailModal";
import {
  Entity,
  EntityDetail,
  EntityEmail,
  TabType,
  EntityTier,
  EntityDiveProps,
} from "./entity_dive/types";
import { EntityDirectorySidebar } from "./entity_dive/EntityDirectorySidebar";
import { EntityProfileHeader } from "./entity_dive/EntityProfileHeader";
import { EntityStatCards } from "./entity_dive/EntityStatCards";
import { EntityCommunicationPartners } from "./entity_dive/EntityCommunicationPartners";
import { EntityMessagesExplorer } from "./entity_dive/EntityMessagesExplorer";

export function EntityDiveView({ caseId, evidenceFilter }: EntityDiveProps) {
  const [entities, setEntities] = useState<Entity[]>([]);
  const [selectedEntity, setSelectedEntity] = useState<EntityDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [diveLoading, setDiveLoading] = useState(false);
  const [searchTerm, setSearchTerm] = useState("");
  const [entityTier, setEntityTier] = useState<EntityTier>("key");
  const [sortOption, setSortOption] = useState<"total" | "sent" | "received" | "name">("total");

  const [activeTab, setActiveTab] = useState<TabType>("all");
  const [partnerFilter, setPartnerFilter] = useState<string>("");
  const [emailSearch, setEmailSearch] = useState("");
  const [dateFrom, setDateFrom] = useState("");
  const [dateTo, setDateTo] = useState("");
  const [hasAttachment, setHasAttachment] = useState(false);

  const [emails, setEmails] = useState<EntityEmail[]>([]);
  const [emailsLoading, setEmailsLoading] = useState(false);
  const [selectedEmail, setSelectedEmail] = useState<EntityEmail | null>(null);
  const [modalEmail, setModalEmail] = useState<EmailModalData | null>(null);
  const [settingTarget, setSettingTarget] = useState(false);
  const [notification, setNotification] = useState<string | null>(null);

  const handleSelectEmail = async (em: EntityEmail) => {
    if (selectedEmail?.id === em.id) {
      setSelectedEmail(null);
      return;
    }
    if (!em.body_text && !em.body_html) {
      try {
        const full = await invoke<any>("email_get", { id: em.id });
        if (full) {
          setSelectedEmail({
            ...em,
            body_text: full.body_text,
            body_html: full.body_html,
            headers_raw: full.headers_raw,
          });
          return;
        }
      } catch (e) {
        console.error("Failed to load full email body:", e);
      }
    }
    setSelectedEmail(em);
  };

  useEffect(() => {
    loadEntities();
  }, [caseId, evidenceFilter]);

  const loadEntities = async () => {
    setLoading(true);
    try {
      let data = await invoke<Entity[]>("entity_list", { input: { case_id: caseId, evidence_id: evidenceFilter || undefined } });
      if (data.length === 0) {
        await invoke<number>("extract_entities", { input: { case_id: caseId } });
        data = await invoke<Entity[]>("entity_list", { input: { case_id: caseId, evidence_id: evidenceFilter || undefined } });
      }
      setEntities(data);
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
        input: { case_id: caseId, evidence_id: evidenceFilter || undefined, email_address: email },
      });
      setSelectedEntity(data);
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
          evidence_id: evidenceFilter || undefined,
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
      setNotification(`🎯 ${selectedEntity.display_name || selectedEntity.email} set as primary target!`);
      setTimeout(() => setNotification(null), 3500);
    } catch (e: any) {
      console.error("Failed to set target:", e);
    } finally {
      setSettingTarget(false);
    }
  };

  const handleReExtract = async () => {
    setLoading(true);
    try {
      await invoke<number>("extract_entities", { input: { case_id: caseId } });
      await loadEntities();
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  // Dynamically detect primary organization domain from case entities
  const primaryDomain = useMemo(() => {
    const domainCounts = new Map<string, number>();
    for (const e of entities) {
      const parts = e.email_address.split("@");
      if (parts.length === 2) {
        const domain = parts[1].toLowerCase();
        domainCounts.set(domain, (domainCounts.get(domain) || 0) + (e.sent_count + e.received_count));
      }
    }
    let topDomain = "";
    let topCount = 0;
    for (const [dom, count] of domainCounts.entries()) {
      if (count > topCount) {
        topCount = count;
        topDomain = dom;
      }
    }
    return topDomain;
  }, [entities]);

  const filteredEntities = useMemo(() => {
    let result = entities.filter((e) => {
      const matchSearch =
        e.email_address.toLowerCase().includes(searchTerm.toLowerCase()) ||
        (e.display_name || "").toLowerCase().includes(searchTerm.toLowerCase()) ||
        (e.aliases || "").toLowerCase().includes(searchTerm.toLowerCase());
      if (!matchSearch) return false;

      const total = e.sent_count + e.received_count;
      const isAuto = /^(noreply|no-reply|donotreply|notifications|news|info|marketing|alerts|support|hello|shop|order|billing)@/i.test(e.email_address);

      if (entityTier === "key") {
        return (total >= 5 && !isAuto) || (e.sent_count > 0 && e.received_count > 0) || total >= 50;
      } else if (entityTier === "internal") {
        return primaryDomain ? e.email_address.toLowerCase().endsWith(`@${primaryDomain}`) : total >= 10;
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
            Entity Profiles &amp; Person Resolution
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
            ⚡ Re-Extract &amp; Unify
          </button>
          <button className="btn btn-ghost btn-sm" onClick={loadEntities}>
            ↻ Refresh
          </button>
        </div>
      </div>

      <div style={{ display: "grid", gridTemplateColumns: "350px 1fr", gap: 16, alignItems: "start" }}>
        {/* Left Column: Entity Directory */}
        <EntityDirectorySidebar
          entitiesCount={entities.length}
          filteredEntities={filteredEntities}
          selectedEmail={selectedEntity?.email}
          searchTerm={searchTerm}
          setSearchTerm={setSearchTerm}
          entityTier={entityTier}
          setEntityTier={setEntityTier}
          sortOption={sortOption}
          setSortOption={setSortOption}
          onSelectEntity={loadEntityDive}
        />

        {/* Right Column: Selected Entity Investigation Hub */}
        <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
          {diveLoading ? (
            <div className="card" style={{ padding: 48, textAlign: "center" }}>
              <div className="empty">Loading unified person profile...</div>
            </div>
          ) : selectedEntity ? (
            <>
              {notification && (
                <div
                  className="card mb-3"
                  style={{
                    background: "rgba(34,197,94,0.15)",
                    border: "1px solid #22c55e",
                    color: "#4ade80",
                    padding: "10px 16px",
                    fontWeight: 600,
                    fontSize: 13,
                  }}
                >
                  {notification}
                </div>
              )}

              <EntityProfileHeader
                selectedEntity={selectedEntity}
                onSetAsTarget={handleSetAsTarget}
                settingTarget={settingTarget}
              />

              <EntityStatCards
                selectedEntity={selectedEntity}
                activeTab={activeTab}
                onTabSelect={handleTabSelect}
              />

              <EntityCommunicationPartners
                selectedEntity={selectedEntity}
                partnerFilter={partnerFilter}
                onPartnerSelect={handlePartnerSelect}
              />

              <EntityMessagesExplorer
                caseId={caseId}
                activeTab={activeTab}
                emails={emails}
                emailsLoading={emailsLoading}
                partnerFilter={partnerFilter}
                onClearPartner={() => handlePartnerSelect("")}
                hasAttachment={hasAttachment}
                onToggleAttachment={(checked) => {
                  setHasAttachment(checked);
                  handleFilterChange(activeTab, partnerFilter, dateFrom, dateTo, checked, emailSearch);
                }}
                emailSearch={emailSearch}
                onSearchChange={(val) => {
                  setEmailSearch(val);
                  handleFilterChange(activeTab, partnerFilter, dateFrom, dateTo, hasAttachment, val);
                }}
                dateFrom={dateFrom}
                onDateFromChange={(val) => {
                  setDateFrom(val);
                  handleFilterChange(activeTab, partnerFilter, val, dateTo, hasAttachment, emailSearch);
                }}
                dateTo={dateTo}
                onDateToChange={(val) => {
                  setDateTo(val);
                  handleFilterChange(activeTab, partnerFilter, dateFrom, val, hasAttachment, emailSearch);
                }}
                selectedEmail={selectedEmail}
                onSelectEmail={handleSelectEmail}
                onOpenModal={(em) => setModalEmail({
                  id: em.id,
                  message_id: em.id,
                  from_addr: em.from_addr,
                  from_display: em.from_display,
                  to_addrs: em.to_addrs,
                  cc_addrs: em.cc_addrs || "",
                  subject: em.subject,
                  date_sent: em.date_sent,
                  date_sent_utc: em.date_sent_utc,
                  headers_raw: em.headers_raw,
                  body_text: em.body_text,
                  body_html: em.body_html,
                  folder_name: em.folder_category,
                  folder_category: em.folder_category,
                })}
                onClosePreview={() => setSelectedEmail(null)}
              />
            </>
          ) : (
            <div className="card empty">Select an entity from the list to explore</div>
          )}
        </div>
      </div>

      {modalEmail && (
        <EmailDetailModal
          email={modalEmail}
          onClose={() => setModalEmail(null)}
          titleSuffix="Return to Entity Dive"
        />
      )}
    </div>
  );
}
