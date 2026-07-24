<img src="https://r2cdn.perplexity.ai/pplx-full-logo-primary-dark%402x.png" style="height:64px;margin-right:32px"/>

# The Dutch Way

## Corpus Protocol

**Status:** Founding draft — v0.1
**Governing documents:** *Constitution v0.1*, *Operating Method v0.1*, and *Field Manual v0.1*
**Function:** To govern how the Nation collects, preserves, evaluates, derives, retrieves, corrects, uses, and retires knowledge.

> **The corpus is not a pile of documents. It is maintained infrastructure for evidence, memory, decisions, and learning.**

The Protocol applies to project records, source material, research, conversations, data, code, measurements, AI outputs, decisions, incidents, operating records, and derived doctrine. It translates the Blueprint’s canonical-log principles—append-only history, mandatory provenance, raw-data primacy, visible failures, and controlled sensitive scope—into a general knowledge-governance system.[^1]

***

## 1. Purpose

The corpus exists so that the Nation can:

- Preserve evidence and decision history
- Distinguish facts, claims, observations, inferences, and opinions
- Reconstruct why a decision was made
- Reuse knowledge without losing context
- Identify uncertainty, contradiction, and missing evidence
- Improve systems through actual results rather than memory or confidence
- Prevent hidden dependency on a single person, chat, application, or vendor
- Support reliable human and AI reasoning without treating unverified text as truth
- Retain only knowledge whose value justifies its cost, sensitivity, and maintenance burden

The corpus does **not** exist to collect everything. It exists to retain what the Nation can lawfully, ethically, securely, and usefully govern.

***

## 2. The corpus model

The corpus has six layers. They must remain distinguishable even when stored in the same technical environment.


| Layer | Contents | Primary rule |
| :-- | :-- | :-- |
| **1. Raw record** | Original documents, source files, transcripts, images, measurements, logs, messages, data exports | Preserve the original; do not silently alter it |
| **2. Source record** | Metadata describing origin, rights, date, creator, acquisition method, and integrity | Every retained source has an identity and provenance |
| **3. Extracted material** | Searchable text, OCR, structured fields, quotations within permitted use, parsed data | Extraction is derived, not the original |
| **4. Claims and evidence** | Atomic assertions, supporting and contradicting evidence, confidence, scope, and status | A claim is never identical to its source |
| **5. Derived knowledge** | Summaries, maps, models, decisions, lessons, doctrine, recommendations, analysis | Derivations must point back to their basis |
| **6. Operational use** | Retrieval systems, AI prompts, dashboards, reports, models, workflows, decisions | Use must respect rights, scope, confidence, and access rules |

A searchable summary is useful. A preserved source is authoritative. A recommendation is neither raw fact nor permanent doctrine unless it passes the appropriate decision process.

***

## 3. Core principles

### Raw is preserved

Original material remains available in its acquired form whenever lawful and practical. Corrections, annotations, clean copies, OCR, summaries, and translations are new linked records, not replacements.

### Provenance is mandatory

Every corpus object must answer:

- Where did this come from?
- Who created or supplied it?
- When was it created, published, observed, or acquired?
- How did it enter the corpus?
- What transformations were performed?
- What rights, restrictions, and sensitivity apply?
- What supports, contradicts, or supersedes it?

The Blueprint applies this same rule to every event: the system must distinguish self-report, system measurement, derived information, external ingestion, and system-generated records.[^1]

### Derivation is visible

A summary, claim, model, recommendation, or AI answer must identify the sources and transformations from which it was created.

No output may become “known” merely because it has been repeated often.

### Uncertainty remains attached

The corpus must retain uncertainty, disagreement, contested terms, incomplete evidence, and confidence limits. Cleaning away ambiguity creates a false system, not a reliable one.

### Failure is knowledge

Failed experiments, abandoned options, incidents, negative results, missing data, contradictory evidence, and rejected decisions are corpus assets where their retention is proportionate and lawful.

### Sensitivity is controlled at the boundary

A corpus must not become a diary, surveillance system, or uncontrolled archive by accident. New sensitive categories require explicit admission and appropriate authority; they are never added merely because data collection is technically possible.[^1]

### Retrieval is not authority

A retrieved item is a candidate for inspection. It does not automatically establish truth, permission, relevance, or a decision.

***

## 4. Authority and responsibilities

