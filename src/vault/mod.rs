use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, Direction, FontId, Key, Layout, Rect, RichText,
    Sense, Stroke, StrokeKind, TextEdit, Ui, UiBuilder,
};

use crate::{fonts, icons, theme};

// ----- public state -----

pub struct State {
    root: PathBuf,
    tree: Option<Folder>,
    selected: Option<PathBuf>,
    buffer: String,
    dirty: bool,
    expanded: HashSet<PathBuf>,
    error: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        let root = default_vault_root();
        let _ = ensure_seed(&root);
        let tree = scan_folder(&root).ok();
        let mut expanded = HashSet::new();
        if let Some(t) = &tree {
            expanded.insert(t.path.clone());
            for sub in &t.folders {
                expanded.insert(sub.path.clone());
            }
        }
        Self {
            root,
            tree,
            selected: None,
            buffer: String::new(),
            dirty: false,
            expanded,
            error: None,
        }
    }
}

impl State {
    /// Open a file by absolute path — loads its contents into the editor,
    /// marks ancestors as expanded in the tree, and selects it. No-op if the
    /// file can't be read.
    pub fn open(&mut self, path: PathBuf) {
        match fs::read_to_string(&path) {
            Ok(content) => {
                self.buffer = content;
                self.dirty = false;
                self.error = None;
                let mut cursor = path.parent();
                while let Some(p) = cursor {
                    self.expanded.insert(p.to_path_buf());
                    if p == self.root.as_path() {
                        break;
                    }
                    cursor = p.parent();
                }
                self.selected = Some(path);
            }
            Err(err) => {
                self.error = Some(format!("Could not open {}: {err}", path.display()));
            }
        }
    }
}

// ----- model -----

struct Folder {
    name: String,
    path: PathBuf,
    folders: Vec<Folder>,
    files: Vec<FileEntry>,
}

struct FileEntry {
    name: String,
    path: PathBuf,
}

fn default_vault_root() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("Documents/Epidote/Vault")
}

fn ensure_seed(root: &Path) -> std::io::Result<()> {
    if root.exists() {
        return Ok(());
    }
    fs::create_dir_all(root)?;
    fs::create_dir_all(root.join("Architecture"))?;
    fs::create_dir_all(root.join("Meetings"))?;

    fs::write(
        root.join("README.md"),
        "# Welcome to your Vault\n\nThis is your personal knowledge graph. Notes are plain \
         markdown files organized in folders. Reference other notes with `[[Note Name]]`.\n\n\
         ## Folders\n- **Architecture** — system designs, RFCs, benchmarks.\n- **Meetings** — \
         transcripts and summaries.\n",
    )?;
    fs::write(
        root.join("Architecture/Embedding Architecture.md"),
        "# Embedding Architecture\n\nThe RAG pipeline currently uses 1024-token windows with \
         128-token overlap.\n\n## References\n- [[Chunking Benchmarks]]\n- [[Caching RFC v2]]\n\n\
         ## Open questions\n- Should we move to per-section chunking?\n",
    )?;
    fs::write(
        root.join("Architecture/Chunking Benchmarks.md"),
        "# Chunking Benchmarks\n\nWindow sizes evaluated: 256 / 512 / 1024.\n\nOverlap: 0 / 64 \
         / 128 tokens.\n\nRecall@10 peaks at 512/64 for the meetings corpus.\n",
    )?;
    fs::write(
        root.join("Architecture/Caching RFC v2.md"),
        "# Caching RFC v2\n\nProposed write-through cache layer with TTL bucketing for \
         high-frequency reads. Targets a 92%+ hit rate on hot keys without blowing the memory \
         ceiling.\n",
    )?;
    fs::write(
        root.join("Meetings/Q3 Sync.md"),
        "# Q3 Data Architecture Sync\n\nAttendees: J. Doe, M. Antunes\n\n## Decisions\n- Move \
         caching layer ownership to the platform team.\n- Spawn TSK-104 to evaluate \
         [[Caching RFC v2]].\n",
    )?;
    fs::write(
        root.join("Meetings/RAG Pipeline Optimization.md"),
        "# RAG Pipeline Optimization\n\nReviewed retrieval quality on the meetings corpus. \
         Action items captured under TSK-092. See [[Embedding Architecture]] and \
         [[Chunking Benchmarks]].\n",
    )?;
    Ok(())
}

fn scan_folder(path: &Path) -> std::io::Result<Folder> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Vault".to_string());
    let mut folders = Vec::new();
    let mut files = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            // Skip dotfiles like .obsidian, .git
            if entry_path
                .file_name()
                .map(|n| n.to_string_lossy().starts_with('.'))
                .unwrap_or(false)
            {
                continue;
            }
            folders.push(scan_folder(&entry_path)?);
        } else if file_type.is_file() {
            if entry_path.extension().is_some_and(|e| e == "md") {
                let name = entry_path
                    .file_stem()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                files.push(FileEntry {
                    name,
                    path: entry_path,
                });
            }
        }
    }
    folders.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(Folder {
        name,
        path: path.to_path_buf(),
        folders,
        files,
    })
}

