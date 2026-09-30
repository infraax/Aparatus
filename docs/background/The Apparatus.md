<img src="https://r2cdn.perplexity.ai/pplx-full-logo-primary-dark%402x.png" style="height:64px;margin-right:32px"/>

# The Apparatus

### Foundation Architecture v0.1

**Status:** Exploration → proposed technical baseline
**Purpose:** A general-purpose, owner-operated system of systems for AI-assisted projects, knowledge, agents, protocols, data, and tools.

The Apparatus converts documents, prompts, research, and project work from disconnected reference material into governed operational infrastructure. It is designed for individual builders, research groups, and small teams—not for one person’s workflows alone—and must remain comprehensible, portable, auditable, and usable without any particular model vendor.[^1]

It extends the RWS Blueprint’s Owner / Manager / Provider separation, one canonical road, append-only history, schema-first admission, explicit maintenance, and reserve capacity. It also applies the research findings that authoritative facts need designated ownership, failures must be recorded, interface risks need their own governance, and evidence, judgment, and decision must remain distinct.[^2][^3][^4]

***

## 1. Core proposition

> **The Apparatus is a local-first coordination and record system.**
> It accepts work from people, software, agents, and external AI services; validates and records it; routes it through declared protocols; and exposes only the minimum authorised context and action surface needed by each participant.

The Apparatus is **not**:

- A chatbot wrapper
- A giant agent framework
- A single universal database
- A cloud control plane
- A replacement for Git, files, normal project repositories, or domain-specific software
- A system that gives an LLM unrestricted access to a machine

It is the stable layer beneath those things: the **road network**, registry system, protocol engine, evidence record, and secure interchange.

***

## 2. Design laws

These are proposed constitutional invariants. They should be few, strict, and difficult to bypass.

1. **Local ownership first.** Core records, identities, secrets, policies, and high-sensitivity assets remain owner-controlled by default.
2. **One law for exchanges.** Every system-to-system action is expressed as a validated command, event, artifact, query, or protocol run; no hidden side channel is permitted.
3. **Many project stores, no universal soup.** Each project or domain owns its own records and database. Cross-project use occurs through deliberate, typed exports, grants, and references—not direct arbitrary reads.
4. **Raw records are immutable; views are replaceable.** An original intake, event, artifact receipt, or decision record is never silently overwritten. Corrections and supersessions are additional records that explicitly refer to the prior one.
5. **Every meaningful object has identity, time, provenance, and authority.** UUIDv7, UTC timestamp, source/provenance chain, ownership, classification, and schema version are mandatory.
6. **Policy, coordination, execution, and evidence are separate.** The Owner defines limits; the Core enforces protocol and record law; providers execute; independent evaluators may assess. This follows the Owner / Manager / Provider separation in the Blueprint and Dutch systems research.[^3][^2]
7. **No automatic escalation of authority.** An AI can propose, classify, retrieve, plan, and execute only inside a granted capability. Irreversible, high-impact, external, financial, destructive, or privacy-sensitive actions require an explicit approval gate.
8. **Failure is first-class data.** Denied requests, malformed submissions, access failures, failed jobs, missing evidence, model errors, and recovery actions become auditable system events.[^4][^3]
9. **The apparatus must degrade safely.** Loss of an external model, cloud service, network path, or optional index may reduce convenience but must not corrupt records or prevent local recovery.
10. **Dependencies are declared assets.** Every external library, model, tool, service, and binary is inventoried, version-pinned, risk-rated, replaceable where possible, and subject to maintenance policy.

***

## 3. System shape

```text
                                  ┌─────────────────────┐
                                  │   ASSET OWNER(S)    │
                                  │ Policy · approval   │
                                  │ risk tolerance      │
                                  └─────────┬───────────┘
                                            │
                                  ┌─────────▼───────────┐
                                  │  APPARATUS CONTROL  │
                                  │ Charter · identity  │
                                  │ registry · policy   │
                                  │ audit · capability  │
                                  └─────────┬───────────┘
                                            │
      ┌─────────────────────────────────────┼─────────────────────────────────────┐
      │                                     │                                     │
┌─────▼─────┐                       ┌───────▼───────┐                     ┌───────▼──────┐
│ Project A │                       │ Project B     │                     │ Shared Assets │
│ Database  │                       │ Database      │                     │ Registry      │
│ Files     │                       │ Files         │                     │ Protocols     │
│ Tasks     │                       │ Tasks         │                     │ Schemas       │
└─────┬─────┘                       └───────┬───────┘                     └───────┬──────┘
      │                                     │                                     │
      └───────────────────────┬─────────────┴──────────────┬──────────────────────┘
                              │                            │
                    ┌─────────▼─────────┐        ┌─────────▼─────────┐
                    │ Apparatus Gateway │        │ Local Runtime     │
                    │ API + MCP facade  │        │ jobs · agents     │
                    │ query/action gate │        │ indexing · backup │
                    └─────────┬─────────┘        └─────────┬─────────┘
                              │                            │
             ┌────────────────┼─────────────┐              │
             │                │             │              │
      ┌──────▼──────┐ ┌───────▼──────┐ ┌────▼─────┐ ┌─────▼──────────┐
      │ Local apps  │ │ Local models │ │ CLI/CI   │ │ External AI     │
      │ Web / React │ │ Tools        │ │ agents   │ │ clients/models  │
      └─────────────┘ └──────────────┘ └──────────┘ └────────────────┘
```

The MCP layer is an **adapter at the edge**, not the core of the Apparatus. MCP defines prompts, resources, and tools as distinct primitives, with prompts user-controlled, resources client-managed, and tools model-controlled; this is useful for interoperability but insufficient as the system’s security, audit, identity, or durable-record model.[^5]

***

## 4. Authority model