| Role | Corpus responsibility |
| :-- | :-- |
| **Asset Owner** | Defines corpus purpose, allowed categories, risk tolerance, sensitive-scope boundaries, access model, retention principles, and approval of material exceptions |
| **Corpus Asset Manager** | Maintains schemas, identifiers, registries, quality rules, storage, backups, lifecycle reviews, access controls, and auditability |
| **Source Steward** | Registers a source, confirms its provenance and rights, classifies it, and records its acquisition context |
| **Claim Steward** | Maintains important claims, their evidence links, scope, status, confidence, and review date |
| **Contributor** | Supplies material honestly, labels uncertainty, declares source and transformation, and does not silently alter canonical records |
| **AI Service Provider** | May retrieve, extract, classify, summarise, compare, draft, and flag uncertainty within its mandate; it must not silently admit sensitive material, erase records, or upgrade an inference into a fact |
| **Reviewer / Auditor** | Tests provenance, accuracy, rights, controls, consistency, and recoverability at a level proportionate to consequence |

No contributor may be both the unreviewed source of a high-consequence claim and the sole authority that certifies it as established.

***

## 5. Admission rules

Every incoming object passes through an admission decision.

### Admission outcomes

| Outcome | Meaning |
| :-- | :-- |
| **Admit as raw source** | Preserve original and create Source Record |
| **Admit as derived record** | Store the transformed material with full source linkage |
| **Admit with restricted access** | Retain only within an approved sensitivity and access boundary |
| **Quarantine** | Preserve separately while rights, integrity, safety, or relevance are assessed |
| **Reference only** | Store metadata and external link, but not the content |
| **Reject** | Do not retain content; preserve only a minimal rejection record if useful and lawful |
| **Delete / do not ingest** | Required where retention is not lawful, authorised, necessary, or proportionate |

### Admission questions

1. Does the object serve a legitimate corpus purpose?
2. Is the origin known or honestly marked unknown?
3. Is storage permitted by law, contract, consent, licence, confidentiality, and ethical responsibility?
4. Does it contain sensitive, personal, confidential, copyrighted, or security-relevant material?
5. Is full retention necessary, or is metadata, a reference, or a narrow extract sufficient?
6. What is the expected value, maintenance cost, and risk of keeping it?
7. What access classification and retention rule apply?
8. Can it be linked to a stable identifier and provenance chain?
9. Does it require human review before retrieval or use?
10. Is it contaminated, malicious, unreliable, duplicate, or materially incomplete?

The default is not “collect first, decide later.” The default is **purpose before ingestion**.

***

## 6. Source Record

Every admitted source receives a stable Source ID and a Source Record.

```markdown
# Source Record

## Identity
- Source ID:
- Title / description:
- Source type:
- Version / edition:
- Language:
- Format:
- Hash / integrity reference, where practical:

## Origin
- Creator / publisher / supplying party:
- Original creation or publication date:
- Acquisition date:
- Acquisition method:
- Original location / URL / repository:
- Custody notes:

## Rights and use
- Rights holder:
- Licence / permission / contractual restriction:
- Permitted internal use:
- Permitted external use:
- Attribution requirement:
- Copyright / quotation restriction:
- Retention limitation:

## Classification
- Reliability tier:
- Sensitivity class:
- Access class:
- Subject tags:
- Project / programme links:
- Geographic / temporal scope:

## Integrity and quality
- Original available?:
- Complete / partial / excerpted:
- OCR / transcription quality:
- Known alterations:
- Duplicate / related source IDs:
- Contradictions or limitations:

## Lifecycle
- Source Steward:
- Admission decision:
- Retention rule:
- Review date:
- Retirement / deletion condition:
```


### Stable identifiers

A Source ID never changes and is never reused. Titles, tags, descriptions, and classifications may evolve through versioned records.

Example format:

```text
SRC-2026-000184
```

A claim, decision, summary, embedding, or answer must reference the Source ID rather than relying only on a title or file path.

***

## 7. Reliability tiers

Reliability is contextual. It measures the evidential strength of an object for a particular claim, not its general prestige.


