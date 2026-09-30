# The Apparatus  
### Foundation Architecture v0.1  
**Status:** Exploration → proposed technical baseline  
**Purpose:** A general-purpose, owner-operated system of systems for AI-assisted projects, knowledge, agents, protocols, data, and tools.

The Apparatus converts documents, prompts, research, and project work from disconnected reference material into governed operational infrastructure. It is designed for individual builders, research groups, and small teams—not for one person’s workflows alone—and must remain comprehensible, portable, auditable, and usable without any particular model vendor.[1]

It extends the RWS Blueprint’s Owner / Manager / Provider separation, one canonical road, append-only history, schema-first admission, explicit maintenance, and reserve capacity. It also applies the research findings that authoritative facts need designated ownership, failures must be recorded, interface risks need their own governance, and evidence, judgment, and decision must remain distinct.[2][3][4]

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

6. **Policy, coordination, execution, and evidence are separate.** The Owner defines limits; the Core enforces protocol and record law; providers execute; independent evaluators may assess. This follows the Owner / Manager / Provider separation in the Blueprint and Dutch systems research.[3][2]

7. **No automatic escalation of authority.** An AI can propose, classify, retrieve, plan, and execute only inside a granted capability. Irreversible, high-impact, external, financial, destructive, or privacy-sensitive actions require an explicit approval gate.

8. **Failure is first-class data.** Denied requests, malformed submissions, access failures, failed jobs, missing evidence, model errors, and recovery actions become auditable system events.[4][3]

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

The MCP layer is an **adapter at the edge**, not the core of the Apparatus. MCP defines prompts, resources, and tools as distinct primitives, with prompts user-controlled, resources client-managed, and tools model-controlled; this is useful for interoperability but insufficient as the system’s security, audit, identity, or durable-record model.[5]

***

## 4. Authority model

| Layer | Owns | May do | Must not do |
|---|---|---|---|
| **Asset Owner** | Charter, risk tolerance, project admission, data-class rules, high-impact approvals | Amend policy, appoint maintainers, approve destructive/external actions | Directly bypass recorded protocol without creating an exception record |
| **Apparatus Core** | Identity, registry, policy enforcement, audit, routing, protocol state | Validate, grant/revoke capabilities, append records, schedule work, expose APIs | Hold project-domain business logic |
| **Project Steward** | Project scope, project registry, project database, project protocols | Admit artifacts, define project schemas, approve project-level actions | Read unrelated projects without explicit grant |
| **Service Provider** | UI, CLI, agent adapter, editor, dashboard, automation | Submit typed requests and consume authorised views | Open another project’s store directly |
| **Worker / Agent** | A time-bounded task lease | Read granted context, run approved tools, submit outputs | Expand its own permissions or make unapproved external actions |
| **Independent Evaluator** | Review, audit, test, incident analysis | Assess records and issue findings | Alter operational history or execute production actions |

This retains the Blueprint’s rule that service providers communicate through the Core rather than directly reading each other’s underlying data.[3]

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
|---|---|---|
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
|---|---|---|
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
|---|---|---|
| Immutable files / object store | Original documents, media, code snapshots, transcripts, bundles, generated artifacts | Yes, for artifact bytes |
| Append-only ledger | Receipts, events, decisions, protocol transitions, audit trail | Yes, for history |
| Relational project database | Current operational state, structured objects, task queues, metadata, links | Yes, for current project state |
| Derived indexes | Full text, embeddings, graphs, dashboards, caches, projections | No; always rebuildable |

This preserves the useful part of the existing JSONL canon while allowing structured, transactional project work. The Blueprint’s raw-canonical / projections-disposable rule remains correct; the Apparatus generalises it from a personal event log to every artifact and protocol record.[3]

### Database choice

Turso/libSQL is viable as a **project-database option**, but the architecture must not depend on Turso Cloud. libSQL is a production-ready SQLite fork compatible with SQLite’s file format and API; its distributed/embedded-replica capabilities can be useful where they have a demonstrated need. It is not itself the “Rust rewrite of SQLite”; Turso separately introduced Limbo as an experimental complete rewrite direction.[6][7]

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

This implements the research distinction between measured fact, forecast, judgment, and political/owner choice.[2]

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
|---|---|
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

MCP resources should expose approved read-only views. MCP tools should expose only narrow, auditable actions. MCP prompts may expose reusable workflows, but no prompt should grant authority by itself. MCP’s own primitives distinguish tools, resources, and prompts; the Apparatus adds the missing governance layer around them.[5]

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

The worker does not manually perform archival bookkeeping; it hands the Apparatus a structured bundle. This directly implements the transcript’s intended division of labour.[1]

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

This resembles the research distinction between a signal body, a decision body, and an executing body.[4][2]

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

Interface incidents must be separately visible: many failures occur not inside one component but at the handoff between components, which is exactly the kind of risk treated as its own governed category in the aviation research.[2]

***

## 15. MCP server design

The Apparatus should expose **one MCP server per trust boundary**, not one universal super-server.

| MCP server | Audience | Permitted scope |
|---|---|---|
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
|---|---|
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
|---|---|---|
| **SQLite** | Durable local relational storage | Base format; always exportable |
| **libSQL / Turso** | Optional replication, remote sync, vector features | Do not make cloud hosting mandatory; retain SQLite portability [6] |
| **Git** | Source, registry, and configuration history | Signed commits for high-value changes where practical |
| **OpenAPI** | API contract publication and client generation | Generated from canonical schemas |
| **JSON Schema** | Boundary validation | Keep schemas versioned |
| **MCP** | AI-client interoperability | Expose through capability-controlled facade [5] |
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
|---|---|
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
|---|---|---|
| **A — Foundation** | Failure threatens core records or authority | SQLite, crypto primitives, Rust runtime, filesystem |
| **B — Operational** | Failure reduces service but recovery path exists | Tailscale, backup tool, observability |
| **C — Project utility** | Useful within specific projects | OCR, embeddings, document parsers |
| **D — Experimental** | Trial only; removable without migration | Agent framework, new vector engine, novel model router |

A Tier A or B addition requires a formal dependency admission record. A Tier C project dependency needs project steward approval. Tier D cannot become load-bearing.

***

## 20. Hardware and ownership stack

| Location / zone | Hardware role | Owns |
|---|---|---|
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
|---|---|---|
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

The transcript’s Stockfish insight is right in principle: specialised engines can outperform general models when the environment is formally represented and the search space is well defined. The Apparatus should therefore treat Stockfish-like tools as **domain engines**, invoked only where their assumptions hold—not as metaphors that justify applying a chess engine to ambiguous human/project evidence.[1]

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

The apparatus must measure itself, as the Blueprint frames infrastructure as an information system that records its own failures.[3]

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
|---|---|---|
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

The practical first move is deliberately narrow: **project bootstrap, artifact intake, immutable receipts, and session close**. Everything else—including agents, local models, vector retrieval, distributed replication, and rich visual surfaces—can attach later through the roads already laid. This retains the RWS Blueprint principle: build only what is needed now, but place the pipes so future projects do not require excavation through the foundation.[4][2][3]


[7] Introducing Limbo: A complete rewrite of SQLite in Rust https://turso.tech/blog/introducing-limbo-a-complete-rewrite-of-sqlite-in-rust
[8] Reference https://docs.turso.tech/sdk/rust/reference
[9] GitHub - tursodatabase/libsql: libSQL is a fork of SQLite that is both Open Source, and Open Contributions. https://github.com/tursodatabase/libsql
[10] Embedded Replicas https://docs.turso.tech/features/embedded-replicas/introduction
[11] Prompts https://modelcontextprotocol.io/specification/2025-06-18/server/prompts
[12] Examples of using Turso/libSQL embedded replica sync capability. https://github.com/tursodatabase/embedded-replica-examples
[13] Turso - Prisma ORM https://www.prisma.io/docs/orm/v6/overview/databases/turso
[14] agent-skills/skills/turso-db/SKILL.md at main - GitHub https://github.com/tursodatabase/agent-skills/blob/main/skills/turso-db/SKILL.md
[15] Production SQLite with Turso and libSQL https://www.youtube.com/watch?v=oSbdNY2n8Qk
[16] Prompts https://modelcontextprotocol.info/docs/concepts/prompts/
[17] Learning Resources and Documentation https://modelcontextprotocol.be/resources
[18] Turso: A Rust Rewrite of SQLite. Setup Guide and Whether ... https://dev.to/arshtechpro/turso-a-rust-rewrite-of-sqlite-setup-guide-and-whether-its-worth-your-time-16lk
[19] Turso Sync: a much, much, much better way to sync https://turso.tech/blog/sync-benchmark







# Apparatus Core  
### Package Architecture v0.1  
**Scope:** Rust-only core package — libraries, skeleton, protocols, in-house vs open-source split, and learning map  
**Status:** Implementation-ready design baseline  
**Companion to:** The Apparatus Foundation Architecture v0.1

The Core is the **Asset Manager** of the Apparatus: a small, durable Rust system that owns identity, registries, append-only history, artifacts, policy, protocols, tasks, and the gateway. It does not own project business logic, UI product features, model training, or agent personality. Projects attach through permits; agents attach through capability grants.[1]

***

## 1. Mission of the Core

The Core package must make these operations safe, boring, and complete:

1. Admit a project  
2. Accept an immutable artifact  
3. Append a verified receipt  
4. Authorize a request against policy  
5. Run a named protocol with explicit gates  
6. Lease a bounded task to a worker  
7. Close a session into a structured handoff  
8. Expose a thin API / MCP facade  
9. Maintain itself at night  
10. Restore from backup without the original machine  

If a feature does not serve one of those, it is not Core.

***

## 2. Package topology

```text
apparatus/
├── Cargo.toml                          # workspace root
├── rust-toolchain.toml
├── deny.toml                           # cargo-deny policy
├── .cargo/config.toml
│
├── crates/
│   ├── apparatus-types/                # pure types, IDs, enums, errors
│   ├── apparatus-schema/               # JSON Schema load/validate/compile
│   ├── apparatus-crypto/               # hashing, signing, content addressing
│   ├── apparatus-time/                 # clocks, Unix ms, UUIDv7 helpers
│   ├── apparatus-store/                # SQLite/libsql + filesystem artifact store
│   ├── apparatus-registry/             # projects, tools, models, protocols, schemas
│   ├── apparatus-ledger/               # append-only receipts + integrity chain
│   ├── apparatus-policy/               # capability grants + classification gates
│   ├── apparatus-artifacts/            # intake, hash, store, retrieve, manifest
│   ├── apparatus-protocols/            # protocol definitions + state machine runner
│   ├── apparatus-tasks/                # tasks, leases, results, idempotency
│   ├── apparatus-gateway/              # HTTP API surface (Axum)
│   ├── apparatus-mcp/                  # thin MCP adapter over gateway/services
│   ├── apparatus-observe/              # tracing, metrics, structured logs
│   └── apparatus-maintenance/          # backup, integrity, dependency checks
│
├── bins/
│   ├── apparatusd/                     # main daemon
│   ├── apparatus-cli/                  # operator / developer CLI
│   └── apparatus-worker/               # controlled local worker runner
│
├── schemas/
│   ├── core/
│   │   ├── object_header.schema.json
│   │   ├── artifact.schema.json
│   │   ├── record_receipt.schema.json
│   │   ├── session_bundle.schema.json
│   │   ├── task.schema.json
│   │   ├── capability_grant.schema.json
│   │   └── protocol_run.schema.json
│   ├── protocols/
│   │   ├── project_bootstrap.v1.json
│   │   ├── artifact_intake.v1.json
│   │   ├── session_close.v1.json
│   │   ├── task_lease.v1.json
│   │   └── high_impact_action.v1.json
│   └── project_templates/
│       └── default_project.v1.yaml
│
├── migrations/
│   ├── control/
│   │   └── 0001_init.sql
│   └── project_template/
│       └── 0001_init.sql
│
├── tests/
│   ├── contract/
│   ├── integration/
│   └── restore/
│
└── docs/
    ├── CORE.md
    ├── PROTOCOLS.md
    └── DEPENDENCY_POLICY.md
```

### Crate dependency direction

```text
types
  ↑
time, crypto, schema
  ↑
store
  ↑
registry, ledger, artifacts, policy
  ↑
protocols, tasks
  ↑
gateway, mcp, maintenance, observe
  ↑
bins: apparatusd, apparatus-cli, apparatus-worker
```

Rules:

- `apparatus-types` depends on almost nothing
- lower crates never import gateway/MCP
- project modules never import private store internals
- only `gateway` and `mcp` speak to the outside world
- only `store` touches filesystem paths and SQL connections

***

## 3. What each crate owns

| Crate | Owns | Does not own |
|---|---|---|
| `apparatus-types` | IDs, headers, enums, error taxonomy, serde models | IO, SQL, networking |
| `apparatus-schema` | schema registry, compile/cache, validate payloads | business rules beyond shape |
| `apparatus-crypto` | SHA-256, Blake3, content addresses, signatures | key custody UI / HSM policy product |
| `apparatus-time` | trusted clock source, Unix ms, UUIDv7 factory | scheduling product logic |
| `apparatus-store` | control DB, project DB open/migrate, blob paths | domain interpretation of rows |
| `apparatus-registry` | projects/tools/models/protocols/modules admission | runtime execution |
| `apparatus-ledger` | receipts, hash chain, checkpoints, verify | mutation of subject objects |
| `apparatus-policy` | authorize / deny / require-approval decisions | executing approved actions |
| `apparatus-artifacts` | intake pipeline, manifests, retrieval handles | OCR, embeddings, summarization |
| `apparatus-protocols` | protocol defs + runner + gates | arbitrary agent planning |
| `apparatus-tasks` | queue, lease, result, expiry | sandbox implementation details beyond lease terms |
| `apparatus-gateway` | authn edge, HTTP routes, request IDs, rate limits | core domain algorithms |
| `apparatus-mcp` | MCP tool/resource/prompt mapping onto Core services | becoming a second API law |
| `apparatus-observe` | logs/metrics/traces conventions | business dashboards |
| `apparatus-maintenance` | backup, integrity jobs, restore drills | interactive recovery UI |

***

## 4. Core runtime services

Inside `apparatusd`, the process holds a small service graph:

```text
                    ┌──────────────────────┐
                    │     Config + Paths   │
                    └──────────┬───────────┘
                               │
                    ┌──────────▼───────────┐
                    │     ServiceContext   │
                    │ clock, crypto, store │
                    └──┬───────┬───────┬───┘
           ┌───────────┘       │       └───────────┐
           │                   │                   │
   ┌───────▼──────┐   ┌────────▼────────┐   ┌──────▼───────┐
   │ Registry     │   │ Ledger          │   │ Policy       │
   └───────┬──────┘   └────────┬────────┘   └──────┬───────┘
           │                   │                   │
   ┌───────▼───────────────────▼───────────────────▼───────┐
   │ Artifacts · Protocols · Tasks · Maintenance           │
   └───────────────────────────┬───────────────────────────┘
                               │
                    ┌──────────▼───────────┐
                    │ Gateway + MCP facade │
                    └──────────────────────┘
```

### `ServiceContext` responsibilities

- Resolve configured roots (`APPARATUS_HOME`)
- Open control DB read/write
- Provide clock and ID factory
- Provide crypto helpers
- Provide schema validator
- Expose metric handles
- Never hold request-scoped secrets longer than needed

***

## 5. Dependency stack

### 5.1 Core open-source stack (use directly)

| Crate / tool | Role | Why it fits Core |
|---|---|---|
| **tokio** | async runtime | Standard durable service runtime [2] |
| **axum** | HTTP gateway | Thin, Tower-native, production-proven API layer [2][3] |
| **tower / tower-http** | middleware | timeouts, compression, trace, CORS, request limits [2] |
| **hyper** | HTTP internals via Axum | Battle-tested transport |
| **serde / serde_json** | serialization | Boundary contracts and durable JSON objects |
| **schemars** | Rust → JSON Schema export | Keep Rust types and external schemas aligned |
| **jsonschema** | runtime JSON Schema validation | Validate envelopes and protocol payloads at the gate [4] |
| **rusqlite** (`bundled`) | SQLite access | Local-first relational substrate; portable files |
| **refinery** or **rusqlite_migration** | SQL migrations | Versioned control/project schema evolution |
| **uuid** (`v7`) | identity | Time-sortable unique IDs |
| **time** or **chrono** | timestamps | Prefer `time` for stricter modern API; either is fine if one is chosen and locked |
| **blake3** | fast content hashing | Artifact addressing, integrity, dedup |
| **sha2** | SHA-256 | Interoperable content hashes and external compatibility |
| **ed25519-dalek** | signatures | Receipt/manifest signing later |
| **rand** | secure randomness | Nonces, tokens where needed |
| **thiserror / anyhow** | error handling | `thiserror` in libraries, `anyhow` in bins |
| **tracing / tracing-subscriber** | structured logs | Operational truth without ad-hoc println |
| **metrics / metrics-exporter-prometheus** | metrics | Health and maintenance telemetry |
| **clap** | CLI parsing | `apparatus-cli` and worker args |
| **camino** | UTF-8 paths | Avoid path encoding footguns |
| **fs-err** | filesystem IO with better errors | Clearer operational failures |
| **tempfile** | tests and atomic write staging | Safe write patterns |
| **tokio-util** | helpers | sync/io utilities |
| **bytes** | efficient buffers | artifact streaming |
| **mime_guess / mediatype** | media type detection | artifact intake |
| **hex** | digest encoding | stable hash rendering |
| **base64** | transport encoding | tokens/signatures where needed |
| **zeroize** | secret scrubbing | reduce secret residue in memory |
| **secrecy** | secret wrapper types | prevent accidental logging |
| **parking_lot** | locks if needed | faster/std-friendly mutexes |
| **once_cell** / std once locks | lazy static config | schema caches etc. |
| **regex** | constrained parsing only | never as primary validator |
| **semver** | version comparisons | schema/protocol version rules |
| **ignore / walkdir** | controlled tree walks | maintenance and intake scans |
| **flate2 / zstd** | compression | backup bundles and export packs |
| **tar** | archive format | export/import bundles |
| **rmp-serde** optional | MessagePack | later efficient internal transport if JSON becomes hot |

### 5.2 Strong candidates to evaluate next

| Candidate | What it unlocks | Admission caution |
|---|---|---|
| **libsql / libsql-client** | embedded replicas, remote sync options | Keep plain SQLite export path mandatory |
| **sqlx** | compile-checked SQL, async DB | Heavier than rusqlite; useful if async DB becomes dominant |
| **object_store** | S3/local/Azure blob interface | Only when multi-backend artifact storage is real |
| **opendal** | unified storage abstraction | Powerful, but abstraction cost; wait for need |
| **jsonwebtoken** | JWT edge auth | Useful for remote clients; not first identity system |
| **oauth2 / openidconnect** | external identity providers | Multi-user phase, not M0 |
| **rustls / tokio-rustls** | TLS without OpenSSL pain | Needed for non-Tailscale remote TLS |
| **x509-parser** | cert inspection | mTLS later |
| **cacache** or custom content store | content-addressed caches | Can inspire artifact store design |
| **sled** | embedded KV | Do not replace SQLite for relational core |
| **redb** | embedded KV | Same caution |
| **tantivy** | full-text search engine | Projection layer, not core record law |
| **qdrant-client** | vector DB | Project-level intelligence only |
| **ort / candle / llama.cpp bindings** | local model runtime | Never Core authority |
| **wasmtime** | sandboxed plugin/runtime | Attractive for safe extensions later |
| **cap-std / cap-async-std** | capability-based FS access | Excellent sandbox direction for workers |
| **nix** | Unix process controls | Worker isolation primitives |
| **landlock** / seccomp crates | Linux sandboxing | High value for worker isolation on Linux |
| **iroh** | peer-to-peer backed networking | Experimental federation research |
| **tonic + prost** | gRPC | Only if binary internal RPC becomes necessary |
| **utoipa** | OpenAPI generation from Axum | Good API docs once routes stabilize |
| **insta** | snapshot testing | Excellent for protocol fixtures |
| **proptest** | property testing | High value for ledger/hash invariants |
| **criterion** | benchmarks | Intake/ledger hot paths later |
| **cargo-deny** | license/advisory/ban policy | Supply-chain gate |
| **cargo-audit** | vulnerability advisories | CI maintenance |
| **cargo-cyclonedx** or **syft** | SBOM | release inventory |
| **just** / **mise** | developer task runner | ergonomics, not runtime |