// ----- entry point -----

pub fn show(ui: &mut Ui, state: &mut State) {
    if ui
        .ctx()
        .input(|i| i.modifiers.command && i.key_pressed(Key::S))
    {
        save_current(state);
    }

    let total = ui.available_size_before_wrap();
    let sidebar_w = 260.0;
    let gap = 12.0;
    let origin = ui.cursor().min;

    let sidebar_rect = Rect::from_min_size(origin, vec2(sidebar_w, total.y));
    let editor_rect = Rect::from_min_size(
        pos2(origin.x + sidebar_w + gap, origin.y),
        vec2(total.x - sidebar_w - gap, total.y),
    );

    ui.allocate_rect(
        Rect::from_min_size(origin, vec2(total.x, total.y)),
        Sense::hover(),
    );

    let mut sidebar_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(sidebar_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    show_sidebar(&mut sidebar_ui, state);

    let mut editor_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(editor_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    show_editor(&mut editor_ui, state);
}

// ----- sidebar / file tree -----

fn show_sidebar(ui: &mut Ui, state: &mut State) {
    let outer = ui.max_rect();
    ui.painter()
        .rect_filled(outer, 4.0, theme::SURFACE_CONTAINER_LOW);

    let inner = outer.shrink(12.0);
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::top_down(Align::Min)),
    );

    child.horizontal(|ui| {
        ui.label(
            RichText::new("VAULT")
                .font(fonts::display(11.0))
                .strong()
                .color(theme::DIM_TEXT),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let (rect, response) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::click());
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                icons::REFRESH,
                fonts::icon(14.0),
                if response.hovered() {
                    theme::TEXT
                } else {
                    theme::DIM_TEXT
                },
            );
            if response.clicked() {
                if let Ok(t) = scan_folder(&state.root) {
                    state.tree = Some(t);
                }
            }
        });
    });
    child.add_space(4.0);
    child.label(
        RichText::new(state.root.display().to_string())
            .size(10.0)
            .color(theme::DIM_TEXT),
    );
    child.add_space(8.0);

    if let Some(err) = &state.error {
        child.label(
            RichText::new(err)
                .size(11.0)
                .color(Color32::from_rgb(0xff, 0xb4, 0xab)),
        );
        child.add_space(6.0);
    }

    egui::ScrollArea::vertical()
        .id_salt("vault_tree_scroll")
        .auto_shrink([false, false])
        .show(&mut child, |ui| {
            if let Some(tree) = &state.tree {
                for sub in &tree.folders {
                    show_folder(
                        ui,
                        sub,
                        0,
                        &mut state.expanded,
                        &mut state.selected,
                        &mut state.buffer,
                        &mut state.dirty,
                    );
                }
                for file in &tree.files {
                    show_file(
                        ui,
                        file,
                        0,
                        &mut state.selected,
                        &mut state.buffer,
                        &mut state.dirty,
                    );
                }
            }
        });
}

fn show_folder(
    ui: &mut Ui,
    folder: &Folder,
    depth: usize,
    expanded: &mut HashSet<PathBuf>,
    selected: &mut Option<PathBuf>,
    buffer: &mut String,
    dirty: &mut bool,
) {
    let is_open = expanded.contains(&folder.path);
    let row_h = 22.0;
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), row_h), Sense::click());
    let painter = ui.painter_at(rect);
    if response.hovered() {
        painter.rect_filled(rect, 2.0, theme::SURFACE_CONTAINER);
    }
    let indent = depth as f32 * 14.0;
    painter.text(
        pos2(rect.left() + indent + 4.0, rect.center().y),
        Align2::LEFT_CENTER,
        if is_open {
            icons::EXPAND_MORE
        } else {
            icons::CHEVRON_RIGHT
        },
        fonts::icon(14.0),
        theme::DIM_TEXT,
    );
    painter.text(
        pos2(rect.left() + indent + 22.0, rect.center().y),
        Align2::LEFT_CENTER,
        icons::FOLDER,
        fonts::icon(14.0),
        theme::DIM_TEXT,
    );
    painter.text(
        pos2(rect.left() + indent + 42.0, rect.center().y),
        Align2::LEFT_CENTER,
        &folder.name,
        FontId::proportional(12.5),
        theme::TEXT,
    );

    if response.clicked() {
        if is_open {
            expanded.remove(&folder.path);
        } else {
            expanded.insert(folder.path.clone());
        }
    }

    if is_open {
        for sub in &folder.folders {
            show_folder(ui, sub, depth + 1, expanded, selected, buffer, dirty);
        }
        for file in &folder.files {
            show_file(ui, file, depth + 1, selected, buffer, dirty);
        }
    }
}

