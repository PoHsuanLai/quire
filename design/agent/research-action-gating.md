<!-- paths: skip -->
<!-- Reference copy of ~/rs-wt/agent-spec/research-action-gating.md for the companion freeze (2026-10-02). It names files of other repos and of crates that do not exist yet, so the path check skips it. -->
# Runtime action-gating for autonomous agents: state of the art, late 2026

Research only, written 2026-10-02. All web pages were fetched or searched on 2026-10-02; arXiv ids
carry their month (2605 = May 2026). "[unverified]" = my recollection or a secondary source I could
not confirm against a primary one. Several arXiv pages were read through a summarising fetcher, so
numbers from them are marked "abstract-level" where I could not see the tables. Compared against:
/home/pohsuanlai/rs-wt/companion/COMPANION.md, Security section (lines 67-102).

## 1. What others have built

### 1.1 Coding-agent products

**Claude Code auto mode** (Anthropic engineering post, Mar 2026; default for Pro/Max/Team from 2026-08-14)
- https://www.anthropic.com/engineering/claude-code-auto-mode ; https://www.helpnetsecurity.com/2026/08/10/anthropic-claude-code-auto-mode/
- Tiers: built-in safe-tool allowlist, then in-project edits, then a transcript classifier for shell, web, external operations.
- Two stages: stage 1 is a single-token yes/no tuned to over-block; stage 2 runs chain-of-thought only on what stage 1 flagged.
- The classifier's input is stripped: assistant prose, tool-call descriptions and tool outputs are removed. It sees user messages plus the executable payload only. Reason given: the agent cannot talk the judge round, and injected tool text never reaches it.
- Separately, a server-side probe screens tool outputs before they enter the agent's context and adds a warning when hostile.
- Deny-and-continue: a block goes back to the agent, which must find another way. After 3 consecutive or 20 total denials it escalates to the human (or ends a headless run).
- Published numbers (Anthropic post): real traffic n=10,000: stage-1 FPR 8.5%, full pipeline FPR 0.4%. Real "overeager" actions n=52: FNR 6.6% at stage 1, 17% for the full pipeline. Synthetic exfiltration n=1,000: FNR 1.8% then 5.7%. The 17% is the headline weakness.
- Secondary (Help Net Security, Aug 2026, not checked against a primary): a 1,053-tester study where humans blocked 13.6% of dangerous commands vs 89% for auto mode; attack miss rate 12% to 7% after improvements; users approve ~97% of prompts without close review; Anthropic stops charging for classifier compute. The same article's cross-vendor injection numbers (e.g. "0% vs 5.83%") are vendor-comparison claims I would not rely on.
- Latency/cost: no figures found; the two-stage design exists to keep the common path to one token.

**OpenAI Codex approval modes, sandbox, Auto-review ("guardian")**
- https://alignment.openai.com/auto-review/ ; https://learn.chatgpt.com/docs/sandboxing/auto-review ; https://developers.openai.com/codex/agent-approvals-security
- Layers: sandbox modes (read-only, workspace-write, danger-full-access) times approval policies (untrusted, on-request, never). Auto-review replaces the human at the sandbox boundary: a separate agent grades each boundary-crossing request (shell escalation, blocked network, out-of-workspace edit, MCP tools needing approval) on user intent, environment, policy and likely impact.
- Reviewer model: GPT-5.4 Thinking at low reasoning effort; OpenAI says stronger models detect risk and intent better.
- Published evals: 99.1% of escalated actions approved; 99.93% effective approval over all actions (about 200x fewer human stops); overeagerness recall 90.3%; prompt-injection recall 99.3%; MonitoringBench-Hard recall 96.1%. No latency or cost numbers. They say plainly it is not a security guarantee and cannot defend against a scheming model.
- Circuit breaker: 3 consecutive denials or 10 in the last 50 reviews halts the turn. On denial the agent is told to find a materially safer path or ask the user; the user can `/approve` a single retry.
- Policy: org-level `guardian_policy_config` in managed settings beats local `[auto_review].policy` (natural-language reviewer policy).
- Search snippet only [unverified in detail]: "Guardian v2" reviews computer-use steps with screenshots.