### 5.3 Tools outside the Rust crates, but part of Core ops

| Tool | Use |
|---|---|
| **restic** | encrypted backups |
| **age / SOPS** | secret encryption at rest / in repo |
| **cosign + sigstore** | signed releases |
| **Trivy / Grype / OSV-Scanner** | vulnerability scanning |
| **Tailscale or WireGuard** | Z1 mesh |
| **systemd** | daemon supervision on Linux |
| **OpenTelemetry Collector** optional | telemetry pipeline later |

***

## 6. In-house code we must own

These are not “nice customizations.” They are the Core’s law.

### Must build in-house

| Feature | Why own it |
|---|---|
| Object header + canonical envelope | Defines every durable object |
| UUIDv7 + multi-timestamp discipline | Identity/time law |
| Content-addressed artifact manifests | Portability and integrity |
| Append-only receipt ledger format | History no module can rewrite |
| Registry admission model | Project/tool/protocol permit system |
| Capability grant vocabulary | Security boundary language |
| Policy decision engine interface | authorize / deny / require-approval |
| Protocol state machine format | Deterministic operational workflows |
| Session-close protocol | Primary continuity automation |
| Project bootstrap protocol | Compatible growth mechanism |
| Task lease semantics | Bounded worker authority |
| Classification + provenance model | Evidence integrity |
| Core error taxonomy | Stable operator/agent feedback |
| Backup/restore layout contract | Sovereignty and recovery |
| MCP/tool mapping policy | Prevents ungoverned model action |
| Dependency inventory records | Supply-chain memory |
| Maintenance job model | Nachtonderhoud as code |

### May wrap open source instead of reinventing

| Need | Prefer open source | In-house only glue |
|---|---|---|
| HTTP server | Axum/Tower | route policy + request context |
| JSON Schema validation | jsonschema + schemars | schema registry layout |
| SQLite | rusqlite/libsql | multi-DB topology + migrations |
| Hashing/signing | blake3/sha2/ed25519-dalek | receipt chain format |
| CLI | clap | command taxonomy |
| Logging/metrics | tracing/metrics | field conventions + redaction |
| Backups | restic | what/when/verify protocol |
| Packaging/signing | cosign/cargo-dist later | release policy |

***

## 7. Control database skeleton

`migrations/control/0001_init.sql` (conceptual):

```sql
-- identity and authority
CREATE TABLE principals (
  id            TEXT PRIMARY KEY,         -- uuid v7
  kind          TEXT NOT NULL,            -- human|device|service|agent|external_client
  name          TEXT NOT NULL,
  status        TEXT NOT NULL,            -- active|disabled|revoked
  created_at_ms INTEGER NOT NULL,
  meta_json     TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE projects (
  id              TEXT PRIMARY KEY,
  slug            TEXT NOT NULL UNIQUE,
  steward_id      TEXT NOT NULL,
  classification  TEXT NOT NULL,
  db_path         TEXT NOT NULL,
  artifact_root   TEXT NOT NULL,
  status          TEXT NOT NULL,          -- active|archived|suspended
  created_at_ms   INTEGER NOT NULL,
  schema_namespace TEXT NOT NULL,
  FOREIGN KEY (steward_id) REFERENCES principals(id)
);

CREATE TABLE schema_registry (
  id            TEXT PRIMARY KEY,
  name          TEXT NOT NULL,
  version       TEXT NOT NULL,
  schema_hash   TEXT NOT NULL,
  body_path     TEXT NOT NULL,
  status        TEXT NOT NULL,            -- active|deprecated
  created_at_ms INTEGER NOT NULL,
  UNIQUE(name, version)
);

CREATE TABLE protocol_registry (
  id            TEXT PRIMARY KEY,
  name          TEXT NOT NULL,
  version       TEXT NOT NULL,
  definition_path TEXT NOT NULL,
  definition_hash TEXT NOT NULL,
  status        TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  UNIQUE(name, version)
);

CREATE TABLE tool_registry (
  id            TEXT PRIMARY KEY,
  name          TEXT NOT NULL,
  version       TEXT NOT NULL,
  kind          TEXT NOT NULL,            -- core|worker|external
  risk_tier     TEXT NOT NULL,            -- A|B|C|D
  status        TEXT NOT NULL,
  spec_json     TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  UNIQUE(name, version)
);

CREATE TABLE capability_grants (
  id              TEXT PRIMARY KEY,
  subject_id      TEXT NOT NULL,
  capability      TEXT NOT NULL,
  resource_scope  TEXT NOT NULL,          -- JSON array
  constraints_json TEXT NOT NULL,
  granted_by      TEXT NOT NULL,
  created_at_ms   INTEGER NOT NULL,
  expires_at_ms   INTEGER,
  revoked_at_ms   INTEGER,
  FOREIGN KEY (subject_id) REFERENCES principals(id)
);

CREATE TABLE ledger_receipts (
  pos             INTEGER PRIMARY KEY AUTOINCREMENT,
  id              TEXT NOT NULL UNIQUE,
  subject_id      TEXT NOT NULL,
  subject_type    TEXT NOT NULL,
  operation       TEXT NOT NULL,
  actor_id        TEXT NOT NULL,
  authority_id    TEXT NOT NULL,
  project_id      TEXT,
  recorded_at_ms  INTEGER NOT NULL,
  payload_hash    TEXT NOT NULL,
  prev_hash       TEXT,
  receipt_hash    TEXT NOT NULL,
  body_json       TEXT NOT NULL
);

CREATE TABLE artifacts (
  id              TEXT PRIMARY KEY,
  project_id      TEXT,
  content_hash    TEXT NOT NULL,
  media_type      TEXT NOT NULL,
  byte_size       INTEGER NOT NULL,
  storage_path    TEXT NOT NULL,
  classification  TEXT NOT NULL,
  provenance_json TEXT NOT NULL,
  created_at_ms   INTEGER NOT NULL,
  status          TEXT NOT NULL
);

CREATE TABLE protocol_runs (
  id              TEXT PRIMARY KEY,
  protocol_name   TEXT NOT NULL,
  protocol_version TEXT NOT NULL,
  project_id      TEXT,
  state           TEXT NOT NULL,
  current_step    TEXT,
  initiated_by    TEXT NOT NULL,
  input_bundle_id TEXT,
  created_at_ms   INTEGER NOT NULL,
  updated_at_ms   INTEGER NOT NULL,
  result_json     TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE tasks (
  id              TEXT PRIMARY KEY,
  project_id      TEXT NOT NULL,
  kind            TEXT NOT NULL,
  state           TEXT NOT NULL,
  requested_by    TEXT NOT NULL,
  assigned_to     TEXT,
  input_bundle_id TEXT,
  created_at_ms   INTEGER NOT NULL,
  updated_at_ms   INTEGER NOT NULL,
  deadline_ms     INTEGER,
  spec_json       TEXT NOT NULL
);

CREATE TABLE task_leases (
  id              TEXT PRIMARY KEY,
  task_id         TEXT NOT NULL,
  worker_id       TEXT NOT NULL,
  issued_by       TEXT NOT NULL,
  issued_at_ms    INTEGER NOT NULL,
  expires_at_ms   INTEGER NOT NULL,
  revoked_at_ms   INTEGER,
  grant_ids_json  TEXT NOT NULL,
  FOREIGN KEY (task_id) REFERENCES tasks(id)
);

CREATE INDEX idx_ledger_subject ON ledger_receipts(subject_id);
CREATE INDEX idx_artifacts_hash ON artifacts(content_hash);
CREATE INDEX idx_tasks_state ON tasks(state);
CREATE INDEX idx_grants_subject ON capability_grants(subject_id);
```

Project DBs get a narrower schema: project objects, claims, evidence, local sessions, local task mirrors if needed, and projection metadata. They do **not** get global principal authority tables.

***

## 8. Core types skeleton

```rust
// crates/apparatus-types/src/lib.rs

pub type UuidV7 = uuid::Uuid;
pub type UnixMs = i64;
pub type Sha256Hex = String;
pub type Blake3Hex = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    Public,
    Shared,
    Project,
    Private,
    Sensitive,
    Restricted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceKind {
    HumanStated,
    HumanAuthored,
    SystemMeasured,
    SystemGenerated,
    AgentGenerated,
    ExternalImport,
    ExternalModelOutput,
    Derived,
    Corrected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectHeader {
    pub id: UuidV7,
    pub type_name: String,
    pub schema_version: String,
    pub created_at_unix_ms: UnixMs,
    pub recorded_at_unix_ms: UnixMs,
    pub occurred_at_unix_ms: Option<UnixMs>,
    pub owner_id: UuidV7,
    pub project_id: Option<UuidV7>,
    pub classification: Classification,
    pub provenance: Vec<ProvenanceKind>,
    pub source_refs: Vec<String>,
    pub content_hash: Option<Sha256Hex>,
    pub status: ObjectStatus,
    pub supersedes_id: Option<UuidV7>,
    pub correlation_id: Option<UuidV7>,
    pub causation_id: Option<UuidV7>,
}
```

### What this handles

- Stable identity
- Explicit time semantics
- Ownership and project scope
- Evidence-grade provenance
- Logical supersession instead of silent mutation

***

## 9. Essential traits and functions

### 9.1 Clock and IDs

```rust
pub trait Clock: Send + Sync {
    fn now_unix_ms(&self) -> UnixMs;
}

pub trait IdFactory: Send + Sync {
    fn new_v7(&self) -> UuidV7;
}
```

**Handles:** deterministic testing with fake clocks; production UUIDv7 issuance; refusal of client-supplied “now” as authority.

***

### 9.2 Schema validation

```rust
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

pub trait SchemaValidator: Send + Sync {
    fn validate_json(
        &self,
        schema_name: &str,
        schema_version: &str,
        value: &serde_json::Value,
    ) -> Result<(), Vec<ValidationError>>;
}
```

**Handles:** every gateway write path validates before side effects; protocol inputs/outputs validated by versioned schema; unknown schema versions rejected.

***

### 9.3 Artifact store

```rust
pub struct ArtifactPut {
    pub project_id: Option<UuidV7>,
    pub bytes: bytes::Bytes,
    pub media_type: String,
    pub original_filename: Option<String>,
    pub classification: Classification,
    pub provenance: Vec<ProvenanceKind>,
    pub source_refs: Vec<String>,
}

pub trait ArtifactStore: Send + Sync {
    fn put(&self, actor: PrincipalId, input: ArtifactPut) -> Result<Artifact>;
    fn open(&self, id: UuidV7) -> Result<ArtifactHandle>;
    fn metadata(&self, id: UuidV7) -> Result<Artifact>;
}
```

**Handles:**

- hash-first content addressing
- atomic write (temp file → fsync → rename)
- dedup by hash optional, identity still unique
- no overwrite of blob bytes
- retrieval by ID only through authorized path

Snippet of intake logic:

```rust
// conceptual
let sha = sha256_hex(&bytes);
let b3  = blake3_hex(&bytes);
let path = layout.blob_path(&sha); // artifacts/sha256/ab/cd/<fullhash>

if !path.exists() {
    write_atomic(&path, &bytes)?;
}

let artifact = Artifact {
    header: header_new(...),
    content_hash: sha,
    secondary_hash: b3,
    byte_size: bytes.len() as u64,
    storage_path: path.to_string(),
    ...
};

ledger.append(receipt_for_create(&artifact))?;
db.insert_artifact(&artifact)?;
```

***

### 9.4 Ledger

```rust
pub trait Ledger: Send + Sync {
    fn append(&self, receipt: NewReceipt) -> Result<Receipt>;
    fn get(&self, id: UuidV7) -> Result<Receipt>;
    fn verify_chain(&self) -> Result<IntegrityReport>;
    fn checkpoint(&self) -> Result<Checkpoint>;
}
```

**Handles:**

- monotonic position
- `prev_hash` linkage
- receipt hash over canonical payload
- verify job in maintenance window
- never update old receipt rows

Canonical receipt hash input should be stable:

```text
receipt_hash = sha256(
  id || subject_id || subject_type || operation ||
  actor_id || authority_id || recorded_at_ms ||
  payload_hash || prev_hash
)
```

***

### 9.5 Policy engine

```rust
#[derive(Debug, Clone)]
pub enum PolicyEffect {
    Allow { grant_id: UuidV7 },
    Deny { reason: String, rule_id: String },
    RequireApproval { reason: String, approver_roles: Vec<String> },
}

pub struct AuthzRequest {
    pub principal_id: UuidV7,
    pub action: String,              // "artifact.read"
    pub resource: String,            // "project:mapa/artifact:..."
    pub network_zone: NetworkZone,
    pub context: serde_json::Value,  // request metadata, never secrets
}

pub trait PolicyEngine: Send + Sync {
    fn authorize(&self, req: &AuthzRequest) -> Result<PolicyEffect>;
}
```

**Handles:**

- capability match by action + resource scope
- expiry and revocation
- classification ceilings by zone
- high-impact actions forced into approval
- deny-by-default

***

### 9.6 Protocol runner

```rust
pub struct ProtocolDefinition {
    pub name: String,
    pub version: String,
    pub steps: Vec<ProtocolStep>,
}

pub struct ProtocolStep {
    pub id: String,
    pub kind: StepKind, // Validate | Authorize | Transform | Persist | WaitApproval | CallTool | Emit
    pub input_schema: Option<SchemaRef>,
    pub output_schema: Option<SchemaRef>,
    pub on_fail: OnFail, // Abort | Compensate | Wait | Incident
}

pub trait ProtocolRunner: Send + Sync {
    fn start(&self, req: StartProtocol) -> Result<ProtocolRun>;
    fn advance(&self, run_id: UuidV7, event: ProtocolEvent) -> Result<ProtocolRun>;
    fn get(&self, run_id: UuidV7) -> Result<ProtocolRun>;
}
```

**Handles:**

- explicit step graph
- durable run state
- no hidden side effects outside step kinds
- every transition emits ledger receipt
- approval gates are first-class states

***

### 9.7 Task broker

```rust
pub trait TaskBroker: Send + Sync {
    fn submit(&self, task: NewTask) -> Result<Task>;
    fn lease(&self, worker_id: UuidV7, filter: LeaseFilter) -> Result<Option<TaskLease>>;
    fn heartbeat(&self, lease_id: UuidV7) -> Result<()>;
    fn complete(&self, lease_id: UuidV7, result: TaskResult) -> Result<Task>;
    fn fail(&self, lease_id: UuidV7, err: TaskError) -> Result<Task>;
    fn expire_due_leases(&self, now: UnixMs) -> Result<u64>;
}
```

**Handles:**

- single-worker lease semantics
- short expiry
- idempotent completion
- automatic expiry in maintenance
- result bundle validation before accept

***

## 10. Core protocols to implement first

### P0 — `project.bootstrap.v1`

**Input:** slug, steward, classification, template  
**Steps:**

1. Validate request schema  
2. Authorize `project.create`  
3. Create project row  
4. Create project DB from template migrations  
5. Create artifact root  
6. Register default tool allow-list  
7. Write bootstrap receipt  
8. Return project descriptor  

**Done when:** project is isolated, restorable, and visible in registry.

***

### P1 — `artifact.intake.v1`

**Input:** bytes/stream, project, classification, provenance  
**Steps:**

1. Validate headers/metadata  
2. Authorize `artifact.write`  
3. Hash bytes  
4. Store blob immutably  
5. Insert artifact metadata  
6. Append ledger receipt  
7. Queue optional derived extraction job  
8. Return artifact ID + hashes  

**Done when:** bytes are recoverable by ID and integrity-verifiable.

***

### P2 — `session.close.v1`

This is the highest-leverage early protocol.

**Input:** `SessionBundle`  
**Steps:**

1. Validate bundle schema  
2. Authorize `session.close` for project  
3. Ensure all referenced artifacts exist or intake them  
4. Persist session object  
5. Persist declared tasks/decisions/unresolved questions  
6. Build handoff artifact (deterministic JSON + optional markdown rendering later)  
7. Append one close receipt + child receipts  
8. Mark session `processed` or `review_required`  
9. Queue projection/index update  

**Done when:** a worker can finish work by submitting one bundle and walking away.

***

### P3 — `task.lease.v1`

**Input:** worker identity + capabilities  
**Steps:**

1. Authenticate worker  
2. Find oldest matching queued task  
3. Create lease with expiry and narrowed grants  
4. Mark task `leased`  
5. Return task input bundle references only  
6. Accept completion/failure only from lease holder before expiry  

**Done when:** no worker can act outside lease scope or after expiry.

***

### P4 — `high_impact.action.v1`

**Input:** proposal artifact + requested action  
**Steps:**

1. Validate proposal  
2. Policy returns `RequireApproval`  
3. Create decision request  
4. Wait for owner/steward decision  
5. On approve, execute narrow tool call  
6. Persist result + evidence  
7. On deny/expire, record terminal state  

**Done when:** irreversible actions cannot occur from model initiative alone.

***

### P5 — `maintenance.nightly.v1`

**Steps:**

1. Verify ledger chain  
2. DB integrity_check  
3. Snapshot control + active project DBs  
4. Backup artifact tree incrementally  
5. Expire leases/grants  
6. Emit health metrics  
7. Write maintenance receipt  
8. On failure, open incident record  

***

## 11. Gateway surface (minimal)

`apparatus-gateway` exposes only stable routes:

```text
POST   /v1/projects
GET    /v1/projects/:id

POST   /v1/artifacts
GET    /v1/artifacts/:id
GET    /v1/artifacts/:id/content

POST   /v1/sessions/close
GET    /v1/sessions/:id

POST   /v1/tasks
POST   /v1/tasks/lease
POST   /v1/tasks/leases/:id/complete
POST   /v1/tasks/leases/:id/fail

POST   /v1/protocols/:name/@:version/runs
GET    /v1/protocols/runs/:id
POST   /v1/protocols/runs/:id/events

GET    /v1/registry/tools
GET    /v1/registry/schemas/:name/@:version

GET    /healthz
GET    /readyz
GET    /metrics
```

Every mutating route:

1. Authenticate  
2. Create request ID  
3. Validate JSON schema  
4. Authorize  
5. Execute protocol/service  
6. Append receipts  
7. Return typed response  

***

## 12. MCP adapter mapping

MCP is a facade. It must not invent a second authority model.

| MCP tool | Core protocol/service | Notes |
|---|---|---|
| `apparatus.project.get` | registry read | read-only |
| `apparatus.artifact.get_metadata` | artifacts.metadata | no raw restricted content by default |
| `apparatus.artifact.submit` | `artifact.intake.v1` | size/class limited |
| `apparatus.session.close` | `session.close.v1` | preferred agent exit path |
| `apparatus.task.submit_result` | task complete | lease required |
| `apparatus.search` later | projection query | not M0 |
| `apparatus.shell` | **never** | not a Core tool |

MCP resources should expose:

- approved project descriptors
- protocol descriptors
- schema descriptors
- task lease descriptors

Not:

- raw SQL
- secret handles
- arbitrary filesystem paths
- admin grant APIs

***

## 13. Daemon layout and config

```toml
# apparatus.toml

home = "/var/lib/apparatus"
listen = "127.0.0.1:7420"
network_zone = "z2"

[store]
control_db = "control.db"
projects_dir = "projects"
artifacts_dir = "artifacts"
ledger_dir = "ledger"

[security]
require_auth = true
default_deny = true
max_artifact_bytes = 52428800

[maintenance]
nightly_cron = "0 3 * * *"
backup_dir = "backups"
verify_ledger = true

[observe]
log_json = true
metrics = true
```

`apparatusd` boot sequence:

1. Load config  
2. Resolve absolute paths  
3. Open control DB + migrate  
4. Load schema registry into memory cache  
5. Verify latest ledger checkpoint  
6. Start maintenance scheduler  
7. Start task expiry loop  
8. Bind gateway on localhost / private interface  
9. Optionally start MCP stdio/HTTP bridge process  
10. Emit `core.booted` receipt  

***

## 14. Worker model

`apparatus-worker` is intentionally dumb:

```text
loop:
  lease = POST /v1/tasks/lease
  if none: sleep
  fetch input artifacts metadata/content as granted
  execute allowed tool in sandbox
  write output artifacts via intake API
  complete/fail lease
```

Worker constraints:

- no direct control DB access
- no direct artifact filesystem access
- credentials are short-lived
- tools allow-listed per lease
- network disabled unless grant says otherwise
- all outputs re-enter through Core intake

