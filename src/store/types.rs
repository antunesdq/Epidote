use std::path::PathBuf;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

// ----- Meetings -----

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Meeting {
    pub id: String,
    pub title: String,
    pub project: String,
    pub state: MeetingState,
    pub started_at: DateTime<Utc>,
    pub duration_min: u32,
    #[serde(default)]
    pub elapsed: Option<String>,
    #[serde(default)]
    pub processing: Option<ProcessingState>,
    #[serde(default)]
    pub pinned: bool,
    pub preview: String,
    pub participants: Vec<Participant>,
    pub agenda: Vec<AgendaItem>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub audio_path: Option<PathBuf>,
    #[serde(default)]
    pub transcript_path: Option<PathBuf>,
    #[serde(default)]
    pub task_ids: Vec<String>,
    #[serde(default)]
    pub proposal_ids: Vec<String>,
    #[serde(default)]
    pub doc_paths: Vec<PathBuf>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MeetingState {
    Live,
    Processing,
    Archived,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessingState {
    pub step: String,
    pub pct: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Participant {
    pub initials: String,
    pub name: String,
    pub accent: AccentColor,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccentColor {
    Green,
    Purple,
    Amber,
    Grey,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgendaItem {
    pub num: String,
    pub title: String,
    pub duration: String,
}

// ----- Transcripts (kept on disk, loaded lazily) -----

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transcript {
    pub meeting_id: String,
    pub entries: Vec<TranscriptEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranscriptEntry {
    pub speaker: String,
    pub initials: String,
    pub timestamp_ms: u64,
    pub text: String,
    #[serde(default)]
    pub highlight: Option<String>,
    #[serde(default)]
    pub live: bool,
}

// ----- Tasks -----

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub state: TaskState,
    pub owner: String,
    #[serde(default)]
    pub due: Option<NaiveDate>,
    #[serde(default)]
    pub source_meeting_id: Option<String>,
    #[serde(default)]
    pub source_proposal_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Triage,
    Active,
    Blocked,
    Done,
}

// ----- Proposals -----

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub rationale: String,
    pub kind: ProposalKind,
    pub confidence: u8,
    pub state: ProposalState,
    #[serde(default)]
    pub source_meeting_id: Option<String>,
    #[serde(default)]
    pub linked_task_ids: Vec<String>,
    #[serde(default)]
    pub linked_doc_paths: Vec<PathBuf>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProposalKind {
    Decision,
    Document,
    Followup,
    Owner,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProposalState {
    Pending,
    Accepted,
    Rejected,
}
