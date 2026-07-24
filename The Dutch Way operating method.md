<img src="https://r2cdn.perplexity.ai/pplx-full-logo-primary-dark%402x.png" style="height:64px;margin-right:32px"/>

# The Dutch Way

## Operating Method

**Status:** Founding draft — v0.1
**Governing document:** *The Dutch Way — Constitution v0.1*
**Purpose:** A domain-agnostic method for turning a need, opportunity, failure, or idea into a maintained and accountable system outcome.

> **The Constitution defines what must govern. The Operating Method defines how work proceeds.**

This method adapts the Dutch pattern of broad exploration, explicit decision moments, integrated planning, adaptive delivery, and lifecycle management. Its source logic is MIRT-style funneling from exploration to preferred decision, project decision, and realisation; it is not a requirement to make every task heavy or slow.[^1]

***

## 1. The universal route

Every Level 1–3 task follows this route at a degree proportionate to its consequence:

$$
\text{Signal}
\rightarrow
\text{Map}
\rightarrow
\text{Define}
\rightarrow
\text{Coordinate}
\rightarrow
\text{Calculate}
\rightarrow
\text{Choose}
\rightarrow
\text{Design}
\rightarrow
\text{Validate}
\rightarrow
\text{Commit}
\rightarrow
\text{Execute}
\rightarrow
\text{Operate}
\rightarrow
\text{Maintain}
\rightarrow
\text{Learn}
$$

For a small reversible task, this can occur in minutes and fit on a single page. For a critical system, each stage becomes a formal project artefact, decision gate, and recorded body of evidence.

The route is deliberately circular rather than terminal:

$$
\text{Learn}
\rightarrow
\text{Updated map, standards, assets, and future decisions}
$$

A completed project must strengthen the Nation’s ability to make the next project better.

***

## 2. Before starting

Before work begins, classify the work.


| Level | Use when | Required treatment |
| :-- | :-- | :-- |
| **0 — Free exploration** | Thought, creative inquiry, informal discussion, reversible tinkering | Capture useful insight; no gate or formal plan |
| **1 — Reversible task** | Bounded, low-cost work with no material dependency or maintenance burden | Define goal, owner, constraints, and done condition |
| **2 — Operational project** | Work affecting assets, time, money, shared systems, dependencies, or future maintenance | Run the full method in compact form |
| **3 — Critical system** | High consequence, high cost, irreversible work, sensitive data, external parties, safety exposure, or long-lived operational impact | Formal evidence base, governance, risk register, decision gates, operating model, and review |

If classification is unclear, select the higher level until evidence justifies a lighter approach.

Do not confuse urgency with a lower level. A rushed project may require *more* discipline because it carries a higher chance of hidden planning debt.

***

## 3. Phase 0 — Signal

### Purpose

Capture the initiating condition without prematurely deciding the solution.

A signal may be:

- A user need or system failure
- An opportunity
- A recurring cost or friction
- An observed risk
- A maintenance finding
- A legal, technical, or operational change
- A request from the Owner
- A new strategic possibility
- An insight from research or the corpus


### Required output

Write a one-sentence signal statement:

> “A condition exists that may require action because…”

Then state:

- Who noticed it
- Who is affected
- What is known versus assumed
- Whether there is an immediate risk or deadline
- The provisional project level


### Rule

A signal is **not** a solution request.

“Build an app,” “hire a contractor,” “buy a tool,” or “make a dashboard” should be translated back into the underlying condition before action proceeds.

***

## 4. Phase 1 — Map

### Verkenning: understand the terrain

### Purpose

Discover the actual system before choosing an intervention.

The mapping phase asks not “How do we build the requested thing?” but “What is really happening, where does it sit, what causes it, and what else does it affect?”

### Map the following

| Dimension | Questions |
| :-- | :-- |
| **Purpose** | What outcome is the system trying to create, protect, or restore? |
| **Problem** | What is visibly wrong, missing, blocked, costly, unsafe, or inefficient? |
| **Root condition** | What produces the problem underneath the immediate symptom? |
| **System boundary** | What is inside the project, adjacent to it, and explicitly outside it? |
| **Users and stakeholders** | Who uses, funds, operates, maintains, depends on, or is affected by it? |
| **Assets** | What already exists: data, documents, tools, skills, code, infrastructure, contracts, knowledge, or relationships? |
| **Dependencies** | What must be available, stable, approved, or completed first? What depends on this result later? |
| **Constraints** | What limits exist: money, time, attention, law, privacy, safety, access, competence, technical reality, or physical conditions? |
| **Failure modes** | How could this fail, create harm, become expensive, or become hard to reverse? |
| **Future demand** | What foreseeable future work will touch this same system, interface, site, data, or capability? |

