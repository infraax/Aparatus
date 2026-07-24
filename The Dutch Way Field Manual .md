<img src="https://r2cdn.perplexity.ai/pplx-full-logo-primary-dark%402x.png" style="height:64px;margin-right:32px"/>

# The Dutch Way

## Field Manual

**Status:** Founding draft — v0.1
**Governing documents:** *Constitution v0.1* and *Operating Method v0.1*
**Function:** Reusable working instruments for applying The Dutch Way to projects, systems, research, construction, software, AI work, organisations, and operations.

> **Use the smallest instrument that makes the work clear, accountable, maintainable, and recoverable.**

The Field Manual converts the Constitution and Operating Method into practical artefacts. It follows the RWS/PARKING logic of explicit responsibilities, transparent decision points, lifecycle cost, early interface coordination, adaptive planning, and proactive maintenance.[^1][^2]

***

## 1. Selecting the toolkit

Start by assigning the project level.


| Level | Typical work | Use these tools |
| :-- | :-- | :-- |
| **0 — Free exploration** | Thought, brainstorming, informal research, reversible experiments | Insight Capture; optional Question Map |
| **1 — Reversible task** | Small analysis, repair, short document, bounded code change | Task Card; Definition of Done; Completion Note |
| **2 — Operational project** | Tool, research programme, renovation element, workflow, software module | Terrain Map; Project Charter; Decision Calculation; Risk Register; Lifecycle Plan; Handover; Outcome Review |
| **3 — Critical system** | Long-lived infrastructure, multi-agent system, sensitive corpus, business-critical workflow, safety or external-impact work | All Level 2 tools plus formal interface register, governance record, validation plan, recovery drill, operating manual, audit trail, and scheduled reviews |

Do not use every template because it exists. Use it because its absence would make a material uncertainty, responsibility, or future burden invisible.

***

## 2. One-page Task Card

Use for **Level 1** work.

```markdown
# Task Card

## Task
[One sentence: what will be delivered]

## Purpose
[Why this matters; what user/system condition it improves]

## Owner
[Who accepts the outcome]

## Constraints
- Time:
- Budget / tools:
- Must not:
- Dependencies:

## Definition of Done
- [Observable completion condition]
- [Quality or validation condition]
- [Where the output is stored / handed over]

## Result
- Delivered:
- Actual time / cost:
- Deviation or learning:
- Follow-up / maintenance:
```


### Example

```markdown
## Task
Create a structured index of all source documents for the Dutch Way corpus.

## Purpose
Make source material searchable and prevent repeated manual discovery.

## Definition of Done
- Every source has a stable ID, title, type, date, author, and location
- Each source has a short scope note and provenance status
- Index is stored in the canonical corpus directory
```

If the task develops dependencies, recurring maintenance, external cost, sensitive data, or an irreversible choice, reclassify it to Level 2.

***

## 3. Terrain Map

Use at the beginning of **Level 2 and 3** work.

```markdown
# Terrain Map

## Signal
[A condition exists that may require action because...]

## Actual problem
[Describe the condition without embedding a solution]

## Purpose
[The durable outcome to create, protect, or restore]

## System boundary
### Inside
- 
### Adjacent
- 
### Outside
- 

## Who is affected
| Party / system | Role | Need or exposure | Consultation required? |
|---|---|---|---|
| | | | |

## Existing assets
| Asset | Current condition | Owner | Reuse / constraint |
|---|---|---|---|
| | | | |

## Dependencies and interfaces
| Dependency / interface | Direction | Owner | Status / risk |
|---|---|---|---|
| | | | |

## Constraints
- Time:
- Money:
- Attention:
- Technical / physical:
- Legal / ethical:
- Privacy / dignity:
- Safety / irreversibility:
- Other:

## Root conditions
[What appears to cause the visible problem?]

## Unknowns and assumptions
| Item | Known / assumed / unknown | Why it matters | How it will be checked |
|---|---|---|---|
| | | | |

## Future access opportunity
[What could be prepared now while the system, site, model, or interface is already open?]

## Initial project level
[0 / 1 / 2 / 3, with reason]
```


### Map before solution

A weak Terrain Map starts with: “We need a dashboard.”

A Dutch Way Terrain Map starts with: “Project status, dependencies, resource commitments, and maintenance obligations are not visible across active programmes; this causes duplicated work, missed handovers, and unplanned Owner attention.”

The first statement prescribes an interface. The second identifies a system condition that may or may not need a dashboard.

***

## 4. Project Charter

Use after the terrain is sufficiently understood.