This is the difference between “an agent with SSH” and “a worker in a governed machine.”

***

## 15. Suggested Cargo workspace dependencies (root sketch)

```toml
[workspace]
resolver = "2"
members = [
  "crates/*",
  "bins/*",
]

[workspace.package]
edition = "2021"
license = "MIT OR Apache-2.0"
version = "0.1.0"

[workspace.dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros", "fs", "io-util", "time", "signal"] }
axum = "0.8"
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "limit", "timeout", "request-id"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "0.8"
jsonschema = "0.28"
rusqlite = { version = "0.32", features = ["bundled"] }
uuid = { version = "1", features = ["v7", "serde"] }
blake3 = "1"
sha2 = "0.10"
ed25519-dalek = "2"
thiserror = "2"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
metrics = "0.24"
clap = { version = "4", features = ["derive"] }
camino = "1"
fs-err = "3"
bytes = "1"
hex = "0.4"
zeroize = "1"
secrecy = "0.10"
semver = "1"
time = { version = "0.3", features = ["serde", "formatting", "parsing"] }
```

Pin exact versions in the lockfile; the numbers above are indicative starting points, not a substitute for current crates.io resolution at implementation time.

***

## 16. Test strategy for Core

### Unit

- header serialization stability
- receipt hash stability
- capability scope matching
- schema reject/accept fixtures
- lease expiry edge cases

### Integration

- bootstrap → intake → session close → restore
- unauthorized read denied and receipted
- worker cannot complete another worker’s lease
- ledger verify detects tampered receipt

### Contract

- JSON fixtures for every protocol input/output
- OpenAPI/MCP tool descriptors match schemas

### Restore drill

- build DB + artifacts
- backup
- destroy home
- restore
- verify chain + artifact hashes + project open

A Core that cannot restore is not a Core.

***

## 17. Implementation order inside Core

### Sprint A — pure law

- `apparatus-types`
- `apparatus-time`
- `apparatus-crypto`
- `apparatus-schema`
- receipt hash golden tests

### Sprint B — durable memory

- `apparatus-store`
- control migrations
- `apparatus-ledger`
- `apparatus-artifacts`
- CLI: `artifact put/get`, `ledger verify`

### Sprint C — authority

- `apparatus-registry`
- `apparatus-policy`
- principals + grants
- bootstrap protocol

### Sprint D — work

- `apparatus-protocols`
- `apparatus-tasks`
- `session.close.v1`
- `task.lease.v1`

### Sprint E — edges

- `apparatus-gateway`
- `apparatus-cli`
- `apparatus-worker`
- `apparatus-mcp` thin facade
- `apparatus-maintenance`
- `apparatus-observe`

Do not start with MCP or agents. Start with artifacts, receipts, and bootstrap.

***

## 18. Design constraints that keep Core elegant

1. **One mutating path in:** services mutate state; bins/UI/MCP only call services.  
2. **No ORM maze:** explicit SQL for Core tables.  
3. **No plugin ABI in M0:** protocols and tools are data + supervised workers.  
4. **No global shared mutable “context soup”:** pass `ServiceContext` and request-scoped `AuthContext`.  
5. **Serializable boundaries:** if it cannot be JSON-schema’d, it is not an external API object.  
6. **Deletes are logical:** physical cleanup is a maintenance protocol with retention rules.  
7. **Projection envy is a sin:** search/embeddings/graphs stay out until intake/ledger/policy are dull and reliable.  
8. **Small public surface:** fewer routes/tools beat a clever universe of methods.

***

## 19. Learning / inspiration map

These are chosen because each teaches one thing the Core needs.

### 19.1 Systems and architecture (deep study)

| Source | Why study it | What to steal for Apparatus |
|---|---|---|
| **NASA Flight Software / FSW patterns** (Core Flight System concepts, command/telemetry discipline) | High-reliability command handling, state, fault response | Command acceptance, reject-with-reason, telemetry as first-class, safe mode thinking |
| **NASA CFS (Core Flight System)** | Modular flight software with clear app boundaries | Core apps as isolated services with message contracts |
| **Apollo / JPL operations culture writings** (incl. failure review discipline) | Evidence, procedures, no silent improvisation under pressure | Protocol checklists, anomaly records, go/no-go gates |
| **Rijkswaterstaat / Delta Programme governance model** (your own research corpus) | Separation of roles, long memory, maintenance windows | Owner/manager/provider split, nachtonderhoud, reserve capacity [5][6][1] |
| **Dutch basisregistraties / authentiek gegeven concept** | One authoritative source per fact | Registry ownership of authentic fields, once-only reuse [6] |
| **Portbase model** | Neutral coordination utility | Gateway as coordination layer, not owner of all business logic [5] |

### 19.2 Software projects to read as code

| Project | Focus | What to look at |
|---|---|---|
| **Git** | content-addressed objects + refs | Object store by hash, immutability, refs as movable names over immutable content |
| **SQLite** | reliable embedded database philosophy | “Lite” as disciplined scope; file durability; testing culture; corrosion-resistant simplicity |
| **rusqlite** | clean Rust boundary over SQLite | API ergonomics around a mature C core without over-abstracting |
| **Axum + Tower examples** | composable service middleware | Request-id, timeouts, layers, state injection without global mutability [2] |
| **Tokio mini-redis / Tokio tutorials** | small durable network services | Connection lifecycle, graceful shutdown, supervised tasks |
| **OpenBao / Vault architecture docs** (concepts, not necessarily adoption) | secrets, leases, audit | Short-lived credentials, audit log centrality, lease expiry |
| **sigstore / cosign** | signing and provenance | Release evidence, keyless/signing flows, verification mindset |
| **restic** | backup correctness | Dedup, encryption, verify/restore as equal to backup |
| **OPA (Open Policy Agent)** | policy as data | Separate policy decision from policy enforcement; deterministic decisions |
| **Cedar (AWS)** or **OSO polar concepts** | authorization modeling | Explicit principal/action/resource shape |
| **Temporal** (read conceptually) | durable workflow execution | Workflow histories; wait-states; deterministic replay ideas — then implement a *much smaller* protocol runner |
| **FFmpeg** | codec/protocol modularity + operational ruthlessness | Tiny focused tools, sharp interfaces, “does one hard job extremely well,” regression discipline |
| **tinygrad** | minimalist compute graph ambition | Keep core small enough to hold in one head; fight abstraction obesity |
| **comma.ai openpilot architecture overviews** | real-world agentic system with hard safety constraints | Runtime vs training split, safety-critical path minimalism, logging everything that matters |
| **Firecracker** | microVM isolation elegance | Inspiration for later worker isolation boundaries |
| **gVisor / Bubblewrap / Bubblewrap-adjacent sandbox tools** | syscall/FS isolation | Worker sandbox patterns on Linux |
| **Content-addressed stores: IPFS specs (selectively), Git, Nix store** | deterministic storage | Hash → bytes absolute truth; names are indirection |
| **Nix / NixOS module system** (conceptually) | pure configuration and reproducibility | Declarative system config; pin inputs; pure builds as aspiration |
| **QGIS / PDAL / GDAL** only if geospatial appears | mature C/Rust-adjacent data pipelines | Not Core, but excellent “library quality” reference for domain engines |
| **Stockfish** | specialized perfect-information engine | How a narrow engine exposes a clean protocol (UCI) and stays world-class by scope control |

### 19.3 Specific ideas worth hunting inside codebases

| Look for this | In | Why |
|---|---|---|
| Atomic write patterns (`write tmp → fsync → rename`) | SQLite, various Rust FS crates, Git object write paths | Artifact store durability |
| Hash chain / Merkle-ish verification | Git, transparency logs, simple blockchain-free ledgers | Ledger verify jobs |
| Capability tokens / attenuations | Tahoe-LAFS concepts, macaroons papers, modern cap-std | Grants that can only shrink |
| Idempotency keys | Stripe API docs + good HTTP APIs | Task completion and intake retries |
| Envelope/event validation before persist | your RWS Blueprint event envelope thinking | One law for all writes [1] |
| Reject-unknown-fields discipline | strict serde + schema validation | Prevent silent API drift |
| Graceful shutdown order | Tokio service examples | No half-written receipts on stop |
| Snapshot + WAL thinking | SQLite backup APIs | Control DB backup correctness |
| “Machinery not magic” CLI UX | `git`, `ffmpeg`, `restic`, `sqlite3` | Core CLI should feel like infrastructure tooling |

### 19.4 Books and long-form documents

| Book / doc | Use |
|---|---|
| ***Designing Data-Intensive Applications*** — Martin Kleppmann | Logs, authenticity of data, derived views, replay, storage tradeoffs |
| ***Database Internals*** — Alex Petrov | Enough engine literacy to respect SQLite and not reinvent it badly |
| ***Thinking in Systems*** — Donella Meadows | Stocks/flows/feedback; maintenance and buffers as system design |
| ***Release It!*** — Michael Nygard | Stability patterns, timeouts, backpressure, critical dependency control |
| ***The Phoenix Project*** / ***The Unicorn Project*** (selective) | Ops feedback loops; not literal adoption of enterprise DevOps theater |
| ***An Elegant Puzzle*** — Will Larson | Org/systems boundaries; useful later for multi-steward operation |
| **RFC 4122 / UUIDv7 drafts & final specs** | ID semantics |
| **JSON Schema specification** | Boundary law |
| **OpenAPI specification** | Gateway contract publication |
| **MCP specification** | Edge adapter only; study tools/resources/prompts separation |
| **SLSA / in-toto provenance docs** | Supply-chain evidence model |
| **OWASP ASVS** (selected controls) | Concrete security control checklist |
| **SEI CERT Rust Secure Coding rules** | Unsafe avoidance and secure Rust habits |
| **NASA Software Engineering Handbook / NPR practices** (selected) | Gate reviews, assurance evidence, configuration discipline |
| **“The Architecture of Open Source Applications”** essays | Case studies in real system boundaries |
| **SQLite’s “Literal” documentation pages** (“How SQLite Is Tested”, “Atomic Commit”) | Durability worldview |

### 19.5 Papers / concepts (even if you only read summaries deeply)

| Concept | Why it matters |
|---|---|
| **Capability-based security** | Core grants should behave like capabilities, not ambient authority |
| **Command-query separation** | Read paths and mutate paths stay clean |
| **Event sourcing (tempered)** | Ledger is not full ES theater; borrow receipt/replay ideas carefully |
| **CALM theorem** (coordination avoidance) | Helps decide what must be centralized in Core vs local project |
| **End-to-end principle** | Keep Core dumb where project intelligence can live at edges |
| **Crash-only software** | Design so recovery is normal, not exceptional |
| **Defense in depth** | Zone separation + policy + sandbox + audit |

### 19.6 Apps and operational systems to study as products

| System | What to notice |
|---|---|
| **Linear / Jira (selectively)** | Issue state machines, but avoid their complexity max |
| **GitHub Actions** | Job permissions, least privilege tokens, ephemeral runners |
| **1Password / Bitwarden architecture docs** | Secret isolation mental models |
| **Tailscale ACL policy model** | Identity-aware network policy as code |
| **Cloudflare Workers limits model** | Hard resource ceilings as design, not punishment |
| **Postgres row-level security** (concept) | Inspiration for project isolation patterns later |
| **Kubernetes admission controllers** (concept only) | Permit/admit pattern for modules — do not adopt K8s early |
| **FFmpeg filter graphs** | Explicit pipeline composition with clear frames/packets boundaries |
| **tmux / systemd** | Session and service supervision minimalism |

### 19.7 One-line inspirations worth keeping above the desk

- **Git:** *names move; content does not.*  
- **SQLite:** *a database should be a file you can email.*  
- **FFmpeg:** *sharp tools, brutal reliability, no lifestyle framework.*  
- **tinygrad:** *if the core cannot stay small, the architecture is lying.*  
- **NASA style:** *every command either executes under rule or is rejected with a reason.*  
- **RWS style:** *maintenance is part of the machine, not an afterthought.*[1]
- **Portbase style:** *coordination infrastructure should not become a rent-seeking brain.*[5]

***

## 20. What to study first before writing serious Core code

If you only do a short, high-yield preparation pass, do this sequence:

1. **SQLite atomic commit docs** + “how SQLite is tested”  
2. **Git object model** (blobs/trees/commits/refs) — half a day  
3. **Axum + Tower middleware model** — build one tiny authenticated service  
4. **Kleppmann DDIA chapters on logs, replication, derived views**  
5. **Your RWS Blueprint authority/canon sections again** with Core eyes[1]
6. **OPA or Cedar concepts** for policy decision shape  
7. **restic backup/restore model**  
8. **MCP spec only at the edge**, after Core intake/ledger works  
9. **FFmpeg mental model** of modular tools with strict stream contracts  
10. **one NASA CFS or FSW overview** for command/telemetry discipline  

That set is enough to start Sprint A/B without drowning.

***

## 21. Definition of done for Apparatus Core v0.1

The Core package is real when:

1. `apparatusd` boots from a clean home directory  
2. `apparatus project bootstrap` creates an isolated project  
3. `apparatus artifact put` stores immutable bytes with hash + receipt  
4. `apparatus ledger verify` passes  
5. `apparatus session close` accepts a bundle and produces a handoff artifact  
6. a worker can lease one task and cannot exceed its grant  
7. unauthorized requests are denied and recorded  
8. nightly maintenance snapshot + restore drill succeeds  
9. MCP facade, if enabled, only calls the same services as CLI/HTTP  
10. a second machine can restore the home directory and continue  

***

## 22. Closing

The Core should feel closer to **SQLite + Git + a tiny NASA command router** than to an AI agent framework. Rust is the right language for that boundary: explicit types, strong ownership, one static daemon, boring durability.

Build the law first:

> **identity → schema → artifact → receipt → policy → protocol → lease**

Everything else — models, search, MCP richness, beautiful React consoles, distributed replicas — can wait until those seven verbs are dull, tested, and recoverable.

[7] GitHub - blue-build/jsonschema: A high-performance JSON Schema validator for Rust https://github.com/blue-build/jsonschema
[8] Rust Web Frameworks in 2026: Axum vs Actix vs Rocket vs ... https://www.youtube.com/watch?v=d6VWjKvr4_I
[9] How do I validate JSON using an existing schema file in Rust? https://stackoverflow.com/questions/44733603/how-do-i-validate-json-using-an-existing-schema-file-in-rust
[10] libsql-rusqlite 0.33.0 https://docs.rs/crate/libsql-rusqlite/latest
[11] Getting Started with Axum - Rust's Fastest-Growing Web Framework https://www.rustfinity.com/blog/axum-rust-tutorial
[12] Complete Axum 0.8 Tutorial: Build Production REST APIs in ... https://www.youtube.com/watch?v=Ka7mRKsTCyE
[13] GitHub - json-schema-language/json-schema-language-rust: A Rust implementation of JSON Schema Language https://github.com/json-schema-language/json-schema-language-rust
[14] foundation_jsonschema 0.0.1 - Docs.rs https://docs.rs/crate/foundation_jsonschema/latest
[15] json-schema-validator-core — Rust parser // Lib.rs https://lib.rs/crates/json-schema-validator-core
[16] Rust Web Frameworks in 2026: Axum vs Actix Web vs Rocket ... https://aarambhdevhub.medium.com/rust-web-frameworks-in-2026-axum-vs-actix-web-vs-rocket-vs-warp-vs-salvo-which-one-should-you-2db3792c79a2
[17] Rust Web Backend Frameworks 2026 Deep Dive - Axum, Actix-web ... https://www.youngju.dev/blog/culture/2026-05-16-rust-web-backend-frameworks-2026-axum-actix-web-rocket-poem-loco-pavex-salvo-warp-deep-dive.en
[18] s-panferov/valico: Rust JSON Schema validator and ... https://github.com/s-panferov/valico


# Apparatus Core  
## Decision Record ADR-0001  
### Open-Source Alignment & Build Split  
**Status:** Accepted for Core v0.1 planning  
**Date:** 2026-07-23  
**Supersedes:** Gemini open-source v1 enthusiasm stack; interim compliance rewrite  
**Based on:** Apparatus Foundation Architecture, Core Package Architecture, Deep Research Report v2, and adversarial review  

This record freezes the choices we are willing to build against. It separates **locked defaults**, **selected external solutions**, **in-house law**, **spikes required before freeze**, and **explicit rejects**.[1]

***

## 1. Decision summary

Apparatus Core will be a **small Rust daemon** with:

- proven **SQLite** as the default substrate  
- **in-house** artifact store, ledger, protocols, leases, and capability vocabulary  
- **Cedar** as the leading pure policy evaluator behind an Apparatus trait  
- **official MCP Rust SDK (`rmcp`)** as a thin edge facade only  
- **OS-backed worker isolation** before broad agent tooling  
- **no** experimental DB, enterprise MCP, vector canon, or ambient shell/FS tools in v0.1  

> **Apparatus owns the institution.  
> Open source supplies proven mechanisms.  
> AI remains a supervised worker at the edge.**[1]

***

## 2. Locked architectural defaults

These are law for Core v0.1 unless a listed reversal bar is met.

| ID | Decision | Rationale |
|---|---|---|
| D1 | Core language is **Rust** | Durability, ownership, static binary, boundary safety |
| D2 | Default store is **SQLite via `rusqlite`** | File-local, proven, exportable, restorable [1] |
| D3 | All storage access sits behind **traits/adapters** | Future libSQL/Limbo possible without rewrite [1] |
| D4 | Canonical data = **artifacts + ledger + relational DBs**; indexes are disposable | Prevents projection-as-truth [1] |
| D5 | **Custom CAS** for artifacts | Provenance and atomic write path must be owned [1] |
| D6 | **Custom append-only ledger** | History and deny/failure records are Core law [1] |
| D7 | **Custom task/lease broker** | Capability-narrowed leases cannot be generic job-queue semantics [1] |
| D8 | Policy decisions via **`PolicyEngine` trait**; Cedar is first implementation | Replaceable; workflow not outsourced [1] |
| D9 | `RequireApproval` is **Apparatus protocol state**, not Cedar workflow state | Cedar remains stateless Allow/Deny evaluator [1] |
| D10 | MCP is **edge-only**, zero inherent authority | Confused-deputy containment [1] |
| D11 | Workers never open Core DBs or artifact roots directly | One law for exchange |
| D12 | Network zones **Z0 / Z1 / Z2** enforced by bind address, auth, and policy | External AI never touches Z2 stores [1] |
| D13 | Linux workers first; macOS sandbox best-effort until spiked | Landlock path is clearer than Seatbelt maturity [1] |
| D14 | Dependency admission via **cargo-deny + cargo-audit + SBOM + signed releases** | Supply chain is infrastructure [1] |
| D15 | Build order is fixed: **base → authority → work → edge → projections** | Stops integration enthusiasm [1] |

***

## 3. Reversal bars

Defaults reverse only with evidence.

| Default | Minimum evidence to reverse |
|---|---|
| SQLite/`rusqlite` default | Candidate engine shows stable semver ≥ 1 year, crash-recovery parity proof, Apparatus workload fuzzing, plain-file export compatibility [1] |
| Cedar as policy impl | Alternative embeds cleanly in Rust, equal or better auditability, no weaker local-first story |
| Custom leases | External queue can express capability grants, expiry, idempotent complete, ledgered failures without owning our law |
| MCP official SDK | Spike fails; another thin SDK is simpler and equally maintained |
| Linux-first workers | macOS isolation proves equal enforceability with maintainable tooling |
| Defer vectors | Product need + rebuildable projection design + portable install story |

***

## 4. Selected external libraries & projects

### 4.1 Tier A — foundation now