| Layer | Owns | May do | Must not do |
| :-- | :-- | :-- | :-- |
| **Asset Owner** | Charter, risk tolerance, project admission, data-class rules, high-impact approvals | Amend policy, appoint maintainers, approve destructive/external actions | Directly bypass recorded protocol without creating an exception record |
| **Apparatus Core** | Identity, registry, policy enforcement, audit, routing, protocol state | Validate, grant/revoke capabilities, append records, schedule work, expose APIs | Hold project-domain business logic |
| **Project Steward** | Project scope, project registry, project database, project protocols | Admit artifacts, define project schemas, approve project-level actions | Read unrelated projects without explicit grant |
| **Service Provider** | UI, CLI, agent adapter, editor, dashboard, automation | Submit typed requests and consume authorised views | Open another project’s store directly |
| **Worker / Agent** | A time-bounded task lease | Read granted context, run approved tools, submit outputs | Expand its own permissions or make unapproved external actions |
| **Independent Evaluator** | Review, audit, test, incident analysis | Assess records and issue findings | Alter operational history or execute production actions |

This retains the Blueprint’s rule that service providers communicate through the Core rather than directly reading each other’s underlying data.[^3]

***

## 5. Network zones

The transcript correctly identifies that “internal” and “external” cannot be one vague category. The architecture should use **three zones**, with one direction of increasing trust:

```text
Public / Internet  →  Controlled Remote Mesh  →  Core Private Network
     Z0                        Z1                       Z2
```


### Z0 — Public / external zone

**Purpose:** Interact with the ordinary internet and outside services.

Examples:

- Browser-facing web application
- Public documentation site
- External model APIs
- Webhooks from approved providers
- Remote CI runners
- Package mirrors and software updates

Rules:

- Contains no canonical databases, root secrets, or unrestricted administration
- May receive requests, but all requests terminate at a hardened gateway
- External AI services receive only explicitly selected, policy-permitted context
- Outputs from external AI are classified as **untrusted external derived artifacts** until validated


### Z1 — Controlled remote mesh

**Purpose:** Private access for trusted devices and operators across networks.

Recommended base:

- Tailscale or a self-hosted WireGuard control-plane equivalent
- Device identity and ACLs
- Per-service network policy rather than broad “everyone on the tailnet can reach everything”

This zone is for authorised owner devices, administrative access, development systems, and trusted local workers. It is **not** equivalent to the core zone.

### Z2 — Core private network

**Purpose:** Hold the canonical records, secrets, internal databases, backup authority, and sensitive local services.

Rules:

- No inbound public internet
- No direct external-model access
- Outbound access only through explicit update, backup, or broker paths
- Core databases bind only to loopback or private interface
- Administration requires a trusted device and a higher-privilege identity
- Ideally separate VLAN/subnet, with firewall rules enforced at router and host levels


### Critical decision

Do **not** treat “air-gapped” as a casual label. A machine that synchronises, receives updates, accepts Tailscale access, or sends backups is not air-gapped. It is a **restricted private zone**. True air-gapping should be reserved for offline key material, recovery archives, or exceptional high-sensitivity assets.

***

## 6. Core components

The Core must remain small and boring. It should not become an orchestration framework, a general AI agent, or an all-purpose product platform.


| Component | Responsibility | Core status |
| :-- | :-- | :-- |
| **Identity Authority** | Users, devices, services, agents, keys, roles, capability grants | Required |
| **Registry Authority** | IDs, schemas, projects, modules, protocols, tools, models, classifications | Required |
| **Record Ledger** | Append-only record receipts, hashes, timestamps, provenance | Required |
| **Artifact Store** | Immutable files/blobs plus content hashes and manifests | Required |
| **Policy Engine** | Evaluates whether a request is allowed under charter, classification, role, and capability | Required |
| **Gateway** | HTTPS API, authentication, rate limits, request validation, audit emission | Required |
| **Protocol Engine** | Runs named workflows with gates, state transitions, and evidence requirements | Required |
| **Task Broker** | Creates task leases, queues work, receives results | Required |
| **Projection Service** | Builds disposable search indexes, dashboards, reports, embeddings, summaries | Required |
| **MCP Adapter** | Maps approved MCP resources/tools/prompts onto internal capability-controlled operations | Required, thin |
| **Scheduler** | Nightly maintenance, backups, integrity checks, index jobs, expiry/revocation | Required |
| **Observability Stack** | Logs, metrics, traces, health checks, alerts | Required |
| **Model Router** | Selects local or approved external model/tool based on task policy | Later core-adjacent |
| **Agent Runtime** | Executes longer-lived autonomous workflows | Optional / later |
| **Vector Search** | Semantic retrieval over approved artifacts | Optional / project-level |
| **Graph Reasoner / solver** | Domain-specific optimisation or planning | Optional / project-level |


***

## 7. Technology language split

The transcript names React as the base. React is not a programming language; it is a UI library. The recommended split is:


| Layer | Recommended language | Reason |
| :-- | :-- | :-- |
| Web UI / desktop UI | **TypeScript + React** | Mature interface ecosystem, typed forms, reusable component model |
| Shared contracts | **TypeScript schemas plus JSON Schema** | Good boundary language for APIs, tools, generated clients, validation |
| Gateway / core API | **Rust** or **Go** | Strong concurrency, explicit resource handling, single static binaries, low operational surface |
| Project automation / analysis | **Python** | Best ecosystem for data, research, ML, document processing, rapid project tooling |
| Local native integrations | Platform-native where necessary: Swift, Rust, C/C++ | Use only at hardware/OS boundaries |
| Databases | SQLite / libSQL-compatible SQL | Portable, inspectable, local-first relational base |

### Recommendation

Use **TypeScript + React** for the user-facing apparatus surfaces, **Rust** for the durable core daemon where reliability/security boundaries matter, and **Python** only as a controlled worker language for analysis, ingestion, experimentation, and project tools.

This preserves elegance:

- React does not leak into the core
- Python does not become the authority boundary
- Rust is reserved for the small, high-value engine
- Every layer communicates by versioned contracts, not shared memory or private database access

***

## 8. Storage architecture

The Apparatus should use **three storage forms**, each for a different purpose.


| Storage form | Role | Canonical? |
| :-- | :-- | :-- |
| Immutable files / object store | Original documents, media, code snapshots, transcripts, bundles, generated artifacts | Yes, for artifact bytes |
| Append-only ledger | Receipts, events, decisions, protocol transitions, audit trail | Yes, for history |
| Relational project database | Current operational state, structured objects, task queues, metadata, links | Yes, for current project state |
| Derived indexes | Full text, embeddings, graphs, dashboards, caches, projections | No; always rebuildable |

