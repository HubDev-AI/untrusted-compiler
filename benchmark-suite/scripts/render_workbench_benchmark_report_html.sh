#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 4 ]; then
  echo "usage: $0 <runs_json> <compare_matrix_json> <analysis_json> <out_report_html>" >&2
  exit 2
fi

runs_path="$1"
compare_path="$2"
analysis_path="$3"
out_path="$4"

for path in "$runs_path" "$compare_path" "$analysis_path"; do
  if [ ! -f "$path" ]; then
    echo "input file not found: $path" >&2
    exit 2
  fi
done

runs_json="$(cat "$runs_path")"
compare_json="$(cat "$compare_path")"
analysis_json="$(cat "$analysis_path")"
generated_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"

data_json="$(jq -nc \
  --arg generatedUtc "$generated_utc" \
  --arg repoRoot "$repo_root" \
  --argjson runs "$runs_json" \
  --argjson compare "$compare_json" \
  --argjson analysis "$analysis_json" \
  '
  def sanitize:
    if type == "string" then
      if startswith($repoRoot + "/") then .[($repoRoot | length + 1):] else . end
    elif type == "object" then
      with_entries(.value |= sanitize)
    elif type == "array" then
      map(sanitize)
    else
      .
    end;
  {
    generatedUtc: $generatedUtc,
    runs: ($runs | sanitize),
    compare: ($compare | sanitize),
    analysis: ($analysis | sanitize)
  }')"

mkdir -p "$(dirname "$out_path")"