| Component | Selection | Role in Apparatus | Notes |
|---|---|---|---|
| Async runtime | **tokio** | daemon, gateway, timers | Standard Rust service runtime |
| HTTP gateway | **axum + tower + tower-http** | authenticated API edge | Request IDs, limits, timeouts |
| DB binding | **rusqlite (`bundled`)** | control DB + project DBs | WAL mode; `spawn_blocking` for queries [1] |
| Migrations | **refinery** or **rusqlite_migration** | schema evolution | Pick one in M0 spike; pin it |
| Serde | **serde / serde_json** | envelopes and API payloads | Boundary objects are JSON-schema’d |
| Schema export | **schemars** | Rust type → JSON Schema | Keep contracts aligned |
| Schema validate | **jsonschema** | admit/reject payloads | Gate every mutating write |
| IDs | **uuid (`v7`)** | object identity | Time-sortable IDs |
| Time | **time** | Unix ms + formatting | One time library only |
| Hashing | **sha2**, **blake3** | content addressing + receipt hashing | SHA-256 primary interoperability; Blake3 secondary/fast |
| Signatures | **ed25519-dalek** | optional receipt/manifest signatures later | Not required to store first bytes |
| Errors | **thiserror** (libs), **anyhow** (bins) | error surfaces | Stable taxonomy in types crate |
| Logging | **tracing** + **tracing-subscriber** | structured logs | JSON logs in daemon |
| Metrics | **metrics** (+ Prometheus exporter optional) | health/maintenance | Keep light in v0.1 |
| CLI | **clap** | `apparatus` / worker CLIs | Operator surface |
| Paths/FS helpers | **camino**, **fs-err**, **tempfile** | safer path/IO ergonomics | CAS temp files |
| Bytes | **bytes**, **hex** | buffers and digest rendering | Intake/streaming |
| Versions | **semver** | schema/protocol version compares | Registry admission |
| Supply chain | **cargo-deny**, **cargo-audit** | CI dependency law | Deny unknown licenses/advisories [1] |

### 4.2 Tier A/B — authority & secrecy now/next

| Component | Selection | Role | Status |
|---|---|---|---|
| Policy engine | **cedar-policy** | pure Allow/Deny evaluation | **Adopt as first `PolicyEngine` impl** after spike [1] |
| Secret wrappers | **secrecy**, **zeroize** | prevent accidental log/leak of secrets | Adopt with policy that secrets never enter ledger/model context [1] |
| TLS | **rustls / tokio-rustls** | remote TLS when not mesh-only | When Z1/Z0 HTTPS needed |
| Encrypted secret files | **age** and/or **SOPS** | secrets at rest outside ledger | Ops path, not Core logic |

### 4.3 Tier B — work & edge next

| Component | Selection | Role | Phase |
|---|---|---|---|
| MCP facade | **official `modelcontextprotocol/rust-sdk` (`rmcp`, `rmcp-macros`)** | thin tool/resource mapping | M2 after protocols exist [2][3][1] |
| Capability FS | **cap-std** (+ cap-tempfile as needed) | worker path bounding | M1 worker path [1] |
| Linux kernel FS policy | **landlock** crate / Landlock LSM | hard path deny at kernel | M1 Linux workers [1] |
| Optional stronger sandbox | **bubblewrap** | namespace isolation helper | M1/M2 where useful |
| Compression/archive | **zstd** / **flate2**, **tar** | export/backup bundles | M1 maintenance |
| OpenAPI later | **utoipa** optional | route docs once stable | after gateway stabilizes |

### 4.4 Tier C — derived intelligence later

| Component | Selection | Role | Phase |
|---|---|---|---|
| Vector projection | **sqlite-vec** leading candidate | rebuildable semantic index | M3+ only; never canon [1] |
| FTS | SQLite FTS5 and/or **tantivy** later | text search projections | after intake quality exists |
| Local models | out of Core; worker/provider concern | inference only under lease | not Core authority |

### 4.5 Tier D — adapter watchlist only

| Project | Position | Reversal condition |
|---|---|---|
| **Limbo / Turso rewrite** | experimental adapter slot | maturity + crash parity + export compatibility [1] |
| **libSQL** | optional replication adapter | plain SQLite export preserved [1] |
| **Wasmtime** | future untrusted plugin runtime | need for portable plugin ABI |
| **Firecracker** | future hard isolation | hostile code execution at scale |
| **sqlx** | not chosen | only if async DB ergonomics dominate and migrations stay clean |

### 4.6 Ops assets outside Cargo

| Asset | Use |
|---|---|
| **restic** | encrypted backup of DB snapshots + artifact tree |
| **Tailscale or WireGuard** | Z1 mesh |
| **systemd** | Linux daemon supervision |
| **cosign / sigstore** | signed releases |
| **Syft or cargo-cyclonedx** | SBOM generation |
| **Trivy / Grype / OSV-Scanner** | vulnerability scanning |

***

## 5. Explicit reject list (Core v0.1)

| Reject | Why |
|---|---|
| Limbo as default DB | Alpha/maturity risk; not institutional basement [1] |
| prism-mcp-rs / enterprise MCP stacks | Complexity and attack surface before tool governance [1] |
| apalis-sqlite as lease authority | Owns lifecycle we must own; custom broker required [1] |
| cacache as artifact canon | Opaque cache ≠ auditable CAS [1] |
| Generic FS / shell / SQL MCP tools | Ambient authority through the AI door [1] |
| External vector DBs in Core | Dual-write, non-ownership, ops burden [1] |
| Kafka / NATS / Rabbit / Temporal in Core | Distributed OS around a local institution [1] |
| Keycloak/Dex-first IAM | Overkill for owner-operated local core [1] |
| SurrealDB / multi-model “AI databases” | Blurs canon and projection |
| Agent frameworks as record substrate | Wrong layer |
| Cloud-required Tier A dependency | Breaks local ownership law |

***

## 6. Custom code we will build in-house

This is the non-delegable Core law.

### 6.1 Object & metadata law
**Crate intent:** `apparatus-types`

Build:

- `ObjectHeader` (UUIDv7, timestamps, owner, project, classification, provenance, status, supersedes, correlation/causation)
- classification enum and provenance kinds  
- stable error taxonomy  
- capability action/resource vocabulary types  
- network zone enum  
- serde envelopes for all external objects  

Do not outsource this to an “agent memory” product.

### 6.2 Content-addressed artifact store
**Crate intent:** `apparatus-artifacts` (+ store paths)

Build:

- layout: `artifacts/sha256/ab/cd/<hash>`  
- atomic write: `temp → write → fsync → hash verify → rename`  
- manifest with media type, size, hashes, classification, provenance  
- dedup-by-hash optional; identity still unique  
- retrieval handles only through authorized services  
- no overwrite of blob bytes  

Inspired by Git object store; not a Git embed.

### 6.3 Append-only ledger
**Crate intent:** `apparatus-ledger`

Build:

- receipt rows with monotonic position  
- `prev_hash` / `receipt_hash` chain  
- operations: created/validated/rejected/superseded/denied/failed/...  
- `ledger.verify` full-chain job  
- checkpoints for faster verify  
- never update old receipts  

No blockchain, no required Merkle tree in v0.1.

### 6.4 Registry authority
**Crate intent:** `apparatus-registry`

Build:

- principals (human/device/service/agent)  
- projects admission  
- schema registry  
- protocol registry  
- tool/model registry  
- dependency inventory records  

### 6.5 Policy wrapper & capability grants
**Crate intent:** `apparatus-policy`

Build:

- `PolicyEngine` trait:
  - `Allow`
  - `Deny { reason, rule_id }`
  - `RequireApproval { reason, approver_roles }`
  - optional `AllowWithConstraints` later  
- capability grant records with scope, expiry, zone, max calls/bytes  
- Cedar adapter translating Apparatus principal/action/resource/context  
- static policy load from Zone 2 files only  
- denial and approval-required events ledgered  

**Not in-house:** Cedar evaluator math.  
**In-house:** vocabulary, grants, approval bridge, enforcement side effects.[1]

### 6.6 Protocol engine
**Crate intent:** `apparatus-protocols`

Build state machines for:

| Protocol | Must handle |
|---|---|
| `project.bootstrap.v1` | create isolated project DB/roots/registry row |
| `artifact.intake.v1` | validate → authorize → CAS put → receipt |
| `session.close.v1` | bundle validate → persist outputs/tasks/decisions → handoff artifact |
| `task.lease.v1` | grant narrowed lease, heartbeat, complete/fail |
| `high_impact.action.v1` | force approval gate before execution |
| `maintenance.nightly.v1` | verify, snapshot, backup, expire leases/grants, health receipt |

Every transition emits receipts. No hidden side channels.

### 6.7 Task & lease broker
**Crate intent:** `apparatus-tasks`

Build:

- task rows and states  
- lease issuance with expiry  
- server-side re-check on every worker action  
- idempotency keys for complete/fail  
- heartbeat + expiry sweeper  
- poison/failed task ledger events  
- no direct worker DB access  

v0.1 truth is the **lease table**, not distributed bearer-token IAM. Signed tokens optional later.

### 6.8 Gateway
**Crate intent:** `apparatus-gateway`

Build:

- authn edge  
- request IDs + idempotency keys  
- schema validation  
- policy check  
- protocol dispatch  
- minimal route set only  

### 6.9 MCP facade mapping
**Crate intent:** `apparatus-mcp`

Build:

- allow-listed tools only  
- map tool calls → protocol requests  
- no ambient FS/shell/SQL  
- host validation / rebinding protections on HTTP transport  
- first tools:
  - `apparatus.artifact.get_metadata`
  - `apparatus.session.close`
  - later carefully: `apparatus.task.submit_result`  

Transport/SDK borrowed; authority mapping owned.[2][1]

### 6.10 Worker runtime
**Bin:** `apparatus-worker`

Build:

- poll/lease loop  
- fetch only granted inputs via API  
- execute allow-listed tool in sandbox  
- submit outputs via `artifact.intake`  
- complete/fail lease  
- local Linux isolation launcher (landlock/bwrap)  

### 6.11 Maintenance & restore
**Crate intent:** `apparatus-maintenance`

Build:

- nightly protocol  
- SQLite snapshot strategy (`VACUUM INTO` / backup API)  
- artifact tree consistency checks  
- restic integration contract  
- restore drill command  

### 6.12 Observe conventions
**Crate intent:** `apparatus-observe`

Build:

- field naming  
- redaction rules  
- health endpoints  
- correlation with request/receipt IDs  

***

## 7. Borrow vs build matrix

| Need | Solution | Own or borrow |
|---|---|---|
| SQL engine | SQLite + rusqlite | Borrow |
| HTTP server | axum/tower | Borrow |
| JSON Schema validation | jsonschema/schemars | Borrow |
| Policy math | cedar-policy | Borrow evaluator |
| Capability vocabulary | Apparatus actions/resources | **Own** |
| Approval workflow | protocol state machine | **Own** |
| Hash/sign primitives | sha2/blake3/ed25519-dalek | Borrow |
| Artifact CAS layout/atomics | custom | **Own** |
| Ledger format | custom | **Own** |
| Task leases | custom SQL broker | **Own** |
| MCP transport | official rmcp SDK | Borrow |
| MCP authority mapping | custom facade | **Own** |
| Worker sandbox mechanism | cap-std + landlock/bwrap | Borrow mechanisms |
| Worker policy binding | lease + grants | **Own** |
| Backup engine | restic | Borrow |
| What/when/verify backup | maintenance protocol | **Own** |
| Vector search | sqlite-vec later | Borrow as projection |
| Meaning of evidence/claims | object model + protocols | **Own** |

***

## 8. Workspace skeleton (accepted)

```text
apparatus/
├── Cargo.toml
├── deny.toml
├── rust-toolchain.toml
├── schemas/
├── migrations/
│   ├── control/
│   └── project_template/
├── crates/
│   ├── apparatus-types
│   ├── apparatus-schema
│   ├── apparatus-crypto
│   ├── apparatus-time
│   ├── apparatus-store
│   ├── apparatus-artifacts
│   ├── apparatus-ledger
│   ├── apparatus-registry
│   ├── apparatus-policy
│   ├── apparatus-protocols
│   ├── apparatus-tasks
│   ├── apparatus-gateway
│   ├── apparatus-mcp
│   ├── apparatus-observe
│   └── apparatus-maintenance
└── bins/
    ├── apparatusd
    ├── apparatus-cli
    └── apparatus-worker
```

Dependency direction remains strict: types → crypto/schema/time → store → ledger/artifacts/registry/policy → protocols/tasks → gateway/mcp/maintenance → bins.

***

## 9. Phased delivery with acceptance tests

### M0 — Immutable base
**Build:** types, time, crypto, schema, store, artifacts, ledger  
**External:** rusqlite, serde, uuid, sha2/blake3, jsonschema/schemars  

**Done when:**

1. bootstrap creates control home + project DB files  
2. artifact put survives process kill mid-write without corrupt canon  
3. receipt chain verifies  
4. clean restore of DB + artifacts on another directory opens and verifies  

### M1 — Authority
**Build:** registry, policy wrapper, capability grants, denial receipts  
**External:** cedar-policy (after spike), secrecy/zeroize  

**Done when:**

1. unauthorized read/write denied and ledgered  
2. grant expiry works  
3. high-impact action path can enter `PendingApproval` without executing  

### M2 — Work
**Build:** protocols, tasks/leases, worker bin, Linux sandbox launcher  
**External:** cap-std, landlock/bwrap  

**Done when:**

1. worker leases one task and cannot complete another worker’s lease  
2. expired lease cannot act  
3. worker cannot read outside granted directory under kernel enforcement  
4. `session.close` produces handoff artifact + receipts  

### M3 — Edge
**Build:** gateway stabilization, MCP facade allow-list  
**External:** axum stack, official rmcp SDK  

**Done when:**

1. external client can call only allow-listed tools  
2. tool call cannot bypass policy/approval  
3. no FS/shell/SQL raw tools exist  
4. DNS/host validation configured on HTTP MCP path  

### M4 — Projections & adapters
**Optional:** sqlite-vec/FTS, libSQL adapter experiments, richer model router  

**Done when:** projections rebuild from canon after wipe.

***

## 10. Required spikes before irreversible coding choices

| Spike | Question | Pass criteria | Fail fallback |
|---|---|---|---|
| S1 Cedar | Can we implement `Allow/Deny/RequireApproval` cleanly? | Static policies, diagnostics usable, no string-built policy | Keep trait; temporary rules engine in Rust |
| S2 CAS durability | Are atomic writes + restore robust? | Kill tests + verify + restore drill | Adjust fsync/dir fsync strategy |
| S3 Leases | Idempotent complete/fail + expiry | Double-complete safe; expired deny receipted | Simplify states; still custom SQL |
| S4 MCP rmcp | Thin facade fit | 2 tools mapped through policy | Evaluate alternate thin SDK; still MCP protocol |
| S5 Linux sandbox | Kernel deny works | Out-of-scope open fails even if app tries | bubblewrap-first or Linux-only worker support note |

No spike → no Tier lock beyond current defaults.

***

## 11. First allow-listed surfaces

### HTTP (Core)
- `POST /v1/projects`  
- `POST /v1/artifacts`  
- `GET /v1/artifacts/:id`  
- `POST /v1/sessions/close`  
- `POST /v1/tasks`  
- `POST /v1/tasks/lease`  
- `POST /v1/tasks/leases/:id/complete`  
- `POST /v1/tasks/leases/:id/fail`  
- `POST /v1/protocols/:name/@:version/runs`  
- `GET /healthz` `/readyz` `/metrics`  

### MCP tools v0
- `apparatus.project.get`  
- `apparatus.artifact.get_metadata`  
- `apparatus.session.close`  

### MCP tools explicitly forbidden
- shell/exec  
- arbitrary filesystem read/write  
- raw SQL  
- grant admin  
- unrestricted network fetch  

***

## 12. Control DB ownership split

| Store | Contents | Owner |
|---|---|---|
| `control.db` | principals, projects, schemas, protocols, tools, grants, audit indexes | Core |
| `ledger` receipts | append-only history | Core |
| `artifacts/` CAS | immutable bytes | Core |
| `projects/<slug>/project.db` | project objects/tasks/claims/sessions | Project steward via Core APIs |
| `projects/<slug>/derived/` | search/embeddings/caches | Disposable |

No cross-project SQL. Shares use export/grant protocols only.

***

## 13. Dependency admission policy

| Tier | Examples | Admission bar |
|---|---|---|
| A Foundation | rusqlite, tokio, serde, sha2 | multi-year proof, replaceability plan, no cloud hard-dep, license clean |
| B Operational | cedar-policy, rmcp, cap-std, axum | active maintenance, spike passed, bounded use-case |
| C Utility | sqlite-vec, tar/zstd helpers | degrade safely; cannot become canon |
| D Experimental | Limbo, Firecracker integrations | feature-flag only; never default path |

Rules:

1. New Tier A/B needs owner approval note in ADR or deny.toml exception  
2. Pin versions in lockfile  
3. CI fails on advisory/license violations  
4. SBOM on release  
5. Prefer fewer deps over clever deps  

***

## 14. Selected “inspiration projects” retained as design references

Not dependencies. Study targets for implementation quality:

| Reference | Steal this idea |
|---|---|
| SQLite | file durability, disciplined scope, testing culture |
| Git | immutable content, movable names/refs |
| FFmpeg | sharp tools, strict interfaces, regression ruthlessness |
| tinygrad | keep core small enough to hold in one head |
| NASA C&DH / FSW patterns | command accept/reject with reason; telemetry |
| Cedar examples | principal/action/resource separation |
| restic | backup equals verify/restore |
| official MCP rust-sdk | transports and tool schema shape only [2] |
| cap-std / landlock | kill ambient authority |

***

## 15. Final selected stack poster

### Build with
Rust workspace · rusqlite/SQLite · axum/tokio · cedar-policy · rmcp · cap-std/landlock · sha2/blake3 · serde/jsonschema · tracing · cargo-deny · restic  

### Build ourselves
headers · CAS · ledger · registry · grants · protocols · leases · session close · maintenance · MCP allow-list mapping · worker lease loop  

### Do not build on
Limbo-default · prism-mcp · apalis-as-law · cacache-as-canon · vector DB core · shell MCP · Kafka/Temporal core · Keycloak-first  

***

## 16. Acceptance statement

We accept ADR-0001 as the open-source and custom-code split for Apparatus Core v0.1.

**Core success condition remains unchanged:**

1. bootstrap project  
2. intake immutable artifact  
3. verify ledger  
4. deny unauthorized action with receipt  
5. lease bounded task  
6. close session by bundle  
7. restore on a clean machine  
8. optional MCP facade calls the same law as CLI/HTTP  

Anything that does not serve those outcomes is deferred.

***

## 17. Next document after this

When ready to implement, next artifact should be:

**M0 Engineering Spec**  
- exact SQL migrations  
- CAS path layout  
- receipt hash canonical string  
- `ObjectHeader` fields finalized  
- CLI commands for put/get/verify/restore  

No more broad stack research required before M0.

[2] The official Rust SDK for the Model Context Protocol https://github.com/modelcontextprotocol/rust-sdk
[3] rust-sdk/crates/rmcp/README.md at main https://github.com/modelcontextprotocol/rust-sdk/blob/main/crates/rmcp/README.md
[4] Model Context Protocol https://github.com/modelcontextprotocol
[5] Releases · modelcontextprotocol/rust-sdk https://github.com/modelcontextprotocol/rust-sdk/releases
[6] Cargo.toml - modelcontextprotocol/rust-sdk https://github.com/modelcontextprotocol/rust-sdk/blob/main/Cargo.toml
[7] Repositories 27 https://github.com/orgs/modelcontextprotocol/repositories
[8] SDKs https://modelcontextprotocol.io/docs/sdk
[9] Model Context Protocol https://github.com/orgs/modelcontextprotocol/repositories?q=lang:rust&type=all
[10] GitHub - jeanlucthumm/modelcontextprotocol-rust-sdk: The UNofficial Rust SDK for Model Context Protocol servers and clients https://github.com/jeanlucthumm/modelcontextprotocol-rust-sdk
[11] barndoor-ai/official-mcp-rust-sdk https://github.com/barndoor-ai/official-mcp-rust-sdk
[12] GitHub - Derek-X-Wang/mcp-rust-sdk: Rust SDK for the Model Context Protocol (MCP) https://github.com/Derek-X-Wang/mcp-rust-sdk
[13] modelcontextprotocol/rust-sdk - GitHub Stars Leaderboard https://githublb.vercel.app/repo/modelcontextprotocol/rust-sdk
[14] Two New Open Source Rust Crates Create Easier Cedar ... https://aws-news.com/article/2023-12-14-two-new-open-source-rust-crates-create-easier-cedar-policy-management
[15] Model Context Protocol (modelcontextprotocol) https://githublb.vercel.app/owner/modelcontextprotocol
[16] Two New Open Source Rust Crates Create Easier Cedar ... https://aws.amazon.com/blogs/opensource/easier-cedar-policy-management/


# Apparatus × Engram  
## Technical Integration & Rewrite Specification  
**Document:** `APPARATUS-ENGRAM-001`  
**Status:** Design baseline for first in-Apparatus project  
**Date:** 2026-07-24  
**Sources read:** `infraax/engram`, `infraax/chimera-brain` Engram/Vector docs, frequency/symphony research, rewrite geodesic, ADR seams, capacity results, and Core ADR-0001  