### Required output

Create a **Terrain Map** containing:

- Problem statement
- System boundary
- Stakeholder and owner map
- Existing-asset inventory
- Dependency map
- Constraint list
- Unknowns and assumptions
- Initial opportunities for reuse, prevention, or reserve capacity


### Exit condition

The team can explain the problem without naming a preferred solution.

### Dutch Way test

If the problem is “traffic congestion,” do not begin with “add lanes.” Identify movement patterns, underlying demand, adjacent infrastructure, incentives, alternatives, and capacity constraints first. The PARKING report uses this exact logic: a congestion problem may be better solved through cycling connections, employer arrangements, and traffic-flow measures than through road widening.[^1]

***

## 5. Phase 2 — Define

### Make the assignment real

### Purpose

Convert the mapped condition into a clear and governable assignment.

### Define

- **Purpose:** the durable outcome sought
- **Scope:** what the project will and will not do
- **Success criteria:** observable conditions that prove the purpose is met
- **Failure criteria:** conditions that mean the approach has failed or must pause
- **Owner:** who holds the purpose, risk tolerance, and final authority
- **Asset Manager:** who coordinates the plan, interfaces, record, and lifecycle
- **Delivery responsibility:** who executes the work
- **Operator / maintainer:** who takes responsibility after handover
- **Project level:** 0–3, with rationale
- **Time horizon:** immediate, current cycle, annual, multi-year, or strategic horizon
- **Decision rights:** what can be decided autonomously and what requires escalation


### Required output

A **Project Charter** no longer than necessary, but sufficient for another competent contributor to understand:

> What is being done, why it matters, for whom, within what boundary, by whom, and how success will be recognised.

### Rule

If no one can own the completed asset, the project is not ready to build.

***

## 6. Phase 3 — Coordinate

### Put the parking before the building

### Purpose

Ensure that every relevant system, contributor, and future interface can work together before implementation hardens decisions.

### Coordination actions

- Identify active programmes and projects that overlap in assets, people, data, budget, timing, or technical interfaces.
- Identify opportunities to reuse existing work instead of rebuilding it.
- Identify conflicts, duplicated effort, competing demands, and required sequence.
- Bring needed expertise and affected parties into the process early.
- Decide what shared definitions, data structures, standards, formats, or interfaces are required.
- Reserve future connection points where justified.
- Record agreed responsibilities and handovers.


### Required output

A **Coordination Plan** containing:

- Interface register
- Dependency sequence
- Stakeholder consultation need
- Shared standards and source-of-truth location
- Reuse opportunities
- Known clashes and resolutions
- Future extension hooks or reserve capacity
- Named escalation route for unresolved conflict


### Rule

No important interface may remain “someone else’s problem.”

The RWS Blueprint uses a single validated road for module traffic and requires new modules to declare identity, purpose, data categories, event namespace, and schema before admission. That is the digital form of coordinated infrastructure rather than accidental connections.[^2]

***

## 7. Phase 4 — Calculate

### Make the whole-system case

### Purpose

Compare realistic paths before commitment.

The calculation is not a ritual for producing a spreadsheet. It is the explicit reasoning that makes a decision shareable, reviewable, and correctable.

### Minimum options

For Level 2 and 3 work, assess at least:

1. **Do nothing / accept the condition**
2. **Minimal intervention**
3. **Preferred intervention**
4. **Alternative or phased intervention**
5. **Defer, investigate, or pilot first**

### Calculate for each credible option

| Dimension | What to assess |
| :-- | :-- |
| **Purpose value** | How well does it solve the actual need? |
| **Reliability** | Will it remain workable and understandable? |
| **Resilience** | What happens under failure, change, or disruption? |
| **Economic viability** | Direct cost, value, avoided cost, and systemic return |
| **Quality / elegance** | Clarity, coherence, durability, and fit with the wider system |
| **Speed** | Time to value, critical deadlines, and cost of delay |
| **Maintenance** | Operating work, skill requirements, support, inspection, and replacement |
| **Risk and reversibility** | What can go wrong, who bears it, and how difficult is recovery? |
| **Dependencies** | What it enables, blocks, duplicates, or destabilises |
| **Optionality** | Future choices preserved or destroyed |
| **Protected considerations** | Safety, legality, ethics, privacy, autonomy, dignity, and externalities |
| **Confidence** | Quality of evidence, uncertainty, and assumptions |