| Tier | Description | Typical examples |
| :-- | :-- | :-- |
| **A — Primary / authoritative** | Direct record, original measurement, official instrument, authenticated firsthand evidence | Signed contract, source dataset, official legal text, system log, original research data |
| **B — Strong secondary** | Competent analysis with clear method and citations | Systematic review, audited report, well-documented technical analysis |
| **C — Informative secondary** | Useful interpretation or reporting with limited verification | Journalism, practitioner analysis, unsystematic review |
| **D — Unverified / contextual** | Leads, opinions, recollections, unattributed content, informal discussion | Chat messages, social posts, unverified personal reports |
| **E — Synthetic / derived** | AI output, summary, translation, extraction, model output, internal synthesis | Draft memo, OCR text, AI answer, embedding, auto-tagging result |

Tier E material may be valuable but cannot independently establish a high-consequence fact. It must lead back to inspectable evidence.

A Tier A source may still be weak for a claim outside its scope. An official road-maintenance report may be authoritative about its maintenance programme but not about general human behaviour or AI architecture.

***

## 8. Sensitivity and access

### Sensitivity classes

| Class | Meaning | Default handling |
| :-- | :-- | :-- |
| **S0 — Public** | Suitable for public retention and sharing | Normal access |
| **S1 — Internal** | Non-public operational material | Controlled internal access |
| **S2 — Confidential** | Business, project, contractual, or personally sensitive material | Need-to-know access; secure storage |
| **S3 — Restricted** | High-impact personal, security, legal, financial, medical, authentication, or highly sensitive material | Explicit Owner-approved scope; strict access and retention |
| **S4 — Prohibited / exceptional** | Material that must not normally be retained | Do not ingest unless a separately authorised, lawful protocol exists |

### Access rule

Access must be the minimum necessary to perform a defined role.

AI access must be separately assessed. “The AI can technically retrieve it” is not a sufficient basis for access.

### Sensitive admission rule

No new S3 category is admitted through routine project work or a code change. It requires an explicit Owner decision that states:

- Corpus purpose
- Necessity
- Scope
- Access roles
- Retention period
- Security controls
- Retrieval limitations
- Audit and review requirements
- Deletion or exit condition

This directly follows the Blueprint’s sensitive-scope rule: expanding the data categories held by the canon is a charter-level decision, not an implementation detail.[^1]

***

## 9. The provenance chain

Every transformation creates a new object with a traceable parent chain.

$$
\text{Raw Source}
\rightarrow
\text{Extraction}
\rightarrow
\text{Claim}
\rightarrow
\text{Analysis}
\rightarrow
\text{Decision}
\rightarrow
\text{Outcome}
\rightarrow
\text{Lesson}
$$

### Transformation record

```markdown
# Transformation Record

- Derived object ID:
- Parent object IDs:
- Transformation type:
  - OCR
  - Transcription
  - Translation
  - Parsing
  - Extraction
  - Classification
  - Summary
  - Embedding
  - Analysis
  - AI generation
  - Human editorial synthesis
- Tool / model / method:
- Tool or model version:
- Operator:
- Date:
- Parameters or prompt reference:
- Quality check performed:
- Known limitations:
- Output location:
```


### Rule

No derived object may overwrite, relabel, or silently replace its parents.

If OCR incorrectly reads a date, correct the extracted record; do not alter the scanned original. If an AI summary is wrong, revise the summary and preserve its link to the source and correction history.

***

## 10. Claims and evidence

The corpus should manage material claims as independent, inspectable objects.

```markdown
# Claim Record

## Claim
[One atomic, testable statement]

## Claim ID
[CLM-YYYY-NNNNNN]

## Type
- Fact claim
- Causal claim
- Forecast
- Interpretation
- Recommendation
- Normative position
- Decision premise

## Scope
- Subject:
- Location / system:
- Time period:
- Conditions / qualifiers:

## Status
- Proposed
- Supported
- Contested
- Inconclusive
- Superseded
- Refuted
- Withdrawn

## Evidence
| Source / object ID | Supports / contradicts / contextualises | Relevant passage or data | Reliability for this claim |
|---|---|---|---|
| | | | |

## Confidence
- High / medium / low
- Reason:

## Steward and review
- Claim Steward:
- Last reviewed:
- Next review:
- Superseded by:
```


### Claim rules

- One claim should express one proposition.
- A source may support, contradict, contextualise, or be irrelevant to a claim.
- Claims must preserve material qualifications such as date, population, location, conditions, and uncertainty.
- A decision may rely on a claim while still recording that the claim is uncertain.
- A recommendation must distinguish its factual premises from its value judgment.

