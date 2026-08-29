import { useState, useRef, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { SearchEmail, SortField, SearchProps } from "./search/types";
import { SearchBar } from "./search/SearchBar";
import { SearchResultsTable } from "./search/SearchResultsTable";
import { SearchMessageInspector } from "./search/SearchMessageInspector";

export type { SearchEmail as Email };

interface FtsResponse {
  total_hits: number;
  execution_ms: number;
  query_parsed: string;
  items: SearchEmail[];
}

export function SearchView({ caseId, evidenceFilter, onViewEntity }: SearchProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchEmail[]>([]);
  const [loading, setLoading] = useState(false);
  const [searched, setSearched] = useState(false);
  const [sortField, setSortField] = useState<SortField>("date");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("desc");
  const [selectedEmail, setSelectedEmail] = useState<SearchEmail | null>(null);
  const [searchMetrics, setSearchMetrics] = useState<{ hits: number; ms: number } | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
    doSearch("");
  }, [caseId, evidenceFilter]);

  const doSearch = async (searchQuery?: string) => {
    const q = (searchQuery !== undefined ? searchQuery : query).trim();
    setLoading(true);
    setSearched(true);
    setSelectedEmail(null);
    try {
      if (q) {
        const res = await invoke<FtsResponse>("fts_search", {
          input: { case_id: caseId, evidence_id: evidenceFilter || undefined, query: q, limit: 500 },
        });
        setResults(res?.items || []);
        setSearchMetrics({ hits: res?.total_hits || 0, ms: res?.execution_ms || 0 });
      } else {
        const res = await invoke<SearchEmail[]>("advanced_search", {
          input: { case_id: caseId, evidence_id: evidenceFilter || undefined, query: "", limit: 500 },
        });
        setResults(res || []);
        setSearchMetrics(null);
      }
    } catch (e) {
      console.error("Search failed:", e);
      // Fallback on advanced_search if FTS syntax error
      try {
        const fallback = await invoke<SearchEmail[]>("advanced_search", {
          input: { case_id: caseId, evidence_id: evidenceFilter || undefined, query: q, limit: 500 },
        });
        setResults(fallback || []);
      } catch {
        setResults([]);
      }
      setSearchMetrics(null);
    } finally {
      setLoading(false);
    }
  };

  const sortedResults = useMemo(() => {
    let sorted = [...results];
    sorted.sort((a, b) => {
      let cmp = 0;
      switch (sortField) {
        case "date":
          cmp = (a.date_sent_utc || "").localeCompare(b.date_sent_utc || "");
          break;
        case "from":
          cmp = (a.from_display || a.from_addr).localeCompare(b.from_display || b.from_addr);
          break;
        case "subject":
          cmp = (a.subject || "").localeCompare(b.subject || "");
          break;
        case "risk":
          cmp = a.risk_score - b.risk_score;
          break;
        case "rank":
          cmp = (a.match_rank || 0) - (b.match_rank || 0);
          break;
      }
      return sortDir === "asc" ? cmp : -cmp;
    });
    return sorted;
  }, [results, sortField, sortDir]);

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortField(field);
      setSortDir(field === "rank" ? "asc" : "desc");
    }
  };

  return (
    <div>
      {/* Top Title */}
      <div className="row between mb-4">
        <div>
          <h2 style={{ fontSize: 22, fontWeight: 700, color: "var(--text-0)" }}>
            Forensic &amp; eDiscovery Full-Text Search
          </h2>
          <p className="muted" style={{ fontSize: 12 }}>
            High-speed SQLite FTS5 inverted index with Porter Stemming, Boolean operators (<code>AND</code>, <code>OR</code>, <code>NOT</code>), proximity (<code>NEAR/5</code>), and attachment indexing.
          </p>
        </div>
      </div>

      {/* Main Search Bar Card */}
      <SearchBar
        query={query}
        setQuery={setQuery}
        loading={loading}
        inputRef={inputRef}
        onSearch={doSearch}
        searchMetrics={searchMetrics}
      />

      {/* Results Area */}
      {loading ? (
        <div className="card empty">Executing sub-millisecond FTS5 index lookup...</div>
      ) : searched && results.length === 0 ? (
        <div className="card empty">No emails or attachments match the query "{query}"</div>
      ) : (
        <div
          style={{
            display: "grid",
            gridTemplateColumns: selectedEmail ? "1fr 420px" : "1fr",
            gap: 16,
            alignItems: "start",
          }}
        >
          <SearchResultsTable
            caseId={caseId}
            results={sortedResults}
            selectedEmail={selectedEmail}
            sortField={sortField}
            sortDir={sortDir}
            onSort={handleSort}
            onSelectEmail={setSelectedEmail}
            onViewEntity={onViewEntity}
          />

          {selectedEmail && (
            <SearchMessageInspector
              caseId={caseId}
              selectedEmail={selectedEmail}
              onClose={() => setSelectedEmail(null)}
            />
          )}
        </div>
      )}
    </div>
  );
}