**Goose (Block, now Linux Foundation AAIF)** adversary mode
- https://goose-docs.ai/blog/2026/03/31/adversary-mode/
- Hidden reviewer agent sees original task, recent conversation and the tool call; returns ALLOW or BLOCK. Rules are plain markdown (`adversary.md`). Same model as the main agent. **Fail-open**: if the reviewer errors, the call goes through. Also a deterministic injection detector and modes Auto / Approve / SmartApprove / Chat.
- Open issue that default Auto approves everything: https://github.com/aaif-goose/goose/issues/12567

**OpenHands** (https://docs.openhands.dev/sdk/guides/security ; https://github.com/OpenHands/software-agent-sdk/issues/4157)
- Analyzers: LLM, GraySwan (external), Pattern, PolicyRail (deterministic). The LLM analyzer reads the risk label the acting model wrote on its own action; a reported flaw is that LOW auto-executes. Default ConfirmRisky threshold is HIGH. Lesson: never let the actor rate itself.

**Gemini CLI** (https://geminicli.com/docs/reference/policy-engine/)
- TOML policy engine: allow / deny / ask_user per tool and argument pattern, scoped by approval mode (yolo, autoEdit, plan). Persistent approvals are mode-aware to stop trust leaking from edit mode into plan mode (PR 23257). Rule `modes` field being made mandatory (issue 24797). No model judge found.

**Cursor / Windsurf = Devin Desktop / GitHub Copilot / Cline**
- Cursor: default "Auto-Run in Sandbox", with allowlist; users report the allowlist is silently ignored in sandbox mode (forum.cursor.com threads, 2026).
- Windsurf (renamed Devin Desktop 2026-06-02): prefix/exact allow-lists, domain allow-list cards; CVE-2025-62353 bypassed its deny-list via injection (https://drel.ai/blog/windsurf-security-review). Lesson: string deny-lists on shell commands are brittle.
- Copilot cloud agent: network firewall on by default, org-managed (https://github.blog/changelog/2026-04-03-organization-firewall-settings-for-copilot-cloud-agent/); workflows from agent PRs need a human with write access.
- Cline: per-category auto-approve (read, edit, command, browser, MCP), plus YOLO. No model judge (https://docs.cline.bot/features/auto-approve).
- Devin (cloud), Jules, Amp: no primary source found on action gating; not covered.

### 1.2 Computer-use and browser agents

- **Claude in Chrome** (https://claude.com/blog/claude-for-chrome ; https://claude.com/blog/claude-in-chrome-generally-available): 123 test cases / 29 attack scenarios gave 23.6% attack success without mitigations; less than 0.08% in the current configuration on their internal set. One classifier screens incoming content, another checks every action before it runs. Site permissions, blocklists, confirmations for downloads and sensitive input. Internal-set numbers only.
- **Claude desktop computer use**: per-app tiers fixed by app category (browsers view-only, terminals/IDEs click-only, others full), app blocklist, action review (https://www.reworked.co/digital-workplace/anthropics-claude-can-now-control-your-mac/, secondary).
- **OpenAI Atlas / ChatGPT agent** (https://openai.com/index/prompt-injections/ ; https://openai.com/safety/prompt-injections/): confirmations before consequential steps, logged-out mode, Watch Mode (agent pauses if the user leaves a sensitive tab), prompt-injection monitoring. OpenAI says injection is "unlikely to ever be fully solved" (https://techcrunch.com/2025/12/22/openai-says-ai-browsers-may-always-be-vulnerable-to-prompt-injection-attacks/).
- **Gemini computer use** (https://ai.google.dev/gemini-api/docs/computer-use ; Gemini 3.5 Flash post): each proposed function call may carry `safety_decision` = regular / require_confirmation / blocked from a Google-side classifier. The host app must prompt the user and return `safety_acknowledgement`; terms forbid bypassing it. Adds "automated task stopping" on detected threats. Notably the host app, not Google, draws the confirmation, which the model cannot press only if the host implements it so.
- **Perplexity Comet / BrowseSafe** (https://www.perplexity.ai/hub/blog/building-safer-ai-browsers-with-browsesafe): fine-tuned Qwen3-30B scans raw HTML for injection before the assistant sees it; open model plus BrowseSafe-Bench. Lasso Security reported a 36% bypass using simple encodings (https://www.lasso.security/blog/red-teaming-browsesafe-perplexity-prompt-injections-risks). This is content classification, not action gating.
- **Windows Agent Workspace / Copilot Actions** (https://learn.microsoft.com/en-us/windows/security/book/operating-system-agentic-security ; https://blogs.windows.com/windowsexperience/2025/10/16/securing-ai-agents-on-windows/): separate standard agent account, separate desktop session, known-folder-only access until the user authorises more, off by default, user can watch and take over. OS-level identity isolation; no published action classifier.
- **Apple** (WWDC26 sessions 343, 240): App Intents confirmation is the final step after parameters are gathered, justified by LLM side-effect risk. A proposed risk-class scheme (always confirm for financial, etc.) is in a NowSecure post, not Apple primary (https://www.nowsecure.com/blog/2026/08/05/...). Apple's design is per-intent declared confirmation, not a judge.

### 1.3 Research

**Judge/monitor family**
- **LlamaFirewall** (Meta, arXiv 2505.03574, MIT for code, Llama licence for models): PromptGuard 2 (86M classifier), AlignmentCheck (LLM audits the agent's reasoning trace against the user goal), CodeShield. AgentDojo: baseline ASR 17.6% / utility 47.7%; PromptGuard alone 7.5% / 47.0%; AlignmentCheck (Llama 4 Maverick) 2.89% / 43.1%; both 1.75%. Note the utility cost of the judge.
- **Agent-as-a-Proxy** (arXiv 2602.05066): an adaptive injection that also targets the monitor beats AlignmentCheck: 42.3% success against Llama-3.1-8B as monitor, 90.7-93.3% against 7B monitors; up to 99% vs PromptGuard 2 + AlignmentCheck. Abstract-level conclusion: model scale and runtime observation alone do not give security; need structural guarantees. Directly relevant because a monitor that sees agent reasoning is attackable through the agent.
- **Adaptive attacks on trusted monitors** (arXiv 2510.09462, Oct 2025) and **The Attacker Moves Second** (arXiv 2510.09023, authors from OpenAI/Anthropic/GDM): adaptive attackers bypass 12 published defences at over 90% success.
- **Ctrl-Z** (Redwood, arXiv 2504.10374): BashBench, 257 tasks. Defer-to-resample plus history analysis cut attack success from 58% to 7% for about 5% usefulness loss. A later finding (search summary, not verified at source) says resampling can amplify injections by turning them into best-of-n attacks.
- **ShieldAgent** (arXiv 2503.22738, ICML 2025): extracts verifiable rules from policy documents into rule circuits; claims +11.3% over prior guards, rule recall 90.1%, 64.7% fewer API calls.
- **AgentSpec** (arXiv 2503.18666, ICSE 2026): DSL of trigger/predicate/enforcement; blocks over 90% of unsafe code-agent executions, all embodied hazards (abstract-level).
- **GuardAgent**, **ToolEmu**, **Llama Guard / ShieldGemma / Granite Guardian**: from memory, [unverified] this round. These are mostly content-safety classifiers or emulators for finding risky tool use, not per-action authorisers on an agent trajectory. OSGuard (arXiv 2606.15034, Jun 2026) is a new computer-use safety benchmark with an action-level evaluation section and compares against WebGuard/ShieldAgent/GuardAgent/SafePred; I could only read its outline, not the numbers.

**Structural / deterministic family**
- **CaMeL** (Google DeepMind 2025): interpreter with capability metadata; public artefact completes all 949 AgentDojo cases at 60.1% utility (via https://arxiv.org/html/2608.22868 summary).
- **FIDES** (Microsoft, arXiv 2505.23643): integrity/confidentiality labels, deterministic policy; with policy checks stops all AgentDojo injections (summary).
- **Progent** (arXiv 2504.11703): LLM drafts a per-task symbolic privilege policy from the user's request; SMT check makes updates narrowing (auto) or expanding (needs approval). AgentDojo ASR 39.9% to 1.0%, ASB 70.3% to 3.9%, utility kept.
- **Conseca** (Google, arXiv 2501.17070): LLM sees only trusted context, emits a just-in-time policy with human-readable rationale; enforcement is deterministic.
- **AgentFlow** (arXiv 2608.22868, Aug 2026): flow-centric policy language; confirmed compromise 33.0% to 0.0% and utility 46.7% to 63.3% on 949 AgentDojo cases (abstract-level; note the baseline here is a weak agent).
- **ARGUS** (arXiv 2605.03378): per-call auditor tracing each state-changing argument to supporting evidence spans and validating against the user query; **AIRGuard** (arXiv 2605.28914, repo github.com/Sophie508/AIRGuard): runtime authority control on normalised tool calls; **AgentTrust** (2605.04785, 2606.08539); **Before the Tool Call** (2603.20953): deterministic pre-action authorisation argued to beat LLM gating. I could not extract numbers for these; treat as direction signals, read before citing.
- **Influence is not authority** (arXiv 2608.29942): causal guardrail signals flag legitimate tool use as attacks; relevant to false-ask rate (abstract-level).
- **Agents Rule of Two** (Meta, 2025-10-31) and the lethal trifecta (Willison, 2025-06-16): an unsupervised agent should hold at most two of untrusted input, sensitive data, external communication (https://simonwillison.net/2025/Nov/2/new-prompt-injection-papers/).

### 1.4 Open-source we could adopt or learn from

| Project | Licence | Lang | What | Activity |
|---|---|---|---|---|
| Cedar (AWS, now CNCF) | Apache-2.0 | Rust | Policy language; used by AgentCore Policy: default deny, forbid-overrides-permit, NL to Cedar (https://aws.amazon.com/blogs/security/why-policy-in-amazon-bedrock-agentcore-chose-cedar-for-securing-agentic-workflows/) | active |
| LlamaFirewall / PurpleLlama | MIT code, Llama licence on models | Python | PromptGuard 2, AlignmentCheck, CodeShield | active; models not OSI-open |
| NVIDIA OpenShell / NemoClaw | Apache-2.0 | Rust+Python [unverified split] | Landlock+seccomp+netns sandbox, out-of-process policy engine (YAML), privacy router; does not stop injection steering the model | active, v0.0.x |
| Invariant Guardrails | Apache-2.0 | Python | Rule language over agent traces, proxy for MCP/LLM | company acquired/changed [unverified] |
| Goose adversary | Apache-2.0 [unverified] | Rust | Reviewer agent, markdown rules, fail-open | active |
| OpenHands SDK analyzers | MIT [unverified] | Python | Pattern, PolicyRail analyzers | active |
| BrowseSafe + Bench | open (licence not checked) | model | HTML injection detector | released Dec 2025 |
| Progent, AgentSpec, CaMeL, AgentFlow, AIRGuard | research licences, not checked | Python | reference implementations | research-grade |

## 2. Comparison table (action gating specifically)

| System | Deterministic rules | Model judge | Judge input | Tool output seen by judge | Human UI | Denial feedback | Fail mode | Published eval |
|---|---|---|---|---|---|---|---|---|
| Claude Code auto | allowlist tiers, protected paths | 2-stage classifier | user msgs + payload | stripped | prompt after 3/20 denials | yes, continue | escalates to human | FPR 0.4%, FNR 17% (n=52) |
| Codex Auto-review | sandbox + policy | GPT-5.4 low | intent, env, policy, impact | not stated | `/approve` retry | yes, "safer path" | halts turn on breaker | recall 90-99%, 99.9% approve |
| Goose adversary | modes | same-model reviewer | task, recent chat, call | not stated | none | block | **fail-open** | none |
| OpenHands | pattern, PolicyRail | self-rated or LLM | action | n/a | confirm HIGH | n/a | defaults weak | none |
| Gemini CLI | TOML policy | none | n/a | n/a | ask_user | deny msg | rule default | none |
| Gemini computer use | n/a | Google classifier per action | action (+screen) | n/a | host app must draw | blocked | host-defined | not published |
| Claude in Chrome | site perms, tiers | input + per-action classifiers | n/a | screened | confirmations | n/a | n/a | 23.6% to <0.08% (internal) |
| Atlas / agent | logged-out, site rules | injection monitor | n/a | n/a | confirm, Watch Mode | n/a | pause | none numeric |
| Windows agent workspace | account + folder ACL | none published | n/a | n/a | approve/take over | n/a | off by default | none |
| LlamaFirewall | none | AlignmentCheck + PG2 | reasoning trace | yes | none | block | n/a | ASR 17.6% to 1.75%, utility -4.6 pts |
| Progent | symbolic per-task policy | LLM drafts policy | user task | no (policy from user) | expansion needs approval | n/a | deny | ASR 39.9% to 1.0% |
| CaMeL / FIDES | interpreter + labels | planner only | user query | quarantined | n/a | n/a | deny | ~0% ASR, 60% utility |
| Our draft | Cedar | auto-mode classifier | trusted requests, action, taint, history | quoted data only | compositor-drawn | yes | Ask | none yet |

## 3. Cutting edge vs what our draft has

Already at or near the frontier in the draft:
- Independent judge with stripped, typed input and untrusted content only as quoted data (matches Claude Code's strongest design choice).
- Three-valued Allow / Deny{reason} / Ask{reason} with denial fed back to the agent (matches Claude Code and Codex; reason strings go beyond both as published).
- Fail-closed to Ask (better than Goose fail-open).
- Compositor-drawn confirmation the agent cannot press: stronger than any product above. Gemini and OpenAI leave the UI to the host; Windows has take-over but not an unforgeable surface.
- Cedar deterministic layer and taint labels (CaMeL/FIDES style) in front of a model judge: the layering the literature converges on.
- Planner / reader split in the existing section.

Genuinely cutting edge elsewhere that the draft lacks, or states only loosely:
1. Two-stage cascade (cheap single-token filter, reasoning on flagged) with published FPR/FNR on real traffic: Claude Code.
2. Circuit breakers on repeated denials (3 consecutive; 20 or 10-of-50 totals): Claude Code, Codex.
3. Per-task privilege derived from the user request and made monotonic (Progent, Conseca): narrowing is automatic, widening needs the human.
4. Honest evaluation: real overeager-action sets, MonitoringBench, and adaptive-attacker tests against the monitor itself.
5. Reasoning/trajectory alignment checks (LlamaFirewall AlignmentCheck, Codex reviewer), but these are the most attackable (Agent-as-a-Proxy).
6. Resampling / defer-to-trusted as an alternative to a hard deny (Ctrl-Z), with its own injection-amplification caveat.
7. OS-level identity separation for the agent (Windows agent account), which the draft's Landlock/bubblewrap story partly covers.

## 4. Gaps and improvements for our pipeline

1. **Judge must be assumed attackable.** Adaptive attacks beat monitors at 40-99% (2602.05066, 2510.09023). Keep the judge's input free of agent prose and tool output, as Claude Code does; do not add trajectory/CoT to its input without measuring. Make the judge's power asymmetric: it can only move Allow toward Ask/Deny; Cedar `forbid` and the human confirmation are never overridable by it. Treat judge Allow as valid only for actions Cedar already marks "judgeable" (reversible, in-scope).
2. **Open-model reality.** The draft commits to open small models. Published judges used Sonnet-class or GPT-5.4. A small local judge will have worse FNR; the 8B monitors in Agent-as-a-Proxy were the weakest. Plan: cascade (tiny classifier then a larger local model on flagged), measure on our own set, and shrink the Allow region until measured FNR is acceptable. Judge latency matters on a desktop; budget it per Space strictness.
3. **Derive a per-task capability envelope from the trusted request** (Progent/Conseca): at task start, a policy-writer model sees only the user's request and emits Cedar-checkable constraints (which folders, recipients, domains, effect classes). Cedar then enforces it deterministically; expansions become Ask. This is the best defence-per-complexity item and fits "scoped expiring authorizations".
4. **Trifecta rule in Cedar.** Encode "untrusted input in context + sensitive data read + outbound channel" as a hard forbid-unless-confirmed, Meta's Rule of Two. The taint labels already enable it; make it an explicit named rule and test it.
5. **Arguments, not just actions.** Windsurf's CVE and Cursor's ignored allowlist show string matching on commands fails. Gate typed, parsed actions (the cua-action crate idea) and validate that each outbound argument (recipient, URL, path) traces to a trusted source or a user-visible quote (ARGUS-style provenance of arguments).
6. **Circuit breaker and anomaly detection.** Adopt 3-consecutive / N-of-50 denial limits that stop the turn and surface to the human; add rate limits per action class, burst detection for mass actions, and a cost cap. Also detect "denial probing" (same goal, many reformulations) as a distinct signal.
7. **Denial feedback design.** Codex tells the agent to find a safer path or stop; Ctrl-Z warns that resampling can amplify injection. Feed back a minimal reason (never the classifier's full reasoning or policy text, to avoid giving an attacker an oracle) and count retries towards the breaker.
8. **Second opinion on high-impact Asks only.** Two-model checks (different families) for actions that would otherwise be Allow in the destructive/outbound classes; disagreement becomes Ask. Do not use same-model review (Goose) for this.
9. **Human-side honesty.** 93-97% of prompts are approved reflexively (Anthropic). So Ask must be rare and specific: show the concrete diff/recipient/amount in the compositor surface, not the agent's description of it, and require a distinct gesture for destructive/financial. Track the Ask rate and approval rate as health metrics.
10. **Expiring authorizations.** Bind them to (Space, task id, action class, parameter pattern), store in the audit log, invalidate on taint changes, and never let the judge mint one.
11. **Ship a red-team suite with the product.** Contents: (a) AgentDojo/ASB-style injection tasks adapted to desktop (mail, web, files, calendar); (b) a real-overeager-actions set collected from our own logs, like Anthropic's n=52; (c) synthetic exfiltration, 1,000+ items; (d) adaptive tests against the judge (GCG-style suffix in a tool result, encoding tricks like the 36% BrowseSafe bypass, multi-step fragmented exfiltration, "approval-lookalike" text in screenshots); (e) benign-task suite measuring false Ask/Deny per Space strictness; (f) confirm-UI spoofing tests (agent-drawn windows, synthetic input via reis/uinput). Report FPR, FNR, p50/p95 latency and cost per 1,000 actions on every release, with confidence intervals (n=52 gives wide ones).
12. **Screenshot channel.** Computer-use judges see pixels; injected text in images bypasses text stripping. Judge the typed action plus the target element's accessibility label, not the screenshot, where possible; if the screen is shown, mark it as untrusted quoted data.

## 5. Open questions

- What FNR can a local open-weight judge reach on our desktop action set, and at what latency? No published number exists for small open judges on computer-use actions (OSGuard may help; I only read its outline).
- Is the 17% FNR on real overeager actions mostly "agent was told vaguely" cases? How should a judge treat user-ambiguous authority ("clean up my downloads")? Anthropic's post discusses this; worth a closer read.
- Do reasoning traces help or hurt? AlignmentCheck lowers ASR but is the most adaptively attackable; Codex's reviewer uses reasoning at "low" effort. Need our own A/B with adaptive attack.
- Is a judge needed at all for Space strictness levels where Cedar + confirmations suffice? Deterministic-only pre-action authorisation (2603.20953) is argued to beat LLM gating on injection robustness but loses on coverage of novel actions.
- Per-task policy writer (item 3): how often does it over-narrow and cause Ask fatigue? Progent reports utility kept; check on desktop tasks.
- Which of ARGUS, AIRGuard, AgentTrust, AgentFlow have reproducible code and real numbers? I did not get past abstracts; read before citing.
- Licensing: LlamaFirewall models are under the Llama licence, which conflicts with an open-source-models-only stance if that means OSI-open; BrowseSafe licence unchecked.
- Not covered for lack of primary sources: Devin cloud, Jules, Amp, Operator internals, GuardAgent, ToolEmu, ShieldGemma, Granite Guardian, NeMo Guardrails (all [unverified] from memory).

<!-- paths: end -->