This preserves the useful part of the existing JSONL canon while allowing structured, transactional project work. The Blueprint’s raw-canonical / projections-disposable rule remains correct; the Apparatus generalises it from a personal event log to every artifact and protocol record.[^3]

### Database choice

Turso/libSQL is viable as a **project-database option**, but the architecture must not depend on Turso Cloud. libSQL is a production-ready SQLite fork compatible with SQLite’s file format and API; its distributed/embedded-replica capabilities can be useful where they have a demonstrated need. It is not itself the “Rust rewrite of SQLite”; Turso separately introduced Limbo as an experimental complete rewrite direction.[^6][^7]

### Storage rule

> **SQLite-compatible files are the long-lived substrate; hosted replication is an optional transport capability.**

That means:

- A project can run on plain local SQLite from day one
- libSQL/Turso replication can be enabled later
- A project must remain exportable as ordinary SQLite + files + manifests
- No metadata model may require a hosted Turso-only feature
- Vector indexes are derived and rebuildable, never the sole copy of knowledge

***

## 9. Database topology

```text
apparatus/
├── control.db                  # Core control plane; Z2 only
├── ledger/
│   ├── 2026/07/records.jsonl   # Append-only signed/hashed receipts
│   └── checkpoints/
├── registries/
│   ├── schemas/
│   ├── projects/
│   ├── protocols/
│   ├── tools/
│   ├── models/
│   └── classifications/
├── artifacts/
│   └── sha256/ab/cd/<hash>     # Content-addressed immutable blobs
├── projects/
│   ├── mapa/
│   │   ├── project.db
│   │   ├── artifacts/
│   │   ├── workspace/
│   │   └── derived/
│   ├── dutch-way/
│   │   ├── project.db
│   │   ├── artifacts/
│   │   ├── workspace/
│   │   └── derived/
│   └── vector-brain/
│       └── ...
└── backups/
```


### Core control database

`control.db` is not a place for project content. It stores only:

- Identity and device registration
- Project registration and ownership
- Schema registry
- Protocol definitions and versions
- Capability grants
- Tool and model registry
- Audit indexes
- Secret references, never plaintext secrets
- System health and maintenance records


### Project database

Each project receives an isolated `project.db` with:

- Project-local objects
- Project-local tasks
- Project-local claims and evidence
- Project-local artifact references
- Project-local agent sessions
- Project-local decisions
- Project-local derived search metadata

Direct cross-project SQL reads are forbidden. Any cross-project transfer must use a typed `export_bundle`, `reference_grant`, or `artifact_share` protocol.

***

## 10. Universal object model

Every object must be serialisable as JSON, identifiable, time-addressable, attributable, classifiable, and versioned.

```ts
type UUID = string;          // UUIDv7 at creation
type UnixMs = number;        // integer milliseconds since epoch
type SHA256 = string;        // lowercase hex
type URI = string;

type Classification =
  | "public"
  | "shared"
  | "project"
  | "private"
  | "sensitive"
  | "restricted";

type ProvenanceKind =
  | "human_stated"
  | "human_authored"
  | "system_measured"
  | "system_generated"
  | "agent_generated"
  | "external_import"
  | "external_model_output"
  | "derived"
  | "corrected";

interface ObjectHeader {
  id: UUID;
  type: string;
  schema_version: string;

  created_at_unix_ms: UnixMs;
  recorded_at_unix_ms: UnixMs;
  occurred_at_unix_ms?: UnixMs;

  owner_id: UUID;
  project_id?: UUID;

  classification: Classification;
  provenance: ProvenanceKind[];
  source_refs: URI[];
  content_hash?: SHA256;

  status: "active" | "superseded" | "deprecated" | "revoked" | "deleted_logically";
  supersedes_id?: UUID;
  correlation_id?: UUID;
  causation_id?: UUID;
}
```


### Why both UUIDv7 and Unix time

- **UUIDv7** provides globally unique, roughly time-sortable identity
- **Unix milliseconds** provide a clear, portable machine timestamp
- An ISO-8601 UTC rendering should additionally be stored or generated for human inspection

The timestamp must distinguish:

- `occurred_at` — when the described event happened
- `created_at` — when the object was created
- `recorded_at` — when the Apparatus accepted it
- `processed_at` — when a worker/index/model handled it

This prevents later imports from pretending they happened when they were ingested.

***

## 11. Required data objects

### Artifact

An immutable file, bundle, transcript, image, code snapshot, model output, or document.

```ts
interface Artifact extends ObjectHeader {
  type: "artifact";
  media_type: string;
  byte_size: number;
  storage_uri: URI;
  original_filename?: string;

  artifact_kind:
    | "document"
    | "source_code"
    | "dataset"
    | "image"
    | "audio"
    | "video"
    | "transcript"
    | "model_output"
    | "archive"
    | "other";

  extraction_status: "not_required" | "queued" | "complete" | "failed";
  parent_artifact_id?: UUID;
}
```


### Record receipt

A ledger entry proving the Apparatus accepted an object/action.

```ts
interface RecordReceipt extends ObjectHeader {
  type: "record_receipt";
  subject_id: UUID;
  subject_type: string;
  operation: "created" | "updated" | "superseded" | "validated" | "rejected" | "deleted_logically";

  actor_id: UUID;
  authority_id: UUID;
  request_id: UUID;

  previous_receipt_hash?: SHA256;
  receipt_hash: SHA256;
  signature_ref?: URI;
}
```


### Claim and evidence

This prevents an agent summary, a source quotation, an observation, and a decision from collapsing into one ambiguous text block.

```ts
interface Claim extends ObjectHeader {
  type: "claim";
  statement: string;
  claim_kind: "fact" | "estimate" | "forecast" | "interpretation" | "recommendation";
  confidence: number; // 0.0–1.0; never a substitute for provenance
  evidence_ids: UUID[];
  validity_window?: { start_unix_ms?: UnixMs; end_unix_ms?: UnixMs };
}

interface Evidence extends ObjectHeader {
  type: "evidence";
  artifact_id: UUID;
  locator: {
    page?: number;
    section?: string;
    line_start?: number;
    line_end?: number;
    timestamp_start_ms?: number;
    timestamp_end_ms?: number;
  };
  extraction_method: "manual" | "parser" | "ocr" | "model_assisted";
  quotation?: string;
}
```

