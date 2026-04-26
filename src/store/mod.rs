// Step-1 foundation: on-disk JSON store under ~/Documents/Epidote/. The panels
// still drive their own seeded mocks; this module is wired in but not yet read
// from. Migration to FK-driven panels happens in step 5.
#![allow(dead_code)]

mod types;

pub use types::*;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::{TimeZone, Utc};
use serde::{de::DeserializeOwned, Serialize};

const MEETINGS_DIR: &str = "meetings";
const TASKS_DIR: &str = "tasks";
const PROPOSALS_DIR: &str = "proposals";
const TRANSCRIPTS_DIR: &str = "transcripts";
const VAULT_DIR: &str = "Vault";
const AUDIO_DIR: &str = "meetings/audio";

#[derive(Debug)]
pub enum StoreError {
    Io(io::Error),
    Json(serde_json::Error),
    HomeNotFound,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Io(e) => write!(f, "io: {e}"),
            StoreError::Json(e) => write!(f, "json: {e}"),
            StoreError::HomeNotFound => write!(f, "could not resolve user Documents directory"),
        }
    }
}
impl std::error::Error for StoreError {}
impl From<io::Error> for StoreError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

#[derive(Debug)]
pub struct Store {
    root: PathBuf,
    pub meetings: Vec<Meeting>,
    pub tasks: Vec<Task>,
    pub proposals: Vec<Proposal>,
}

impl Store {
    /// Load from `~/Documents/Epidote`, seeding on first run.
    pub fn load() -> Result<Self, StoreError> {
        let root = default_root().ok_or(StoreError::HomeNotFound)?;
        Self::load_from(root)
    }