### Required output

A **Decision Calculation** with:

- Options considered
- Evidence and assumptions
- Comparison against constitutional priorities
- Recommendation
- Explicit unknowns
- Required resources and maintenance burden
- Consequences of delay or inaction


### Rule

Do not invent precision. Use ranges, confidence labels, scenario descriptions, and professional judgment where exact measurement is unavailable.

A decision is sufficiently calculated when a competent reviewer can see the reasoning path and challenge a specific assumption rather than merely disagreeing with a conclusion.

***

## 8. Gate 1 — Preferred Decision

### Voorkeursbeslissing

### Purpose

Select the preferred path before detailed design or significant commitment.

### Decision outcomes

- Proceed to design
- Proceed with a limited pilot
- Merge with another programme
- Defer pending evidence, capacity, or timing
- Return to mapping because the problem is not understood
- Reject or archive the initiative


### Gate questions

- Does the preferred option serve the actual purpose?
- Is it stronger than doing nothing, a smaller intervention, or a phased route?
- Does it protect long-term reliability and recoverability?
- Is its economic model credible at system level?
- Is maintenance accepted and owned?
- Are material dependencies coordinated?
- Are any constitutional conditions unresolved?
- Is the project level still correct?


### Required record

The **Preferred Decision Record** states:

- Option selected
- Why it won
- What was rejected and why
- Owner or delegated authority
- Conditions for moving to detailed design
- Review triggers and unresolved uncertainties

This gate follows the MIRT pattern of broad-to-specific funneling: exploration precedes an explicit preferred decision; a choice is not allowed to emerge invisibly through momentum.[^1]

***

## 9. Phase 5 — Design

### Design the whole lifecycle

### Purpose

Transform the selected option into a buildable, operable, maintainable, and recoverable design.

### Design must include

- Functional requirements: what it must do
- Non-functional requirements: reliability, performance, privacy, safety, access, usability, compatibility, and documentation
- System architecture and interfaces
- Delivery plan and sequence
- Resource plan: time, money, attention, tools, compute, materials, external services
- Risk register and mitigations
- Test and acceptance plan
- Operating and maintenance model
- Failure response and recovery plan
- Migration, retirement, or replacement assumptions
- Monitoring and performance signals
- Documentation and handover plan
- Reserve capacity and future-extension hooks where justified


### The parking test

Before building the visible asset, ask:

> “What support, access, maintenance, data, interfaces, permissions, capacity, or future routes must exist so that this can operate without creating avoidable friction later?”

Build or reserve that foundation first where the calculation supports it.

### Required output

A **Lifecycle Design Package** proportionate to the project level.

For a small software utility, this may be a one-page architecture, checklist, and README. For a critical system, it may include technical design, contracts, operations manual, maintenance plan, recovery runbook, and formal validation evidence.

***

## 10. Phase 6 — Validate

### Find clashes before construction

### Purpose

Challenge the design before it becomes expensive to change.

### Validation modes

- Evidence review
- Technical review
- Stakeholder / user review
- Interface or dependency review
- Prototype or proof of concept
- Pilot
- Simulation
- Security, privacy, legal, ethical, or safety review
- Cost and lifecycle review
- Recovery drill or restore test
- Independent red-team or adversarial review where consequence warrants it


### Required questions

- Does the design satisfy the defined success criteria?
- Are all interfaces known, compatible, and owned?
- Are assumptions tested, or merely written down?
- Is failure observable?
- Can the system be repaired or restored?
- Does the maintainer have the tools, access, knowledge, and time required?
- Does the resource plan remain credible?
- Has the project created hidden obligations for another programme?
- Is the proposed reserve capacity useful, justified, and non-speculative?


### Required output

A **Validation Record**:

- What was checked
- By whom
- Evidence obtained
- Defects, clashes, or uncertainties found
- Required corrections
- Residual risk accepted
- Whether the design is ready for commitment


### Rule

Validation failure is not project failure. It is a successful early discovery.

The architecture blueprint adopts this directly: schema and registry validation are designed to identify incompatible modules before they can write to the canonical system.[^2]

***

## 11. Gate 2 — Project Decision

### Commit only when it can be carried

### Purpose

Authorize material expenditure, irreversible work, external commitments, or transition to live operation.

### Commitment requires

- Approved preferred decision
- Validated lifecycle design
- Named owner, manager, executor, and maintainer
- Sufficient resources or an explicit phased allocation
- Accepted risk position
- Defined completion and acceptance criteria
- Confirmed dependencies and delivery sequence
- Operating, maintenance, and recovery readiness
- An agreed reporting rhythm
- A clear decision on what will happen if scope, cost, timing, or risk changes