```markdown
# Project Charter

## Identity
- Project name:
- Programme:
- Project level:
- Start date:
- Target decision / completion date:

## Purpose
[What durable value is this project intended to create?]

## Scope
### Included
- 
### Explicitly excluded
- 

## Success criteria
- 
- 
- 

## Failure / stop criteria
- 
- 
- 

## Authority
| Role | Named person / entity | Responsibility |
|---|---|---|
| Asset Owner | | Purpose, risk tolerance, final authority |
| Asset Manager | | Integrated planning, interfaces, records, lifecycle |
| Delivery lead | | Execution |
| Operator / maintainer | | Normal service, maintenance, recovery |

## Resources
- Time:
- Budget:
- Owner attention:
- Tools / compute / materials:
- External services:

## Key dependencies
- 

## First decision required
[What must be decided next?]
```


### The ownership test

Do not approve a charter if any answer is missing:

- Who can decide when the project must trade cost, quality, or speed?
- Who will operate the asset after delivery?
- Who bears the maintenance burden?
- Where will the authoritative record live?

The RWS Blueprint makes this separation explicit: the Owner holds charter and risk authority, the Core/manager holds coordination and records, and service providers deliver through declared interfaces.[^1]

***

## 5. Decision Calculation

Use before a Preferred Decision or material commitment.

```markdown
# Decision Calculation

## Decision required
[What exactly must be chosen?]

## Non-decision baseline
[What happens if we do nothing or defer?]

## Options
1. Do nothing / accept condition
2. Minimal intervention
3. Preferred intervention
4. Alternative or phased intervention
5. Pilot / gather evidence first

## Comparison
| Dimension | Option 1 | Option 2 | Option 3 | Option 4 | Option 5 |
|---|---|---|---|---|---|
| Purpose value | | | | | |
| Reliability / maintainability | | | | | |
| Resilience / recoverability | | | | | |
| Economic viability / return | | | | | |
| Quality / elegance | | | | | |
| Speed | | | | | |
| Maintenance burden | | | | | |
| Risk / reversibility | | | | | |
| Dependencies and system effects | | | | | |
| Optionality created / lost | | | | | |
| Protected considerations | | | | | |
| Evidence confidence | | | | | |

## Assumptions
- 

## Recommendation
[Preferred path and direct reasoning]

## Conditions
[What must be true before proceeding?]

## Review triggers
[What evidence or event would reopen this decision?]
```


### Practical scoring rule

Numbers are optional. When used, they must reveal assumptions rather than hide them.

Use one of three approaches:

- **Qualitative:** low / medium / high; strong / adequate / weak.
- **Scenario-based:** best case / expected case / adverse case.
- **Range-based:** e.g., 4–8 hours, €50–€100, 1–3 months of maintenance effort.

The Constitution requires shared calculability, not fictional precision.

***

## 6. Lifecycle Cost Register

Use whenever a choice could create recurring cost, dependency, or maintenance work.

```markdown
# Lifecycle Cost Register

| Cost / benefit area | Build / acquisition | Operation | Maintenance | Failure / recovery | Renewal / exit | Evidence / confidence |
|---|---:|---:|---:|---:|---:|---|
| Time | | | | | | |
| Money | | | | | | |
| Owner attention | | | | | | |
| Compute / data / energy | | | | | | |
| Skill / staffing | | | | | | |
| User disruption | | | | | | |
| Dependency / lock-in | | | | | | |
| Risk reduction / avoided cost | | | | | | |
| Reusable asset value | | | | | | |
```


### Rule

The lowest upfront cost is not automatically the lowest total cost.

The source research emphasises Dutch lifecycle-cost management: asset decisions are evaluated across construction, maintenance, renewal, and disruption rather than only on the initial build price.[^2]

***

## 7. Interface and dependency register

Use when more than one system, person, agent, vendor, programme, or technical component must coordinate.

```markdown
# Interface and Dependency Register

| ID | Interface / dependency | Producer | Consumer | Contract / expected behaviour | Failure effect | Validation | Owner | Status |
|---|---|---|---|---|---|---|---|---|
| INT-01 | | | | | | | | |

## Rules
- Every interface has one accountable owner.
- Every dependency has a visible status.
- No dependency remains “assumed to work.”
- A change to a shared interface requires notification and compatibility assessment.
- Undocumented direct access is a sluiproute: a side route that bypasses the intended system road.
```


### Digital example

A research agent writes its findings to a shared canon only through an approved schema and source record; it does not silently alter another project’s private notes or derived conclusions.

### Physical example

Before reopening a floor or trench, map every existing and planned cable, pipe, route, access point, and future requirement; coordinate the work so the same surface does not need to be opened repeatedly.