This implements the research distinction between measured fact, forecast, judgment, and political/owner choice.[^2]

### Decision

```ts
interface Decision extends ObjectHeader {
  type: "decision";
  title: string;
  decision_state: "proposed" | "approved" | "rejected" | "deferred" | "reversed";
  decided_by?: UUID;
  rationale_claim_ids: UUID[];
  alternatives_considered: UUID[];
  consequences: string[];
  review_at_unix_ms?: UnixMs;
}
```


### Task and lease

```ts
interface Task extends ObjectHeader {
  type: "task";
  title: string;
  task_kind: "research" | "coding" | "analysis" | "ingestion" | "maintenance" | "review";
  requested_by: UUID;
  assigned_to?: UUID;

  input_bundle_id?: UUID;
  required_capabilities: string[];
  allowed_tool_ids: UUID[];
  allowed_model_ids: UUID[];

  state: "queued" | "leased" | "running" | "blocked" | "completed" | "failed" | "cancelled";
  deadline_unix_ms?: UnixMs;
}

interface TaskLease extends ObjectHeader {
  type: "task_lease";
  task_id: UUID;
  worker_id: UUID;
  issued_by: UUID;
  expires_at_unix_ms: UnixMs;
  capability_grant_ids: UUID[];
}
```


### Session bundle

This directly answers the transcript’s desired “end session and let the internal system do the logging” mechanism.

```ts
interface SessionBundle extends ObjectHeader {
  type: "session_bundle";
  session_kind: "human" | "agent" | "hybrid";
  project_id: UUID;

  participants: UUID[];
  source_artifact_ids: UUID[];
  input_ids: UUID[];
  output_ids: UUID[];

  task_ids: UUID[];
  decision_ids: UUID[];
  unresolved_question_ids: UUID[];
  handoff_summary_artifact_id?: UUID;

  close_state: "open" | "submitted" | "processed" | "review_required" | "closed";
}
```


### Protocol run

```ts
interface ProtocolRun extends ObjectHeader {
  type: "protocol_run";
  protocol_id: UUID;
  protocol_version: string;
  initiated_by: UUID;
  input_bundle_id?: UUID;

  state:
    | "requested"
    | "validated"
    | "awaiting_approval"
    | "running"
    | "blocked"
    | "completed"
    | "failed"
    | "cancelled";

  current_step_id?: string;
  output_ids: UUID[];
  incident_ids: UUID[];
}
```


### Capability grant

```ts
interface CapabilityGrant extends ObjectHeader {
  type: "capability_grant";
  subject_id: UUID;         // agent, service, human, device
  capability: string;       // e.g. artifact.read, git.commit, shell.run.sandboxed
  resource_scope: URI[];
  constraints: {
    max_calls?: number;
    max_bytes?: number;
    expires_at_unix_ms?: UnixMs;
    require_approval?: boolean;
    network_zone?: "z0" | "z1" | "z2";
  };
  granted_by: UUID;
  revoked_at_unix_ms?: UnixMs;
}
```


***

## 12. Metadata standard

Every imported file, generated artifact, tool call, prompt, session, decision, protocol run, and cross-network transfer must carry the following minimum metadata:


| Category | Required fields |
| :-- | :-- |
| Identity | UUIDv7, object type, schema version |
| Time | created, occurred where applicable, recorded, processed |
| Ownership | owner, project, steward |
| Classification | public/shared/project/private/sensitive/restricted |
| Provenance | source kind, source references, actor, model/tool version |
| Integrity | SHA-256 content hash, size, media type, signature if available |
| Lineage | parent, supersedes, correlation, causation |
| Authority | capability grant, approval reference, protocol run |
| Retention | retention policy, legal/owner hold, deletion eligibility |
| Reproducibility | environment, dependency lock reference, prompt/template version, seed where meaningful |

### Model-output metadata

Every AI-produced artifact must additionally store:

- Provider and model identifier
- Model version/date where available
- Invocation timestamp
- System prompt/template identifier
- Input bundle references
- Tool calls and their receipts
- Parameters relevant to reproducibility
- Human review status
- Whether output is draft, proposed, accepted, or rejected

An external model output is never itself a verified fact. It becomes a claim or proposal whose evidence must be separately linked.

***

## 13. Communication layers

### Layer A — Human interface

- React web console
- Desktop shell later if necessary
- CLI for developers and operators
- Mobile UI only for narrow interactions, approvals, notifications, and capture


### Layer B — Apparatus API

- HTTPS + JSON
- OpenAPI contract generated from schemas
- Authentication at gateway
- Request IDs, idempotency keys, signed receipts
- Streaming only where clearly needed


### Layer C — Internal command bus

Start simple:

- Durable task table in SQLite/libSQL
- Worker polling or server-sent events
- Explicit state transitions
- Idempotent handlers

Do **not** begin with Kafka, NATS, RabbitMQ, Temporal, Kubernetes, or an event mesh. A durable relational task queue plus append-only receipts is sufficient until measured load or reliability requirements prove otherwise.

### Layer D — MCP interoperability facade

MCP tools must map to internal capabilities.

Example:

```text
MCP tool: apparatus.search_project
        ↓
Gateway validates client identity
        ↓
Policy engine checks project.read grant
        ↓
Project query service executes scoped search
        ↓
Receipt written to ledger
        ↓
Result returned with artifact IDs + evidence locators
```

MCP resources should expose approved read-only views. MCP tools should expose only narrow, auditable actions. MCP prompts may expose reusable workflows, but no prompt should grant authority by itself. MCP’s own primitives distinguish tools, resources, and prompts; the Apparatus adds the missing governance layer around them.[^5]

### Layer E — External service broker

No external service calls the core directly.

The broker:

- Redacts or selects permitted context
- Logs request metadata and payload policy
- Enforces provider-specific quotas
- Applies egress rules
- Stores output as external derived artifact
- Prevents secrets or restricted data from leaving the authorised boundary
- Can be disabled without breaking the local record system

***

## 14. Key protocols

### Project bootstrap protocol

A new project receives:

1. Project ID and steward
2. Isolated project database
3. Artifact directory
4. Default project policy
5. Schema namespace
6. Git repository or repository binding
7. Tool allow-list
8. Model policy
9. Basic task queue
10. Backup inclusion
11. Admission receipt

No project directly edits Apparatus core tables.

### Session close protocol

This is the first high-value automation to build.

```text
Agent/human invokes apparatus.session.close
        ↓
Validate session bundle shape
        ↓
Collect declared inputs, outputs, artifacts, task references
        ↓
Store and hash missing artifacts
        ↓
Extract metadata and provenance
        ↓
Classify by policy
        ↓
Create claims, decisions, unresolved questions where declared
        ↓
Generate concise handoff artifact
        ↓
Append ledger receipts
        ↓
Queue optional local indexing / summarisation
        ↓
Return immutable session receipt
```

The worker does not manually perform archival bookkeeping; it hands the Apparatus a structured bundle. This directly implements the transcript’s intended division of labour.[^1]

### Artifact intake protocol

1. Accept file or URL through gateway
2. Virus/malware scan where applicable
3. Hash content
4. Store immutable original
5. Create Artifact object
6. Extract text/OCR in a derived artifact
7. Assign provenance and classification
8. Queue optional indexing
9. Return receipt

### Agent task protocol

1. Human or system creates typed Task
2. Policy engine determines permitted tools/models/data
3. Task broker creates short-lived lease
4. Worker receives only scoped input bundle
5. Worker emits progress receipts
6. Worker submits output bundle
7. Output enters validation/review state
8. Session-close protocol packages the work
9. Capability expires automatically

### High-impact action protocol

For actions such as sending external messages, modifying production infrastructure, publishing, deleting, spending money, changing policy, or granting permissions:

1. Agent prepares a proposal
2. Apparatus generates a **fact / inference / recommendation / requested decision** brief
3. Owner or authorised steward approves/rejects
4. Execution occurs via narrow capability
5. Result and evidence are recorded
6. Post-condition checks run
7. Failure or divergence creates an incident

This resembles the research distinction between a signal body, a decision body, and an executing body.[^4][^2]

### Incident and recovery protocol

- Capture incident
- Classify: infrastructure, interface, security, data integrity, external dependency, model behaviour, operator error, unknown
- Contain
- Preserve evidence
- Restore service
- Create root-cause analysis
- Record corrective action
- Verify corrective action
- Close only when the recovery threshold is met

Interface incidents must be separately visible: many failures occur not inside one component but at the handoff between components, which is exactly the kind of risk treated as its own governed category in the aviation research.[^2]

***

## 15. MCP server design

The Apparatus should expose **one MCP server per trust boundary**, not one universal super-server.


| MCP server | Audience | Permitted scope |
| :-- | :-- | :-- |
| `apparatus-mcp-local` | Local development tools | Selected project tools, sandbox actions, local queries |
| `apparatus-mcp-remote` | Trusted remote AI clients | Read-only or approval-gated project operations |
| `apparatus-mcp-research` | Research agents | Artifact upload, source capture, claim/evidence submission |
| `apparatus-mcp-admin` | Owner only | Registry, policy, recovery, maintenance; never exposed externally |
| `project-<id>-mcp` | Project-specific workers | Only that project’s granted protocols and tools |

### MCP tool categories

**Read-only tools**

- `project.search`
- `artifact.get_metadata`
- `artifact.get_text_excerpt`
- `claim.list`
- `protocol.describe`
- `task.get`
- `registry.list_allowed_tools`

**Bounded write tools**

- `artifact.submit`
- `claim.submit`
- `task.submit_result`
- `session.close`
- `incident.report`
- `protocol.start`

**Never exposed as ordinary model tools**

- Raw shell access to Z2
- Unbounded database SQL
- Secret read/export
- Policy amendment
- Permanent capability grant
- Direct deletion
- Raw network scanning
- Router/firewall control
- Root-level host administration

***

## 16. Core library layout

```text
apparatus-core/
├── crates/
│   ├── apparatus-types/        # Stable object definitions, IDs, enums
│   ├── apparatus-schema/       # JSON Schema generation and validation
│   ├── apparatus-registry/     # Project/module/tool/model/protocol registry
│   ├── apparatus-ledger/       # Receipts, hash chain, integrity checkpoints
│   ├── apparatus-policy/       # Capability and classification decisions
│   ├── apparatus-artifacts/    # Content-addressed storage + manifests
│   ├── apparatus-protocols/    # State machines and protocol runner
│   ├── apparatus-tasks/        # Tasks, leases, idempotency, worker results
│   ├── apparatus-gateway/      # API/auth/rate limit/audit middleware
│   ├── apparatus-mcp/          # Thin MCP adapter
│   ├── apparatus-observe/      # Structured logs, metrics, traces
│   └── apparatus-maintenance/  # Backup, integrity, dependency checks
├── services/
│   ├── apparatusd/             # Main daemon
│   ├── apparatus-worker/       # Controlled local worker runner
│   └── apparatus-cli/          # Operator/developer command line
├── web/
│   └── console/                # TypeScript + React operator UI
├── schemas/
│   ├── core/
│   ├── protocols/
│   └── project_templates/
└── docs/
```


### Stable core interfaces

```rust
trait Registry {
    fn resolve_project(&self, project_id: ProjectId) -> Result<Project>;
    fn resolve_protocol(&self, protocol_id: ProtocolId) -> Result<ProtocolDefinition>;
    fn resolve_tool(&self, tool_id: ToolId) -> Result<ToolDefinition>;
}

trait Ledger {
    fn append(&self, receipt: RecordReceipt) -> Result<LedgerPosition>;
    fn verify_chain(&self) -> Result<IntegrityReport>;
}

trait PolicyEngine {
    fn authorize(&self, request: AuthorizationRequest) -> Decision<CapabilityGrant>;
}

trait ArtifactStore {
    fn put(&self, input: ArtifactInput) -> Result<Artifact>;
    fn get(&self, artifact_id: ArtifactId) -> Result<ArtifactStream>;
}

trait ProtocolRunner {
    fn start(&self, request: ProtocolStartRequest) -> Result<ProtocolRun>;
    fn advance(&self, run_id: ProtocolRunId, event: ProtocolEvent) -> Result<ProtocolRun>;
}
```

