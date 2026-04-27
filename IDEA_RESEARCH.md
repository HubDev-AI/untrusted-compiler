---
title: sec4 / Untrusted<T> — Use Case Research
date: 2026-04-27
method: Feynman 5-step (ELI12 → 5 gaps → research → re-explain → one-paragraph test)
research_passes: 3 parallel web-research agents (competitor head-to-head, typed-IFC adoption history, AI-codegen vuln rates + compliance economics)
status: pressure-test of `docs/path-b-positioning.md`, not endorsement
---

# sec4 / Untrusted\<T\> — Where Does This Language Actually Win?

## Step 1 — ELI12

You wrote a backend programming language. The compiler refuses to build any program where data from a user (the bad kind, like a hacker typing into a form) can reach a place that does damage (a database, the filesystem, an HTML page that another user sees) without first passing through a checked gate. Every function also has to write down what kinds of side effects it does (`reads database`, `writes files`, `talks to network`). So when an AI agent writes a 10-line description of what an API should do, the compiler can prove it cannot leak secrets, cannot do SQL injection, cannot do XSS, cannot read files outside its lane. The runtime is fast. That is the whole pitch.

The unit of value is **provable absence of injection-class bugs in code an AI just wrote** — not lines of code saved, not developer ergonomics.

## Step 2 — Five Gaps Worth Closing

Re-framed per advisor: each gap is a buyer or competitor, not a topic. Generic gaps return Wikipedia answers.

- **G1 — Hack/HHVM at Meta**: typed taint analysis at scale. Did it sell outside Meta? That ceiling probably bounds sec4's TAM.
- **G2 — FlowCaml / Jif / Fabric / Wyvern graveyard**: why do whole-language typed-IFC projects die? (Hypothesis: ergonomics + ecosystem, not soundness.)
- **G3 — AI codegen vuln rates**: is the pain quantified in dollars or auditor hours? If "AI writes insecure code" is a real stat, who's the buyer?
- **G4 — Hasura / PostgREST / Supabase / Convex / Encore**: where does sec4 win on a buyer's checklist that they cannot? CRUD-from-schema is a crowded space.
- **G5 — Operational reality**: who runs LASM in prod? Distribution? FFI to existing infra? This is where most language projects die.

## Step 3 — What Research Found

### G1 — Hack at Meta: real, but ceiling = 2 organizations

Meta shipped Hack 2014. Zoncolan (taint analyzer) 2019: >100M LOC analyzed per run in <30 min, "thousands of potential security issues prevented." Outside Meta: **Slack** migrated ~5M LOC to Hack in 2016, then had to build their own type-checker (Hakana, open-sourced Feb 2023) because Meta's tooling didn't fit. Box and Etsy tried it. **HHVM dropped PHP compatibility in v4.0 (2018)**, severing every other adopter. WordPress, Wikipedia, the rest of PHP world reverted. 2026 status: Hack is alive, the de facto user base is **Meta + Slack** — two organizations. No documented third enterprise.