The PARKING report treats underground planning and BIM-style interface coordination as a way to identify conflicts before execution rather than after assets are physically committed.[^2]

***

## 8. Risk and recovery register

Use for Level 2 and mandatory for Level 3.

```markdown
# Risk and Recovery Register

| ID | Risk / failure mode | Likelihood | Impact | Early signal | Prevention / mitigation | Recovery action | Owner | Residual risk | Review date |
|---|---|---|---|---|---|---|---|---|---|
| R-01 | | Low / Med / High | Low / Med / High | | | | | | |

## Escalation thresholds
- Pause work when:
- Escalate to Owner when:
- Activate recovery when:
- Notify affected parties when:
```


### Good risk entry

| Risk / failure mode | Early signal | Mitigation | Recovery |
| :-- | :-- | :-- | :-- |
| External AI provider changes pricing or access | Notice of policy change; cost rise; degraded API | Avoid provider-specific core logic; keep exportable records; cost threshold alert | Switch provider, reduce usage, or use local fallback |

### Weak risk entry

“Technology might fail.”

A risk is useful only when it identifies a condition, an early signal, a preventive action, and a recovery route.

The RWS Blueprint provides a practical model: it records likelihood, blast radius, and mitigation for risks such as re-sign failure, platform policy changes, widget limitations, homelab loss, and scope creep.[^1]

***

## 9. Reserve-capacity assessment

Use whenever systems, buildings, data models, contracts, workflows, or interfaces are being altered.

```markdown
# Reserve-Capacity Assessment

## Current access opportunity
[What is open, being changed, or easy to modify now?]

## Foreseeable future need
[What plausible work may require this access later?]

## Preparation option
[What conduit, interface, metadata, spare capacity, documentation, or modular boundary could be added now?]

## Cost now
- Money:
- Time:
- Complexity:
- Maintenance:
- Other:

## Expected later cost if omitted
- Rework:
- Disruption:
- Lost opportunity:
- Access difficulty:
- Compatibility / migration cost:

## Decision
- Build reserve capacity now
- Record and defer
- Reject as speculative

## Reasoning
[Why the option is or is not justified]
```


### Examples

- When designing a data model, add versioning, provenance, and module identifiers before data volume makes migration painful.
- When opening a wall, place an empty conduit for credible future wiring or network needs.
- When building an AI workflow, define an API boundary and export format before adding specialised tools.
- When documenting a project, establish stable identifiers before multiple agents begin generating derived materials.

The RWS Blueprint calls these “pipes laid now for 2028”: small fields and interfaces that reduce later rewrite cost without forcing premature feature construction.[^1]

***

## 10. Validation plan

Use before Gate 2: Project Decision.

```markdown
# Validation Plan

## Claims requiring validation
| Claim / requirement | Why it matters | Validation method | Evidence threshold | Owner | Result |
|---|---|---|---|---|---|
| | | Review / test / pilot / prototype / simulation / audit | | | |

## Interface checks
- 
- 

## Acceptance tests
- 
- 

## Recovery checks
- Backup / restore:
- Rollback:
- Failover / fallback:
- Manual operating route:

## Remaining uncertainty
- 

## Decision
- Ready for commitment
- Ready for limited pilot only
- Return to design
- Stop / defer
```


### Rule

Do not mark a design “validated” because it has been discussed. Validate claims through the cheapest credible method: inspection, source check, test, prototype, pilot, simulation, or recovery drill.

A backup that has never been restored is not verified recovery capacity. The RWS Blueprint explicitly schedules restore testing as routine maintenance rather than trusting backups by assumption.[^1]

***

## 11. Change and speling record

Use when work deviates from the plan.

```markdown
# Change / Speling Record

## Change
[What changed from the approved plan?]

## Trigger
[Why did this become necessary or beneficial?]

## Classification
- Minor and reversible
- Material but manageable
- Constitutional / authority issue

## Whole-system calculation
- Cost of acting now:
- Cost of pausing:
- Effect on dependencies:
- Reversibility:
- Maintenance impact:
- Risk if continued:
- Risk if not continued:

## Authority
[Who may decide?]

## Decision
- Proceed within speling
- Modify and continue
- Pause pending decision
- Return to earlier gate

## Record location
[Where evidence and decision are stored]
```


### Meaning of speling

*Speling* is controlled margin, not informal permission. It is appropriate when a small adjustment prevents wider disruption and remains transparent, reversible, and inside delegated authority. It is not appropriate for hidden scope expansion, silent risk transfer, or bypassing the Owner’s reserved decisions.

***

## 12. Handover and operating record

Use when a project becomes an asset.