### Decision outcomes

- Commit to delivery
- Commit to pilot only
- Commit in phases with gates
- Return for redesign
- Defer
- Stop


### Required record

The **Project Decision** includes the authorised scope, resource envelope, authority, material risks, dependencies, reporting requirements, and next decision point.

No project becomes real because work has already begun. It becomes real because it has passed a conscious commitment decision.

***

## 12. Phase 7 — Execute

### Deliver with discipline

### Purpose

Build or carry out the approved work without losing the original purpose, system integration, or ability to recover.

### Execution rules

- Work within the authorised scope and resource envelope.
- Maintain the canonical record of material decisions, changes, evidence, and actual resource use.
- Protect interfaces and shared standards.
- Report deviations early.
- Do not conceal defects to preserve appearance or schedule.
- Use planned maintenance or low-disruption windows where practical.
- Do not add features merely because access is convenient; reserve capacity only when the calculation justifies it.
- Escalate before a deviation crosses constitutional boundaries or consumes another party’s authority.


### Progress reporting

Every material project reports:

- Work completed
- Work remaining
- Actual versus expected cost and time
- Risks changed
- New dependencies or blockers
- Quality and validation status
- Decisions required
- Maintenance or operational implications discovered
- Recommendation: continue, modify, pause, or stop


### Change control

A change requires a proportionate recalculation when it materially affects:

- Purpose or scope
- Cost, time, or attention
- Reliability, resilience, or recoverability
- Interfaces and dependencies
- Maintenance burden
- Protected considerations
- Reversibility or external impact

Small, reversible changes may use documented *speling*. Material changes return to the appropriate decision gate.

***

## 13. Phase 8 — Accept and hand over

### From project to beheer

### Purpose

Transfer a completed output into normal management and operation.

A build that is not handed over is a future incident.

### Acceptance requires

- Success criteria demonstrably met
- Remaining limitations and known defects recorded
- Owner and maintainer formally known
- Documentation delivered and accessible
- Monitoring, inspection, or review rhythm active
- Maintenance tasks scheduled or assigned
- Dependencies and access rights verified
- Recovery procedure tested where appropriate
- Budget or capacity for normal operation confirmed
- The asset registered in the Nation’s inventory


### Required output

An **Acceptance and Handover Record** containing:

- What was delivered
- What it does and does not do
- Acceptance evidence
- Owner, operator, and maintainer
- Known limitations and residual risks
- Maintenance schedule
- Recovery and support route
- First review date

The constitutional principle is simple: delivery is not completion; operational handover is completion.

***

## 14. Phase 9 — Operate

### Make reality observable

### Purpose

Ensure that the asset continues to serve its purpose under real conditions.

### Operation includes

- Normal use and service delivery
- Monitoring of meaningful health, quality, cost, and outcome indicators
- Incident response
- User feedback
- Routine reporting
- Dependency monitoring
- Identification of wear, drift, debt, or changing demand
- Ongoing protection of records, access, and knowledge


### What to observe

Choose only signals that lead to action. Examples:

- Reliability and availability
- Actual user value or service outcome
- Cost versus expected benefit
- Response time, error rate, defects, or rework
- Maintenance workload
- Data quality and integrity
- Dependency health
- User friction and complaints
- Security, privacy, safety, or compliance events
- Capacity headroom and future demand

The RWS-inspired principle is that infrastructure is also an information system: measurement is not decoration, but a way to steer maintenance, operations, and intervention.[^2][^1]

***

## 15. Phase 10 — Maintain and renew

### Prevent failure from becoming the maintenance plan

### Purpose

Preserve system value and intervene before deterioration becomes crisis.

### Maintenance types

| Type | Purpose |
| :-- | :-- |
| **Routine maintenance** | Recurring work that keeps the system healthy |
| **Preventive maintenance** | Work performed because evidence predicts future degradation |
| **Corrective maintenance** | Repair after a defect or failure |
| **Adaptive maintenance** | Adjustment to changing conditions, users, technology, law, or interfaces |
| **Renewal / replacement** | Planned major intervention when the asset reaches its lifecycle limit |

### Maintenance method

1. Maintain an asset register and known service condition.
2. Define inspection or review intervals.
3. Reserve sufficient time, money, and attention.
4. Prefer low-disruption maintenance windows where possible.
5. Report unavoidable disruption honestly and early.
6. Record work performed, defects found, and capacity consumed.
7. Feed observed failure patterns back into future design standards.