The important architectural rule: **project modules depend on interfaces and schemas, never on the core daemon’s private implementation.**

***

## 17. Project module contract

A project can be a research corpus, robotics programme, personal instrument, product, agent system, or operational service.

Every project module must declare:

```yaml
project:
  id: mapa
  version: 0.1.0
  steward: owner:project-mapa
  classification_default: project

data:
  schema_namespace: "org.apparatus.mapa"
  database: "projects/mapa/project.db"
  artifact_root: "projects/mapa/artifacts"

interfaces:
  inbound_protocols:
    - artifact_intake
    - task_submission
    - session_close
  outbound_exports:
    - project_summary_bundle

tools:
  allow:
    - git
    - python.analysis
    - local_embedding
  approval_required:
    - network.egress
    - external_model.invoke
    - publish

models:
  local_allowed:
    - embedding-small
  external_allowed:
    - approved-research-model

retention:
  raw_artifacts: retain
  derived_indexes: rebuildable
  task_logs_days: 180
```

A module cannot:

- Invent unnamed data categories
- Use another project database directly
- Add an external dependency without inventory registration
- Make high-impact action capabilities default
- Store a secret in project metadata or source code
- Convert external model output into authoritative fact automatically

***

## 18. Tool, library, and asset policy

### Build ourselves

These are too foundational to outsource as opaque dependencies.


| Asset | Why own it |
| :-- | :-- |
| Core object schemas | They define the system’s law |
| Registry model | It determines authority and compatibility |
| Capability and approval rules | Central security boundary |
| Ledger receipt format | Core record integrity and auditability |
| Protocol definitions | The apparatus’s operational logic |
| Session-close protocol | Primary automation and continuity mechanism |
| Project bootstrap template | Ensures compatible growth |
| Artifact manifest format | Long-term portability |
| Dependency inventory | Supply-chain control |
| MCP adapter policy layer | Prevents model tools becoming ungoverned access |

### Use directly

| Tool / asset | Use | Conditions |
| :-- | :-- | :-- |
| **SQLite** | Durable local relational storage | Base format; always exportable |
| **libSQL / Turso** | Optional replication, remote sync, vector features | Do not make cloud hosting mandatory; retain SQLite portability [^6] |
| **Git** | Source, registry, and configuration history | Signed commits for high-value changes where practical |
| **OpenAPI** | API contract publication and client generation | Generated from canonical schemas |
| **JSON Schema** | Boundary validation | Keep schemas versioned |
| **MCP** | AI-client interoperability | Expose through capability-controlled facade [^5] |
| **Tailscale / WireGuard** | Trusted remote mesh | ACLs, device posture, no broad implicit trust |
| **OpenTelemetry** | Traces, metrics, structured observability | No sensitive payloads by default |
| **Prometheus-compatible metrics** | System health monitoring | Local-first deployment |
| **Grafana** | Operational dashboards | Optional, read-only projection layer |
| **SOPS + age** | Encrypted configuration/secrets in version control | Keys outside repository |
| **restic** | Encrypted, deduplicated backups | Mandatory restore testing |
| **Trivy / OSV-Scanner** | Dependency and image vulnerability scanning | Run in CI and scheduled maintenance |
| **Sigstore / cosign** | Artifact signing and verification | Use for release and binary provenance |
| **Syft + Grype** | SBOM generation and vulnerability scanning | Attach SBOM to releases |
| **Semgrep** | Static application security analysis | Baseline rule set plus local rules |
| **Playwright** | UI regression testing | Project-level |
| **pytest / cargo test / Vitest** | Test frameworks | Match language layer |
| **Stockfish** | Chess-specific deterministic analysis | Only as a project-specific engine, never a general reasoning substitute |

### Evaluate before admission

| Candidate | Why not core by default |
| :-- | :-- |
| LangChain / LlamaIndex | Useful adapters, but fast-moving abstractions; can be project-level only |
| Temporal | Strong workflow engine, but too heavy until protocol throughput proves need |
| NATS / RabbitMQ / Kafka | Valuable at scale; premature for first implementation |
| Kubernetes | Operational complexity vastly exceeds initial needs |
| Neo4j / graph database | Add only when graph queries cannot be served from relational projections |
| Qdrant / Weaviate | Useful vector stores, but should not replace SQLite/files until genuine scale requires it |
| Postgres | Excellent when multi-user concurrency and server workloads justify it; not default for local project stores |
| Full local LLM serving stack | Valuable later; needs hardware/quality evaluation, not a foundation dependency |


***

## 19. Supply-chain standards

The core should adopt a conservative dependency posture inspired by durable infrastructure rather than rapid prototype culture.

### Required controls

- Lockfiles committed for every language ecosystem
- Exact version pinning for core dependencies
- SBOM produced for every release
- Dependency inventory includes owner, purpose, license, version, replacement plan, and risk tier
- Automated vulnerability scan in CI and monthly maintenance
- Signed release artifacts
- No unreviewed install scripts running with administrative privileges
- No direct dependency from arbitrary Git branch or URL in core
- Minimal dependency count in Rust core
- Separate “experimental” dependencies from “core-approved” dependencies
- Reproducible build target where practical
- Offline recovery bundle for core bootstrap


### Dependency tiers

| Tier | Meaning | Examples |
| :-- | :-- | :-- |
| **A — Foundation** | Failure threatens core records or authority | SQLite, crypto primitives, Rust runtime, filesystem |
| **B — Operational** | Failure reduces service but recovery path exists | Tailscale, backup tool, observability |
| **C — Project utility** | Useful within specific projects | OCR, embeddings, document parsers |
| **D — Experimental** | Trial only; removable without migration | Agent framework, new vector engine, novel model router |

A Tier A or B addition requires a formal dependency admission record. A Tier C project dependency needs project steward approval. Tier D cannot become load-bearing.

***

## 20. Hardware and ownership stack