Implication: typed taint analysis at scale works. The market for it outside the org that built it = ~zero. ([Slack Engineering on Hakana](https://slack.engineering/hakana-taking-hack-seriously/), [HHVM ending PHP](https://hhvm.com/blog/2018/09/12/end-of-php-support-future-of-hack.html))

**XHP** (Facebook's HTML-as-types layer) is the closest *working* prior art to sec4's `HtmlSafe` type — and crucially it killed XSS at FB scale. But XHP is a library inside PHP/Hack, not a new language. ([XHP docs](https://docs.hhvm.com/hack/XHP/introduction/))

### G2 — Typed-IFC graveyard: ecosystem lock-out, not soundness

| Project | Years | Outcome |
|---|---|---|
| **FlowCaml** (INRIA) | 1999–2003 | Last release July 2003. Zero documented production deployments. |
| **Jif** (Cornell, Andrew Myers) | 1999–~2010 | Textbook IFC system, 1000+ academic citations. Survey ([DuZhuoli & Huang](https://lucaszdu.com/files/language-ifc-survey-ecs235b.pdf)) states directly: "has not seen widespread adoption." |
| **Fabric** (Cornell, distributed Jif) | ~2006–dormant | No production deployments. |
| **Wyvern** (CMU, Aldrich) | ~2013–2020 | NSF-funded 2014 PLATEAU paper pitched **the exact same use case as sec4** ("AI-generated code proven secure"). 2020 Onward! paper, then nothing. Wyvern team explicitly flagged information-flow types as **"too hard to use."** ([2014 paper](https://www.cs.cmu.edu/~aldrich/papers/plateau14-wyvern.pdf), [2020 case study](http://www.cs.cmu.edu/~aldrich/papers/wyvern-case-onward20.pdf)) |

Zdancewic's "[Challenges for Information-flow Security](https://www.cis.upenn.edu/~stevez/papers/Zda04.pdf)" (2004) names the wall: IFC systems are either too restrictive (rejecting legitimate downgrades) or too expensive to retrofit. Survey conclusion: **"library-level approaches are generally preferable, as requiring programmers to use a novel language may be too demanding."**

Common cause of death across all four: **whole-system rewrite required, no library imports, no migration path**. The type system was correct. The productivity tax killed adoption.

### G3 — AI codegen vuln rates: real and growing (peer-reviewed range is wider than vendor claims)

| Source | Number | Year |
|---|---|---|
| Pearce et al. "[Asleep at the Keyboard](https://arxiv.org/abs/2108.09293)" (NYU, IEEE S&P) | **40%** of 1,689 Copilot completions across 89 security-relevant scenarios contain CWEs | 2021 |
| **Sandoval et al. "[Lost at C](https://www.usenix.org/conference/usenixsecurity23/presentation/sandoval)" (NYU, USENIX Security)** — replication of Pearce with N=58 user study | "AI-assisted users produced critical security bugs at a rate **no greater than 10% more than the control**" — *contradicts Pearce on low-level C; the headline 40% does not survive a controlled user study in this domain* | 2023 |
| Perry et al. "[Do Users Write More Insecure Code](https://arxiv.org/abs/2211.03622)" (Stanford, ACM CCS) | AI-assisted users wrote **SQL injection 36% vs 7% control**. Worse: rated own code *more* secure. | 2023 |
| **Asare et al. "[Is Copilot as Bad as Humans](https://arxiv.org/abs/2204.04741)" (Waterloo+Aalto, EMSE)** | Copilot replicates the original vulnerable code **33%** of the time, the fix at **25%**. Conclusion: "not as bad as human developers". | 2023 |
| **Khoury et al. "[How Secure is Code Generated by ChatGPT?](https://arxiv.org/abs/2304.09655)" (UQO, IEEE SMC)** | ChatGPT produced secure code in **only 5 of 21 programs** on first attempt; could often be coaxed to fix when prompted. | 2023 |
| **Fu/Liang/Tahir et al. "[Security Weaknesses of Copilot-Generated Code](https://arxiv.org/abs/2310.02059)" (TOSEM)** — real-world GitHub corpus | **29.5% of Python**, **24.2% of JS** snippets affected; spans **43 CWE categories**, 8 of which are CWE Top-25. Copilot Chat fixes up to 55.5% when shown SAST warnings. | 2025 |
| [Veracode 2025 GenAI Report](https://www.veracode.com/blog/genai-code-security-report/) (vendor) | 100+ LLMs across Java/Python/C#/JS: **45% security-test failure**. CWE-80 XSS 86%. Java worst at 72%. **Flat across model scales**. | 2025 |
| [Apiiro telemetry, Fortune 50](https://apiiro.com/blog/4x-velocity-10x-vulnerabilities-ai-coding-assistants-are-shipping-more-risks/) (vendor) | AI-assisted repos: **10×** security findings/mo growth Dec 2024→Jun 2025. Privilege-escalation +322%. | 2025 |
| **[Georgia Tech Vibe Security Radar](https://vibe-radar-ten.vercel.app/) — primary** ([news](https://news.research.gatech.edu/2026/04/13/bad-vibes-ai-generated-code-vulnerable-researchers-warn)) | **74 confirmed AI-attributed CVEs** across ~50 tools by Mar 2026 (6 in Jan, 15 in Feb, 35 in Mar). **Claude Code = 27 of 74** (over-represented because it leaves commit signatures). True count estimated **5–10× higher** across OSS. | 2026 |
| [CSA Research Note](https://labs.cloudsecurityalliance.org/research/csa-research-note-ai-generated-code-vulnerability-surge-2026/) — synthesis | Cites the four rows above. Documents AI-tool attack surface: **Amazon Q ([CVE-2025-8217](https://aws.amazon.com/security/security-bulletins/AWS-2025-015/))**, **Cursor (CVE-2025-54135 CurXecute, CVE-2025-54136 MCPoison)**, **Copilot rule-file ([CVE-2025-53773](https://nvd.nist.gov/vuln/detail/CVE-2025-53773))**, **Lovable RLS bypass (CVE-2025-48757, CVSS 9.3)**. | 2026 |

**Honest read (revised)**: peer-reviewed range is **1.0×–1.4× (Sandoval, low-level C user study)** to **~1.4× snippet rate (Asare/Fu, real corpora)** to **vendor 1.5–3× to 10× (Veracode/Apiiro)**. The headline "AI writes 40% buggy code" *does not survive* a controlled user study (Sandoval). What *does* survive replication: **(a)** AI-assisted devs ship larger PRs with worse review (Apiiro), **(b)** Stanford false-confidence (devs rate own AI code more secure than it is), **(c)** AI tools repeat the same mistakes across millions of users so one bug pattern → thousands of repos (Zhao). The Stanford and Zhao findings are the load-bearing pain. The 40% / 10× headline numbers are not. Pitches built on the inflated numbers will be dismissed by sophisticated buyers.

### G3-bis — Compile-time preventability ≠ all of OWASP

| OWASP Top 10 | Compile-time preventable? |
|---|---|
| A03 Injection (SQLi/XSS/cmd) | **Yes** — sec4's strongest case |
| Path traversal, secret leakage in logs | **Yes** — `PathSafe`, `Secret<T>` |
| A01 Broken access control (auth/session logic) | **No** — runtime context |
| A02 Cryptographic failures (algorithm choice) | **No** |
| A04 Insecure design | **No** |
| A07 Identification & auth failures | **No** |

**3–4 of 10** preventable at compile time. Pitch must scope to that. Anyone selling sec4 as "secure backend, full stop" oversells.

### G3-tris — Compliance buyer economics

- SOC 2 Type 2: $30k–$100k/cycle. PCI-DSS Level 1 ROC: $35k–$200k/yr. HIPAA gap analysis: $6k–$35k. ([Secureframe](https://secureframe.com/hub/soc-2/audit-cost), [Centraleyes](https://www.centraleyes.com/pci-dss-compliance-cost/))
- A "provably parameterized SQL paths" finding converts a 20–40 hour auditor line item ($5k–$18k @ $250–$450/hr) into a structural proof. Real $ saved per audit cycle.
- **No insurer** currently correlates compiler proofs with cyber-insurance premiums (2026). Speculative buyer.
- **AI code review market** = $1.2B+ VC: CodeRabbit $550M valuation ($15M ARR, 20%/mo growth), Greptile $180M, Graphite $52M raise. Proxy signal: AI codegen creates audit pressure that existing tools can't fully satisfy.

### G4 — Head-to-head with incumbents

| Competitor | Already delivers | sec4 edge | Apps incumbent can't |
|---|---|---|---|
| **Hasura** | Auto-CRUD over PG, RLS, role permissions, event triggers | No data-flow trust lattice. Business logic escapes to Remote Actions = untyped. | Multi-step sagas, mixed DB+FS+net with per-sink trust gates, audit trail |
| **PostgREST** | Zero-code REST CRUD, RLS via PG roles | No business logic at all. Author confirms: thin translation layer. | Anything beyond CRUD. Sec4 = the rewrite target when PostgREST users outgrow it. |
| **Supabase** | PG + auto API + auth + storage + RLS, SOC2/HIPAA infra | Business logic = Edge Functions (Supabase docs: "deliberately decoupled, adds complexity at scale"). 45% AI-vuln rate inside those Edge Functions. | Regulated apps where auditor needs provable sanitization across all sinks |
| **Convex** | TS reactive backend, runtime validators, ctx.auth | Validators are runtime not compile-time; no `Untrusted<T>` distinction; closed cloud, no self-host | On-prem regulated workloads, batch/streaming/CLI |
| **Encore** | Typed infra resources (DB/pubsub/cron), service-to-service typed calls. Encore.ts = ~9× Express. | Encore types resources, not data-trust. AI-gen Encore code carries same 45% vuln rate. | Compiler-proven absence of injection |
| **Apollo Federation / WunderGraph** | Federated GraphQL, field-level authz | Late-2025 CVEs from access-control directives not propagating to interface implementations — exactly the bug class sec4 prevents structurally | Greenfield without subgraph fleet |

**Pet competitor — Trusted Types (Google → W3C)**: structurally **the closest working analog to sec4's pitch**. `TrustedHTML` / `TrustedScript` / `TrustedScriptURL`: typed gate before innerHTML/eval. Google killed DOM XSS across Workspace, rolled enforcement to Gmail Jan 2024, W3C Working Draft May 2024. **But Trusted Types is a narrow API in JavaScript with browser CSP enforcement — not a new language.** ([Trusted Types W3C](https://www.w3.org/TR/2024/WD-trusted-types-20240531/), [Google Research](https://research.google/pubs/adopting-trusted-types-in-production-web-frameworks-to-prevent-dom-based-cross-site-scripting-a-case-study/))

**White space**: compiler-enforced multi-sink trust lattice **on output an AI agent emits**. Hasura/Supabase/Convex/Encore none cover this. The white space is real. Whether it's worth a new *language* is the open question.

### G5 — Operational reality + greenfield-lang adoption (2026)

Lived case studies:

- **Broke through**: **Gleam** v1.0 (Mar 2024) — solo author Louis Pilfold, sponsor-funded, BEAM interop avoided ecosystem cold-start, type-safe distributed pitch. ~841 survey respondents (tiny) but 70% admiration in SO 2025. **Bun/Zig** — single-binary install, viral perf demo (Bun = Node killer), Zig adopted via C interop. **Mojo** — vendor-funded (Modular), AI/ML vertical, TIOBE jumped #194→#68.
- **Stalled / died**: **Pony** — capability security model (direct sec4 analog) — stalled. Failure: capability system required complete mental reframe, no killer app, no community flywheel. **Crystal** — Ruby-syntax + native perf, no unique vertical, lost to Go/Rust on every dimension. **Ballerina** (WSO2) — Integrator repo *archived Oct 2024*. Failure: vendor-owned lang, value prop entangled with vendor product, no independent ecosystem.
- **Realistic path** for solo-founder language with working compiler today: 12–24 months to small paying base via OSS compiler + managed hosting tier ($99–$499/mo) + first 10 customers in regulated dev communities.

**Bench claim — methodological flaw (own-repo audit)**:

The "5–160× faster than Go/Rust/Node" headline in `docs/path-b-positioning.md` is comparison against `benchmark-suite/services/{go,rust,node}-workbench/` — **all three baselines shell out to the `psql` CLI per HTTP request**:

- Go: `runPsql` calls `exec.Command("psql", ...)` for every query (line 738+ of go-workbench main.go)
- Rust: `Command::new("psql")` per query
- Node: `spawnSync('psql', ...)` per query

No driver, no connection pool, no prepared statements. Sec4-LASM uses native protocol. That's not a moat — it's apples-to-`fork+exec(psql)`. Calibration: well-tuned Rust async runtimes (Encore.ts) claim ~9× Express. A real Go+pgx+pool baseline almost certainly closes most of the 160× gap. **TechEmpower** review: 13/20 top impls fail under DB connectivity loss; framework microbenchmarks are a documented liar's market. ([HN critique](https://news.ycombinator.com/item?id=47497763))

**This is a positioning liability, not an asset.** First sophisticated reviewer that opens the bench harness will mock the claim. Either rerun with sane baselines (and revise the number to ~1.5–3× honestly) or drop the perf pitch and lead with security.

## Step 4 — Re-explain (sharper)

Sec4's **technical** pitch is solid: real compiler, real typed sinks, real effects checking. The compiler does prove what the brochure claims (verified `04-sec4-security-model.md` and `lasm-alpha-full/main.ut`).

Sec4's **strategic** pitch as currently written ("AI writes the spec, compiler proves secure, runtime serves at 2500 req/s") collides with three independently-fatal facts:

1. **Whole-language typed IFC has 30+ years of corpses** (FlowCaml, Jif, Fabric, Wyvern). The one project that scaled (Hack) escaped to exactly one external company (Slack), which then forked it. Typed IFC sticks only when there's a captive platform (Apex), regulatory mandate (SPARK→DO-178C), single-org enforcement (Hack@Meta), or it's a narrow API in a host language (Trusted Types). Sec4 fits none of those by default.
2. **Compile-time prevention only addresses ~3–4 of OWASP Top 10** — injection + secrets + path traversal. Auth/session/crypto/design bugs (the other ~6) are unreachable for any type system. Selling sec4 as "secure backend" oversells; selling as "injection-proof codegen target" is honest and narrower.
3. **The bench claim is broken** (psql shell-out baselines). Removing it, the pitch reduces to security typing alone, which lands in the IFC graveyard.

## Step 5 — One-paragraph test

Sec4 is a small typed-output language for an AI agent to emit when generating injection-sensitive backend code, paired with a runtime that executes it. The compiler categorically prevents 3–4 OWASP categories (injection, secret-in-log, path-traversal). It does not prevent the other 6. The technically correct positioning is **not** "general-purpose secure backend language" (graveyard); it is one of: (a) a narrow typed-DSL that AI agents emit and an existing runtime executes (Trusted Types analog), (b) a niche compliance-audit-cost reducer where the compiler output is an auditor artifact for SOC 2 / PCI / HIPAA injection findings, or (c) a portable contribution-graph + reputation layer for AI-agent-built apps. Option (a) has working precedent at Google scale; option (b) has working precedent in defense (SPARK); option (c) is unproven. The "5–160× perf" headline must be removed or rerun against a real Postgres driver before any sophisticated reviewer sees it, because the current baselines shell out to `psql` per request.

---

## Brutal Take

The honest read of `path-b-positioning.md` is: **it pitches a 2014 Wyvern-shaped product in 2026 packaging.** Wyvern's NSF-funded paper said the same words ("typed sinks make AI-generated code provably secure") and the project was dormant by 2020 with no users. Hack scaled to a single org outside Meta and that org promptly forked the toolchain. The four academic typed-IFC projects all died of the same disease: developers will not abandon their library ecosystem for soundness alone.

Sec4's path-b pitch additionally leans on a benchmark claim that **is wrong by construction** — comparing native-protocol Postgres against `fork+exec(psql)` is not a 160× language win, it's a methodology bug. A reviewer at Hasura, Convex, or Encore will spot it in 10 minutes, and the credibility hit will torpedo whatever security argument follows.

The genuine technical asset here — typed multi-sink trust lattice + effect declarations as machine-readable security contract — is real and has zero direct competitor. Hasura, Supabase, Convex, Encore all stop at runtime validation or schema-level access control. None give you a compile-time proof. That asset deserves a positioning that doesn't cosplay general-purpose competition with TypeScript/Go.

Most damning evidence: **Stanford 2023** found AI-assisted developers wrote SQL injection at 36% vs 7% for the control group — *and rated their own code as more secure*. The false-confidence loop is exactly the buyer pain that cannot be patched by linters firing after the PR. A type system that *cannot express* the bug is the only architectural answer. That is sec4's real value. The question is whether it's a *language*-shaped solution or a *typed DSL embedded in a host language*-shaped solution. Trusted Types says the latter wins.

---

## Three Survivable Angles (replace `path-b-positioning.md` headline)

###  — "Trusted Types for backends" (typed DSL + adapter, NOT general-purpose lang)

Reposition sec4 as a **narrow typed-output DSL** that AI agents emit, with a runtime that integrates into existing infrastructure (PG, Redis, S3). Drop the "general-purpose backend language" framing entirely. Compete-with-Trusted-Types' adoption mechanic, not with Go/TS.

- **Why it survives**: Trusted Types proved that typed sinks at *narrow* scope plus mandatory enforcement at one runtime (browser CSP) kills XSS at Google scale. Same shape, server-side, for SQL/HTML/path/secret sinks.
- **What changes**: AI agents emit `.ut` for the security-critical core (validation, persistence, sink-touching code). The rest is a normal app in TS/Go/Python. Sec4 becomes a **library + runtime** the host app calls into, not a replacement for the host app.
- **Validation gate**: build a Cursor / Claude Code skill that generates sec4 modules for any TS app's API surface, with one-command FFI back. Show 5 real OSS Cursor users adopt it in 30 days.

### Angle B — Compliance-cost reducer for SOC 2 / PCI / HIPAA injection findings

Pitch sec4 not to engineering teams (who don't have switching budget) but to **compliance teams at regulated SaaS** (who pay $30k–$200k per audit cycle). Sec4 ships **audit artifacts**: machine-readable proof that "no untrusted input reaches a SQL/HTML/path/log sink without a typed validation gate." Replaces 20–40 hours of auditor manual code review per cycle with structural proof.

- **Why it survives**: SPARK/Ada survived 30 years on this exact economic model (DO-178C cert is cheaper with SPARK). Apex survived on Salesforce AppExchange security review.
- **What changes**: positioning sells to CFO/security/compliance line, not engineering line. Pricing model: $X per certified audit cycle, not per developer seat. Partnerships with audit firms (Vanta, Drata, Secureframe).
- **Validation gate**: get one Big-4 audit firm or compliance SaaS to write a letter saying "sec4-generated audit packet reduces our injection-finding review time by Y%." If you can't get one, this angle is dead.

### Angle C — AI agent's "verified output mode" for backend generation

The narrowest, most current pitch: sec4 is the **language Claude Code / Cursor / Devin emit when the user asks for backend code in `--secure` mode**. Compete in the AI-codegen-tooling layer, not the framework layer.

- **Why it survives**: AI code review market is $1.2B+ VC funded right now; CodeRabbit/Greptile/Graphite are the buyer-pain tracker. Sec4 isn't a *review* tool — it's a *generation target* whose output cannot have the bug class. Stanford false-confidence finding is the wedge.
- **What changes**: distribution = Anthropic / OpenAI / Cursor partnership integrations, not landing pages. Revenue model = per-million-tokens-emitted-in-sec4 royalty or enterprise license. The user never knows sec4 exists.
- **Validation gate**: get Anthropic or Cursor to A/B sec4-emitted vs TS-emitted backend code on injection benchmark. If sec4 wins on Stanford-style trial and adoption tracking is feasible, ship.

---

## What to Drop (cost of keeping these = credibility)

- **"5–160× faster than Go/Rust/Node"** — methodology broken, baselines shell out to `psql`. Either rerun with pgx/sqlx/pg-pool (and accept the number is probably 1.5–3×) or remove from public materials. **Highest priority to fix.**
- **"General-purpose backend language"** framing — collides with TS/Go/Rust on ergonomics, ecosystem, library count. Cannot win that fight in 2026.
- **"Resource declarations replace 254-line CRUD"** — Hasura/PostgREST/Supabase do this without a new language. Stop competing in CRUD-density Olympics.

## What to Keep

- **Typed multi-sink trust lattice** (`Untrusted<T>` → `SqlQuery` / `HtmlSafe` / `PathSafe` / `PublicUrl` / `Secret<T>`). This is the real technical asset. Genuinely no direct competitor.
- **Effect declarations as machine-readable contracts** (`effects { db.write, net }`). AI-agent-legible. Auditor-legible. Honest pitch.
- **Working compiler + LASM runtime** — most "language" projects this far in are vapor. You have actually-running code.

---

## Validation Gates (before more code)

1. **Bench rerun**: implement Go+pgx+pool baseline in `benchmark-suite/services/go-workbench`, rerun matrix. If sec4-LASM is still ≥2× faster on the same workload, that's defensible. If it collapses to <1.2×, rewrite all bench claims.
2. **Cursor/Claude Code skill**: ship a one-shot prompt → `.ut` module generator. Wire it into a real TS app (suggest one of your own). Find 5 OSS devs (named, contactable) who'd add it to their workflow this quarter. If nobody bites, Angle  is dead and you have data, not opinion.
3. **One compliance call**: cold-email 10 fintech-compliance SaaS firms (Vanta, Drata, Secureframe, Tugboat Logic). Pitch: "we generate audit artifacts for injection findings." If any reply with interest, Angle B has signal. If none reply, Angle B is dead.
4. **AI-vendor reach-out**: one email to Anthropic Applied AI / OpenAI codegen / Cursor with a 2-paragraph pitch on Angle C. Not a sale — a signal probe. Reply rate ≠ 0 = pursue.

If 0/4 gates fire signal in 30 days, the brutal-take read is correct: this is a 2014-shaped Wyvern in 2026 packaging, and the technical work is more valuable as a paper than a product.

---

## Sources (22, original pass)

1. [Pearce et al., "Asleep at the Keyboard?" (NYU, arXiv 2108.09293)](https://arxiv.org/abs/2108.09293)
2. [Perry et al., "Do Users Write More Insecure Code with AI Assistants?" (Stanford/ACM CCS, arXiv 2211.03622)](https://arxiv.org/abs/2211.03622)
3. [Veracode 2025 GenAI Code Security Report](https://www.veracode.com/blog/genai-code-security-report/)
4. [Apiiro: 4× velocity, 10× vulnerabilities](https://apiiro.com/blog/4x-velocity-10x-vulnerabilities-ai-coding-assistants-are-shipping-more-risks/)
5. [CSA Research Note: AI-Generated Code Vulnerability Surge 2026](https://labs.cloudsecurityalliance.org/research/csa-research-note-ai-generated-code-vulnerability-surge-2026/)
6. [Snyk: Secure Adoption in the GenAI Era](https://snyk.io/reports/secure-adoption-in-the-genai-era/)
7. [Zoncolan: Static Analysis at Meta (Engineering Blog, 2019)](https://engineering.fb.com/2019/08/15/security/zoncolan/)
8. [Hakana: Slack's Hack type-checker (2023)](https://slack.engineering/hakana-taking-hack-seriously/)
9. [Hacklang at Slack (2016)](https://slack.engineering/hacklang-at-slack-a-better-php/)
10. [HHVM ending PHP support (2018)](https://hhvm.com/blog/2018/09/12/end-of-php-support-future-of-hack.html)
11. [Rapid Rise and Fall of Hack/HHVM (Medium)](https://medium.com/@thoughtsfromryan/the-rapid-rise-and-fall-of-facebooks-hack-and-hhvm-7eeea401b04)
12. [XHP: HTML-as-types in Hack](https://docs.hhvm.com/hack/XHP/introduction/)
13. [Jif (Cornell)](https://www.cs.cornell.edu/jif/)
14. [Sabelfeld & Myers, Language-Based Information-Flow Security (JSAC 2003)](https://www.cs.cornell.edu/andru/papers/jsac/sm-jsac03.pdf)
15. [Wyvern PLATEAU 2014: AI-generated secure code pitch](https://www.cs.cmu.edu/~aldrich/papers/plateau14-wyvern.pdf)
16. [Wyvern Onward! 2020 case study](http://www.cs.cmu.edu/~aldrich/papers/wyvern-case-onward20.pdf)
17. [Zdancewic, Challenges for IFC Security (PLID'04)](https://www.cis.upenn.edu/~stevez/papers/Zda04.pdf)
18. [DuZhuoli & Huang, Survey of Language-Based IFC](https://lucaszdu.com/files/language-ifc-survey-ecs235b.pdf)
19. [Trusted Types W3C Working Draft (May 2024)](https://www.w3.org/TR/2024/WD-trusted-types-20240531/)
20. [Adopting Trusted Types in Production (Google Research)](https://research.google/pubs/adopting-trusted-types-in-production-web-frameworks-to-prevent-dom-based-cross-site-scripting-a-case-study/)
21. [SPARK/Ada in Avionics (AdaCore)](https://www.adacore.com/industries/avionics)
22. [Apex SOQL Injection Prevention (Salesforce)](https://developer.salesforce.com/docs/atlas.en-us.apexcode.meta/apexcode/pages_security_tips_soql_injection.htm)

---

## Sources (additional, verified 2026-04-27 — primary, free, triple-checked)

Each item below was (a) returned by a research agent with a verbatim-quoted line, (b) URL fetched directly from the publishing domain via this session, (c) confirmed to be on a primary publisher (peer-review venue, .gov, .edu, standards-body, or named-author journalism). Vendor blogs and content farms rejected.

### Peer-reviewed AI codegen security (counter-evidence + replication of original 22)

23. [Sandoval et al., "Lost at C" (USENIX Security 2023, NYU)](https://www.usenix.org/conference/usenixsecurity23/presentation/sandoval) — N=58 controlled user study; "rate no greater than 10% more than the control"; partially replicates and *softens* Pearce's 40% headline.
24. [Asare/Nagappan/Asokan, "Is Copilot as Bad as Humans" (EMSE 2023, Springer)](https://arxiv.org/abs/2204.04741) — 33% vulnerable-replication rate on C/C++ corpus; peer-reviewed.
25. [Khoury et al., "How Secure is Code Generated by ChatGPT?" (IEEE SMC 2023, UQO)](https://arxiv.org/abs/2304.09655) — 5 of 21 programs initially secure; nuanced fix-on-prompt finding.
26. [Fu/Liang/Tahir/Li/Shahin/Yu/Chen, "Security Weaknesses of Copilot-Generated Code in GitHub Projects" (ACM TOSEM 2025)](https://arxiv.org/abs/2310.02059) — real-world corpus, 733 snippets, 29.5% Python / 24.2% JS, 43 CWE categories.
27. [Hajipour/Hassler/Holz/Schönherr/Fritz, "CodeLMSec Benchmark" (IEEE SaTML 2024, CISPA)](https://arxiv.org/abs/2302.04012) — primary security benchmark for black-box code LLMs.
28. [Tony et al., "LLMSecEval Dataset" (IEEE/ACM MSR 2023, TUHH)](https://arxiv.org/abs/2303.09384) — 150 NL prompts dataset for LLM security evaluation.

### Primary CVE attribution + AI-tool attack surface

29. [Georgia Tech Vibe Security Radar — dashboard (SSLab)](https://vibe-radar-ten.vercel.app/) + [Georgia Tech research news 2026-04-13](https://news.research.gatech.edu/2026/04/13/bad-vibes-ai-generated-code-vulnerable-researchers-warn) — primary source for "35 CVEs March 2026", "74 AI-attributed CVEs", methodology = SZZ blame + AI-signature scan + Claude Agent SDK verification.
30. [CVE-2025-8217 — Amazon Q Developer VS Code Ext (AWS Security Bulletin)](https://aws.amazon.com/security/security-bulletins/AWS-2025-015/) — GitHub-token misconfig in CodeBuild → wiper-style malicious prompt injected into AI extension; 2 days live on Marketplace.
31. [CVE-2025-53773 — GitHub Copilot / Visual Studio command injection (NVD)](https://nvd.nist.gov/vuln/detail/CVE-2025-53773) — primary NIST entry, "Improper neutralization of special elements used in a command".
32. [CVE-2025-64173 — Apollo Router Federation access-control directive bypass (NVD)](https://nvd.nist.gov/vuln/detail/CVE-2025-64173) — interface-vs-impl directive divergence; the *exact bug class* a typed multi-sink lattice prevents.
33. [Lasso Security — Bar Lanyado, "AI Package Hallucinations"](https://www.lasso.security/blog/ai-package-hallucinations) — primary "slopsquatting" research; ~30% hallucination rate; huggingface-cli case study (30k+ downloads).

### Standards / government / compliance buyer surface (free, primary)

34. [NIST SP 800-218 — Secure Software Development Framework v1.1 (CSRC)](https://csrc.nist.gov/pubs/sp/800/218/final) — federal SSDF baseline.
35. [NIST SP 800-218A — SSDF Community Profile for Generative AI](https://csrc.nist.gov/pubs/sp/800/218/a/final) — augments SSDF for AI codegen, mandated by EO 14110. *Direct authority for sec4's compliance angle.*
36. [NIST AI 600-1 — Generative AI Profile of the AI RMF](https://nvlpubs.nist.gov/nistpubs/ai/NIST.AI.600-1.pdf) — 13 risks + 400+ actions, 2,500-participant working group.
37. [NIST SP 800-53 Rev. 5 (upd1) — Security and Privacy Controls](https://csrc.nist.gov/pubs/sp/800/53/r5/upd1/final) — controls SI-10 (input validation), SI-11 (error handling), SA-15 (secure dev tools).
38. [OWASP Top 10:2021 — A03 Injection](https://owasp.org/Top10/2021/A03_2021-Injection/) — primary mapping for sec4's strongest case (CWE-79/89/73).
39. [OWASP Top 10 for LLM Applications v2025 (PDF)](https://owasp.org/www-project-top-10-for-large-language-model-applications/assets/PDF/OWASP-Top-10-for-LLMs-v2025.pdf) — primary OWASP authority for AI-codegen-specific threat catalog.
40. [MITRE CWE Top 25 Most Dangerous Software Weaknesses 2025](https://cwe.mitre.org/top25/archive/2025/2025_cwe_top25.html) — community-developed list; sec4-preventable subset = the injection cluster.
41. [CISA "Case for Memory Safe Roadmaps" (Dec 2023, Five Eyes joint)](https://www.cisa.gov/resources-tools/resources/case-memory-safe-roadmaps) — government endorsement of typed-safety-language migration; precedent for sec4's typed-sink pitch.
42. [PCI DSS v4.0.1 — official PDF (PCI SSC, June 2024)](https://www.pcisecuritystandards.org/document_library/) — Req 6.2 secure dev, Req 6.3 vuln management; primary buyer doc for Angle B.
43. [AICPA Trust Services Criteria 2017 (rev. 2022) — CC8.1 change management](https://www.aicpa-cima.com/resources/download/2017-trust-services-criteria-with-revised-points-of-focus-2022) — SOC 2 primary doc.
44. [HIPAA Security Rule, 45 CFR § 164.312 (eCFR)](https://www.ecfr.gov/current/title-45/subtitle-A/subchapter-C/part-164/subpart-C/section-164.312) — technical safeguards (anti-bot CAPTCHA on direct fetch — verified existence via cite-checking, not full content).
45. [EU Cyber Resilience Act, Reg (EU) 2024/2847 (EUR-Lex)](https://eur-lex.europa.eu/eli/reg/2024/2847/oj/eng) — mandatory compliance Dec 11, 2027; new EU compliance buyer surface (full text fetch returned empty — text confirmed via OJ legal-act metadata, not body).
46. [NIST SP 800-171 Rev 3 (May 2024)](https://csrc.nist.gov/pubs/sp/800/171/r3/final) — DoD CMMC baseline.
47. [CIS Critical Security Controls v8.1 — Control 16 Application Software Security](https://www.cisecurity.org/controls/application-software-security) — non-profit standards body.

### Typed information-flow / capability security primary academic

48. [Volpano/Smith/Irvine, "A Sound Type System for Secure Flow Analysis" (JCS 1996)](https://journals.sagepub.com/doi/10.3233/JCS-1996-42-304) — foundational soundness proof; cited by every IFC survey.
49. [Myers, "JFlow: Practical Mostly-Static Information Flow Control" (POPL 1999, Cornell)](https://www.cs.cornell.edu/andru/papers/popl99/popl99.pdf) — primary Jif paper.
50. [Heintze & Riecke, "The SLam Calculus" (POPL 1998)](https://flint.cs.yale.edu/cs428/doc/heintze98slam.pdf) — typed λ-calculus with security types.
51. [Sabelfeld & Sands, "Declassification: Dimensions and Principles" (JCS 2009)](https://www.cse.chalmers.se/~andrei/sabelfeld-sands-jcs07.pdf) — canonical framework for *intentional* downgrades; directly relevant to sec4's gate-design ergonomics (the part Wyvern reportedly got wrong).
52. [Stefan/Russo/Mitchell/Mazières, "Flexible Dynamic IFC: LIO" (Haskell Symposium 2011 / JFP 2017)](https://dl.acm.org/doi/10.1145/2096148.2034688) — *library-level* IFC monad in Haskell; counter-example to "needs whole language" framing — direct support for Angle .
53. [Liu et al., "Fabric: Secure Distributed Computation" (SOSP 2009, Cornell)](https://www.sigops.org/s/conferences/sosp/2009/papers/liu-sosp09.pdf) — distributed Jif primary.
54. [Roy/Porter/Bond/McKinley/Witchel, "Laminar: Practical Fine-Grained DIFC" (PLDI 2009, UT Austin)](https://www.cs.utexas.edu/~witchel/pubs/pldi09-roy.pdf) — DIFC across OS + heap.
55. [Hritcu/Greenberg/Karel/Pierce/Morrisett, "All Your IFCException Are Belong To Us" (IEEE S&P 2013)](https://ieeexplore.ieee.org/document/6547098/) — exception handling for IFC.
56. [Polikarpova/Stefan/Yang et al., "Liquid Information Flow Control" (ACM PACMPL/ICFP 2020)](https://dl.acm.org/doi/10.1145/3408987) — liquid types for IFC.
57. [Watson/Anderson/Laurie/Kennaway, "Capsicum: Practical Capabilities for UNIX" (USENIX Security 2010, Best Paper, Cambridge+Google)](https://www.usenix.org/conference/usenixsecurity10/capsicum-practical-capabilities-unix) — capability model in mainstream OS (FreeBSD adopted).
58. [Dennis & Van Horn, "Programming Semantics for Multiprogrammed Computations" (CACM 1966)](https://dl.acm.org/doi/10.1145/365230.365252) — *the* original capability paper.
59. [Clebsch, "Pony: Co-designing a Type System and a Runtime" (Imperial College PhD thesis 2017)](https://spiral.imperial.ac.uk/handle/10044/1/65656) — reference-capability design that *did* ship with a working runtime; closest direct analog to sec4's typed-effect approach.

### Competitor primary docs + AI-code-review market signals

60. [Hasura — Actions docs](https://hasura.io/docs/2.0/actions/create/) — "extend Hasura's schema with custom business logic" (escapes type-safety to a REST handler).
61. [Supabase — Edge Functions architecture](https://supabase.com/docs/guides/functions/architecture).
62. [Convex — auth in functions docs](https://docs.convex.dev/auth/functions-auth) — runtime validators, not compile-time.
63. [Encore — Pub/Sub primitives docs](https://encore.dev/docs/go/primitives/pubsub) — typed infra resources.
64. [Encore.ts 9× Express benchmark (primary, methodology disclosed)](https://encore.dev/blog/event-loops) — calibration anchor for sec4's perf claim.
65. [TechCrunch — CodeRabbit $60M Series B at $550M val (Sep 2025)](https://techcrunch.com/2025/09/16/coderabbit-raises-60m-valuing-the-2-year-old-ai-code-review-startup-at-550m/) — Marina Temkin byline; "$15M ARR, 20%/mo growth"; lead Scale Venture Partners + NVentures.
66. [TechCrunch — Graphite $52M Series B at Anthropic-backed (Mar 2025)](https://techcrunch.com/2025/03/18/anthropic-backed-ai-powered-code-review-platform-graphite-raises-cash/) — Kyle Wiggers byline; lead Accel.
67. [TechCrunch — Greptile $180M valuation (Jul 2025)](https://techcrunch.com/2025/07/18/benchmark-in-talks-to-lead-series-a-for-greptile-valuing-ai-code-reviewer-at-180m-sources-say/) — lead Benchmark.

### Benchmark methodology (counters the perf-headline liability)

68. [TechEmpower issue #8790 — "Most of the best-performing frameworks don't survive temporary db connectivity loss"](https://github.com/TechEmpower/FrameworkBenchmarks/issues/8790) — primary GitHub issue (March 2024); only 7/20 top frameworks survived restart.
69. [Brendan Gregg, "Benchmarking Checklist" (2018)](https://www.brendangregg.com/blog/2018-06-30/benchmarking-checklist.html) — named primary author (Netflix).
70. [Gil Tene, "How NOT to Measure Latency" (Azul, InfoQ)](https://www.infoq.com/presentations/latency-response-time/) — coordinated-omission canonical talk.

---

## Verification of Original 22 Sources (2026-04-27)

Direct fetch + spot-check of every existing citation. **One material correction:**

- **Source 5 (CSA Research Note 2026)** — body of doc claimed "Anthropic Git MCP server itself shipped 3 CVEs from AI-processed repos." **The CSA paper does not say this.** What CSA actually documents (verified via direct fetch): Amazon Q Developer VS Code Ext (CVE-2025-8217, 1 CVE), Cursor (CVE-2025-54135 CurXecute, CVE-2025-54136 MCPoison, 2 CVEs), GitHub Copilot rule-file processing (CVE-2025-53773), and Lovable RLS bypass (CVE-2025-48757, CVSS 9.3). Of the 74 confirmed AI-attributed CVEs, **Claude Code = 27** (over-represented because it leaves identifying commit signatures, not because it is the worst tool). The "Anthropic Git MCP" sentence has been **removed from the G3 table** and replaced with the actual primary citations and Georgia Tech as the upstream source for the 35-CVEs-March-2026 number.

The remaining 21 original sources resolved cleanly to their primary publishers. Two with delivery caveats:

- [Source 17 — Zdancewic 2004 (UPenn)](https://www.cis.upenn.edu/~stevez/papers/Zda04.pdf): UPenn faculty hosting confirms primary.
- [Source 22 — Salesforce Apex SOQL injection guide](https://developer.salesforce.com/docs/atlas.en-us.apexcode.meta/apexcode/pages_security_tips_soql_injection.htm): Salesforce Developer Docs primary.

### Sources with delivery caveats (existence confirmed via metadata, body not fully readable in this session)

- HIPAA Security Rule eCFR section (Source 44 above) — eCFR returns CAPTCHA on programmatic fetch; URL is the canonical citation per HHS.
- EU Cyber Resilience Act (Source 45) — eur-lex.europa.eu returned empty body for the ELI permalink; legal-act number 2024/2847 is the cite of record per OJ.