The PARKING report describes RWS maintenance as proactive, lifecycle-led work, including scheduled night, weekend, and holiday windows to reduce disruption while keeping assets serviceable.[^1]

***

## 16. Phase 11 — Learn

### Convert outcome into national capacity

### Purpose

Close the loop between intent, reality, and future work.

At a meaningful milestone, after delivery, after a failure, or at the end of an operating cycle, conduct an **Outcome Review**.

### Ask

- Did the project achieve its original purpose?
- What actually happened compared with the forecast?
- Which assumptions were correct, weak, or false?
- What was the real total cost, including maintenance and disruption?
- What unexpected benefit or harm appeared?
- Did the chosen level of process fit the consequence?
- Which interfaces worked or failed?
- What debt was created, reduced, or exposed?
- What must change in the asset, project method, doctrine, or field manual?
- What evidence should enter the corpus?


### Required output

An **Outcome and Learning Record**:

- Intended outcome
- Actual outcome
- Forecast versus actual resources
- Deviations and root conditions
- Remaining risks and maintenance obligations
- Reusable assets produced
- Lessons for standards, templates, or future project selection
- Recommendation: continue, improve, replace, retire, or scale

No outcome review is a punishment exercise. It is how the Nation prevents each project from paying tuition for lessons that the next project ignores.

***

## 17. The four decision gates

| Gate | Decision | Minimum evidence | Authority |
| :-- | :-- | :-- | :-- |
| **G0 — Classification** | What level of discipline applies? | Signal, initial consequence assessment | Contributor within mandate |
| **G1 — Preferred Decision** | Which path should be pursued, if any? | Terrain Map and Decision Calculation | Owner or delegated authority |
| **G2 — Project Decision** | May material commitment and delivery begin? | Validated Lifecycle Design Package | Owner or delegated authority |
| **G3 — Acceptance** | May the asset enter normal operation? | Acceptance evidence and handover readiness | Owner, manager, or designated maintainer |

A task may return to an earlier gate whenever new evidence changes the calculation. Returning is not failure. Continuing under a disproven assumption is failure of method.

***

## 18. Autonomy matrix

| Situation | Contributor action |
| :-- | :-- |
| Low-cost, reversible work within mandate | Decide and execute; record outcome proportionately |
| Unclear objective or system boundary | Pause execution; return to Map or Define |
| Minor deviation with clear whole-system benefit | Apply documented *speling* if reversible and within mandate |
| New dependency, meaningful cost increase, or material scope drift | Recalculate; inform the Asset Manager; seek the relevant decision |
| Constitutional conflict, irreversible harm, or unclear authority | Halt progression pending a decision |
| Urgent failure or threat | Stabilise the system within competence; protect assets and people; record facts; escalate with cost-of-delay and recovery calculation |
| Discovery that a project should not continue | State it directly; recommend stop, defer, merge, or redesign |

The desired contributor is neither passive nor reckless: it is autonomous, direct, evidence-aware, and responsible for the consequences of its work.

***

## 19. Compact operating prompt

When asked to perform work under **The Dutch Way**, an AI should internally apply this sequence:

1. Identify the signal, purpose, owner, and project level.
2. Map the system before assuming the requested solution is correct.
3. Identify existing assets, dependencies, constraints, stakeholders, and future maintenance.
4. Propose options, including minimal intervention, phased delivery, deferral, and doing nothing when relevant.
5. Calculate options against constitutional priorities: purpose, reliability, resilience; then economic viability, quality, and speed.
6. Make assumptions, uncertainty, and protected considerations explicit.
7. Select or recommend a path with a clear rationale and decision point.
8. Design lifecycle, interfaces, validation, handover, maintenance, recovery, and learning—not merely initial delivery.
9. Act autonomously within mandate; use bounded *speling* only where justified and recordable.
10. Escalate rather than silently override a constitutional boundary.
11. Record material decisions, deviations, failures, and results so later contributors can reproduce the calculation.
12. Leave the system clearer, more maintainable, and more capable than it was before.

## 20. Closing rule

> **First understand the terrain. Then make the calculation. Then build only what can be owned, operated, maintained, and justified.**

The next document, **The Dutch Way — Field Manual**, should translate this method into reusable templates: the Terrain Map, Project Charter, Decision Calculation, gate records, risk register, lifecycle design, handover record, outcome review, anti-patterns, and cross-domain examples.

<div align="center">⁂</div>

[^1]: PARKING-Plan-Dutch-Infrastructure-Research.pdf

[^2]: RWS-Blueprint-Foundation-Architecture.md