| Location / zone | Hardware role | Owns |
| :-- | :-- | :-- |
| **Offline recovery device** | Optional encrypted archive / recovery keys | Root recovery material, encrypted backup verification keys |
| **Core node, Z2** | Small always-on Linux machine or dedicated server | `control.db`, ledger, registries, artifact manifests, policy engine, gateway internal endpoint, scheduler |
| **Storage node, Z2** | NAS or directly attached encrypted storage | Artifact bytes, immutable snapshots, backup staging |
| **Local worker node, Z2/Z1** | Compute host with CPU/GPU as needed | OCR, embeddings, indexing, local models, code execution sandboxes |
| **Owner workstation, Z1** | Primary operator/developer environment | React console, CLI, IDE, project workspaces, approved admin client |
| **Mobile capture device, Z1** | Narrow intake/approval surface | Capture, notifications, explicit approvals; no core authority |
| **External AI providers, Z0** | Optional inference services | Receive only brokered authorised bundles; own no canonical Apparatus records |
| **Public web host, Z0** | Optional public/UI edge | Static site or limited gateway; never direct core access |

### Recommended first physical layout

Do not overbuild a datacenter.

1. **One Linux mini-PC or existing capable machine** as Core + controlled worker host
2. **One encrypted external drive** for local backup rotation
3. **One encrypted off-site backup target**
4. **One daily workstation** as developer/operator surface
5. **Tailscale/WireGuard** for Z1 trusted access
6. **Router/firewall rules** creating real Z0/Z1/Z2 segmentation
7. **Optional GPU node later**, not a requirement for M0

The core’s first requirement is recoverability and clarity, not GPU capacity.

***

## 21. Security model

### Identity

Use distinct identities for:

- Human owner
- Human steward
- Device
- Service
- Agent worker
- External AI client
- CI runner
- Recovery operator

Never reuse a human identity as a background service identity.

### Secrets

- Secrets live in a dedicated encrypted secret store
- Applications receive short-lived injected secrets
- Source code, artifacts, logs, prompts, and databases store references—not plaintext
- Rotate credentials after incident or device loss
- Restrict external service keys to project/function scope


### Capability model

A capability has:

- Subject
- Action
- Resource scope
- Network zone
- Expiry
- Call/byte limit
- Approval requirement
- Issuer
- Revocation record

Example:

```text
agent:research-worker-17
may: artifact.read
scope: project:dutch-way/artifacts:*
zone: z1
expires: 2026-07-24T03:00:00Z
max_bytes: 50 MB
```


### Sandboxed execution

Any agent that runs code should execute in a sandbox with:

- Read-only input bundle mount
- Separate writable output directory
- No host filesystem access by default
- No secrets unless explicitly injected
- No network by default
- CPU, memory, runtime, and disk limits
- Complete command and artifact receipt log

***

## 22. Models and reasoning tools

The Apparatus should not assume “more LLMs” equals better architecture. It needs a **model selection policy**, not a permanent collection of models.


| Work type | Preferred engine class | Example |
| :-- | :-- | :-- |
| Schema validation | Deterministic code | JSON Schema / Rust validator |
| Hashing, classification rules, routing | Deterministic code | Core services |
| Search | Full-text index first; embeddings second | SQLite FTS + optional vector index |
| Numerical optimisation | Domain solver | OR-tools, linear algebra, project engine |
| Chess/game perfect-information search | Domain engine | Stockfish |
| Document extraction | Parser/OCR first, model fallback | PDF parser, OCR |
| Code transformation | Strong coding model with tests/sandbox | External or local model by policy |
| Research synthesis | External model plus citation/evidence protocol | Brokered external AI |
| Private summarisation | Local model when quality sufficient | Optional local runtime |
| High-impact decision | Human owner/steward | Never delegated |

The transcript’s Stockfish insight is right in principle: specialised engines can outperform general models when the environment is formally represented and the search space is well defined. The Apparatus should therefore treat Stockfish-like tools as **domain engines**, invoked only where their assumptions hold—not as metaphors that justify applying a chess engine to ambiguous human/project evidence.[^1]

### Model router rules

- Deterministic tool before LLM when feasible
- Local model before external model for sensitive material where quality permits
- Cheapest adequate model before most powerful model
- Strong model only when task complexity and policy justify it
- Every model call is logged as a model invocation object
- Model output cannot mutate core state without protocol/approval
- External model calls pass through the egress broker

***

## 23. Observability and maintenance

The apparatus must measure itself, as the Blueprint frames infrastructure as an information system that records its own failures.[^3]

### Daily maintenance window

- Integrity-check ledger hash chain
- Verify database consistency
- Snapshot control and project databases
- Backup changed artifacts
- Verify critical service health
- Check capability expiry and stale task leases
- Recompute projections/indexes as required
- Run dependency advisory scan
- Emit maintenance receipts


### Weekly maintenance window

- Restore-test one selected backup
- Review failed protocol runs
- Review denied high-impact requests
- Check storage capacity and backup age
- Rotate/verify logs
- Review external-service usage and egress records


### Monthly maintenance window

- Dependency update branch
- SBOM refresh
- Security review
- Schema compatibility tests
- Project registry audit
- Rebuild one derived index from canonical source as a recovery drill
- Review unused capabilities and revoke them


### Metrics

- Record append latency
- Artifact intake success/failure rate
- Protocol completion/failure rate
- Task lease expiry rate
- Backup success and verified restore age
- Dependency vulnerability count by tier
- External model egress volume by project/classification
- Unreviewed model output count
- Cross-project transfer count
- Incident recurrence rate
- Core storage and index growth

***

## 24. Initial implementation sequence

### M0 — Law and record

Build only:

- Core object schemas
- Registry files/database
- `control.db`
- Artifact content hashing
- Append-only ledger receipts
- Project bootstrap
- Minimal CLI
- `POST /artifact`
- `POST /session/close`
- `GET /artifact/:id`
- Backup snapshot script
- One integrity test

**Evidence of success:** a document can be imported, hashed, classified, recorded, retrieved, backed up, and restored on a clean machine.

### M1 — Project and session apparatus

Build:

- Project bootstrap protocol
- Project database template
- Task object and lease model
- Session bundle and session-close protocol
- Basic React console
- Git repository binding
- Local worker queue
- Structured audit views

**Evidence of success:** a coding/research session can close through one command and produce a complete project-bound handoff bundle without manual archival work.

