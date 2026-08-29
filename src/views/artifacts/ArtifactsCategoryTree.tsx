import { TaxonomyDomainSummary } from "./types";

interface Props {
  visibleTaxonomy: TaxonomyDomainSummary[];
  totalAllArtifacts: number;
  selectedDomain: string;
  selectedSubcategory: string;
  showEmptyDomains: boolean;
  setShowEmptyDomains: (v: boolean) => void;
  onSelectDomain: (d: string) => void;
  onSelectSubcategory: (s: string) => void;
}

export function ArtifactsCategoryTree({
  visibleTaxonomy,
  totalAllArtifacts,
  selectedDomain,
  selectedSubcategory,
  showEmptyDomains,
  setShowEmptyDomains,
  onSelectDomain,
  onSelectSubcategory,
}: Props) {
  return (
    <div className="card" style={{ padding: 10, maxHeight: "calc(100vh - 160px)", overflowY: "auto", minWidth: 0 }}>
      <div className="row between mb-2" style={{ padding: "4px 6px" }}>
        <span style={{ fontSize: 10, fontWeight: 800, letterSpacing: "0.8px", color: "var(--text-3)" }}>
          AVAILABLE ARTIFACTS ({visibleTaxonomy.length})
        </span>
        <button 
          className="btn btn-ghost btn-sm"
          style={{ fontSize: 10, padding: "1px 6px", height: "auto" }}
          onClick={() => setShowEmptyDomains(!showEmptyDomains)}
          title={showEmptyDomains ? "Hide empty categories" : "Show all categories including 0"}
        >
          {showEmptyDomains ? "Hide 0s" : "Show All"}
        </button>
      </div>

      {/* All Artifacts Root */}
      <div 
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "7px 8px",
          borderRadius: "var(--r-sm)",
          cursor: "pointer",
          marginBottom: 6,
          background: selectedDomain === "all" ? "var(--accent)" : "transparent",
          color: selectedDomain === "all" ? "#000" : "var(--text-0)",
          fontWeight: selectedDomain === "all" ? 700 : 500,
          fontSize: 12,
        }}
        onClick={() => { onSelectDomain("all"); onSelectSubcategory("all"); }}
      >
        <div className="row gap-1" style={{ alignItems: "center", overflow: "hidden" }}>
          <span>📁</span>
          <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>All Artifacts</span>
        </div>
        <span 
          className="badge" 
          style={{ 
            background: selectedDomain === "all" ? "#000" : "var(--bg-3)", 
            color: selectedDomain === "all" ? "#fff" : "var(--text-1)",
            fontSize: 10 
          }}
        >
          {totalAllArtifacts}
        </span>
      </div>

      {/* Domain List */}
      <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
        {visibleTaxonomy.length === 0 ? (
          <div className="muted text-xs p-3 text-center">No forensic artifacts detected in this case yet.</div>
        ) : (
          visibleTaxonomy.map((dom) => {
            const isDomainSelected = selectedDomain === dom.domain_id;
            return (
              <div key={dom.domain_id} style={{ display: "flex", flexDirection: "column" }}>
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "space-between",
                    padding: "6px 8px",
                    borderRadius: "var(--r-sm)",
                    cursor: "pointer",
                    background: isDomainSelected && selectedSubcategory === "all" ? "var(--bg-3)" : "transparent",
                    color: isDomainSelected ? "var(--accent)" : "var(--text-1)",
                    fontWeight: isDomainSelected ? 700 : 500,
                    fontSize: 12,
                    borderLeft: isDomainSelected ? "3px solid var(--accent)" : "3px solid transparent",
                  }}
                  onClick={() => {
                    onSelectDomain(dom.domain_id);
                    onSelectSubcategory("all");
                  }}
                >
                  <div className="row gap-1" style={{ alignItems: "center", overflow: "hidden", minWidth: 0 }}>
                    <span>{dom.icon}</span>
                    <span style={{ textOverflow: "ellipsis", whiteSpace: "nowrap", overflow: "hidden" }}>{dom.name}</span>
                  </div>
                  <span 
                    style={{ 
                      fontSize: 10.5, 
                      fontFamily: "var(--mono)",
                      color: isDomainSelected ? "var(--accent)" : "var(--text-3)",
                      fontWeight: 600,
                      flexShrink: 0
                    }}
                  >
                    {dom.total_count}
                  </span>
                </div>

                {/* Subcategories */}
                {isDomainSelected && dom.subcategories.filter(s => showEmptyDomains || s.count > 0).length > 0 && (
                  <div style={{ display: "flex", flexDirection: "column", paddingLeft: 18, marginTop: 2, marginBottom: 4, gap: 2 }}>
                    {[...dom.subcategories]
                      .filter(s => showEmptyDomains || s.count > 0)
                      .sort((a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: "base" }))
                      .map((sub) => {
                        const isSubSelected = selectedSubcategory === sub.subcategory_id;
                        return (
                          <div
                            key={sub.subcategory_id}
                            style={{
                              display: "flex",
                              alignItems: "center",
                              justifyContent: "space-between",
                              padding: "3px 6px",
                              borderRadius: "var(--r-sm)",
                              cursor: "pointer",
                              fontSize: 11,
                              color: isSubSelected ? "var(--accent)" : "var(--text-2)",
                              background: isSubSelected ? "rgba(56, 189, 248, 0.1)" : "transparent",
                              fontWeight: isSubSelected ? 700 : 400,
                            }}
                            onClick={(e) => {
                              e.stopPropagation();
                              onSelectSubcategory(sub.subcategory_id);
                            }}
                          >
                            <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>↳ {sub.name}</span>
                            <span style={{ fontSize: 9.5, fontFamily: "var(--mono)", color: "var(--text-3)", flexShrink: 0 }}>
                              {sub.count}
                            </span>
                          </div>
                        );
                    })}
                  </div>
                )}
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}