***

## 0. Executive answer

**Engram does not become Apparatus Core.**  
It becomes the **first governed intelligence project** attached to Apparatus: a memory engine whose certificates, writes, queries, and indexes live under Core law.

| Question | Answer |
|---|---|
| Is Engram Core? | **No.** Core remains identity, ledger, CAS, policy, protocols, leases |
| What is it then? | A **project-scoped memory subsystem** + optional workspace crates |
| Package shape | **Multi-crate Rust package family** + **Python reference lab** (not one mega-module inside `apparatusd`) |
| Rewrite? | **Yes — selective rewrite**, not a blind port of the whole Python tree |
| What stays Python? | Research, golden vectors, experiments, embedder trials |
| What becomes Rust? | Format codec, fingerprint hot path, index adapter, Core protocol bridge |
| Where do bytes live? | `.eng` certificates as **immutable Apparatus artifacts** |
| Where does the index live? | **Derived projection** under `projects/<id>/derived/engram/` — rebuildable |
| How do agents touch it? | Only via **capability-gated protocols/tools**, never raw DB/FS |

One line:

> **Engram is the first “symphony layer” on the Apparatus road network — composed memory with confidence, not a second source of truth.**[1]

***

## 1. What Engram is today (from the repos)

### 1.1 `infraax/engram` — LLM session-memory PoC

Current pipeline: 

```text
text / model state
  → extract state (originally llama.cpp KV-cache)
  → Fourier fingerprint (rFFT, keep f0+f1, L2-norm, concat)
  → write EIGENGRAM certificate (.eng, ~hundreds of bytes)
  → HNSW index
  → multi-stage retrieval (ANN → trajectory/constraints → metadata)
  → MCP / API surfaces
```

**Solid pieces to keep conceptually**

- Fourier fingerprint idea (deterministic geometric projection; no training required for the fingerprint step) 
- Compact binary certificate with confidence/margin fields 
- HNSW-class µs retrieval path 
- Embedder fallback chain idea (model-native → sentence model → hash) 
- Large test culture / round-trip discipline 

**Not production / must not enter Core blindly**

- llama.cpp / GGUF coupling 
- batch session-end write assumptions 
- stub backends (Redis/S3), experimental compression paths 
- alpha MCP session protocol instability 
- treating Engram as the system of record

### 1.2 `chimera-brain/vector_engram` — embodied / situational branch

Generalization already stated in your docs: 

```text
any structured state vector
  → spectral fingerprint
  → compact certificate
  → fast associative retrieval
```

For Vector/robot:

```text
situation_t = concat(vision, audio, touch, imu, emotion, ...)
fp = normalize(concat(|rFFT(situation_t)|[0], |rFFT(...)|[1]))
```

Additional modules in the chimera tree:

- VSA bind/bundle style composition 
- resonance / confidence style retrieval aids 
- compose / two-rate memory experiments 
- store/index/query split in Python 

### 1.3 Philosophy layer (frequency + symphony)

The research docs argue memory should be: 

- **composed / resonant**, not only key-value lookup  
- frequency-structured rather than “notebook taped to a goldfish”  
- confidence-gated (“safe read / safe write”)  
- continuous and lifelong where needed, not only chat-session dumps  

For Apparatus, translate poetry into engineering:

| Philosophy | Engineering meaning inside Apparatus |
|---|---|
| Memory is not a filing cabinet | Engram index ≠ canonical store |
| Resonance over brittle keys | fingerprint + ANN + margin/confidence |
| Self across time | session/project continuity via certificates + Core receipts |
| Safe read/write | policy + confidence thresholds + approval for high impact |
| Symphony / composition | VSA/compose as optional higher layer later |

### 1.4 Capacity / performance signals

From capacity/PoC notes: HNSW-class queries in tens–low hundreds of µs on box-class hardware; high Recall@1 on small N in PoC conditions; robot-class ARM expected much slower without compiled kernels. 

Treat these as **targets and evidence**, not guarantees for Apparatus hardware until re-bench under Core I/O and policy overhead.

***

## 2. Where Engram lives in Apparatus

### 2.1 Layer map

```text
Z0  UI / external AI clients
      │
      │  MCP allow-listed tools / HTTP
      ▼
Z2  APPARATUS CORE
      identity · policy · ledger · CAS · protocols · leases
      │
      │  typed protocol calls only
      ▼
PROJECT: engram (or any project that enables Engram)
      ┌──────────────────────────────────────────────┐
      │  apparatus-engram-* crates (project module)  │
      │  - certify / fingerprint / index / query     │
      │  - never owns root identity or global policy │
      └──────────────────────────────────────────────┘
      │                         │
      ▼                         ▼
  artifacts/*.eng            derived/engram/
  (canonical cert bytes)     (HNSW/FTS projections)
      │
      ▼
  project.db metadata rows (pointers, margins, links)
```

### 2.2 Hard placement rules (aligned to ADR-0001)

1. **Canon:** `.eng` certificate bytes → Apparatus **Artifact** (CAS, hashed, receipted).  
2. **Canon metadata:** certificate header fields mirrored into project DB objects with provenance.  
3. **Non-canon:** HNSW/graph/resonance structures → `derived/`; wipe/rebuild allowed.  
4. **Non-canon:** model embeddings used only as inputs to fingerprints or as separate derived vectors.  
5. **Authority:** every write/query is a protocol under Cedar/policy + leases.  
6. **No ambient FS tools** for models to “just read the engram folder.”[1]

### 2.3 Engram is a *project capability*, not a global brain

Recommended product shape:

- Core can run with **Engram disabled**  
- A project opts in: `engram` module admitted in project registry  
- Multiple projects can each have isolated Engram stores  
- Shared cross-project memory only via explicit export/grant protocols  

This preserves “many project stores, no universal soup.”

***

## 3. Package shape: library, module, crate, or service?

### Verdict

Engram inside Apparatus is **all of the following, in layers**:

| Layer | Form | Name | Language |
|---|---|---|---|
| Reference lab | Python package (existing) | `engram` / `vector_engram` kept external or vendored as `labs/engram-py` | Python |
| Format + domain types | Rust **library crate** | `apparatus-engram-format` | Rust |
| Math / fingerprint | Rust **library crate** (hot path) | `apparatus-engram-math` | Rust |
| Index adapters | Rust **library crate** | `apparatus-engram-index` | Rust |
| Domain services | Rust **library crate** | `apparatus-engram-service` | Rust |
| Core bridge | Rust **library crate** | `apparatus-engram-bridge` | Rust |
| Optional daemon side-car | Rust **bin** (usually not needed v1) | `engramd` only if out-of-process isolation wanted | Rust |
| Operator/agent surface | protocols + MCP tools in Core facade | not a separate product API | — |

### What it is *not*

- Not a single Python module dropped into Core  
- Not a replacement for `apparatus-ledger` or CAS  
- Not an always-on global microservice mesh  
- Not “the agent’s infinite memory OS” with ambient rights  

### Recommended mental model

```text
labs/engram-py/               # golden reference + research (upstream-tracking)
crates/apparatus-engram-*/    # production rewrite surface used by projects
bins/apparatusd               # loads engram bridge as optional module
projects/<slug>/              # data plane for that project's memory
```

**Module** in product language: “Engram module enabled on project X.”  
**Crates** in engineering language: the Rust implementation units.  
**Library** in reuse language: format/math/index are reusable libs.  
**Service** in runtime language: in-process service objects behind traits; out-of-process only if sandbox needs demand it.

***

## 4. Rewrite strategy (Apparatus-native, not Vector-native copy)

Your geodesic rewrite for Vector recommends roughly: 

- C++ on robot reflex  
- Go on box service  
- Python as reference  
- FlatBuffers schema  
- PFFFT / usearch style stack  

**Apparatus is different:** Core is already **Rust**, local-first, SQLite canon, no Anki `vic-engine` constraint.

### 4.1 Adapted language decision for Apparatus

| Component | Vector geodesic | Apparatus decision |
|---|---|---|
| Reference / research | Python | **Keep Python lab** |
| Format codec | FlatBuffers multi-lang | **Rust codec + frozen byte ABI + golden vectors**; FlatBuffers optional if you want multi-lang parity later |
| Fingerprint DSP | C++ / PFFFT | **Rust first**; optional `pffft` FFI only if bench proves need |
| Hot index | C++ ring buffer | **Rust** in-process for project service |
| Archive ANN | C++ usearch via Go | **Rust index adapter** (usearch binding or pure-Rust HNSW candidate) behind trait |
| Service/API | Go box | **Rust bridge inside Apparatus protocols** (Axum already exists) |
| Robot reflex tier | C++ in-engine | **Out of scope for Apparatus Core integration** unless a specific robot project needs a later FFI export |

### 4.2 Rewrite principles

1. **Contract-first** (your ADR Approach 2): freeze byte format + golden vectors before broad ports.   
2. **Python remains executable spec** until Rust matches golden tolerances.   
3. **Do not port stubs** (Redis/S3/PolarQuant theater).  
4. **Decouple state source** via a `StateVector` trait from day one (KV, text embedding stream, perception, session bundle features).   
5. **Every durable write enters Core intake** — Engram never secret-writes disks behind the ledger.  
6. **Index rebuild from artifacts** must work with Engram disabled mid-flight.

### 4.3 Keep / rewrite / drop

| Piece | Action | Notes |
|---|---|---|
| Fourier f0+f1 fingerprint math | **Rewrite in Rust** (keep algorithm) | Core IP of Engram  |
| Certificate envelope | **Rewrite codec + versioning** | Canonical artifact payload |
| Confidence / margin fields | **Keep as first-class metadata** | Safe-read gate  |
| HNSW retrieval | **Adapter rewrite** | Projection only |
| 4-stage retrieval ideas | **Reimplement selectively** | Start with ANN + metadata filters; add constraints later |
| llama.cpp bridge | **Drop from Core path** | Optional worker integrator later |
| MCP engram tools | **Re-map through Apparatus MCP facade** | No direct FS  |
| Knowledge markdown index | **Defer** | Not needed for v1 |
| VSA/compose/symphony layers | **Phase 2+** | After certify/query solid  |
| Streaming writer | **Rebuild against Core intake + queue** | Continuous memory later  |
| Go box service | **Don’t create** for Apparatus | Core gateway replaces it |
| C++ robot lib | **Separate future project export** | Not required to land Engram-in-Apparatus |

***

## 5. Crate architecture (detailed)

```text
apparatus/
├── labs/
│   └── engram-py/                      # git subtree or submodule tracking infraax/engram + vector_engram reference
│       ├── golden/                     # frozen vectors & .eng fixtures
│       └── README.md                   # how to regenerate goldens
│
├── crates/
│   ├── apparatus-engram-format/        # pure codec + header types (no DB, no ANN)
│   ├── apparatus-engram-math/          # fingerprint, norms, optional VSA later
│   ├── apparatus-engram-index/         # Index trait + HNSW/usearch impls
│   ├── apparatus-engram-service/       # certify/query/rebuild domain logic
│   └── apparatus-engram-bridge/        # Core protocols, schema registration, MCP tool specs
│
├── schemas/engram/
│   ├── certificate_v1.schema.json      # JSON metadata companion (not replacing binary)
│   ├── engram_certify.v1.json
│   ├── engram_query.v1.json
│   └── engram_rebuild_index.v1.json
│
└── migrations/project_template/
    └── 00xx_engram.sql                 # optional module tables
```

### 5.1 `apparatus-engram-format` (library crate)

**Responsibility:** binary certificate ABI.

Suggested envelope (logical; finalize with goldens):

```text
magic: b"EGR1" or project-chosen stable magic (track upstream EGR1/EGRV carefully)
version: u16
flags: u16
state_kind: u8          # kv | embedding_seq | perception | session_features | custom
dim_fp: u16
fp_dtype: u8            # f16/f32
created_at_unix_ms: i64
content_hash: [u8;32]   # sha256 of fingerprint payload (+ critical fields)
confidence: f32         # semantic coverage / safe-read score
margin: f32             # retrieval margin if known at write
source_ref_count + refs
fingerprint bytes
optional tail (extensible TLVs): model_id, robot_id, place, emotion, ...
```

Must support:

- encode/decode round-trip  
- unknown-tail skip for forward compat  
- deterministic canonicalization for hashing  
- no_std-friendly core optional later (not required v1)

**Depends on:** almost nothing (`thiserror`, `byteorder`/`zerocopy` style tools).  
**Does not depend on:** rusqlite, axum, faiss, Python.

### 5.2 `apparatus-engram-math` (library crate)

**Responsibility:** state → fingerprint.

```rust
pub trait StateVector {
    fn dims(&self) -> usize;
    fn as_f32_matrix(&self) -> &[f32]; // shape logical [T, D] or flat policy documented
    fn state_kind(&self) -> StateKind;
    fn provenance(&self) -> StateProvenance;
}

pub struct Fingerprint {
    pub components: Vec<f32>, // or fixed array after version freeze
    pub method: FingerprintMethod, // FourierF0F1 {..}
    pub conf_hint: Option<f32>,
}

pub fn fingerprint_fourier_f0_f1(state: &dyn StateVector, cfg: &FpConfig) -> Result<Fingerprint>;
```

Rules:

- pure functions + explicit config  
- golden-tested vs Python within tolerance (e.g. 1e-5 relative/abs policy)   
- SIMD later; correctness first  

Optional later modules in same crate:

- VSA bind/bundle  
- resonance scoring helpers  
- two-rate consolidation math  

### 5.3 `apparatus-engram-index` (library crate)

**Responsibility:** rebuildable ANN projection.

```rust
pub trait EngramIndex: Send + Sync {
    fn upsert(&mut self, id: UuidV7, fp: &[f32], meta: &IndexMeta) -> Result<()>;
    fn remove(&mut self, id: UuidV7) -> Result<()>;
    fn query(&self, fp: &[f32], k: usize, filter: &QueryFilter) -> Result<Vec<Hit>>;
    fn persist(&self, path: &Utf8Path) -> Result<()>;
    fn load(path: &Utf8Path) -> Result<Self> where Self: Sized;
    fn stats(&self) -> IndexStats;
}
```

Implementation plan:

1. **M1:** brute-force exact index (correctness, small N)  
2. **M2:** HNSW-class backend (evaluate `usearch` binding vs pure Rust crate; license Apache-friendly per your geodesic)   
3. Always rebuildable from artifact scan  

### 5.4 `apparatus-engram-service` (library crate)

**Responsibility:** domain operations without HTTP.

Operations:

- `certify(state, meta) -> Certificate + suggested ArtifactPut`  
- `open(cert_bytes) -> CertificateView`  
- `index_attach(cert, artifact_id)`  
- `query(QuerySpec) -> QueryResult` (hits with artifact IDs + margins)  
- `rebuild_index(project_scope) -> RebuildReport`  
- `gc_derived()`  

This crate may know project paths only through injected store traits from Core (`ArtifactStore`, `ProjectDb`, `Clock`, `IdFactory`).

### 5.5 `apparatus-engram-bridge` (library crate)

**Responsibility:** make Engram a lawful Apparatus citizen.

Registers:

- schemas  
- protocols  
- MCP tool specs  
- migration pack for project template  
- policy action names  

No second authority model.

***

## 6. Data model inside a project

### 6.1 Files

```text
projects/<slug>/
├── project.db
├── artifacts/
│   └── sha256/.../<hash>          # includes .eng cert bytes as artifacts
├── workspace/
│   └── engram/
│       └── staging/               # optional temp before intake finalize
└── derived/
    └── engram/
        ├── index.bin              # ANN structure (disposable)
        ├── index.meta.json        # build generation, params (disposable)
        └── stats.json
```

### 6.2 Project DB tables (module migration)

```sql
-- enabled only if project module "engram" admitted
CREATE TABLE engram_certificates (
  id                TEXT PRIMARY KEY,      -- uuid v7
  artifact_id       TEXT NOT NULL UNIQUE,  -- CAS pointer
  content_hash      TEXT NOT NULL,
  state_kind        TEXT NOT NULL,
  fingerprint_method TEXT NOT NULL,
  dim               INTEGER NOT NULL,
  confidence        REAL,
  margin            REAL,
  source_refs_json  TEXT NOT NULL,
  model_id          TEXT,
  session_id        TEXT,
  task_id           TEXT,
  created_at_ms     INTEGER NOT NULL,
  recorded_at_ms    INTEGER NOT NULL,
  classification    TEXT NOT NULL,
  status            TEXT NOT NULL,          -- active|superseded|deleted_logically
  supersedes_id     TEXT,
  ext_json          TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE engram_links (
  id                TEXT PRIMARY KEY,
  cert_id           TEXT NOT NULL,
  rel_type          TEXT NOT NULL,          -- supports|derived_from|contradicts|continues
  target_kind       TEXT NOT NULL,          -- artifact|claim|decision|task|cert
  target_id         TEXT NOT NULL,
  created_at_ms     INTEGER NOT NULL
);

CREATE TABLE engram_index_generations (
  gen               INTEGER PRIMARY KEY,
  built_at_ms       INTEGER NOT NULL,
  cert_count        INTEGER NOT NULL,
  params_json       TEXT NOT NULL,
  index_path        TEXT NOT NULL,
  status            TEXT NOT NULL           -- active|building|failed|superseded
);
```

### 6.3 Object mapping to Core headers

Each certificate row corresponds to:

- one `Artifact` (bytes)  
- one `RecordReceipt` on create  
- optional `Claim`/`Evidence` links if a human or agent promotes a memory to asserted knowledge  
- never auto-promote fingerprint similarity into “fact”

***

## 7. Protocols (Apparatus law for Engram)

### P-E1 `engram.certify.v1`

**Input:** state payload ref (artifact or inline bounded features) + metadata  
**Steps:**

1. Validate schema  
2. Authorize `engram.certify` on project  
3. Load/construct `StateVector`  
4. Fingerprint  
5. Encode `.eng`  
6. `artifact.intake` cert bytes  
7. Insert `engram_certificates` row  
8. Queue derived index upsert job  
9. Receipt  

**Output:** `cert_id`, `artifact_id`, `confidence`, `content_hash`

### P-E2 `engram.query.v1`

**Input:** query state or fingerprint + filters (time, session, classification, k)  
**Steps:**

1. Authorize `engram.query`  
2. Fingerprint query state if needed  
3. ANN query over derived index  
4. Join hits to certificate metadata + artifact IDs  
5. Apply confidence/margin thresholds  
6. Return ranked hits **with provenance**, not naked floats only  
7. Ledger read-audit optional (config; at least metrics)

**Safe-read rule:** if top confidence/margin below project threshold → return `insufficient_resonance` rather than weak junk as truth. 

### P-E3 `engram.rebuild_index.v1`

Maintenance/operator protocol:

1. Authorize maintenance  
2. Scan active cert artifacts  
3. Build new index generation  
4. Atomic switch  
5. Receipt + stats  

### P-E4 `engram.promote.v1` (later)

Promote a certificate (or query hit) into `Claim`/`Evidence` only with explicit human/agent protocol and higher policy tier.

### P-E5 `engram.session_capture.v1` (bridge from old MCP memory)

Maps old “end session dump” idea onto Apparatus `session.close` + optional Engram certify of session feature state. 

Preferred path:

```text
session.close bundle
  → Core stores handoff artifact
  → optional engram.certify(session_features)
```

Not a parallel shadow archive.

***

## 8. Capability vocabulary (Cedar/actions)

Add project-scoped actions:

| Action | Meaning |
|---|---|
| `engram.module.enable` | steward enables module on project |
| `engram.certify` | write new certificate |
| `engram.query` | query memory |
| `engram.rebuild_index` | maintenance |
| `engram.promote` | lift memory into claims/evidence |
| `engram.export` | cross-project export |
| `engram.delete_logical` | supersede/tombstone cert metadata |

Classification ceilings still apply: a Z0 model with `engram.query` on `public/shared` must not see `restricted` certs.

***

## 9. MCP surface (thin, allow-listed)

Through Core MCP facade only:[1]

| Tool | Maps to |
|---|---|
| `apparatus.engram.certify` | `engram.certify.v1` |
| `apparatus.engram.query` | `engram.query.v1` |
| `apparatus.engram.get` | metadata+optional cert artifact fetch if granted |

Forbidden:

- `engram.dump_all`  
- raw path read of `derived/engram`  
- “reindex and delete whatever” without maintenance protocol  
- automatic certify of every tool call without policy/thresholds  