```markdown
# Handover and Operating Record

## Delivered asset
- Name / ID:
- Purpose:
- Location:
- Version / condition:
- Date accepted:

## Ownership
| Role | Name / entity | Responsibility |
|---|---|---|
| Owner | | Purpose and renewal decision |
| Operator | | Daily use / service |
| Maintainer | | Inspection, repair, updates |
| Support / escalation | | Incident route |

## Operating instructions
- Normal operating mode:
- Inputs / outputs:
- Access requirements:
- Dependencies:
- Known limitations:

## Maintenance
| Activity | Frequency | Owner | Evidence / record |
|---|---|---|---|
| | | | |

## Recovery
- Failure indicators:
- First response:
- Restore / rollback process:
- Last tested:
- Maximum acceptable downtime / disruption:

## Next review date
[Date or condition]
```


***

## 13. Outcome and learning review

Use after meaningful delivery, at planned intervals, or following an incident.

```markdown
# Outcome and Learning Review

## Original purpose
[What was this intended to achieve?]

## Actual outcome
[What happened in reality?]

## Forecast versus actual
| Area | Forecast | Actual | Variance | Why |
|---|---|---|---|---|
| Time | | | | |
| Money | | | | |
| Quality | | | | |
| Reliability | | | | |
| Maintenance | | | | |
| User / system value | | | | |

## What proved correct
- 

## What was wrong, incomplete, or unexpectedly costly
- 

## Failure, incident, or friction record
- 

## Assets created or improved
- 

## Debt created, reduced, or exposed
- 

## Changes required
- Asset / system:
- Operating Method:
- Field Manual:
- Constitution:
- Corpus / evidence needs:

## Next action
- Continue
- Improve
- Scale
- Replace
- Retire
- Archive
```


### Rule

The review is complete only when it changes something: the asset, maintenance plan, standard, template, future decision rule, or evidence base.

***

## 14. Maintenance calendar

Use for any lasting asset.


| Rhythm | Purpose | Typical actions |
| :-- | :-- | :-- |
| **Daily / weekly** | Keep service healthy | Check alerts, failures, queues, backups, usage, or wear indicators |
| **Monthly** | Verify integrity | Review status, dependency changes, costs, access, logs, and maintenance backlog |
| **Quarterly** | Test resilience | Restore test, update risk register, inspect lifecycle condition, revise forecasts |
| **Annual** | Renew strategically | Major upgrades, schema migrations, replacement planning, architecture review, long-range capacity assessment |
| **Event-driven** | Respond to change | Incident, new regulation, supplier change, failure pattern, capacity threshold, user need |

Schedule disruptive maintenance in the least harmful window available. For digital work this may be overnight or low-use periods; for construction it may be coordinated access windows; for a personal research system it may mean preserving uninterrupted deep-work time and scheduling administrative maintenance separately.

This operational-discipline principle is drawn directly from the PARKING account of RWS work being planned into night, weekend, and holiday windows where possible.[^2]

***

## 15. Anti-patterns

These are **not the Dutch Way**.


| Anti-pattern | Why it fails | Corrective move |
| :-- | :-- | :-- |
| **Solution-first thinking** | Treats the requested tool or build as the problem | Return to the Signal and Terrain Map |
| **Lowest-price selection** | Pushes lifecycle cost, fragility, and maintenance burden into the future | Use the Lifecycle Cost Register |
| **Heroic rescue culture** | Rewards emergency intervention instead of reliable planning | Build maintenance, recovery, and margin into the plan |
| **Shadow systems** | Creates undocumented dependencies and incompatible sources of truth | Establish one framework and declared interfaces |
| **Invisible ownership** | Leaves decisions, maintenance, and failure response unclaimed | Name Owner, Manager, Executor, and Maintainer |
| **Maintenance by crisis** | Waits for failure, increasing cost and disruption | Monitor condition and schedule preventive work |
| **Meeting-based coordination** | Relies on memory and goodwill rather than durable interfaces | Use registers, decisions, standards, and shared records |
| **False precision** | Uses unsupported numbers to make a preferred choice look rational | State ranges, assumptions, evidence quality, and uncertainty |
| **Urgency theatre** | Uses time pressure to bypass planning and quality | Calculate the cost of delay versus rushed debt; use speling only within mandate |
| **Silent exception** | Conceals changed scope, risk, or authority | Use a Change / Speling Record and escalate if needed |
| **Build-and-abandon** | Delivers an asset without operator, maintenance, documentation, or recovery | Require Handover before acceptance |
| **Feature accumulation** | Treats every future possibility as a reason to build now | Build justified hooks, not speculative complexity |
| **Learning without record** | Repeats expensive mistakes because insights remain informal | Produce an Outcome and Learning Review |