### M2 — Secure agent and MCP boundary

Build:

- Identity service
- Capability grants
- MCP facade
- Tool registry
- Sandbox worker
- External egress broker
- Approval gate for high-impact actions
- Incident record

**Evidence of success:** an external or local AI client can read an approved project bundle, submit a result, and be unable to exceed its permission scope.

### M3 — Derived intelligence

Build only after the record system works:

- Full-text search
- Optional embeddings
- Local document extraction pipeline
- Claim/evidence graph
- Model router
- Dashboard projections
- Research and code-review protocols

**Evidence of success:** a project can query its own approved corpus with traceable evidence locators and no ambiguity about original source versus generated summary.

### M4 — Federation and collaboration

Later:

- Multi-owner organisations
- Shared project grants
- Cross-Apparatus export/import bundles
- Federated registries
- Remote worker pools
- Optional libSQL replication
- Public documentation/external API products

***

## 25. Open decisions

These should remain explicitly unresolved until a focused decision pass.


| Decision | Current recommendation | Why open |
| :-- | :-- | :-- |
| Core implementation language | Rust | Strong fit for small durable daemon; benchmark against Go only if build velocity suffers |
| Database baseline | Plain SQLite first, libSQL-compatible | Keeps portability; replication need not be assumed |
| Core node OS | Linux | Best service/runtime control; exact distribution needs selection |
| Identity provider | Start local/simple, evaluate OIDC later | Avoid introducing an IdP platform before multi-user need |
| Object storage | Local filesystem first | S3/MinIO only when scale or remote durability warrants it |
| Tailscale dependency | Use initially, preserve WireGuard exit path | Convenience is high, but trust/dependency model needs explicit acceptance |
| Artifact encryption | Encrypt full disks plus encrypted backups at M0 | Per-object encryption may be needed later for sharing/revocation |
| Signed ledger | Hash chain at M0; cryptographic signatures at M2 | Key management must be designed before signatures are treated as authoritative |
| Local model runtime | Not core | Hardware, quality, privacy, and maintenance must be evaluated per use case |
| Vector search | Per-project optional | Full-text plus metadata handles more than expected |
| Workflow engine | SQLite task queue at M0–M2 | Adopt Temporal/NATS only on measured requirements |
| Multi-user tenancy | Design identifiers now; do not implement yet | Avoid accidental personal-only architecture without premature IAM complexity |
| Public external API | No | Must follow mature policy, rate-limit, and incident capability |
| Browser extension / mobile agents | No | High privacy and attack-surface implications |
| “Air gap” assets | Offline recovery keys/backups only | Define exact operational procedure before claiming air-gapped security |


***

## 26. Explicit non-goals for v0.1

- Autonomous self-modifying agents
- General web browsing agent with machine control
- Full personal memory system
- Social/multi-tenant collaboration platform
- Kubernetes deployment
- Universal graph database
- A replacement for GitHub/GitLab
- A universal model router across every provider
- A full public SaaS product
- Semantic search as a core dependency
- Automatic truth determination by model confidence
- A single shared database for every project

***

## 27. Definition of done for the foundation

The Apparatus foundation is real when all of the following work:

1. A new project can be created in one typed command.
2. A file can be ingested, content-hashed, classified, stored, and retrieved by immutable ID.
3. Every operation creates a receipt with UUIDv7, Unix time, actor, authority, provenance, and schema version.
4. A session can be closed through a structured bundle rather than manual archival labour.
5. A local worker can receive a scoped task lease and submit output without unrestricted host access.
6. An external AI client can use a limited MCP interface without receiving direct database, filesystem, or secret access.
7. A denied request is visible as an auditable record.
8. Core and project databases plus artifact storage can be restored onto a clean replacement machine.
9. A derived index can be deleted and rebuilt from canonical records.
10. Adding a new project or module requires a permit-like registry admission, not modification of the core.

***

## Closing architecture statement

The Apparatus should be built as **a small sovereign core with many replaceable edges**. Files and SQLite-compatible databases preserve ownership; the ledger preserves history; registries preserve meaning; protocols preserve repeatability; capability grants preserve boundaries; and the MCP/API surfaces make the system usable by future AI clients without letting those clients become the system’s authority.

The practical first move is deliberately narrow: **project bootstrap, artifact intake, immutable receipts, and session close**. Everything else—including agents, local models, vector retrieval, distributed replication, and rich visual surfaces—can attach later through the roads already laid. This retains the RWS Blueprint principle: build only what is needed now, but place the pipes so future projects do not require excavation through the foundation.[^4][^2][^3]
<span style="display:none">[^10][^11][^12][^13][^14][^15][^16][^17][^18][^19][^8][^9]</span>

<div align="center">⁂</div>

[^1]: apparatus_transcript.txt

[^2]: Geodesic-2-Dutch-Stewardship-Evidence-Crisis.md

[^3]: RWS-Blueprint-Foundation-Architecture.md

[^4]: Geodesic-1-Critical-Networks.md

[^5]: https://github.com/modelcontextprotocol/specification/blob/main/docs/specification/2025-03-26/server/_index.md

[^6]: https://docs.turso.tech/libsql

[^7]: https://turso.tech/blog/introducing-limbo-a-complete-rewrite-of-sqlite-in-rust

[^8]: https://docs.turso.tech/sdk/rust/reference

[^9]: https://github.com/tursodatabase/libsql

[^10]: https://docs.turso.tech/features/embedded-replicas/introduction

[^11]: https://modelcontextprotocol.io/specification/2025-06-18/server/prompts

[^12]: https://github.com/tursodatabase/embedded-replica-examples

[^13]: https://www.prisma.io/docs/orm/v6/overview/databases/turso

[^14]: https://github.com/tursodatabase/agent-skills/blob/main/skills/turso-db/SKILL.md

[^15]: https://www.youtube.com/watch?v=oSbdNY2n8Qk

[^16]: https://modelcontextprotocol.info/docs/concepts/prompts/

[^17]: https://modelcontextprotocol.be/resources

[^18]: https://dev.to/arshtechpro/turso-a-rust-rewrite-of-sqlite-setup-guide-and-whether-its-worth-your-time-16lk

[^19]: https://turso.tech/blog/sync-benchmark

