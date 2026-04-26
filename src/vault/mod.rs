use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, Direction, FontId, Key, Layout, Rect, RichText,
    Sense, Stroke, StrokeKind, TextEdit, Ui, UiBuilder,
};

use crate::{fonts, graph, icons, nav::Open, theme};

// ----- public state -----

pub struct State {
    root: PathBuf,
    tree: Option<Folder>,
    selected: Option<PathBuf>,
    buffer: String,
    dirty: bool,
    expanded: HashSet<PathBuf>,
    error: Option<String>,
    view: View,
    graph: graph::State,
}

#[derive(Clone, Copy, PartialEq)]
enum View {
    Document,
    Graph,
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
            view: View::Document,
            graph: graph::State::default(),
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
                self.view = View::Document;
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

pub fn show(ui: &mut Ui, state: &mut State) -> Option<Open> {
    if ui
        .ctx()
        .input(|i| i.modifiers.command && i.key_pressed(Key::S))
    {
        save_current(state);
    }

    let total = ui.available_size_before_wrap();
    let sidebar_w = 240.0;
    let gap = 0.0;
    let origin = ui.cursor().min;

    let sidebar_rect = Rect::from_min_size(origin, vec2(sidebar_w, total.y));
    let main_rect = Rect::from_min_size(
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

    let mut main_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(main_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    show_main(&mut main_ui, state)
}

// ----- main pane: toolbar + active view -----

fn show_main(ui: &mut Ui, state: &mut State) -> Option<Open> {
    let outer = ui.max_rect();
    ui.painter().rect_filled(outer, 0.0, theme::BACKGROUND);

    let mut nav: Option<Open> = None;

    // Toolbar (breadcrumb + view toggle)
    let toolbar_h = 44.0;
    let toolbar_rect = Rect::from_min_size(outer.min, vec2(outer.width(), toolbar_h));
    let mut toolbar_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(toolbar_rect.shrink2(vec2(20.0, 8.0)))
            .layout(Layout::left_to_right(Align::Center)),
    );
    show_toolbar(&mut toolbar_ui, state);
    ui.painter().line_segment(
        [toolbar_rect.left_bottom(), toolbar_rect.right_bottom()],
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
    );

    // Body
    let body_rect = Rect::from_min_size(
        pos2(outer.left(), outer.top() + toolbar_h),
        vec2(outer.width(), outer.height() - toolbar_h),
    );
    let mut body_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(body_rect)
            .layout(Layout::top_down(Align::Min)),
    );

    match state.view {
        View::Document => {
            if let Some(o) = show_editor(&mut body_ui, state) {
                nav = Some(o);
            }
        }
        View::Graph => {
            // The retired top-level graph module is reused as a vault subview.
            if let Some(o) = graph::show(&mut body_ui, &mut state.graph) {
                nav = Some(o);
            }
        }
    }

    nav
}

fn show_toolbar(ui: &mut Ui, state: &mut State) {
    // Breadcrumb
    let folder = state
        .selected
        .as_ref()
        .and_then(|p| p.strip_prefix(&state.root).ok())
        .and_then(|rel| rel.parent())
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    ui.label(
        RichText::new("VAULT")
            .font(FontId::monospace(10.0))
            .color(theme::DIM_TEXT),
    );
    ui.add_space(4.0);
    ui.label(
        RichText::new("/")
            .font(FontId::monospace(10.0))
            .color(theme::DIM_TEXT),
    );
    ui.add_space(4.0);
    ui.label(
        RichText::new(if folder.is_empty() { "All" } else { folder.as_str() })
            .font(fonts::display(12.0))
            .strong()
            .color(theme::TEXT),
    );

    // Right-aligned view toggle
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        view_toggle(ui, &mut state.view);
    });
}