The RWS Blueprint identifies scope creep—where foundation work consumes the intended instrument—as a primary risk; it answers with thin early milestones, explicit gates, and a permit process for future additions.[^1]

***

## 16. Domain translations

The doctrine remains constant; the assets, signals, and validation methods change.


| Domain | “Terrain” means | “Parking first” means | Maintenance means |
| :-- | :-- | :-- | :-- |
| **Software / AI** | Users, data, models, interfaces, deployment, cost, security, dependencies | Schema, API boundaries, observability, export paths, access control, rollback | Updates, backups, tests, monitoring, dependency review |
| **Research** | Questions, sources, evidence quality, theories, methods, gaps, claims | Source registry, provenance, glossary, dataset structure, citation rules | Literature updates, claim review, replication, corpus hygiene |
| **Construction / renovation** | Existing structure, utilities, access, materials, permits, future use | Routes, conduits, drainage, access, service zones, future connections | Inspection, repair, weather protection, replacement planning |
| **Organisation / team** | Purpose, people, roles, incentives, information flow, customers, finances | Decision rights, onboarding, documentation, shared systems, escalation routes | Reviews, skills, succession, process repair, financial control |
| **Personal system** | Time, health, energy, finances, commitments, tools, relationships | Calendars, buffers, file structure, backups, routines, automation boundaries | Reviews, rest, repairs, updates, debt reduction |
| **Corpus / knowledge system** | Source types, rights, provenance, taxonomy, retrieval, model use | Stable IDs, schemas, raw/derived separation, versioning, access control | Ingestion review, deduplication, quality audit, re-indexing |


***

## 17. Worked mini-example

### Case

A system has several AI-assisted projects, but findings, costs, dependencies, and decisions are spread across chats, folders, and tools.

### Weak response

“Build a project-management dashboard.”

### Dutch Way response

**Signal:** Active programmes lack a shared operational view; this causes repeated research, lost decisions, and unclear maintenance obligations.

**Terrain:** Existing assets include project documents, chats, source files, calendars, and code repositories. Constraints include Owner attention, fragmented formats, privacy, and the risk of building an elaborate management system before the core work is stable.

**Options:**

- Do nothing: low immediate cost; continued fragmentation
- Create a manual project register: low cost; partial coherence
- Build a central MCP/Treasury system: high long-term potential; premature at current maturity
- Establish a simple canonical register plus shared templates: moderate cost; creates reusable foundation
- Pilot the register on two active programmes: low-risk evidence path

**Preferred Decision:** Pilot a minimal canonical project and asset register using stable IDs, decision records, maintenance fields, and a weekly review. Do not build a dashboard or MCP server yet.

**Parking first:** define the data schema, IDs, source-of-truth location, access rules, and export path before creating visual surfaces.

**Acceptance:** the system is accepted when two projects can be understood by another contributor without searching chat history.

This example mirrors the Blueprint’s “one log, one API, one schedule” preference: disciplined simple infrastructure first, rather than premature frameworks or interface complexity.[^1]

***

## 18. Core working phrases

Use direct language. Replace vague project speech with operational statements.


| Avoid | Prefer |
| :-- | :-- |
| “We should maybe look into…” | “The signal is X. I will map the root condition before recommending an intervention.” |
| “This seems like the best option.” | “Option B is preferred because it preserves reliability and recoverability while producing the strongest lifecycle value under the stated assumptions.” |
| “It is urgent.” | “The deadline is X. The cost of delay is Y. The cost of rushing is Z. The required decision is…” |
| “We can fix it later.” | “Deferring this creates maintenance debt of X; owner, repayment condition, and review date are…” |
| “This is complete.” | “Delivery is complete; acceptance requires documentation, named maintenance, recovery path, and operating review.” |
| “There is a risk.” | “Failure mode X has likelihood Y, impact Z, early signal A, mitigation B, and recovery C.” |
| “We need more coordination.” | “Interfaces A, B, and C lack owners and declared contracts; the Coordination Plan will resolve them before commitment.” |


***

## 19. Field rule

> **Do not add procedure to look disciplined. Add only enough structure that the next correct decision becomes easier than the next shortcut.**

The next document should be the **Corpus Protocol**: a source-ingestion and evidence-governance specification that determines how raw material, provenance, disagreements, decisions, results, and derived doctrine are collected without contaminating or overreaching the corpus.

<div align="center">⁂</div>

[^1]: RWS-Blueprint-Foundation-Architecture.md

[^2]: PARKING-Plan-Dutch-Infrastructure-Research.pdf