***

## 10. Runtime topology options

### Option A — In-process module (default v1)

```text
apparatusd
  └─ EngramBridge service
       ├─ format/math/index crates
       └─ uses Core ArtifactStore + ProjectDb
```

**Pros:** simple, one binary, easy receipts  
**Cons:** ANN/FFT load shares process with Core  

Mitigation: run heavy certify/rebuild on worker leases, not gateway thread.

### Option B — Worker-specialized compute

```text
apparatusd (law)
apparatus-worker --role engram-compute
  └─ leased tasks: fingerprint heavy / rebuild
```

**Pros:** isolation, matches Core worker model  
**Cons:** more moving parts  

**Recommendation:**  
- v1 queries + small certifies in-process  
- large rebuilds / bulk backfills as leased worker tasks  
- no separate `engramd` until multi-tenant load demands it  

***

## 11. Mapping old Engram components → Apparatus homes

| Upstream component | New home | Notes |
|---|---|---|
| `kvcos/engram/format.py` | `apparatus-engram-format` | rewrite codec  |
| fingerprint math | `apparatus-engram-math` | golden parity |
| `hnsw_index.py` / faiss usage | `apparatus-engram-index` | trait + impl |
| `retrieval.py` stages | service query pipeline | simplify first  |
| `writer.py` / `reader.py` | service + artifact intake | writer must call Core |
| `embedder.py` | optional worker tool | not Core; produce state vectors |
| `knowledge_index.py` | defer | projection/docs later |
| `session_propagator.py` | fold into session.close + certify | |
| `mcp/engram_memory.py` | delete as authority; remap tools | facade only  |
| `vector_engram/vsa.py` etc. | phase 2 math modules | after v1 certify/query |
| `vector_brain` C++/Go plan | not required for Apparatus landing | robot export later  |
| Redis/S3 backends | reject for Core path | local CAS only |

***

## 12. Confidence, frequency, symphony — engineered, not mystical

### 12.1 Confidence gate

Certificate carries coverage/confidence and margin. 

Apparatus rules:

- store confidence with cert metadata  
- project policy sets `min_query_confidence`, `min_margin`  
- below threshold → structured refusal, ledgerable as `engram.weak_match` if desired  
- never auto-execute high-impact actions from weak resonance alone  

### 12.2 Frequency core

v1 implements only the proven narrow method:

- spectral fingerprint f0+f1 (or frozen successor after goldens)  

Defer broader “symphony OS” until:

- certify/query durable  
- rebuild proven  
- capacity benches on target hardware  

### 12.3 Composition layer (phase 2)

VSA bind/bundle/resonance become optional rankers/composers over certificates, still producing either:

- ephemeral query views, or  
- new certificates that reference parents via `engram_links`  

Composition output that matters must be intaken as artifacts again.

***

## 13. Relationship to Core phases (when to build)

Engram is the first *project*, but it still respects Core sequencing.

| Core phase | Engram work allowed |
|---|---|
| M0 Core base (CAS/ledger) | Lab only: freeze format goldens, no production writes |
| M1 policy/registry | Define actions, schemas, module admission |
| M2 tasks/workers/session.close | **Start Engram v1 bridge** (certify/query/rebuild) |
| M3 MCP edge | Expose allow-listed Engram tools |
| M4 projections | HNSW upgrades, VSA/symphony experiments, sqlite-vec complementarity |

**Do not** build Engram before artifact intake + receipts exist.  
Otherwise you recreate a second ungoverned memory disk.

***

## 14. Engram v1 scope (first shippable in-Apparatus slice)

### In scope

1. StateVector trait + one text/session feature extractor path  
2. Fourier fingerprint parity with Python goldens  
3. `.eng` encode/decode + artifact intake  
4. project DB cert registry  
5. brute-force index + optional HNSW  
6. `certify` / `query` / `rebuild_index` protocols  
7. policy actions + tests  
8. CLI: `apparatus engram certify|query|rebuild|verify`  

### Out of scope v1

- llama.cpp native KV extraction in Core  
- robot NEON C++ port  
- Go service  
- full symphony composition language  
- distributed replica memory  
- automatic lifelong 30 fps perception writer  
- claiming Engram replaces claims/evidence system  

***

## 15. Testing & parity plan

### Golden suite (from your ADR seams idea) 

```text
labs/engram-py/golden/
  fingerprint_cases.json
  cert_bytes/*.eng
  query_cases.json
```

Rust tests:

- decode upstream fixtures  
- fingerprint within tolerance  
- encode stable hash  
- query top-1 agreement on frozen sets  

### Integration tests (Apparatus)

- certify → artifact exists → ledger verifies → query finds it  
- deny query without grant  
- wipe `derived/` → rebuild → same top-k  
- restore backup on clean machine → query works after rebuild  

### Capacity benches

Re-run class of experiments from `CAPACITY_RESULTS` under Apparatus paths (not raw Python store assumptions). 

***

## 16. Dependency choices specific to Engram

| Need | Choice | Tier |
|---|---|---|
| FFT | pure Rust first (`rustfft` candidate) | B after spike |
| Optional FFT accel | PFFFT via FFI only if needed | D/B |
| ANN | trait + usearch or pure Rust HNSW candidate | B/C |
| fp16 | `half` crate or manual | B |
| FlatBuffers | optional later if multi-lang robot export | D |
| faiss | avoid as Core dep; optional lab only | D |
| Python lab deps | isolated in labs venv, not linked to apparatusd | lab |

License filter remains: no GPL in shipping Core binary path (your geodesic correctly bans FFTW). 

***

## 17. Security model for Engram

Threats special to memory engines:

| Threat | Mitigation |
|---|---|
| Memory poisoning | certify requires capability; provenance mandatory; optional human confirm for promote |
| Secret leakage into certs | scrubber before fingerprint; classification defaults high if source sensitive; never embed raw secrets |
| Cross-project bleed | isolated project derived/ + DB; no global index |
| Model exfil via query | redaction, k limits, classification filters, rate limits |
| Index tampering | index disposable; authority is cert artifacts + ledger |
| Confused deputy via MCP | tools only call protocols; policy on agent principal |

Engram confidence is **not** authentication.

***

## 18. What “done” means for Engram-in-Apparatus v1

1. Module can be enabled on a fresh project  
2. CLI certifies a session/text state into a CAS `.eng` artifact  
3. Ledger has intake receipt  
4. Query returns the cert with margin/confidence  
5. Derived index deleted and rebuilt with equivalent retrieval quality on golden set  
6. Unauthorized principal denied  
7. Backup/restore + rebuild succeeds  
8. MCP tool path (if enabled) uses same protocols  
9. Core still runs cleanly with Engram module absent  

***

## 19. Suggested implementation order (Engram track)

### E0 — Contracts (3–5 days)
- freeze certificate ABI doc  
- export goldens from Python lab  
- JSON schemas for certify/query  

### E1 — Format + math crates
- Rust codec  
- fingerprint parity tests  

### E2 — Service + SQL + intake bridge
- certify protocol end-to-end through Core artifacts  

### E3 — Query + rebuild
- brute force then HNSW adapter  

### E4 — Policy + CLI + tests

### E5 — MCP allow-list tools

### E6 — Optional symphony/VSA experiments as derived rankers  

***

## 20. Documented decision record addendum

**ADR-0002 — Engram placement**

| Decision | Choice |
|---|---|
| Engram in Core foundation? | **No** |
| Engram form | **Optional project module implemented as Rust crate family + Python lab** |
| Durable memory bytes | **Artifacts (`.eng`) + project metadata** |
| Indexes | **Derived only** |
| Rewrite target language | **Rust (Apparatus-native), Python reference retained** |
| Vector C++/Go robot split | **Deferred** to robot export project |
| First protocols | certify / query / rebuild |
| First agent surface | allow-listed MCP/tools after Core M2/M3 |
| Relationship to claims/evidence | Engram recalls candidates; promotion is separate governed protocol |

***

## 21. Final picture

```text
                    ┌─────────────────────────┐
                    │     Apparatus Core      │
                    │ law · ledger · CAS      │
                    │ policy · leases         │
                    └───────────┬─────────────┘
                                │ protocols
                ┌───────────────┼────────────────┐
                │               │                │
                ▼               ▼                ▼
         project MAPA    project ENGRAM    project ...
                                │
                ┌───────────────┼────────────────┐
                │               │                │
                ▼               ▼                ▼
           .eng artifacts   cert metadata   derived HNSW
           (canonical)      (canonical)     (disposable)
                                │
                                ▼
                     query ≈ resonance candidates
                                │
                                ▼
                     promote? → Claim/Evidence protocol
```

Engram’s rewrite inside Apparatus is therefore:

> **a governed memory engine crate family**,  
> **not a second core**,  
> **not a Python sidecar forever**,  
> **not an agent with filesystem keys** —  
> the first real project that proves Apparatus can host serious intelligence without surrendering institutional law.

***

When you want the next artifact, the natural follow-on is a thin **E0 Contract Pack**: exact `.eng` field offsets, golden generation commands, `engram.certify.v1` JSON schema, and the SQL migration draft — enough to start coding `apparatus-engram-format` without further architecture debate.

Bronnen
[1] tekst.txt https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/attachments/1098824204/bb2fd30f-b715-45c9-be42-68057ade0664/tekst.txt?AWSAccessKeyId=ASIA2F3EMEYESINKEH3U&Signature=LpnvYYeZfIUktbeJFbjq9L%2BumHo%3D&x-amz-security-token=IQoJb3JpZ2luX2VjEEMaCXVzLWVhc3QtMSJHMEUCIC2zLSdr4paudBA6Ikcfz3fFPzBftgMfJPGHNaHUPzeuAiEA5Qwkz34bxCo%2BW9G4qr%2Fg%2FYGbK1B%2FdglyF62xnGxjxA8q8wQIDBABGgw2OTk3NTMzMDk3MDUiDCowuOCr9tsGnvVAwyrQBOmYteHtSbnKgLQ6RAjcFLRG1%2FDIUuB7YseM3LZMA8APsBwJJ9OSUnemvbfgTx2XL%2FuIivauXMnAk%2BoUUwhBfz8Bf%2FLddKT9VCPl2hhRzGBdl0JYHSDs2mLnZ%2FaeFxTxuTakIUFg%2F%2Fvc932T6UFCFn4ginSzTipw5Ca3%2BP2YajpJvANrkpR5AIdhlmRYw1CRPkrfIyQuIJ8slPESC%2FWV967dPVKQ6JqNKe6QkxSE57tY4pErkbD2j9FpfgtzY9KAVW7goMRyZKdd3p%2Bu5kjOFREnBEto1xMf9Net5ePfpQads1ZYW2wpIfZWQ3SvcI5lJ4TRqpJMViCFKmRtJ1H3XS9SUVWeQdC1JiEwIX2v8xEplzsD9jUZEhk9utSM%2FY7RCrolcoY7fWYt5Up4gYA6d6nlrjCzgsgWzfivUJWxWCZLibnfesUZbGVRA03eAmbx0ALkI1uAoivZeN6FBR%2B%2Bd3CHjY%2FsCcC1KC%2FY6Mr8Zn7AHNIkd%2FNXF3r%2BySVyFeqwEs73CqFdHcC6whk9P%2FaNvAvcaDlV0e7WK8J7N6Djd0ztiGFCrYHZxCGSXBQxygDzLGmFcSk4eHctLXwzau%2FGEbCk8fN%2Bl1zQzUQV83HFav1pq0cyRt%2Be%2FhF%2FVYSBK6i6Ssj3VLpWykTjgbJgeY6Lc%2Fz1DpQPUTCnhzSSfNBD75%2FrYYt431JGchl%2BNNsU7U8QOoZHa51eAwK5dUE2naorHfNJaYpRZBBACyMkd%2FYPSmBwiBT0bvgKYo6BbXC5RnZdXXwcDtadXFyqnYxbYWzjE3sw5d6O0wY6mAEy9GJci1cYa67zvl8AaGI4GuRldNLS60aW9KTJC1%2Fs33mHJqOh9jHm6DoCMlBwU2F2tMS4HXLOIcm%2BAmGea5QS2x4x4NvFFwhtgZJcS%2BNrm9s9tn76OeY2g74CvVExMPwGhMSBkgv3MWr0%2BpccEiUINx45MFAYqPd%2B5eXL8Jb6Wc1rHCQ7BhAraY3NyBEjHQx%2BVTcr%2FBOxRw%3D%3D&Expires=1784921400
[2] Mystery of the memory engram: History, current knowledge ... https://www.sciencedirect.com/science/article/pii/S0149763424000435
[3] A Synaptic Framework for the Persistence of Memory ... https://research.vu.nl/en/publications/a-synaptic-framework-for-the-persistence-of-memory-engrams/
[4] Memory engrams: Recalling the past and imagining the future https://pmc.ncbi.nlm.nih.gov/articles/PMC7577560/
[5] An Integrated Index: Engrams, Place Cells, and Hippocampal Memory https://www.sciencedirect.com/science/article/pii/S0896627320305286
[6] : Memory: engram - Loterre https://loterre.istex.fr/P66/en/page/-G4CV58RJ-C
[7] Brain-wide mapping of contextual fear memory engram ... https://www.biorxiv.org/content/10.1101/668483v1.full-text
[8] Sylvia Wirth - Basis of the memory engram - Part 1 https://www.youtube.com/watch?v=p74abtIlSko
[9] Memory engram stability and flexibility - Neuropsychopharmacology https://www.nature.com/articles/s41386-024-01979-z
[10] Engram neurons: Encoding, consolidation, retrieval, and forgetting of memory https://www.nature.com/articles/s41380-023-02137-5
[11] Engram (brain cells involved in memory) https://www.flickr.com/photos/nihgov/49106159066
[12] How Does the Sparse Memory “Engram” Neurons Encode ... https://www.frontiersin.org/journals/neural-circuits/articles/10.3389/fncir.2016.00061/full
[13] ENGRAM: Effective, Lightweight Memory Orchestration for ... https://arxiv.org/html/2511.12960v2
[14] Engram — Provable Agent Memory | Open Source https://engram.to/
[15] In vivo ephaptic coupling allows memory network formation https://www.biorxiv.org/content/10.1101/2023.02.28.530474v1.full
[16] ENGRAM https://engram.cloud/




# Apparatus Ã— Engram â€” E0 Contract Pack

**ID:** APPARATUS-ENGRAM-E0-001  
**Status:** Accepted design contract; implementation begins only after Core M0 artifact intake and ledger verification exist  
**Date:** 2026-07-24  
**Applies to:** the optional, project-scoped Engram module  

---

## 1. Purpose and non-goals

E0 freezes the narrow seam required to implement Engram safely inside Apparatus:

1. a canonical, versioned certificate payload;
2. a Rust/Python golden-vector parity process;
3. three protocol contracts (`certify`, `query`, `rebuild_index`);
4. the project migration which records certificate metadata without duplicating certificate bytes; and
5. the boundaries between canonical certificates and disposable retrieval indexes.

E0 does **not** freeze an ANN implementation, a robot runtime, a model/KV extractor, an embedding model, VSA composition, a Go service, or an MCP surface. Those follow only after the E0/M1 tests pass.

Normative words **MUST**, **MUST NOT**, **SHOULD**, and **MAY** have their ordinary RFC-style meanings.

---

## 2. Architecture decision

### 2.1 Placement

Engram is an optional **project module** implemented by a Rust crate family. It is not Apparatus Core and it is not a standalone authority service.

```text
Apparatus Core
  â”œâ”€â”€ canonical artifact CAS        owns bytes, hashes, receipts
  â”œâ”€â”€ project.db                    owns project metadata and links
  â”œâ”€â”€ policy/protocol engine        owns authority and state transitions
  â””â”€â”€ Engram module
        â”œâ”€â”€ format + math           deterministic libraries
        â”œâ”€â”€ service                 certify/query/rebuild operations
        â””â”€â”€ index                   derived, replaceable projection
```

An Engram certificate is stored as an immutable Core artifact. The index contains only certificate IDs, fingerprints and filterable derived metadata; it can be deleted and rebuilt from authorized certificate artifacts.

### 2.2 Upstream compatibility decision

The upstream repos contain two related formats:

- **EGR1 / EIGENGRAM**: LLM/KV-state-oriented, documented as a 99-byte fixed header.
- **EGRV / SituationCert**: Vector/perception-oriented, variable-length strings and a compact fp16 vector.

Apparatus **will not emit either upstream binary layout as its canonical v1 format**. The EGR1 Python writer adds extension fields before vectors while retaining version `1`; its reader detects those fields by remaining-file-length heuristics. That makes byte-level interoperability ambiguous across versions. EGRV is cleaner for situation memories but is still hand-rolled and currently tied to Vector-specific fields.

Apparatus therefore defines **AEG1** (Apparatus Engram Generation 1): a deterministic envelope, a bounded metadata payload, and explicit versioned representations. Import readers for `EGR1` and `EGRV` MAY be implemented later as non-canonical adapters that emit new AEG1 certificates with provenance naming the imported artifact.

No upstream artifact is silently rewritten. Import is `source artifact -> import receipt -> new AEG1 artifact`.

---

## 3. Canonical artifact contract: AEG1

### 3.1 MIME type and storage

- **Artifact media type:** `application/vnd.apparatus.engram;version=1`
- **Suggested filename:** `<certificate-uuid>.aeg`
- **CAS identity:** the ordinary Apparatus artifact SHA-256 of the complete AEG1 byte stream.
- **CAS location:** determined solely by `apparatus-artifacts`; Engram never chooses a durable absolute path.
- **Compression:** forbidden in AEG1 v1. Compression is an outer artifact concern and must not prevent independent parsing.
- **Encryption:** provided by the project/artifact protection layer, not by the AEG1 envelope.

### 3.2 Byte order and float rules

- Multi-byte integers are **little endian**.
- Floating values use IEEE-754 binary16 or binary32 as declared by the header.
- UUID bytes use RFC 4122 network byte order (the 16 canonical UUID octets, not a language-native integer layout).
- Unix timestamps are signed `int64` Unix milliseconds UTC.
- Text is UTF-8, normalized to Unicode NFC before writing.
- Metadata is UTF-8 **canonical JSON**: object keys lexicographically ordered; no insignificant whitespace; finite numeric values only; duplicate keys forbidden.

### 3.3 Fixed header: exactly 160 bytes

All offsets are decimal byte offsets from the beginning of the artifact.

| Offset | Size | Type | Field | Rule |
|---:|---:|---|---|---|
| 0 | 4 | bytes | `magic` | Exactly ASCII `AEG1` |
| 4 | 2 | u16 | `format_version` | Exactly `1` |
| 6 | 2 | u16 | `header_len` | Exactly `160` in v1 |
| 8 | 4 | u32 | `flags` | Reserved; MUST be `0` in v1 writers |
| 12 | 16 | UUID bytes | `certificate_id` | UUIDv7 generated by Apparatus |
| 28 | 8 | i64 | `observed_at_ms` | Time source state was observed; `0` if unknown |
| 36 | 8 | i64 | `recorded_at_ms` | Core clock time at certify operation |
| 44 | 2 | u16 | `state_kind` | Enum below |
| 46 | 2 | u16 | `representation_kind` | Enum below |
| 48 | 1 | u8 | `vector_dtype` | `1=f16`, `2=f32` |
| 49 | 1 | u8 | `component_count` | Number of declared spectral/components; `0` if not applicable |
| 50 | 2 | u16 | `vector_dim` | Number of scalar vector elements; 1â€“65535 |
| 52 | 4 | u32 | `vector_bytes` | Must equal `vector_dim * dtype_width` in v1 |
| 56 | 4 | u32 | `metadata_bytes` | 2â€“65536 bytes in v1 |
| 60 | 2 | u16 | `source_count` | Number of source refs in metadata; 0â€“1024 |
| 62 | 2 | u16 | `reserved` | MUST be `0` |
| 64 | 32 | bytes | `representation_id_sha256` | SHA-256 of metadata `representation.id` UTF-8 bytes |
| 96 | 32 | bytes | `basis_id_sha256` | SHA-256 of metadata `basis.id`, or 32 zero bytes if no basis |
| 128 | 32 | bytes | `source_set_sha256` | SHA-256 of canonical source-ref byte stream, defined below |

After the 160-byte header:

```text
[160, 160 + vector_bytes)                         fingerprint vector
[160 + vector_bytes, + metadata_bytes)            canonical JSON metadata
EOF                                                required immediately after metadata
```