This lets the Nation disagree constructively: people may share the same evidence but assign different weights, assumptions, or constitutional trade-offs.

***

## 11. Decisions and outcomes

The corpus must link decisions to evidence and then to real outcomes.

```markdown
# Decision Record

## Decision
- Decision ID:
- Date:
- Authority:
- Decision required:
- Chosen option:
- Rejected options:

## Basis
- Relevant Source IDs:
- Relevant Claim IDs:
- Assumptions:
- Constraints:
- Constitutional priority calculation:
- Uncertainty and accepted risk:

## Conditions
- Compensating controls:
- Review point:
- Expiry condition:
- Reversal / repair path:

## Outcome links
- Project / asset:
- Outcome Review ID:
- Incident IDs:
- Learning Record IDs:
```


### Rule

Do not rewrite a past decision to make it appear wiser after results arrive.

If a decision was reasonable with available evidence but produced a poor outcome, record both facts. If it was poorly reasoned, record that too. Transparent learning is more valuable than retrospective appearance management.

The PARKING research treats transparent decision moments and honesty about unavoidable disruption as operational strengths, not public-relations liabilities.[^2]

***

## 12. AI use protocol

AI may assist the corpus, but it must operate as a service provider under declared constraints.

### Permitted AI activities

Subject to access and project authority, AI may:

- Classify and tag sources
- Extract metadata and structured fields
- Produce search indexes and embeddings
- Identify duplicates or likely relationships
- Create summaries and translations
- Propose claims, evidence links, and contradictions
- Draft maps, calculations, reports, and decision records
- Identify missing provenance, uncertainty, weak evidence, or policy conflicts
- Support retrieval with source-linked answers
- Suggest maintenance, review, or retirement actions


### Prohibited AI actions without explicit authority

AI may not:

- Invent a source, provenance chain, quotation, observation, or decision
- Present a derived answer as raw evidence
- Upgrade a weak claim into an established fact
- Alter or delete raw canonical records
- Admit new sensitive categories
- Expand access permissions
- Train, fine-tune, externally transmit, or publish corpus material beyond explicitly approved rights and scope
- Infer sensitive personal attributes from ordinary data unless that specific activity has been separately authorised
- Conceal uncertainty, source conflict, gaps, or retrieval limitations
- Treat retrieval ranking as evidence quality


### AI output label

Every material AI-derived object must carry:

```markdown
- Derivation: AI-generated / AI-assisted
- Model and version:
- Prompt / task reference:
- Source IDs consulted:
- Retrieval date:
- Human review status:
- Confidence and known limitations:
```


### AI answer rule

When an AI uses the corpus to answer a consequential question, it must state:

- What evidence supports the answer
- Whether evidence is direct or derived
- Relevant uncertainty or disagreement
- Whether a decision, rather than a factual conclusion, remains required
- The source and claim identifiers needed for inspection

The Blueprint’s distinction between raw canon and recomputable projections provides the model: AI views, summaries, and scores are derived and disposable; the traceable record remains the source of authority.[^1]

***

## 13. Retrieval protocol

Retrieval must produce relevant, permitted, inspectable material—not merely fluent output.

### Retrieval sequence

1. Identify the user’s question, project, decision, and consequence level.
2. Determine permitted corpus scope based on the requester’s role and data classification.
3. Retrieve raw and high-quality sources first where available.
4. Retrieve related claims, decisions, outcomes, and known contradictions.
5. Check temporal validity: distinguish historical records from current conditions.
6. Check scope: do not apply a source beyond what it establishes.
7. Provide source-linked synthesis rather than untraceable paraphrase.
8. Record material retrievals used in high-consequence decisions.

### Retrieval order

Prefer:

1. Primary and authoritative material relevant to the precise claim
2. Strong secondary analysis
3. Relevant internal decisions and observed outcomes
4. Lower-tier contextual material, clearly labelled
5. AI-generated or derived content only as navigation or synthesis

### Search result rule

Search ranking indicates probable relevance, not truth.

***

## 14. Contradiction protocol

Contradiction is not corruption. Unrecorded contradiction is.

When sources conflict:

1. Preserve both sources.
2. Create or update the relevant Claim Record.
3. Record the precise proposition that conflicts.
4. Compare scope, date, method, incentives, definitions, and evidential strength.
5. State whether the conflict is real, apparent, unresolved, or caused by different conditions.
6. Do not force a single conclusion where evidence does not justify one.
7. Escalate where a high-consequence decision depends on resolution.
```markdown
# Contradiction Note

- Claim ID:
- Object A:
- Object B:
- Exact point of conflict:
- Scope / date / method comparison:
- Current assessment:
  - Resolved
  - Context-dependent
  - Inconclusive
  - Requires decision
- Next evidence needed:
- Steward:
- Review date:
```


***

## 15. Correction, versioning, and deletion

### Corrections

The corpus is append-only in history, not permanently wrong in use.

- Do not silently alter raw material.
- Create a correction, annotation, replacement, or superseding record.
- Link it to the object corrected.
- Explain the reason and authority.
- Preserve enough history to understand the correction, unless law or safety requires removal.


### Versioning

- Stable IDs remain stable.
- Content versions receive clear version labels.
- Derived objects identify parent version(s).
- A new source edition is a new object linked to prior editions.
- Updated guidance does not erase the doctrine or evidence from which it evolved.


### Deletion and legal removal

Deletion may be required for legal, ethical, contractual, security, or retention reasons.

When deletion is permitted or required:

- Remove the protected content from active and backup storage according to the approved procedure.
- Retain only the minimum lawful deletion record: object ID, date, authority, reason category, and verification status.
- Do not retain sensitive content merely to prove it was deleted.
- Update derived assets that can no longer be supported.
- Mark affected claims or outputs as potentially incomplete.

Append-only history does not override law, dignity, security, or a justified right to removal.

***

## 16. Retention and retirement

Every class of corpus material needs a lifecycle.


| Object type | Default retention approach | Retirement question |
| :-- | :-- | :-- |
| Constitutional and doctrine records | Permanent, versioned | Has it been superseded or amended? |
| Decision records | Retain through decision impact and review horizon | Could future audit, maintenance, or learning require it? |
| Raw project evidence | Retain according to purpose, rights, and operational value | Is it still necessary and permitted? |
| Operational logs | Keep at a defined useful resolution and period | Does diagnostic or legal value still exceed cost and sensitivity? |
| Sensitive records | Shortest justified retention | Is the specific purpose still active? |
| Derived summaries and embeddings | Recomputable; retire when sources, method, or purpose change | Can this be regenerated from retained authorised sources? |
| Temporary working material | Short retention | Has it been incorporated, rejected, or become unnecessary? |

### Retirement outcomes

- Keep active
- Archive cold
- Reclassify
- Reindex or regenerate
- Supersede
- Anonymise or aggregate
- Delete under approved procedure

A corpus with no retirement process becomes an unmanaged landfill. Retention itself creates cost, retrieval noise, privacy risk, and maintenance work.

***

## 17. Quality assurance

The Corpus Asset Manager shall run proportionate checks.

### Admission checks

- Stable ID exists
- Required metadata is present
- Provenance is complete or explicitly unknown
- Rights and sensitivity are classified
- Duplicate status is checked
- Raw and derived objects are distinguished
- Storage location and access control are correct


### Periodic checks

| Rhythm | Check |
| :-- | :-- |
| **Routine** | Ingestion failures, broken links, missing metadata, access anomalies |
| **Monthly** | Duplicate candidates, stale claim reviews, failed transformations, index health |
| **Quarterly** | Rights and sensitive-scope review, provenance sampling, contradiction review, restore test |
| **Annual** | Schema review, retention review, corpus health assessment, access recertification, derived-index regeneration, disaster-recovery exercise |

### Corpus health indicators

- Percentage of material with complete provenance
- Percentage of important claims with direct evidence links
- Number of unresolved high-consequence contradictions
- Retrieval quality and citation coverage
- Duplicate rate
- Stale-source and stale-claim rate
- Sensitive-access exceptions
- Failed ingestion, transformation, or backup events
- Restore-test success
- Cost, storage growth, and maintenance burden

Failures must become visible events. The Blueprint treats sync gaps and system failures as first-class canonical records rather than invisible absence; the corpus follows the same doctrine.[^1]

***

## 18. Minimum technical architecture

The Protocol is technology-neutral, but the following logical separation is mandatory.

