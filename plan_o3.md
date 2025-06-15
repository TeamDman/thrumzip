# Meta-Takeout → DuckDB Migration Plan

Our end-goal is to replace a pile of bulky, overlapping Facebook
“Download Your Information” ZIP archives with a single, compressed,
query-friendly DuckDB database.  
After *every* new export you should be able to run a single CLI command:

```bash
meta-takeout ingest --db ./meta.duckdb --zips ~/Downloads/facebook_export
```

and walk away knowing that:

- Any file already present (byte-identical or perceptually identical) is
  deduplicated.
- Brand-new or changed items are appended with full provenance.
- Schema-aware data (messages, reactions, etc.) is split into typed
  columns instead of being stored as raw JSON.
- The CLI warns you, with diff-like details, whenever the ingest detects
  a field that looks “wrong” (hash mismatch, truncated thread, etc.).

---

## 1. Bird’s-eye workflow

```mermaid
flowchart TD
    A[CLI: meta-takeout ingest] -->|1. Scan dir| B[ZIP Enumerator]
    B -->|2. For each ZIP| C[Dedup Check<br/>(against DB)]
    C -->|a. Already known| D[Skip / link provenance]
    C -->|b. Unknown| E[Content Classifier]
    E -->|Image| F[Perceptual Hash&nbsp;+ Resize → Images table]
    E -->|JSON| G[Schema Extractor]
    E -->|Other| H[Binary blob table]
    F & G & H --> I[DuckDB Writer]
    I --> J[Commit + Update Indexes]
    J --> K[Done – “Safe to delete ZIPs?” prompt]
```

Legend  
- Solid arrows = synchronous pipeline stages  
- Dashed arrows = background tasks (hashing, decompression) – will be
  implemented with `tokio`’s task system.

---

## 2. DuckDB schema (v0)

```mermaid
erDiagram
    zips {
        uuid           id PK
        text           path
        timestamp      ingested_at
        text           facebook_export_id
    }
    entries {
        uuid           id PK
        uuid           zip_id FK
        text           inside_path
        text           content_type  "image/png, message/json, …"
        blob           raw_bytes     "only if not handled elsewhere"
        timestamp      seen_at
        text           sha256        "quick equality check"
    }
    images {
        uuid           entry_id  PK  FK
        text           perceptual_hash64
        int            width
        int            height
        int            compressed_size
    }
    messages {
        uuid           entry_id  PK  FK
        text           thread_id
        timestamp      sent_time
        text           sender
        text           message
        json           raw_json   "fallback for unmapped fields"
    }

    zips ||--o{ entries : contains
    entries ||--|| images : "image/*"
    entries ||--|| messages : "message/*"
```

Notes  
- `entries.raw_bytes` is *null* once a specialised table (images,
  messages, …​) has taken ownership of the data.  
- Perceptual hashes use the same gradient algorithm already prototyped
  in `examples/perceptual_equality.rs`.

---

## 3. CLI surface (draft)

- `meta-takeout ingest --db PATH --zips DIR [--force-rehash]`
- `meta-takeout ls            --db PATH [--type image|message]`
- `meta-takeout export        --db PATH --out DIR --query SQL`

`clap` will be used for argument parsing; everything else is async
Tokio. Output is intentionally *plain* text or CSV so you can pipe into
`duckdb`, `jq`, `fzf`, etc.

---

## 4. Implementation phases

1. Bootstrap
   - Create `duckdb::Connection` helper and run `CREATE TABLE` DDL if the
     database is empty.
   - Re-use the image path & hash prototype to fill `entries` and
     `images`.
2. Generalise the **Content Classifier**
   - Walk every extension found in the ZIPs
   - Emit a report of “known” vs “unknown” types so we can prioritise
     next work.
3. Message JSON extractor
   - Facebook stores messages in
     `messages/inbox/*/message_*.json`.  
   - Write a streaming parser (`serde_json::Deserializer::from_slice`)
     to avoid loading multi-GB files at once.
4. Provenance & auditing
   - For every duplicate pick the *smallest* compressed entry but keep a
     `zips_entries` link table with *all* sources so nothing is lost.
   - Add invariants (`CHECK` constraints) and nightly test runs that
     download the latest export and run a dry-run ingest.
5. Space reclamation & safety net
   - Once `entries` for a ZIP report *all* rows present somewhere else,
     print `rm -v <zip>` suggestions.
   - Offer `meta-takeout vacuum` which internally triggers
     `duckdb:PRAGMA optimize`.

---

## 5. Exploratory helper program

A small `cargo run --example scan_extensions` will:

- Iterate every ZIP
- Emit a frequency histogram of extensions it finds
- Sample 5 random files per unseen extension for manual inspection

This feeds back into phase 2’s classifier list.

---

## 6. Long-term vision

- Build a tiny Web UI (Svelte + WASM DuckDB) for full-text search across
  messages and image preview.  
- Plug the same dedup engine into Google Takeout, iCloud, etc.  
- Publish the CLI on `crates.io`; the whole stack is *local-first*,
  private, and script-friendly.

---

Feel free to poke holes in any of the assumptions above – the roadmap is
yours to adapt as you explore edge cases.