Readers MUST reject trailing bytes in AEG1 v1. Future extensions require a new format version or explicitly specified flag/section rule; they MUST NOT repeat the upstream heuristic-extension pattern.

### 3.4 Enums

`state_kind`:

| Value | Name | Meaning |
|---:|---|---|
| 1 | `text_features` | features derived from text/session artifacts |
| 2 | `embedding_sequence` | ordered embedding or hidden-state-derived input |
| 3 | `perception` | sensor/situation state |
| 4 | `session_features` | session close / handoff feature state |
| 5 | `custom` | project-registered state type |

`representation_kind`:

| Value | Name | Meaning |
|---:|---|---|
| 1 | `fourier_f0_f1` | baseline spectral fingerprint |
| 2 | `imported_egr1` | imported legacy EIGENGRAM; no claim of native parity |
| 3 | `imported_egrv` | imported legacy Vector situation certificate |
| 4 | `vsa_composed` | reserved; not E0 implementation |
| 65535 | `custom` | requires project registry entry |

### 3.5 Required metadata schema

Metadata must pass the `engram-certificate-metadata.v1` schema in this pack. Required concepts are:

```json
{
  "representation": {
    "id": "fourier.f0f1.raw.v1",
    "algorithm_version": "1.0.0",
    "parameters": {"frequencies": [0, 1], "normalization": "l2"}
  },
  "basis": {"id": "", "version": ""},
  "confidence": {"coverage": 0.0, "margin": null, "local_density": null},
  "provenance": {"producer": "apparatus-engram-math", "producer_version": "0.1.0"},
  "source_refs": [
    {"artifact_id": "018f...", "sha256": "<64 lowercase hex>", "role": "state_input"}
  ],
  "labels": {"session_id": null, "task_id": null, "model_id": null},
  "extensions": {}
}
```

The metadata MUST NOT contain:

- raw secret values, access tokens, API keys, private prompts, or unredacted credentials;
- raw source content merely for convenience;
- a field that asserts the certificate is a fact, claim, approval, or authorization decision.

### 3.6 Source-set hash algorithm

For each source ref, construct this UTF-8 byte sequence:

```text
artifact_id + "\n" + sha256_lowercase + "\n" + role + "\n"
```

Sort records lexicographically by `(artifact_id, sha256, role)`, concatenate, and SHA-256 the resulting bytes. This is `source_set_sha256`. A `source_count=0` certificate hashes the empty byte string.

### 3.7 Parser validation order

1. Enforce maximum artifact size before allocation: 1 MiB for v1.
2. Check `magic`, version and exact header length.
3. Check reserved fields and flags are zero.
4. Validate enum values and dtype.
5. Compute expected vector byte count with checked arithmetic.
6. Validate exact total file length.
7. Decode vector and reject NaN or infinite values.
8. Parse canonical JSON; validate its schema.
9. Recompute `representation_id_sha256`, `basis_id_sha256`, and `source_set_sha256`.
10. Compare header and metadata state/representation declarations.
11. Only then expose the certificate to query/index code.

A parse success does not grant authority. Artifact classification and policy still govern access.

---

## 4. Protocol contracts

All requests have an Apparatus outer envelope (request id, principal, project id, correlation id, schema version). The following objects describe protocol payloads only.

### 4.1 `engram.certify.v1`

**Purpose:** make one AEG1 certificate from authorized, bounded source artifacts or an approved state payload.

**Authority:** `engram.certify` on the target project and classification.  
**Mutations:** artifact intake; `engram_certificates` insert; ledger receipts; derived-index task enqueue.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "apparatus://schemas/protocol/engram.certify.v1",
  "type": "object",
  "additionalProperties": false,
  "required": ["project_id", "state", "representation", "source_refs"],
  "properties": {
    "project_id": {"type": "string", "format": "uuid"},
    "state": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind", "artifact_id"],
      "properties": {
        "kind": {"enum": ["text_features", "embedding_sequence", "perception", "session_features", "custom"]},
        "artifact_id": {"type": "string", "format": "uuid"},
        "custom_kind": {"type": ["string", "null"], "maxLength": 128}
      }
    },
    "representation": {
      "type": "object",
      "additionalProperties": false,
      "required": ["id", "algorithm_version", "parameters"],
      "properties": {
        "id": {"type": "string", "pattern": "^[a-z0-9][a-z0-9._-]{0,127}$"},
        "algorithm_version": {"type": "string", "maxLength": 64},
        "parameters": {"type": "object", "maxProperties": 32}
      }
    },
    "source_refs": {
      "type": "array", "minItems": 1, "maxItems": 1024,
      "items": {
        "type": "object", "additionalProperties": false,
        "required": ["artifact_id", "role"],
        "properties": {
          "artifact_id": {"type": "string", "format": "uuid"},
          "role": {"type": "string", "pattern": "^[a-z][a-z0-9_]{0,63}$"}
        }
      }
    },
    "observed_at_ms": {"type": ["integer", "null"], "minimum": 0},
    "labels": {"type": "object", "maxProperties": 24},
    "idempotency_key": {"type": "string", "minLength": 16, "maxLength": 128}
  }
}
```

**Required behavior:**

1. Validate schema and project-module admission.
2. Resolve every source artifact through Core access controls; derive source SHA-256 from Core metadata, not caller input.
3. Obtain a `StateVector` only through an approved extractor/worker path.
4. Run deterministic representation implementation.
5. Write AEG1 bytes through `artifact.intake`, never directly to a project path.
6. Insert metadata row and append a receipt.
7. Enqueue an idempotent derived-index update; failure to index does not invalidate the canonical certificate.

**Response:**

```json
{
  "certificate_id": "uuid",
  "artifact_id": "uuid",
  "artifact_sha256": "64 lowercase hex",
  "confidence": {"coverage": 0.72, "margin": null},
  "index_status": "queued"
}
```

### 4.2 `engram.query.v1`

**Purpose:** retrieve candidates, not facts.

**Authority:** `engram.query`; policy filters results by classification, project scope, zone, and caller grant.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "apparatus://schemas/protocol/engram.query.v1",
  "type": "object",
  "additionalProperties": false,
  "required": ["project_id", "query", "limit"],
  "properties": {
    "project_id": {"type": "string", "format": "uuid"},
    "query": {
      "oneOf": [
        {"type": "object", "additionalProperties": false, "required": ["certificate_id"], "properties": {"certificate_id": {"type": "string", "format": "uuid"}}},
        {"type": "object", "additionalProperties": false, "required": ["state_artifact_id", "representation_id"], "properties": {"state_artifact_id": {"type": "string", "format": "uuid"}, "representation_id": {"type": "string", "maxLength": 128}}}
      ]
    },
    "limit": {"type": "integer", "minimum": 1, "maximum": 50},
    "min_score": {"type": "number", "minimum": -1, "maximum": 1, "default": 0.0},
    "min_margin": {"type": ["number", "null"], "minimum": 0},
    "filters": {"type": "object", "additionalProperties": false, "properties": {"after_ms": {"type": "integer"}, "before_ms": {"type": "integer"}, "session_id": {"type": "string"}, "state_kinds": {"type": "array", "items": {"type": "string"}, "maxItems": 8}}},
    "include": {"type": "array", "items": {"enum": ["metadata", "artifact_ref", "links"]}, "uniqueItems": true}
  }
}
```

**Response semantics:**

- Return `status: "ok"` with ordered hits, or `status: "insufficient_resonance"` when no permitted result passes requested/project thresholds.
- Each hit includes certificate/artifact IDs, score, confidence, margin, provenance summary, and permitted metadata.
- A score is not a truth value. The response MUST label all hits as `candidate_memory`.
- The query protocol MUST NOT cause automatic certify, promotion, or mutation.

### 4.3 `engram.rebuild_index.v1`

**Purpose:** reconstruct a disposable index generation solely from active canonical certificates.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "apparatus://schemas/protocol/engram.rebuild_index.v1",
  "type": "object",
  "additionalProperties": false,
  "required": ["project_id"],
  "properties": {
    "project_id": {"type": "string", "format": "uuid"},
    "representation_id": {"type": ["string", "null"], "maxLength": 128},
    "reason": {"type": "string", "maxLength": 256},
    "verify_sample_size": {"type": "integer", "minimum": 0, "maximum": 10000, "default": 100}
  }
}
```

**Authority:** `engram.rebuild_index` or maintenance protocol lease.  
**Execution:** a worker task for large projects; no external worker gets direct project DB write authority.  
**Atomic switch:** build `generation=N+1`, verify, then update one project DB pointer transactionally. On failure retain `N` and append a failure receipt.

---

## 5. Project migration: `00xx_engram.sql`

```sql
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS engram_certificates (
  id                    TEXT PRIMARY KEY,
  artifact_id           TEXT NOT NULL UNIQUE,
  artifact_sha256       TEXT NOT NULL CHECK(length(artifact_sha256) = 64),
  state_kind            TEXT NOT NULL,
  representation_id     TEXT NOT NULL,
  representation_version TEXT NOT NULL,
  vector_dim            INTEGER NOT NULL CHECK(vector_dim > 0),
  vector_dtype          TEXT NOT NULL CHECK(vector_dtype IN ('f16', 'f32')),
  confidence_coverage   REAL,
  retrieval_margin      REAL,
  observed_at_ms        INTEGER NOT NULL,
  recorded_at_ms        INTEGER NOT NULL,
  classification        TEXT NOT NULL,
  status                TEXT NOT NULL CHECK(status IN ('active','superseded','tombstoned')),
  supersedes_id         TEXT REFERENCES engram_certificates(id),
  source_set_sha256     TEXT NOT NULL CHECK(length(source_set_sha256) = 64),
  metadata_json         TEXT NOT NULL,
  created_receipt_id    TEXT NOT NULL,
  UNIQUE(artifact_sha256)
);

CREATE INDEX IF NOT EXISTS idx_engram_cert_active_repr_time
  ON engram_certificates(status, representation_id, observed_at_ms DESC);