fn view_toggle(ui: &mut Ui, view: &mut View) {
    let segments: [(View, &str); 2] = [(View::Document, "Document"), (View::Graph, "Graph")];
    let pad = 12.0;
    let mut total_w = 0.0;
    let font = FontId::proportional(11.5);
    for (_, label) in &segments {
        let g = ui.painter().layout_no_wrap(label.to_string(), font.clone(), theme::TEXT);
        total_w += g.size().x + pad * 2.0;
    }
    let h = 26.0;
    let (rect, _) = ui.allocate_exact_size(vec2(total_w, h), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect(
        rect,
        3.0,
        theme::SURFACE_CONTAINER,
        Stroke::new(1.0, theme::OUTLINE_VARIANT),
        StrokeKind::Inside,
    );
    let mut x = rect.left();
    for (v, label) in segments {
        let g = ui.painter().layout_no_wrap(label.to_string(), font.clone(), theme::TEXT);
        let seg_w = g.size().x + pad * 2.0;
        let seg_rect = Rect::from_min_size(pos2(x, rect.top()), vec2(seg_w, h));
        let seg_resp = ui.interact(seg_rect, egui::Id::new(("vault_view_seg", label)), Sense::click());
        let active = *view == v;
        let bg = if active {
            theme::SURFACE_HIGH
        } else if seg_resp.hovered() {
            theme::SURFACE_HIGH
        } else {
            Color32::TRANSPARENT
        };
        let fg = if active {
            theme::TEXT
        } else {
            theme::DIM_TEXT
        };
        painter.rect_filled(seg_rect, 3.0, bg);
        painter.text(seg_rect.center(), Align2::CENTER_CENTER, label, font.clone(), fg);
        if seg_resp.clicked() {
            *view = v;
        }
        x += seg_w;
    }
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

fn show_editor(ui: &mut Ui, state: &mut State) -> Option<Open> {
    let outer = ui.max_rect();
    ui.painter()
        .rect_filled(outer, 0.0, theme::BACKGROUND);

    let inner = outer.shrink2(vec2(28.0, 20.0));
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
        return None;
    }

    let title = state
        .selected
        .as_ref()
        .and_then(|p| p.file_stem())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    child.allocate_ui_with_layout(
        vec2(child.available_width(), 44.0),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.label(
                RichText::new(&title)
                    .font(fonts::display(22.0))
                    .strong()
                    .color(theme::TEXT),
            );
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

    let wikilinks = parse_wikilinks(&state.buffer);
    let chip_strip_h = if wikilinks.is_empty() { 0.0 } else { 56.0 };
    let editor_h = (child.available_height() - chip_strip_h).max(160.0);

    egui::ScrollArea::vertical()
        .id_salt("vault_editor_scroll")
        .auto_shrink([false, false])
        .max_height(editor_h)
        .show(&mut child, |ui| {
            let response = ui.add_sized(
                vec2(ui.available_width(), ui.available_height().max(160.0)),
                TextEdit::multiline(&mut state.buffer)
                    .desired_width(f32::INFINITY)
                    .font(FontId::monospace(13.0))
                    .frame(false),
            );
            if response.changed() {
                state.dirty = true;
            }
        });

    let mut nav: Option<Open> = None;
    if !wikilinks.is_empty() {
        child.add_space(8.0);
        if let Some(o) = wikilink_strip(&mut child, &wikilinks, state) {
            nav = Some(o);
        }
    }
    nav
}

fn parse_wikilinks(buffer: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let bytes = buffer.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'[' && bytes[i + 1] == b'[' {
            let mut j = i + 2;
            while j + 1 < bytes.len() && !(bytes[j] == b']' && bytes[j + 1] == b']') {
                j += 1;
            }
            if j + 1 < bytes.len() {
                if let Ok(name) = std::str::from_utf8(&bytes[i + 2..j]) {
                    let name = name.trim();
                    if !name.is_empty() && !out.iter().any(|n| n == name) {
                        out.push(name.to_string());
                    }
                }
                i = j + 2;
                continue;
            } else {
                break;
            }
        }
        i += 1;
    }
    out
}

fn wikilink_strip(ui: &mut Ui, links: &[String], state: &State) -> Option<Open> {
    let mut nav: Option<Open> = None;
    ui.label(
        RichText::new("LINKED NOTES")
            .font(FontId::monospace(9.5))
            .strong()
            .color(theme::DIM_TEXT),
    );
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        for name in links {
            let resolved = state
                .tree
                .as_ref()
                .and_then(|t| find_doc_by_name(t, name));
            let clickable = resolved.is_some();
            if wikilink_chip(ui, name, clickable) {
                if let Some(p) = resolved {
                    nav = Some(Open::Vault(p));
                }
            }
        }
    });
    nav
}

fn wikilink_chip(ui: &mut Ui, name: &str, clickable: bool) -> bool {
    let label = format!("[[{name}]]");
    let font = FontId::monospace(11.0);
    let g = ui.painter().layout_no_wrap(label.clone(), font.clone(), theme::TEXT);
    let w = g.size().x + 18.0;
    let sense = if clickable {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(vec2(w, 24.0), sense);
    let painter = ui.painter_at(rect);
    let hovered = response.hovered() && clickable;
    let (bg, fg, stroke) = if !clickable {
        (
            Color32::TRANSPARENT,
            theme::DIM_TEXT,
            Stroke::new(1.0, theme::OUTLINE_VARIANT),
        )
    } else if hovered {
        (
            with_alpha(theme::PRIMARY, 30),
            theme::PRIMARY,
            Stroke::new(1.0, theme::PRIMARY),
        )
    } else {
        (
            theme::SURFACE_CONTAINER,
            theme::PRIMARY,
            Stroke::new(1.0, theme::OUTLINE_VARIANT),
        )
    };
    painter.rect(rect, 3.0, bg, stroke, StrokeKind::Inside);
    painter.text(rect.center(), Align2::CENTER_CENTER, &label, font, fg);
    response.clicked()
}

fn find_doc_by_name(folder: &Folder, name: &str) -> Option<PathBuf> {
    let lower = name.to_lowercase();
    for f in &folder.files {
        if f.name.to_lowercase() == lower {
            return Some(f.path.clone());
        }
    }
    for sub in &folder.folders {
        if let Some(p) = find_doc_by_name(sub, name) {
            return Some(p);
        }
    }
    None
}

fn with_alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
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