```text
Raw Store
    |
    v
Source Registry
    |
    v
Extraction / Transformation Store
    |
    v
Claims, Decisions, Outcomes Registry
    |
    v
Search and Retrieval Index
    |
    v
AI and Human Interfaces
```


### Boundary rules

1. Raw sources are immutable except for lawful removal or technical preservation actions.
2. Derived content must not overwrite raw sources.
3. Search indexes, embeddings, summaries, and model outputs are disposable and regenerable.
4. Every layer uses stable IDs and provenance links.
5. Sensitive material is segregated by access policy rather than merely hidden in a shared folder.
6. Every transformation is attributable to a method, tool, model, or person.
7. Backups are tested through restoration, not assumed to work.

This aligns with the Blueprint’s architecture: raw canonical records remain authoritative, while projections are recomputable views; schema validation catches inconsistency before it reaches the canon.[^1]

***

## 19. Minimum viable corpus

Do not begin by building a large knowledge platform.

For an initial corpus, create only:

- A stable `source_id`
- A Source Record registry
- A raw-source folder or object store
- A clear separation between raw and derived material
- A claim / decision / outcome register
- A rights and sensitivity classification
- A transformation log
- Search that can return source-linked results
- Backups with a tested restore route
- A review cadence

Build advanced retrieval, embedding, graph analysis, multi-agent workflows, dashboards, fine-tuning, and automation only when the basic corpus proves its value and the lifecycle calculation supports expansion.

This is the corpus equivalent of laying reserve pipes rather than building speculative infrastructure: create stable entry points and provenance from day one, then extend only as real use justifies it.[^2][^1]

***

## 20. Corpus templates

### Source intake card

```markdown
# Corpus Intake Card

- Proposed source:
- Why it is needed:
- Project / programme:
- Source type:
- Origin known?:
- Rights / licence known?:
- Sensitivity class:
- Raw file / reference location:
- Required access:
- Expected retention:
- Proposed Source Steward:
- Admission outcome:
```


### AI synthesis record

```markdown
# AI Synthesis Record

- Synthesis ID:
- Task:
- Model / version:
- Date:
- Source IDs consulted:
- Claim IDs consulted:
- Retrieval scope and filters:
- Output location:
- Statements requiring human verification:
- Contradictory evidence surfaced:
- Known limitations:
- Human reviewer:
- Approval status:
```


### Corpus incident record

```markdown
# Corpus Incident Record

- Incident ID:
- Date detected:
- Type:
  - Wrong access
  - Missing provenance
  - Inaccurate extraction
  - Corrupted file
  - Failed ingestion
  - Rights issue
  - Sensitive-data exposure
  - Retrieval hallucination
  - Broken backup / restore
  - Other
- Affected object IDs:
- Immediate containment:
- Root condition:
- Correction / recovery:
- Notification requirement:
- Preventive change:
- Owner:
- Closure evidence:
```


***

## 21. Corpus maxims

1. **A document without provenance is a lead, not evidence.**
2. **Raw is preserved; derived is linked.**
3. **A claim is not its source.**
4. **A retrieval result is not a decision.**
5. **An AI synthesis is not an authority.**
6. **Uncertainty must travel with the statement it qualifies.**
7. **Contradiction is recorded before it is resolved.**
8. **Rights and dignity are admission conditions, not cleanup work.**
9. **Sensitive scope requires explicit authority.**
10. **Failures, gaps, and negative results are knowledge.**
11. **Indexes and embeddings are replaceable; source lineage is not.**
12. **Keep only what can be justified, governed, maintained, and retired.**
13. **A backup is a claim until restoration proves it.**
14. **The corpus must make future reasoning easier, not merely make storage larger.**

***

## 22. Ratification

With this Protocol, **The Dutch Way** has its first complete operating set:


| Document | Role |
| :-- | :-- |
| **Constitution** | What governs: purpose, priorities, authority, boundaries, and values |
| **Operating Method** | How work proceeds: from signal to learning |
| **Field Manual** | What contributors use: templates, registers, gates, and working patterns |
| **Corpus Protocol** | How knowledge is admitted, governed, traced, used, corrected, retained, and retired |

> **First preserve the evidence. Then make the calculation. Then record the decision. Then observe the outcome. Then improve the system.**

<div align="center">⁂</div>

[^1]: RWS-Blueprint-Foundation-Architecture.md

[^2]: PARKING-Plan-Dutch-Infrastructure-Research.pdf