CREATE TABLE IF NOT EXISTS engram_links (
  id                TEXT PRIMARY KEY,
  certificate_id    TEXT NOT NULL REFERENCES engram_certificates(id),
  relation_type     TEXT NOT NULL CHECK(relation_type IN ('derived_from','continues','supports','contradicts','related_to')),
  target_kind       TEXT NOT NULL CHECK(target_kind IN ('artifact','claim','decision','task','certificate')),
  target_id         TEXT NOT NULL,
  created_at_ms     INTEGER NOT NULL,
  created_receipt_id TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_engram_links_certificate
  ON engram_links(certificate_id);

CREATE TABLE IF NOT EXISTS engram_index_generations (
  generation       INTEGER PRIMARY KEY,
  representation_id TEXT NOT NULL,
  path_rel         TEXT NOT NULL,
  certificate_count INTEGER NOT NULL,
  parameters_json  TEXT NOT NULL,
  source_snapshot_receipt_id TEXT NOT NULL,
  built_at_ms      INTEGER NOT NULL,
  activated_at_ms  INTEGER,
  status           TEXT NOT NULL CHECK(status IN ('building','active','failed','superseded')),
  failure_artifact_id TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS one_active_engram_generation_per_repr
  ON engram_index_generations(representation_id)
  WHERE status = 'active';
```

The artifact table, receipt table, and project/module registry are owned by Core migrations and are intentionally not duplicated here.

---

## 6. Golden-vector contract

### 6.1 Directory layout

```text
labs/engram-py/
  golden/
    manifest.json
    fingerprints/
      f0f1-001.input.f32le
      f0f1-001.expected.f32le
      f0f1-001.meta.json
    certificates/
      aeg1-001.aeg
      aeg1-001.decoded.json
    queries/
      exact-small-corpus.json
      expected-topk.json
```

### 6.2 Golden manifest requirements

`manifest.json` records:

- generator git commit and dirty state;
- Python version and numerical library versions;
- algorithm ID/version and exact parameters;
- input shape/dtype/endianness;
- tolerance policy;
- SHA-256 for every fixture;
- expected result ordering and tie-break behavior.

### 6.3 Determinism rules

- Inputs are stored as little-endian `f32` bytes and shape is in `.meta.json`.
- Rust and Python must use the same flattening order: row-major `[time, dimension]`.
- The baseline fingerprint uses the frozen E0 reference algorithm: rFFT on each declared signal axis, magnitude extraction for components `[0, 1]`, concatenate in documented order, then L2-normalize once over the final concatenated vector.
- If a source is too short for component `1`, certify MUST fail with `state_insufficient_for_representation`; it must not silently pad.
- Rust parity acceptance: absolute error <= `1e-5` for `f32` expected values and cosine similarity >= `0.99999` on the full final fingerprint. Any platform-specific SIMD relaxation needs a new manifest and owner decision.
- AEG1 encode/decode must be byte-exact for fixed test inputs where `recorded_at_ms` and UUID are injected.

### 6.4 Golden generator command contract

The Python reference lab must expose these commands; names can be adjusted but output semantics cannot:

```bash
# Freeze algorithm cases and fixture manifest. No network access.
python -m engram_lab.goldens fingerprint \
  --algorithm fourier.f0f1.raw.v1 \
  --input fixtures/states/ \
  --out labs/engram-py/golden/fingerprints/

# Encode deterministic AEG1 fixtures using injected UUID/timestamps.
python -m engram_lab.goldens certificate \
  --fixture labs/engram-py/golden/fingerprints/f0f1-001 \
  --certificate-id 018f0000-0000-7000-8000-000000000001 \
  --recorded-at-ms 1721817600000 \
  --out labs/engram-py/golden/certificates/aeg1-001.aeg

# Verify all hashes and expected outputs before a commit.
python -m engram_lab.goldens verify --manifest labs/engram-py/golden/manifest.json
```

The current upstream Python packages are the research/reference source. A small `engram_lab` adapter must be created rather than making Apparatus production code depend on llama.cpp, Torch, FAISS, or upstream MCP code.

### 6.5 Rust test commands

```bash
cargo test -p apparatus-engram-format --test aeg1_golden
cargo test -p apparatus-engram-math --test f0f1_golden
cargo test -p apparatus-engram-index --test exact_query_golden
cargo test -p apparatus-engram-bridge --test certify_query_rebuild_e2e
```

No golden fixture can be updated merely because a production test fails. Update requires: explicit algorithm/version decision, regenerated manifest, review, and an ADR/receipt in the development project.

---

## 7. Rust crate API seam (E0 skeleton)

```rust
// apparatus-engram-format
pub const AEG1_MAGIC: [u8; 4] = *b"AEG1";
pub const AEG1_HEADER_LEN: usize = 160;

pub struct Aeg1Header { /* exact fields in Â§3.3 */ }
pub struct EngramCertificate {
    pub header: Aeg1Header,
    pub vector: Vec<f32>,
    pub metadata: CertificateMetadata,
}

pub fn encode(cert: &EngramCertificate) -> Result<Vec<u8>, FormatError>;
pub fn decode(bytes: &[u8]) -> Result<EngramCertificate, FormatError>;

// apparatus-engram-math
pub trait StateVector {
    fn state_kind(&self) -> StateKind;
    fn shape(&self) -> StateShape;
    fn values_f32_le(&self) -> &[f32];
}

pub trait Fingerprinter {
    fn representation_id(&self) -> &str;
    fn fingerprint(&self, input: &dyn StateVector) -> Result<Fingerprint, MathError>;
}

// apparatus-engram-index
pub trait EngramIndex {
    fn build(&mut self, entries: impl Iterator<Item = IndexEntry>) -> Result<IndexBuildReport, IndexError>;
    fn query(&self, query: &[f32], limit: usize, filter: &IndexFilter) -> Result<Vec<IndexHit>, IndexError>;
}
```

`apparatus-engram-format` and `apparatus-engram-math` MUST NOT depend on `rusqlite`, `axum`, MCP, or a specific ANN backend.

---

## 8. E0 acceptance tests

E0 is complete only when all are true:

1. A fixed AEG1 fixture decodes identically in Python reference adapter and Rust codec.
2. Rust re-encodes the injected-time/UUID fixture byte-for-byte.
3. Corrupt magic, version, header length, hash, vector length, JSON, NaN and trailing-byte cases are rejected.
4. Rust Fourier f0/f1 fingerprint meets every golden tolerance.
5. `engram.certify.v1` through a test Core creates exactly one artifact, one certificate row and expected receipts.
6. Deleting `derived/engram/` followed by `engram.rebuild_index.v1` restores golden retrieval result order.
7. An unauthorized caller cannot certify, query, rebuild, or read the certificate artifact.
8. An EGR1/EGRV import test, if adapters exist, preserves original bytes as source artifact and emits a distinct AEG1 certificate with import provenance.
9. Backup/restore test restores the certificate artifact and DB row; index may be absent and must rebuild.

---

## 9. Deferred decisions

| Topic | Decision now |
|---|---|
| HNSW/usearch/Rust ANN backend | Interface only; benchmark after exact index works |
| FFT crate or PFFFT FFI | Rust implementation first; optimize only with profiles |
| FlatBuffers | Not needed for AEG1; reconsider for future robot C++/Go export |
| VSA bind/bundle / resonance | Not in E0; later representation kinds |
| Continuous 10â€“30fps writes | Not E0; requires worker/backpressure design |
| MCP tools | M3 edge concern; protocols are the contract now |
| Legacy import | Optional adapter; never makes legacy format canonical |
| Signed certificates | Artifact and ledger integrity first; optional detached signatures later |

---

## 10. Implementation order

1. Create `apparatus-engram-format` with AEG1 parser/encoder and malformed-input tests.
2. Add `engram-certificate-metadata.v1` schema and canonical JSON writer.
3. Build Python `engram_lab` adapter and freeze first goldens.
4. Create `apparatus-engram-math`; pass f0/f1 parity suite.
5. Add project migration and `engram.certify.v1` bridge, calling Core artifact intake.
6. Add exact brute-force index and `engram.query.v1`.
7. Add rebuild protocol, crash-safe generation switch, and restore test.
8. Only then benchmark an ANN implementation and expose MCP tools.

---

## 11. Final invariant

**AEG1 certificates are remembered evidence artifacts. They are not authority, truth, permission, or an alternative database.**

Apparatus keeps the governing record. Engram supplies confidence-scored associative recall under that record's policy, provenance, and recovery rules.



# E0 Contract Pack Created

I created the full implementation-ready E0 pack: **[Download the E0 Contract Pack](sandbox:/output/APPARATUS_ENGRAM_E0_CONTRACT_PACK.md)**. It freezes Engram’s first lawful interface with Apparatus, including byte format, project migration, protocol schemas, golden-vector parity rules, crate seams, and acceptance tests. 

## Key decisions frozen

- Apparatus uses a new canonical **AEG1** certificate format rather than treating upstream `EGR1` or `EGRV` as Core canon. The upstream formats remain importable later with provenance, but their evolving/heuristic extensions are not made institutional law. 
- An Engram certificate is an immutable Core artifact with media type `application/vnd.apparatus.engram;version=1`; indexes are derived, deletable, and rebuildable. 
- The fixed AEG1 header is exactly **160 bytes**, uses deterministic little-endian fields, UUIDv7, timestamps, vector dimensions/type, and hashes binding representation, basis, and source set. 
- Certificate metadata is bounded canonical JSON, while the fingerprint vector stays compact binary f16/f32. The contract explicitly rejects secrets, raw source dumps, and any claim that a memory certificate equals a fact or authority decision. 

## Protocols included

| Protocol | Purpose |
|---|---|
| `engram.certify.v1` | Authorized source artifacts/state → fingerprint → AEG1 artifact → receipt → index task |
| `engram.query.v1` | Recall ranked **candidate memories**, filtered by project policy and classification |
| `engram.rebuild_index.v1` | Rebuild and atomically activate a disposable index generation |

The schemas include validation constraints, authority expectations, response semantics, and the rule that query results are candidates rather than truth claims. 

## Engineering path

The pack specifies these crates:

```text
apparatus-engram-format   # AEG1 encoding/decoding only
apparatus-engram-math     # StateVector + Fourier f0/f1 fingerprint
apparatus-engram-index    # exact index first; ANN backend later
apparatus-engram-service  # certify/query/rebuild domain logic
apparatus-engram-bridge   # Core protocols, migration, policy integration
```

The format and math crates must remain pure: no SQLite, Axum, MCP, or ANN-backend dependencies. 

## First build order

1. Implement `apparatus-engram-format` and malformed-input tests.  
2. Create the small Python reference adapter and freeze initial golden vectors.  
3. Implement Rust Fourier f0/f1 parity.  
4. Add the project migration plus `engram.certify.v1`.  
5. Use a brute-force exact index first.  
6. Implement query and rebuild flows.  
7. Benchmark HNSW/usearch only after correctness, restore, and policy tests pass. 

## Important correction preserved

The upstream Engram EIGENGRAM format has a documented `EGR1` header, but the current writer inserts later extension fields while retaining version `1`, and the reader infers extensions from remaining length. The E0 pack deliberately avoids carrying that ambiguity into Apparatus canon. 

The document ends with the operative invariant:

> **AEG1 certificates are remembered evidence artifacts. They are not authority, truth, permission, or an alternative database.**

Bronnen
# Apparatus Development Environment â€” macOS Bootstrap Runbook

**ID:** APPARATUS-DEV-ENV-001  
**Status:** Developer bootstrap baseline  
**Platform:** macOS on Apple Silicon or Intel MacBook  
**Primary stack:** Rust, SQLite, local files, Python reference lab only  
**IDE recommendation:** VS Code first; IntelliJ IDEA Ultimate + Rust plugin is an equally valid alternative  

---

## 1. Purpose

This runbook sets up a conservative local environment for building Apparatus and the optional Engram module. It installs development mechanisms, not product architecture. The project dependency manifests remain the only authority for application libraries.

**Do not install a database server, vector database, Kafka, Redis, Docker-only stack, agent framework, MCP server, or model runtime merely because it is available.** Apparatus Core begins as a Rust workspace plus SQLite files.

---

## 2. Choose one IDE

### Default: VS Code

Use VS Code when you want a simple workspace, terminal-first flow, and easy use of AI coding tools.

Install these extensions only:

| Extension ID | Purpose | Required |
|---|---|---:|
| `rust-lang.rust-analyzer` | Rust language server, navigation, diagnostics, formatting | Yes |
| `vadimcn.vscode-lldb` | Native debugging via LLDB | Yes |
| `tamasfe.even-better-toml` | Cargo.toml editing | Recommended |
| `redhat.vscode-yaml` | CI/config YAML validation | Recommended |
| `EditorConfig.EditorConfig` | Respect `.editorconfig` | Recommended |
| `ms-python.python` | Python golden/reference lab only | Later when lab starts |
| `ms-python.vscode-pylance` | Python analysis | Later when lab starts |

Do **not** install the deprecated `rust-lang.rust` extension. Use `rust-analyzer`.

### Alternative: IntelliJ IDEA / RustRover / CLion

If using JetBrains, install the **Rust** plugin from `Settings â†’ Plugins â†’ Marketplace`; it is supported across IntelliJ-based IDEs. Use it with the repository root opened as the project, not an individual crate. CLion is useful later only if a C/C++ robot export becomes active.

Use one IDE as primary at a time. Keep project configuration editor-neutral.

---

## 3. Bootstrap prerequisites

Open Terminal and run each section deliberately. Read command output; do not paste the whole file blindly.

### 3.1 Xcode command-line tools

```bash
xcode-select --install
```

After installation:

```bash
xcode-select -p
clang --version
make --version
```

### 3.2 Homebrew

If `brew --version` does not work, install from the official Homebrew site. Then:

```bash
brew update
brew install git gh just jq yq sqlite pkg-config openssl@3 age restic shellcheck
brew install --cask visual-studio-code
```

Optional later tools, **not required for Day 1**:

```bash
brew install cargo-watch hyperfine
brew install --cask docker
```

Docker is optional. Do not use it to replace local Rust/SQLite development.

### 3.3 Git identity and GitHub CLI

```bash
git config --global user.name "YOUR NAME"
git config --global user.email "YOUR EMAIL"
git config --global init.defaultBranch main
git config --global fetch.prune true
gh auth login
```

Confirm:

```bash
git --version
gh auth status
```

Use SSH Git remotes where possible. Do not place GitHub tokens in the repository, shell history, `.env` committed files, artifacts, or Engram certificates.

### 3.4 Rust stable toolchain

Install Rust using the official rustup installer:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Close and reopen Terminal, then run:

```bash
rustup toolchain install stable
rustup default stable
rustup component add rustfmt clippy rust-src
rustup update stable
rustc --version
cargo --version
rustfmt --version
cargo clippy --version
```

Use stable Rust. Do not switch the whole project to nightly without a written ADR and a specific blocked requirement.

### 3.5 Cargo developer tools

Install only tooling, not project runtime dependencies:

```bash
cargo install cargo-deny --locked
cargo install cargo-audit --locked
cargo install cargo-nextest --locked
cargo install cargo-llvm-cov --locked
```

Verify:

```bash
cargo deny --version
cargo audit --version
cargo nextest --version
cargo llvm-cov --version
```

If a `cargo install` fails, preserve the error output, update Rust stable once, and retry. Do not add random `--force`, nightly, or unpinned Git dependencies to bypass a failure.

### 3.6 Python reference-lab environment â€” defer until Engram E0 goldens

Python is for `labs/engram-py` only; it must not become a runtime dependency of `apparatusd`.

When E0 golden fixtures begin:

```bash
brew install uv
uv python install 3.12
cd labs/engram-py
uv venv --python 3.12
source .venv/bin/activate
uv pip install -r requirements-dev.txt
```

The `requirements-dev.txt` must be committed and pinned before installing any Python research dependency. Do not run upstream installation scripts that download model weights or start network services without explicit approval.

---

## 4. Workspace checkout and baseline

Choose a local work directory outside Desktop, iCloud Drive, Dropbox, or synced folders. Example:

```bash
mkdir -p "$HOME/Developer"
cd "$HOME/Developer"
git clone git@github.com:YOUR-ORG/apparatus.git
cd apparatus
```

If the repository does not exist yet:

```bash
mkdir -p "$HOME/Developer/apparatus"
cd "$HOME/Developer/apparatus"
git init
git branch -M main
```

Create these top-level files/directories before writing implementation code:

```text
apparatus/
â”œâ”€â”€ README.md
â”œâ”€â”€ AGENTS.md
â”œâ”€â”€ Cargo.toml
â”œâ”€â”€ Cargo.lock
â”œâ”€â”€ rust-toolchain.toml
â”œâ”€â”€ deny.toml
â”œâ”€â”€ .gitignore
â”œâ”€â”€ .editorconfig
â”œâ”€â”€ .vscode/                 # optional editor config, committed
â”œâ”€â”€ docs/
â”‚   â”œâ”€â”€ adr/
â”‚   â”œâ”€â”€ architecture/
â”‚   â””â”€â”€ research/
â”œâ”€â”€ schemas/
â”œâ”€â”€ migrations/
â”‚   â”œâ”€â”€ control/
â”‚   â””â”€â”€ project_template/
â”œâ”€â”€ crates/
â”œâ”€â”€ bins/
â”œâ”€â”€ tests/
â”œâ”€â”€ scripts/
â”œâ”€â”€ fixtures/
â””â”€â”€ labs/
    â””â”€â”€ engram-py/           # later, reference/golden lab only
```

Copy the previously created architecture reports and ADRs into `docs/architecture/` and `docs/adr/`. Keep raw external research/docs under `docs/research/` with source URL and retrieval date in a small README.

---

## 5. Mandatory starter configuration

### 5.1 `rust-toolchain.toml`

```toml
[toolchain]
channel = "stable"
profile = "minimal"
components = ["rustfmt", "clippy", "rust-src"]
```

### 5.2 `.editorconfig`

```ini
root = true

[*]
charset = utf-8
end_of_line = lf
insert_final_newline = true
indent_style = space
indent_size = 2
trim_trailing_whitespace = true

[*.rs]
indent_size = 4

[*.{md,yml,yaml,json,toml}]
indent_size = 2
```

### 5.3 `.gitignore`

```gitignore
/target/
/.DS_Store
.env
.env.*
!.env.template
*.sqlite-wal
*.sqlite-shm
/data/
/run/
/tmp/
/coverage/
/labs/**/.venv/
/labs/**/__pycache__/
/labs/**/*.py[cod]
.vscode/*.local.json
.idea/workspace.xml
```

Do not ignore migration files, fixture manifests, `Cargo.lock`, golden-vector manifests, ADRs, or test data that is safe to publish.

### 5.4 VS Code workspace recommendations

Create `.vscode/extensions.json`:

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "vadimcn.vscode-lldb",
    "tamasfe.even-better-toml",
    "redhat.vscode-yaml",
    "EditorConfig.EditorConfig"
  ]
}
```

Create `.vscode/settings.json`:

```json
{
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll": "explicit",
    "source.organizeImports": "explicit"
  },
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.cargo.allFeatures": false,
  "rust-analyzer.cargo.features": [],
  "files.watcherExclude": {
    "**/target/**": true
  },
  "search.exclude": {
    "**/target/**": true,
    "**/.git/**": true
  }
}
```

Do not enable `allFeatures` by default: optional experimental adapters must not quietly become part of the everyday build.

### 5.5 `justfile` command surface

Create a `justfile` with only the following safe initial commands:

```make
set shell := ["zsh", "-cu"]

default:
  @just --list

fmt:
  cargo fmt --all -- --check

fmt-fix:
  cargo fmt --all

lint:
  cargo clippy --workspace --all-targets -- -D warnings

test:
  cargo nextest run --workspace

build:
  cargo build --workspace --locked

check:
  just fmt
  just lint
  just test

deny:
  cargo deny check

audit:
  cargo audit

cov:
  cargo llvm-cov nextest --workspace --html
```

Do not add `run`, database-destroying, cache-clearing, network, deployment, or secret-reading recipes until their protocols exist.

---

## 6. Initial Rust workspace policy

The AI or developer must create the workspace with the architectural package boundaries already agreed:

```text
crates/
  apparatus-types
  apparatus-schema
  apparatus-crypto
  apparatus-time
  apparatus-store
  apparatus-artifacts
  apparatus-ledger
  apparatus-registry
  apparatus-policy
  apparatus-protocols
  apparatus-tasks
  apparatus-gateway
  apparatus-observe
  apparatus-maintenance
  apparatus-engram-format       # E0 begins here only after Core M0
  apparatus-engram-math
  apparatus-engram-index
  apparatus-engram-service
  apparatus-engram-bridge
bins/
  apparatusd
  apparatus-cli
  apparatus-worker
```

Start implementation with only the M0 dependency direction:

```text
types/schema/time/crypto
  â†’ store
  â†’ artifacts + ledger
  â†’ protocols/services
  â†’ binaries
```

Do not create empty crates merely to look complete. A crate is added when its public boundary is needed; the directory map is a target, not a mandate to generate boilerplate everywhere on Day 1.

---

## 7. Dependency admission procedure

Before adding a crate, external project, extension, SDK, or service:

1. State the capability needed in the PR/task note.
2. Search existing workspace dependencies first.
3. Check license, maintenance, security advisories, and whether it can be replaced or isolated.
4. Decide tier: A foundation, B operational, C utility/projection, or D experimental.
5. Add the smallest feature set; never use wildcard features by default.
6. Update `Cargo.lock`.
7. Run `cargo deny check`, `cargo audit`, tests, and license review.
8. Record why it is allowed in ADR or dependency inventory.

Initial intended categories:

| Need | Candidate | Notes |
|---|---|---|
| Async/runtime | `tokio` | Core runtime |
| HTTP boundary | `axum`, `tower`, `tower-http` | Later gateway |
| Serialization | `serde`, `serde_json` | Typed envelopes |
| SQLite | `rusqlite` | Canonical DB, behind store trait |
| IDs | `uuid` with v7 support | Object identity |
| Hash | `sha2`, `blake3` | Explicit crypto roles |
| Errors | `thiserror`, `anyhow` | Library vs binary split |
| Logging | `tracing`, `tracing-subscriber` | Structured observable daemon |
| Policy | `cedar-policy` | Behind `PolicyEngine` trait, after spike |
| Schemas | `schemars`, `jsonschema` | Contracts and validation |

Do not add Engram ANN, FFT, MCP SDK, vector extensions, model runtimes, `sqlite-vec`, or worker sandbox crates during the M0 bootstrap. These are phase-gated.

---

## 8. Day-one verification

From repository root run:

```bash
brew --version
git --version
gh auth status
rustup show active-toolchain
cargo --version
cargo fmt --version
cargo clippy --version
cargo deny --version
cargo audit --version
sqlite3 --version
age --version
restic version
just --version
```

After initial workspace exists:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
cargo deny check
cargo audit
```

The initial repository must be clean after verification:

```bash
git status --short
```

Expected output: nothing.

---

## 9. AI-tool operating boundaries

Claude Code, Antigravity, Grok Build, IDE assistants, and other agents are editors and reviewersâ€”not authorities.

They may:

- read local project documents;
- create reviewed source, tests, schemas, migrations and docs;
- run local formatting, checking and tests;
- propose narrowly scoped dependencies after the admission procedure;
- inspect package documentation before implementation.

They must not, without explicit human approval:

- transmit source, reports, secrets, artifacts or project data to external services;
- run destructive filesystem commands or reset/rewrite Git history;
- install a service/database/model runtime outside this runbook;
- use `curl | sh` except the explicit official rustup install above;
- add cloud credentials, telemetry, analytics, auto-updaters, webhooks, or remote MCP servers;
- use `--no-verify`, blanket `unsafe`, wildcard dependency features, or unreviewed Git dependencies;
- change ADR decisions or generated golden fixtures to make tests pass.

Every AI task begins with `AGENTS.md` and the applicable ADR/E0 document. See `APPARATUS_MASTER_STARTING_PROMPT.md`.

---

## 10. Recovery notes

- If Rust tooling becomes inconsistent: `rustup update stable`, reopen terminal, then rerun verification.
- If macOS SDK/compiler errors occur: `xcode-select --install` or update Xcode command-line tools; do not patch toolchain paths inside the repository.
- If a crate needs OpenSSL: prefer system/toolchain compatibility first; record any explicit `OPENSSL_DIR` environment configuration in local setup documentation, never in source.
- If a build depends on non-Rust native code: stop and write a small dependency decision before adding a Homebrew library or FFI layer.
- If an AI tool changes many unrelated files: stop, inspect `git diff --stat` and `git diff`, then revert only reviewed changes.


# Apparatus â€” Master Starting Prompt for AI Development Tools

Copy this entire prompt into Claude Code, Antigravity, Grok Build, or another coding agent **from the repository root**. Replace bracketed fields only where necessary.

---

You are a careful senior Rust infrastructure engineer working inside the local **Apparatus** repository on macOS. Your job is to help implement a local-first, owner-operated, governed systemâ€”not a generic AI agent platform.

## Authority order

Read these local files before changing anything, in this order when they exist:

1. `AGENTS.md`
2. `README.md`
3. `docs/adr/ADR-0001*` or the Open-Source Alignment Decision Record
4. `docs/architecture/` Core architecture documents
5. `docs/architecture/APPARATUS_ENGRAM_E0_CONTRACT_PACK.md` when touching Engram
6. `docs/research/` only as supporting context
7. Current `Cargo.toml`, `deny.toml`, migrations, schemas, and tests

If documents conflict, use this precedence:

```text
accepted ADR / Core Package Architecture
  > E0 contract pack
  > implementation tests and schemas
  > research reports
  > external repository documentation
  > this prompt
```

If a required document is missing, say exactly which document is needed and proceed only with a narrow, reversible setup task. Do not invent architecture.

## Core invariants

1. Apparatus Core owns identity, policy, artifact CAS, append-only ledger, protocols, task leases, and restoreability.
2. SQLite through `rusqlite` is the canonical default store; databases and artifacts are local files.
3. Every durable mutation is validated, authorized, idempotent where relevant, and receipted in the ledger.
4. Artifacts are immutable, content-addressed bytes. Derived indexes/caches are disposable and rebuildable.
5. External AI/MCP clients have zero inherent authority. They use typed, capability-gated protocol calls only.
6. Workers do not access Core databases or artifact roots directly.
7. No raw shell, arbitrary filesystem, arbitrary SQL, or unrestricted network tool is exposed through MCP.
8. Engram certificates are candidate-memory/evidence artifacts, not facts, authorization, or a second database.
9. Prefer a small explicit dependency tree over convenience abstraction.
10. Work in phases. Do not pull future components into M0.

## Current scope

Current task: **[INSERT ONE CONCRETE TASK, e.g. â€œCreate the M0 Rust workspace and implement only apparatus-types + apparatus-store skeleton with compilation tests.â€]**

Allowed phase: **[M0 / E0 / M1 / etc.]**

Explicitly out of scope: **[LIST]**

## Mandatory workflow

Before editing:

1. Read the authority files above.
2. Inspect existing repository state with `git status --short`, workspace manifests, and relevant source/tests.
3. State a concise implementation plan containing: files to change, public interfaces, migration/schema effects, dependencies to add, and tests to run.
4. Identify any ambiguity or conflict. Ask one focused question if it materially changes design; otherwise choose the smallest reversible compliant option.

During implementation:

1. Make the smallest coherent change. Do not refactor unrelated code.
2. Use stable Rust; do not switch to nightly.
3. Preserve crate dependency direction. Keep pure types/format/math crates independent of HTTP, SQLite, MCP, and ANN backends unless the ADR explicitly says otherwise.
4. Add/modify tests with behavior changes.
5. Use structured errors; do not hide errors with `unwrap`, `expect`, broad catch-all behavior, or ignored `Result`s in production paths.
6. Validate all boundary data. Bound allocations and lengths when parsing files/protocol payloads.
7. Do not log secrets, certificate source content, bearer tokens, or full sensitive payloads.
8. Do not add a dependency without explaining its tier, features, license/maintenance suitability, and why an existing dependency cannot do the job.

Before reporting completion run, when applicable:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
cargo deny check
cargo audit
```

For a partial/new workspace, run the applicable subset and state exactly what could not be run and why.

Then report:

1. What changed and why
2. Files changed
3. Public API/schema/migration changes
4. Dependencies added or declined
5. Commands run and results
6. Remaining risks/open questions
7. `git diff --stat` summary

## Dependency gate

Do not add any crate/service/SDK before first checking whether it is already allowed by ADR-0001. Use the following rules:

- Tier A: foundation; requires unusually strong justification.
- Tier B: bounded operational dependency; prefer a short spike/adapter.
- Tier C: utility or projection; must degrade safely.
- Tier D: experimental; feature-gated, never default/canonical.

Forbidden without an explicit new ADR and human approval:

- Limbo as Core storage
- external vector database
- Kafka, NATS, RabbitMQ, Temporal, or a distributed broker
- Keycloak-first identity architecture
- enterprise/dynamic-plugin MCP stack
- generic shell, filesystem, or SQL MCP tool
- cloud-required dependency in the Core path
- model runtime or agent framework as Apparatus substrate
- Redis/S3 as Engram canon
- Git URL dependency, unpublished crate, or unpinned branch dependency

## Engram-specific rules

When task touches Engram, follow `APPARATUS_ENGRAM_E0_CONTRACT_PACK.md` exactly:

- AEG1 is canonical; upstream EGR1/EGRV are optional imports only.
- Certificate bytes enter through Core artifact intake.
- Index files are under derived storage and must be rebuildable.
- Do not invent AEG1 header extensions or trailing-byte heuristics.
- Freeze and test golden vectors before optimizing or adding ANN.
- Start with exact retrieval before HNSW/usearch.
- Do not add Python/Torch/FAISS/llama.cpp dependencies to `apparatusd`.

## Safety and tool restrictions

Never do any of these without explicit human confirmation in the current conversation/task:

- `rm -rf`, destructive database operations, mass deletion, force-push, history rewrite, reset --hard
- sending repository content to web services, telemetry, analytics, or remote agents
- reading/writing credentials, keychains, SSH keys, `.env` secrets
- installing system services, background daemons, databases, model runtimes, or Docker containers
- changing accepted ADRs, cryptographic formats, migrations, or golden fixtures just to satisfy a test
- bypassing checks with `--no-verify`, disabling lints, adding `unsafe`, or weakening validation

You may run read-only inspection, Cargo commands, formatter/linter/test commands, and the narrow setup commands documented in `APPARATUS_DEV_ENV_001.md`.

## Starting response format

Start now by responding with exactly these headings:

```markdown
## Documents Read
## Repository State
## Plan
## Proposed Dependencies
## Verification Plan
## Questions or Assumptions
```

Do not write code until after this plan is shown, unless the task is purely to inspect/report.