{
  cat <<'HTML_HEAD'
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Workbench Benchmark Report</title>
  <style>
    :root {
      --bg: #f6f8fb;
      --card: #ffffff;
      --ink: #0d1b2a;
      --muted: #4f5d75;
      --line: #d6deeb;
      --accent: #0a7ea4;
      --good: #0b7a43;
      --warn: #aa6f00;
      --bad: #b42318;
      --code: #e7edf7;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
      background: linear-gradient(180deg, #f6f8fb 0%, #ecf1f8 100%);
      color: var(--ink);
      line-height: 1.45;
    }
    .shell {
      max-width: 1200px;
      margin: 0 auto;
      padding: 20px 16px 28px;
    }
    h1, h2, h3 { margin: 0; line-height: 1.2; }
    h1 { font-size: 1.6rem; letter-spacing: -0.01em; }
    h2 { font-size: 1.05rem; }
    .lede {
      margin-top: 6px;
      color: var(--muted);
      font-size: 0.95rem;
    }
    .meta {
      margin-top: 10px;
      color: var(--muted);
      font-size: 0.82rem;
      display: flex;
      flex-wrap: wrap;
      gap: 8px 12px;
    }
    .meta code {
      font-family: "IBM Plex Mono", "SFMono-Regular", monospace;
      background: var(--code);
      border: 1px solid var(--line);
      border-radius: 6px;
      padding: 1px 6px;
      font-size: 0.78rem;
    }
    .stats {
      margin-top: 14px;
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(170px, 1fr));
      gap: 10px;
    }
    .stat {
      background: var(--card);
      border: 1px solid var(--line);
      border-radius: 12px;
      padding: 10px 12px;
    }
    .label {
      color: var(--muted);
      font-size: 0.74rem;
      text-transform: uppercase;
      letter-spacing: 0.04em;
    }
    .value {
      margin-top: 3px;
      font-size: 1.18rem;
      font-weight: 700;
    }
    .value.good { color: var(--good); }
    .value.warn { color: var(--warn); }
    .value.bad { color: var(--bad); }
    .section {
      margin-top: 16px;
      background: var(--card);
      border: 1px solid var(--line);
      border-radius: 12px;
      padding: 12px;
    }
    .impl-wins {
      margin-top: 10px;
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
      gap: 8px;
    }
    .impl-win {
      border: 1px solid var(--line);
      border-radius: 10px;
      padding: 8px 10px;
      background: #fbfdff;
    }
    .impl-win .name {
      font-size: 0.8rem;
      color: var(--muted);
    }
    .impl-win .wins {
      margin-top: 2px;
      font-size: 1.1rem;
      font-weight: 700;
    }
    .endpoints {
      margin-top: 16px;
      display: grid;
      gap: 12px;
    }
    .endpoint {
      background: var(--card);
      border: 1px solid var(--line);
      border-radius: 12px;
      overflow: hidden;
    }
    .endpoint-head {
      padding: 10px 12px;
      background: #f3f7fd;
      border-bottom: 1px solid var(--line);
      display: grid;
      gap: 6px;
    }
    .endpoint-title {
      font-size: 1rem;
      font-weight: 700;
      letter-spacing: -0.01em;
    }
    .chips {
      display: flex;
      flex-wrap: wrap;
      gap: 6px;
      color: var(--muted);
      font-size: 0.76rem;
    }
    .chip {
      border: 1px solid var(--line);
      background: #fff;
      border-radius: 999px;
      padding: 2px 8px;
    }
    .rows {
      padding: 8px;
      display: grid;
      gap: 6px;
    }
    .row {
      border: 1px solid var(--line);
      border-radius: 10px;
      padding: 8px;
      display: grid;
      grid-template-columns: 1.3fr repeat(4, minmax(80px, 1fr));
      gap: 8px;
      align-items: center;
      background: #fff;
    }
    .row.top {
      border-color: #9fc7d5;
      background: #f5fbff;
    }
    .cell {
      display: grid;
      gap: 1px;
      min-width: 0;
    }
    .cell .k {
      color: var(--muted);
      font-size: 0.68rem;
      text-transform: uppercase;
      letter-spacing: 0.04em;
    }
    .cell .v {
      font-size: 0.88rem;
      font-weight: 600;
      overflow-wrap: anywhere;
    }
    .row .impl {
      font-size: 0.92rem;
      font-weight: 700;
    }
    .findings {
      margin-top: 8px;
      padding: 0 8px 10px;
      display: grid;
      gap: 6px;
    }
    .finding {
      border: 1px solid var(--line);
      border-radius: 10px;
      padding: 7px 8px;
      background: #fff;
      font-size: 0.83rem;
    }
    .finding strong { margin-right: 4px; }
    .finding.high strong { color: var(--bad); }
    .finding.medium strong { color: var(--warn); }
    .finding.low strong { color: var(--good); }
    @media (max-width: 860px) {
      .row {
        grid-template-columns: repeat(2, minmax(0, 1fr));
      }
    }
  </style>
</head>
<body>
  <main class="shell">
    <header>
      <h1>Workbench Benchmark Report</h1>
      <p class="lede">Cross-backend fixed-target benchmark results (compact matrix view)</p>
      <div id="meta" class="meta"></div>
    </header>
    <section id="summary" class="stats"></section>
    <section class="section">
      <h2>Leader Wins</h2>
      <div id="wins" class="impl-wins"></div>
    </section>
    <section>
      <div id="endpoints" class="endpoints"></div>
    </section>
  </main>
HTML_HEAD
  cat <<'HTML_DATA_OPEN'
  <script id="workbench-data" type="application/json">
HTML_DATA_OPEN
  printf '%s\n' "$data_json"
  cat <<'HTML_TAIL'
  </script>
  <script>
    const data = JSON.parse(document.getElementById("workbench-data").textContent || "{}");

    const nf = new Intl.NumberFormat("en-US", { maximumFractionDigits: 2, minimumFractionDigits: 0 });
    const pct = new Intl.NumberFormat("en-US", { maximumFractionDigits: 1, minimumFractionDigits: 1 });

    function toMs(raw) {
      if (typeof raw !== "string" || raw.length === 0) return null;
      const m = raw.trim().match(/^([0-9]+(?:\.[0-9]+)?)(ms|s)$/i);
      if (!m) return null;
      const n = Number(m[1]);
      if (!Number.isFinite(n)) return null;
      return m[2].toLowerCase() === "s" ? n * 1000 : n;
    }

    function severityClass(value) {
      const v = String(value || "").toUpperCase();
      if (v === "HIGH" || v === "CRITICAL") return "bad";
      if (v === "MEDIUM") return "warn";
      return "good";
    }

    function coverage(entry) {
      const t = Number(entry?.targetRps || 0);
      const r = Number(entry?.requestsPerSec || 0);
      if (!(t > 0)) return null;
      return (r / t) * 100;
    }

    function putSummary() {
      const analysisSummary = data?.analysis?.summary || {};
      const totals = data?.runs?.totals || {};
      const stats = [
        { k: "Impl Runs Passed", v: String(totals.passed ?? 0), cls: "good" },
        { k: "Impl Runs Failed", v: String(totals.failed ?? 0), cls: totals.failed > 0 ? "bad" : "good" },
        { k: "Endpoints", v: String(analysisSummary.endpointCount ?? 0), cls: "good" },
        { k: "Highest Severity", v: String(analysisSummary.highestSeverity || "UNKNOWN"), cls: severityClass(analysisSummary.highestSeverity) }
      ];
      const root = document.getElementById("summary");
      root.innerHTML = stats.map((s) => `
        <article class="stat">
          <div class="label">${s.k}</div>
          <div class="value ${s.cls}">${s.v}</div>
        </article>
      `).join("");
    }

    function putMeta() {
      const generatedUtc = data?.generatedUtc || "-";
      const range = `${data?.runs?.startedAt || "-"} -> ${data?.runs?.finishedAt || "-"}`;
      const impls = (data?.runs?.runs || []).map((r) => r.impl).join(", ");
      const nodes = [
        ["Generated UTC", generatedUtc],
        ["Run Window", range],
        ["Implementations", impls || "-"]
      ];
      const root = document.getElementById("meta");
      root.innerHTML = nodes.map(([k, v]) => `<span><strong>${k}:</strong> <code>${v}</code></span>`).join("");
    }

    function putWins() {
      const counts = new Map();
      (data?.compare?.endpoints || []).forEach((endpoint) => {
        const leader = endpoint?.leader?.impl;
        if (!leader) return;
        counts.set(leader, (counts.get(leader) || 0) + 1);
      });
      const rows = [...counts.entries()].sort((a, b) => b[1] - a[1]);
      const root = document.getElementById("wins");
      root.innerHTML = rows.map(([impl, wins]) => `
        <article class="impl-win">
          <div class="name">${impl}</div>
          <div class="wins">${wins}</div>
        </article>
      `).join("");
    }

    function putEndpoints() {
      const findingsByEndpoint = new Map(
        (data?.analysis?.endpoints || []).map((e) => [e.endpoint, e.findings || []])
      );
      const root = document.getElementById("endpoints");
      root.innerHTML = (data?.compare?.endpoints || []).map((endpoint) => {
        const compared = endpoint?.compared || [];
        const endpointFindings = findingsByEndpoint.get(endpoint?.endpoint) || [];
        const rows = compared.map((entry, idx) => {
          const cov = coverage(entry);
          const covText = cov == null ? "-" : `${pct.format(cov)}%`;
          return `
            <article class="row ${idx === 0 ? "top" : ""}">
              <div class="cell">
                <div class="k">Implementation</div>
                <div class="v impl">${entry.impl || "-"}</div>
              </div>
              <div class="cell">
                <div class="k">Req/s</div>
                <div class="v">${nf.format(Number(entry.requestsPerSec || 0))}</div>
              </div>
              <div class="cell">
                <div class="k">P99</div>
                <div class="v">${entry.p99 || "-"}</div>
              </div>
              <div class="cell">
                <div class="k">Coverage</div>
                <div class="v">${covText}</div>
              </div>
              <div class="cell">
                <div class="k">RSS KB</div>
                <div class="v">${entry.rssKb == null ? "-" : nf.format(Number(entry.rssKb))}</div>
              </div>
            </article>
          `;
        }).join("");
        const findings = endpointFindings.map((f) => `
          <article class="finding ${String(f.severity || "").toLowerCase()}">
            <strong>[${f.severity || "INFO"}]</strong>${f.id || ""}: ${f.message || ""}
          </article>
        `).join("");
        return `
          <section class="endpoint">
            <header class="endpoint-head">
              <div class="endpoint-title">${endpoint.endpoint || "-"}</div>
              <div class="chips">
                <span class="chip">leader: ${endpoint?.leader?.impl || "-"}</span>
                <span class="chip">target: ${endpoint?.leader?.targetRps ?? "-"}</span>
                <span class="chip">generator: ${endpoint?.leader?.loadGenerator || "-"}</span>
                <span class="chip">constant-rate: ${endpoint?.leader?.constantRate === true ? "true" : "false"}</span>
              </div>
            </header>
            <div class="rows">${rows}</div>
            ${findings ? `<div class="findings">${findings}</div>` : ""}
          </section>
        `;
      }).join("");
    }

    putMeta();
    putSummary();
    putWins();
    putEndpoints();
  </script>
</body>
</html>
HTML_TAIL
} > "$out_path"

echo "wrote $out_path"