fn show_file(
    ui: &mut Ui,
    file: &FileEntry,
    depth: usize,
    selected: &mut Option<PathBuf>,
    buffer: &mut String,
    dirty: &mut bool,
) {
    let is_selected = selected.as_ref() == Some(&file.path);
    let row_h = 22.0;
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), row_h), Sense::click());
    let painter = ui.painter_at(rect);

    let bg = if is_selected {
        theme::SURFACE_HIGH
    } else if response.hovered() {
        theme::SURFACE_CONTAINER
    } else {
        Color32::TRANSPARENT
    };
    painter.rect_filled(rect, 2.0, bg);

    let indent = depth as f32 * 14.0;
    let icon_color = if is_selected {
        theme::PRIMARY
    } else {
        theme::DIM_TEXT
    };
    painter.text(
        pos2(rect.left() + indent + 22.0, rect.center().y),
        Align2::LEFT_CENTER,
        icons::ARTICLE,
        fonts::icon(13.0),
        icon_color,
    );
    painter.text(
        pos2(rect.left() + indent + 42.0, rect.center().y),
        Align2::LEFT_CENTER,
        &file.name,
        FontId::proportional(12.5),
        theme::TEXT,
    );

    if response.clicked() && !is_selected {
        if let Ok(content) = fs::read_to_string(&file.path) {
            *selected = Some(file.path.clone());
            *buffer = content;
            *dirty = false;
        }
    }
}

// ----- editor -----

fn show_editor(ui: &mut Ui, state: &mut State) {
    let outer = ui.max_rect();
    ui.painter()
        .rect_filled(outer, 4.0, theme::SURFACE_CONTAINER_LOW);

    let inner = outer.shrink(20.0);
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::top_down(Align::Min)),
    );

    if state.selected.is_none() {
        child.with_layout(
            Layout::centered_and_justified(Direction::TopDown),
            |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(icons::ARTICLE)
                            .font(fonts::icon(40.0))
                            .color(theme::DIM_TEXT),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Select a note from the sidebar")
                            .font(fonts::display(16.0))
                            .color(theme::DIM_TEXT),
                    );
                });
            },
        );
        return;
    }

    let title = state
        .selected
        .as_ref()
        .and_then(|p| p.file_stem())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let breadcrumb = state
        .selected
        .as_ref()
        .and_then(|p| p.strip_prefix(&state.root).ok())
        .and_then(|rel| rel.parent())
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    child.allocate_ui_with_layout(
        vec2(child.available_width(), 44.0),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.vertical(|ui| {
                if !breadcrumb.is_empty() {
                    ui.label(
                        RichText::new(&breadcrumb)
                            .size(10.5)
                            .color(theme::DIM_TEXT),
                    );
                }
                ui.label(
                    RichText::new(&title)
                        .font(fonts::display(22.0))
                        .strong()
                        .color(theme::TEXT),
                );
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                save_button(ui, state);
                if state.dirty {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("• unsaved")
                            .size(11.0)
                            .color(theme::DIM_TEXT),
                    );
                }
            });
        },
    );
    child.add_space(12.0);

    egui::ScrollArea::vertical()
        .id_salt("vault_editor_scroll")
        .auto_shrink([false, false])
        .show(&mut child, |ui| {
            let response = ui.add_sized(
                vec2(ui.available_width(), ui.available_height().max(200.0)),
                TextEdit::multiline(&mut state.buffer)
                    .desired_width(f32::INFINITY)
                    .font(FontId::monospace(13.0))
                    .frame(false),
            );
            if response.changed() {
                state.dirty = true;
            }
        });
}

fn save_button(ui: &mut Ui, state: &mut State) {
    let (rect, response) = ui.allocate_exact_size(vec2(96.0, 28.0), Sense::click());
    let painter = ui.painter_at(rect);
    let (bg, fg, border) = if state.dirty {
        (
            theme::PRIMARY,
            theme::BUTTON_TEXT,
            Stroke::new(1.0, theme::PRIMARY),
        )
    } else {
        (
            theme::SURFACE_CONTAINER,
            theme::DIM_TEXT,
            Stroke::new(1.0, theme::OUTLINE_VARIANT),
        )
    };
    painter.rect(rect, 2.0, bg, border, StrokeKind::Inside);
    painter.text(
        pos2(rect.left() + 14.0, rect.center().y),
        Align2::LEFT_CENTER,
        icons::SAVE,
        fonts::icon(14.0),
        fg,
    );
    painter.text(
        pos2(rect.left() + 36.0, rect.center().y),
        Align2::LEFT_CENTER,
        if state.dirty { "Save" } else { "Saved" },
        FontId::proportional(12.0),
        fg,
    );
    if response.clicked() && state.dirty {
        save_current(state);
    }
}

fn save_current(state: &mut State) {
    if let Some(path) = &state.selected {
        match fs::write(path, &state.buffer) {
            Ok(_) => {
                state.dirty = false;
                state.error = None;
            }
            Err(err) => {
                state.error = Some(format!("Save failed: {err}"));
            }
        }
    }
}