    /// Load from an explicit root. Used by tests.
    pub fn load_from(root: PathBuf) -> Result<Self, StoreError> {
        ensure_dirs(&root)?;
        if !has_json_files(&root.join(MEETINGS_DIR))? {
            write_seed(&root)?;
        }
        let meetings = read_records::<Meeting>(&root.join(MEETINGS_DIR))?;
        let tasks = read_records::<Task>(&root.join(TASKS_DIR))?;
        let proposals = read_records::<Proposal>(&root.join(PROPOSALS_DIR))?;
        Ok(Self {
            root,
            meetings,
            tasks,
            proposals,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Resolve a path from a stored record. Stored paths are relative to the
    /// store root so the user's `~/Documents/Epidote` folder stays portable.
    pub fn resolve(&self, rel: &Path) -> PathBuf {
        if rel.is_absolute() {
            rel.to_path_buf()
        } else {
            self.root.join(rel)
        }
    }

    pub fn save_meeting(&mut self, m: Meeting) -> Result<(), StoreError> {
        write_record(&self.root.join(MEETINGS_DIR), &m.id, &m)?;
        upsert(&mut self.meetings, m, |x| &x.id);
        Ok(())
    }

    pub fn save_task(&mut self, t: Task) -> Result<(), StoreError> {
        write_record(&self.root.join(TASKS_DIR), &t.id, &t)?;
        upsert(&mut self.tasks, t, |x| &x.id);
        Ok(())
    }

    pub fn save_proposal(&mut self, p: Proposal) -> Result<(), StoreError> {
        write_record(&self.root.join(PROPOSALS_DIR), &p.id, &p)?;
        upsert(&mut self.proposals, p, |x| &x.id);
        Ok(())
    }

    pub fn load_transcript(&self, meeting_id: &str) -> Result<Option<Transcript>, StoreError> {
        let path = self
            .root
            .join(TRANSCRIPTS_DIR)
            .join(format!("{meeting_id}.json"));
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(&path)?;
        Ok(Some(serde_json::from_slice(&bytes)?))
    }

    pub fn save_transcript(&self, t: &Transcript) -> Result<(), StoreError> {
        write_record(&self.root.join(TRANSCRIPTS_DIR), &t.meeting_id, t)
    }
}

fn default_root() -> Option<PathBuf> {
    dirs::document_dir().map(|d| d.join("Epidote"))
}

fn ensure_dirs(root: &Path) -> io::Result<()> {
    for sub in [
        MEETINGS_DIR,
        TASKS_DIR,
        PROPOSALS_DIR,
        TRANSCRIPTS_DIR,
        VAULT_DIR,
        AUDIO_DIR,
    ] {
        fs::create_dir_all(root.join(sub))?;
    }
    Ok(())
}

fn has_json_files(p: &Path) -> io::Result<bool> {
    if !p.exists() {
        return Ok(false);
    }
    for entry in fs::read_dir(p)? {
        let entry = entry?;
        if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
            return Ok(true);
        }
    }
    Ok(false)
}

fn read_records<T: DeserializeOwned>(dir: &Path) -> Result<Vec<T>, StoreError> {
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let bytes = fs::read(&path)?;
        out.push(serde_json::from_slice(&bytes)?);
    }
    Ok(out)
}

fn write_record<T: Serialize>(dir: &Path, id: &str, value: &T) -> Result<(), StoreError> {
    fs::create_dir_all(dir)?;
    let final_path = dir.join(format!("{id}.json"));
    let tmp_path = dir.join(format!("{id}.json.tmp"));
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(&tmp_path, &bytes)?;
    fs::rename(&tmp_path, &final_path)?;
    Ok(())
}

fn upsert<T, F: Fn(&T) -> &String>(vec: &mut Vec<T>, item: T, key: F) {
    let k = key(&item).clone();
    if let Some(pos) = vec.iter().position(|x| key(x) == &k) {
        vec[pos] = item;
    } else {
        vec.push(item);
    }
}

// ----- Seed -----

fn write_seed(root: &Path) -> Result<(), StoreError> {
    for m in seed_meetings() {
        write_record(&root.join(MEETINGS_DIR), &m.id, &m)?;
    }
    for t in seed_tasks() {
        write_record(&root.join(TASKS_DIR), &t.id, &t)?;
    }
    for p in seed_proposals() {
        write_record(&root.join(PROPOSALS_DIR), &p.id, &p)?;
    }
    for t in seed_transcripts() {
        write_record(&root.join(TRANSCRIPTS_DIR), &t.meeting_id, &t)?;
    }
    Ok(())
}

fn seed_meetings() -> Vec<Meeting> {
    let started =
        |y, mo, d, h, mi| Utc.with_ymd_and_hms(y, mo, d, h, mi, 0).single().unwrap();
    let rel = |s: &str| PathBuf::from(s);
    vec![
        Meeting {
            id: "mtg-live".into(),
            title: "System Architecture Sync".into(),
            project: "Epidote · Engineering".into(),
            state: MeetingState::Live,
            started_at: started(2026, 4, 22, 14, 0),
            duration_min: 90,
            elapsed: Some("01:23:48".into()),
            processing: None,
            pinned: true,
            preview: "GraphQL nested-relationship latency and DataLoader rollout.".into(),
            participants: vec![
                Participant {
                    initials: "DC".into(),
                    name: "David C.".into(),
                    accent: AccentColor::Purple,
                },
                Participant {
                    initials: "SJ".into(),
                    name: "Sarah J.".into(),
                    accent: AccentColor::Green,
                },
            ],
            agenda: vec![
                AgendaItem {
                    num: "01".into(),
                    title: "Latency baseline review".into(),
                    duration: "20 min".into(),
                },
                AgendaItem {
                    num: "02".into(),
                    title: "DataLoader prototype".into(),
                    duration: "30 min".into(),
                },
            ],
            summary: None,
            audio_path: None,
            transcript_path: Some(rel("transcripts/mtg-live.json")),
            task_ids: vec!["TSK-092".into()],
            proposal_ids: vec!["prop-chunking".into()],
            doc_paths: vec![],
        },
        Meeting {
            id: "mtg-q3roadmap".into(),
            title: "Q3 Roadmap Planning".into(),
            project: "Epidote · Strategy".into(),
            state: MeetingState::Processing,
            started_at: started(2026, 4, 22, 10, 0),
            duration_min: 60,
            elapsed: None,
            processing: Some(ProcessingState {
                step: "Extracting proposals…".into(),
                pct: 64,
            }),
            pinned: false,
            preview: "Sequencing the next quarter against the embedding-pipeline rebuild.".into(),
            participants: vec![
                Participant {
                    initials: "PR".into(),
                    name: "Priya R.".into(),
                    accent: AccentColor::Amber,
                },
                Participant {
                    initials: "DC".into(),
                    name: "David C.".into(),
                    accent: AccentColor::Purple,
                },
            ],
            agenda: vec![
                AgendaItem {
                    num: "01".into(),
                    title: "Q2 retrospective".into(),
                    duration: "15 min".into(),
                },
                AgendaItem {
                    num: "02".into(),
                    title: "Q3 themes".into(),
                    duration: "30 min".into(),
                },
            ],
            summary: Some(
                "Retro covered three big misses; Q3 will lead with the embedding rebuild.".into(),
            ),
            audio_path: Some(rel("meetings/audio/mtg-q3roadmap.wav")),
            transcript_path: Some(rel("transcripts/mtg-q3roadmap.json")),
            task_ids: vec!["TSK-088".into()],
            proposal_ids: vec!["prop-roadmap".into()],
            doc_paths: vec![],
        },
        Meeting {
            id: "mtg-q3arch".into(),
            title: "Q3 Data Architecture".into(),
            project: "Epidote · Engineering".into(),
            state: MeetingState::Archived,
            started_at: started(2026, 4, 18, 15, 30),
            duration_min: 75,
            elapsed: None,
            processing: None,
            pinned: false,
            preview: "Locked in the migration manifesto and the caching ownership model.".into(),
            participants: vec![
                Participant {
                    initials: "SJ".into(),
                    name: "Sarah J.".into(),
                    accent: AccentColor::Green,
                },
                Participant {
                    initials: "DC".into(),
                    name: "David C.".into(),
                    accent: AccentColor::Purple,
                },
                Participant {
                    initials: "PR".into(),
                    name: "Priya R.".into(),
                    accent: AccentColor::Amber,
                },
            ],
            agenda: vec![
                AgendaItem {
                    num: "01".into(),
                    title: "Doc migration plan".into(),
                    duration: "25 min".into(),
                },
                AgendaItem {
                    num: "02".into(),
                    title: "Caching ownership".into(),
                    duration: "30 min".into(),
                },
            ],
            summary: Some("Three decisions: scope, ownership, and rollout sequencing.".into()),
            audio_path: Some(rel("meetings/audio/mtg-q3arch.wav")),
            transcript_path: Some(rel("transcripts/mtg-q3arch.json")),
            task_ids: vec!["TSK-104".into()],
            proposal_ids: vec!["prop-doc-migration".into(), "prop-caching-owner".into()],
            doc_paths: vec![rel("Vault/Architecture/Migration Manifesto v1.md")],
        },
    ]
}

fn seed_tasks() -> Vec<Task> {
    let now = Utc::now();
    let date = |y, m, d| chrono::NaiveDate::from_ymd_opt(y, m, d);
    vec![
        Task {
            id: "TSK-088".into(),
            title: "Migrate legacy logging to Datadog cluster".into(),
            description: "Cut over auth and ingest first; backfill schemas in place.".into(),
            state: TaskState::Blocked,
            owner: "David C.".into(),
            due: date(2026, 5, 6),
            source_meeting_id: Some("mtg-q3roadmap".into()),
            source_proposal_id: None,
            created_at: now,
            updated_at: now,
        },
        Task {
            id: "TSK-092".into(),
            title: "Determine optimal chunking strategy for vector embeddings".into(),
            description: "Benchmark 256/512/1024 with 32/64/128 overlap; pick best F1 on the eval set."
                .into(),
            state: TaskState::Active,
            owner: "Priya R.".into(),
            due: date(2026, 4, 30),
            source_meeting_id: Some("mtg-live".into()),
            source_proposal_id: Some("prop-chunking".into()),
            created_at: now,
            updated_at: now,
        },
        Task {
            id: "TSK-104".into(),
            title: "Review Redis caching strategy for high-frequency reads".into(),
            description: "Decide on TTL bands and ownership across the read path.".into(),
            state: TaskState::Triage,
            owner: "Sarah J.".into(),
            due: None,
            source_meeting_id: Some("mtg-q3arch".into()),
            source_proposal_id: Some("prop-caching-owner".into()),
            created_at: now,
            updated_at: now,
        },
    ]
}

fn seed_proposals() -> Vec<Proposal> {
    let now = Utc::now();
    let rel = |s: &str| PathBuf::from(s);
    vec![
        Proposal {
            id: "prop-chunking".into(),
            title: "Use 512-token windows with 64-token overlap as the RAG default".into(),
            rationale: "Best F1 on internal eval; matches what landed in two recent papers.".into(),
            kind: ProposalKind::Decision,
            confidence: 82,
            state: ProposalState::Pending,
            source_meeting_id: Some("mtg-live".into()),
            linked_task_ids: vec!["TSK-092".into()],
            linked_doc_paths: vec![],
            generated_at: now,
        },
        Proposal {
            id: "prop-roadmap".into(),
            title: "Lead Q3 with the embedding-pipeline rebuild".into(),
            rationale: "Highest-leverage unblock for downstream RAG and observability work.".into(),
            kind: ProposalKind::Decision,
            confidence: 71,
            state: ProposalState::Pending,
            source_meeting_id: Some("mtg-q3roadmap".into()),
            linked_task_ids: vec!["TSK-088".into()],
            linked_doc_paths: vec![],
            generated_at: now,
        },
        Proposal {
            id: "prop-doc-migration".into(),
            title: "Generated: Migration Manifesto v1.md".into(),
            rationale: "Captures the three architecture decisions and the cutover sequence.".into(),
            kind: ProposalKind::Document,
            confidence: 79,
            state: ProposalState::Accepted,
            source_meeting_id: Some("mtg-q3arch".into()),
            linked_task_ids: vec![],
            linked_doc_paths: vec![rel("Vault/Architecture/Migration Manifesto v1.md")],
            generated_at: now,
        },
        Proposal {
            id: "prop-caching-owner".into(),
            title: "Sarah J. owns the Redis caching strategy review".into(),
            rationale: "Closest context — already running the read-path SLO work.".into(),
            kind: ProposalKind::Owner,
            confidence: 88,
            state: ProposalState::Pending,
            source_meeting_id: Some("mtg-q3arch".into()),
            linked_task_ids: vec!["TSK-104".into()],
            linked_doc_paths: vec![],
            generated_at: now,
        },
    ]
}

fn seed_transcripts() -> Vec<Transcript> {
    vec![Transcript {
        meeting_id: "mtg-live".into(),
        entries: vec![
            TranscriptEntry {
                speaker: "David C.".into(),
                initials: "DC".into(),
                timestamp_ms: 4_875_000,
                text: "The main issue with the current architecture is the latency when querying \
                       nested relationships. The GraphQL resolver is hitting the database \
                       sequentially."
                    .into(),
                highlight: None,
                live: false,
            },
            TranscriptEntry {
                speaker: "Sarah J.".into(),
                initials: "SJ".into(),
                timestamp_ms: 4_924_000,
                text: "Right. We discussed implementing DataLoader to batch those requests.".into(),
                highlight: Some("AWS-East-1 · staging".into()),
                live: false,
            },
        ],
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_root() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let mut p = std::env::temp_dir();
        p.push(format!("epidote-store-{pid}-{n}"));
        p
    }

    #[test]
    fn seeds_on_first_load_and_persists_mutation() {
        let root = temp_root();
        let store = Store::load_from(root.clone()).expect("load");
        assert!(!store.meetings.is_empty(), "seeded meetings");
        assert!(!store.tasks.is_empty(), "seeded tasks");
        assert!(!store.proposals.is_empty(), "seeded proposals");

        // Transcript for the live meeting should be on disk and lazy-loadable.
        let t = store.load_transcript("mtg-live").unwrap();
        assert!(t.is_some(), "transcript seeded");

        let mut store = store;
        let mut task = store.tasks[0].clone();
        let new_title = format!("MUTATED {}", task.id);
        task.title = new_title.clone();
        store.save_task(task).unwrap();

        let store2 = Store::load_from(root.clone()).expect("reload");
        let reloaded = store2.tasks.iter().find(|x| x.title == new_title);
        assert!(reloaded.is_some(), "mutation persisted across reloads");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn second_load_does_not_reseed() {
        let root = temp_root();
        let s1 = Store::load_from(root.clone()).expect("load1");
        let count = s1.meetings.len();
        let first_id = s1.meetings[0].id.clone();
        fs::remove_file(root.join(MEETINGS_DIR).join(format!("{first_id}.json"))).unwrap();

        let s2 = Store::load_from(root.clone()).expect("load2");
        assert_eq!(
            s2.meetings.len(),
            count - 1,
            "seeder did not run again on re-load"
        );

        let _ = fs::remove_dir_all(&root);
    }
}
