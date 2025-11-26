# CASS (Coding Agent Session Search) - Comprehensive Improvement Plan

**Document Version:** 1.0
**Created:** 2024-11-25
**Purpose:** Self-contained roadmap for elevating CASS to premium "Stripe-level" quality

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Project Context & Goals](#project-context--goals)
3. [Current State Analysis](#current-state-analysis)
4. [Issue Categories](#issue-categories)
5. [Detailed Issue Analysis](#detailed-issue-analysis)
6. [Implementation Beads (Task Breakdown)](#implementation-beads-task-breakdown)
7. [Dependency Graph](#dependency-graph)
8. [Effort Estimates](#effort-estimates)
9. [Success Metrics](#success-metrics)

---

## Executive Summary

CASS is a Rust-based TUI application that provides unified search across coding agent session histories (Claude Code, Codex, Gemini, OpenCode, Cline, Amp). After fixing critical connector bugs, the application is now functionally correct but needs significant polish to achieve a premium user experience.

This document identifies **25 distinct improvement areas** organized into **68 implementation beads** with full dependency mapping. The improvements range from 5-minute quick fixes to multi-hour feature additions.

**Key Themes:**
- **Correctness**: Fix documentation/code mismatches, unsafe code patterns
- **Usability**: Human-readable dates, persistent settings, better defaults
- **Polish**: Visual hierarchy, animations, contextual feedback
- **Premium Feel**: Autocomplete, fuzzy suggestions, mouse support

---

## Project Context & Goals

### What CASS Does
CASS indexes conversation histories from multiple AI coding assistants and provides:
- Full-text search via Tantivy (with SQLite FTS5 fallback)
- Per-agent color-coded result panes
- Detailed conversation viewer with message highlighting
- Filter by agent, workspace, and time range
- Persistent settings and query history

### Overarching Goals
1. **Developer Productivity**: Help developers quickly find past conversations
2. **Universal Coverage**: Support all major coding agents
3. **Minimal Friction**: Work out-of-the-box with sensible defaults
4. **Professional Quality**: Match the polish of tools like Stripe, Linear, Raycast

### Target Users
- Power users who use multiple AI coding assistants
- Developers who want to reference past problem-solving sessions
- Teams tracking AI-assisted development patterns

---

## Current State Analysis

### Strengths
- ✅ Solid Rust architecture with clean module separation
- ✅ Tantivy provides fast, high-quality full-text search
- ✅ Per-agent visual theming creates clear information hierarchy
- ✅ Extensive keyboard shortcuts for power users
- ✅ Background indexing keeps data fresh

### Weaknesses
- ❌ Unsafe code patterns (transmute for lifetime extension)
- ❌ Inconsistent documentation (F11 vs Ctrl+Del)
- ❌ Poor time filter UX (millisecond timestamps)
- ❌ Information overload in status bar
- ❌ No loading indicators or progress feedback
- ❌ Query history lost on exit
- ❌ Help overlay shows on every launch
- ❌ No mouse support

---

## Issue Categories

### 🔴 P0: Critical (Must Fix)
Issues that cause confusion, potential crashes, or incorrect behavior.

### 🟠 P1: High Priority (Should Fix)
Sub-optimal patterns that significantly impact usability.

### 🟡 P2: Medium Priority (Nice to Have)
UX improvements that enhance the experience.

### 🟢 P3: Polish (Premium Feel)
Refinements that differentiate premium software.

---

## Detailed Issue Analysis

### ISSUE-001: Unsafe Memory Transmute
**Category:** 🔴 P0 Critical
**Location:** `src/ui/tui.rs:336, 382, 407`
**Files Affected:** `src/ui/tui.rs`

**Current Code:**
```rust
.into_iter()
.map(|s| unsafe { std::mem::transmute::<Span<'_>, Span<'static>>(s) })
.collect()
```

**Problem:**
The code uses `unsafe` transmute to extend the lifetime of `Span` and `Line` objects from temporary to `'static`. This is done to satisfy ratatui's widget lifetime requirements, but it's undefined behavior waiting to happen. If the underlying string data is deallocated while the `Span` still references it, we get a use-after-free.

**Root Cause:**
Ratatui widgets often require `'static` lifetimes for text content when using certain APIs. The original developer chose transmute as a quick workaround rather than restructuring the code to use owned data.

**Solution:**
1. Use `String` instead of `&str` throughout the span construction
2. Use `Cow<'static, str>` for flexibility between owned and borrowed
3. Restructure functions to return owned `Line<'static>` by constructing with owned strings

**Why This Matters:**
- Undefined behavior can cause random crashes
- Memory safety is Rust's core value proposition
- This pattern sets a bad precedent for the codebase

**Acceptance Criteria:**
- [ ] All `unsafe { std::mem::transmute }` calls removed from tui.rs
- [ ] Code compiles without warnings
- [ ] All existing tests pass
- [ ] TUI renders correctly with same visual output

---

### ISSUE-002: Incorrect Binary Name in Error Messages
**Category:** 🔴 P0 Critical
**Location:** `src/ui/tui.rs:459`
**Files Affected:** `src/ui/tui.rs`

**Current Code:**
```rust
"Index not present at {}. Run `coding-agent-search index --full` then reopen TUI."
```

**Problem:**
The binary is named `cass` (as defined in Cargo.toml), but error messages reference `coding-agent-search`. Users will try to run a command that doesn't exist.

**Solution:**
Replace all occurrences of `coding-agent-search` with `cass` in user-facing strings.

**Why This Matters:**
- Users cannot follow the instructions
- Creates impression of unpolished/amateur software
- Simple grep-and-replace fix

**Acceptance Criteria:**
- [ ] All user-facing strings use `cass` as the binary name
- [ ] Error messages are actionable and correct

---

### ISSUE-003: Keyboard Shortcut Documentation Mismatch
**Category:** 🔴 P0 Critical
**Location:** `src/ui/components/widgets.rs:41`, `src/ui/tui.rs:117, 424-427`
**Files Affected:** `src/ui/components/widgets.rs`, `src/ui/tui.rs`

**Current Code (widgets.rs:41):**
```rust
"F11 clear"
```

**Current Code (tui.rs:117):**
```rust
"F3 agent | F4 workspace | F5 from | F6 to | Ctrl+Del clear all"
```

**Problem:**
The search bar tip says "F11 clear" but the actual implementation uses `Ctrl+Delete` (tui.rs:1361). The help overlay correctly says "Ctrl+Del" but the persistent search bar shows wrong info.

**Solution:**
1. Update widgets.rs to say "Ctrl+Del clear" instead of "F11 clear"
2. Audit all shortcut documentation for consistency
3. Consider a single source of truth for shortcuts (constant or enum)

**Why This Matters:**
- Users try F11, nothing happens, frustration ensues
- Inconsistency erodes trust in documentation
- Power users rely on shortcuts being reliable

**Acceptance Criteria:**
- [ ] All shortcut references are consistent across: search bar tips, help overlay, footer legend
- [ ] Each shortcut works as documented

---

### ISSUE-004: Time Filter Requires Millisecond Timestamps
**Category:** 🟠 P1 High Priority
**Location:** `src/ui/tui.rs:1324-1334`
**Files Affected:** `src/ui/tui.rs`, potentially new `src/ui/time_parser.rs`

**Current Code:**
```rust
KeyCode::F(5) => {
    input_mode = InputMode::CreatedFrom;
    input_buffer.clear();
    status = "Created-from (ms since epoch): Enter=apply, Esc=cancel".to_string();
}
```

**Problem:**
To filter conversations to "last 7 days", users must:
1. Know the current Unix timestamp in milliseconds
2. Subtract 604,800,000 (7 days in ms)
3. Type: `1732558800000`

This is absurd. No human thinks in milliseconds.

**Solution:**
Accept multiple input formats:
- Relative: `-7d`, `-24h`, `-1w`, `-30m`, `yesterday`, `today`
- Absolute: `2024-11-25`, `Nov 25`, `11/25/2024`
- Unix: `1732558800` (seconds) or `1732558800000` (ms) for power users

Implementation approach:
```rust
fn parse_time_input(input: &str) -> Option<i64> {
    let input = input.trim().to_lowercase();

    // Relative formats
    if input.starts_with('-') {
        return parse_relative_time(&input[1..]);
    }
    if input == "yesterday" {
        return Some(now_ms() - 86_400_000);
    }
    if input == "today" {
        return Some(start_of_today_ms());
    }

    // Try chrono parsing for dates
    if let Ok(date) = NaiveDate::parse_from_str(&input, "%Y-%m-%d") {
        return Some(date.and_hms(0, 0, 0).timestamp_millis());
    }

    // Fallback to numeric (ms or seconds)
    if let Ok(n) = input.parse::<i64>() {
        // Heuristic: if < 10^12, assume seconds; else ms
        return Some(if n < 1_000_000_000_000 { n * 1000 } else { n });
    }

    None
}
```

**Why This Matters:**
- Time filtering is a core feature
- Current UX makes it effectively unusable
- Competitors (Raycast, Alfred) all support human dates

**Acceptance Criteria:**
- [ ] `-7d` sets filter to 7 days ago
- [ ] `2024-11-20` sets filter to that date at midnight
- [ ] `yesterday` works
- [ ] Invalid input shows helpful error message
- [ ] Status bar shows parsed date for confirmation

---

### ISSUE-005: Time Chips Display Raw Milliseconds
**Category:** 🟠 P1 High Priority
**Location:** `src/ui/tui.rs:325-330, 579-586`
**Files Affected:** `src/ui/tui.rs`

**Current Code:**
```rust
format!("[time:{:?}->{:?}]", filters.created_from, filters.created_to)
// Output: [time:Some(1732558800000)->None]
```

**Problem:**
Even after entering a time filter, users see gibberish like `Some(1732558800000)`. This provides zero useful feedback about what filter is active.

**Solution:**
Format timestamps as human-readable dates:
```rust
fn format_time_chip(from: Option<i64>, to: Option<i64>) -> String {
    let fmt = |ms: i64| -> String {
        DateTime::<Utc>::from_timestamp_millis(ms)
            .map(|dt| dt.format("%b %d").to_string())  // "Nov 25"
            .unwrap_or_else(|| "?".into())
    };

    match (from, to) {
        (Some(f), Some(t)) => format!("[time: {} → {}]", fmt(f), fmt(t)),
        (Some(f), None) => format!("[time: {} → now]", fmt(f)),
        (None, Some(t)) => format!("[time: start → {}]", fmt(t)),
        (None, None) => String::new(),
    }
}
```

**Why This Matters:**
- Chips should confirm what filter is active
- Raw debug output looks broken/unfinished
- Quick visual check impossible with current format

**Acceptance Criteria:**
- [ ] Time chips display as `[time: Nov 20 → now]`
- [ ] Year shown only if different from current year
- [ ] "today", "yesterday" used for recent dates

---

### ISSUE-006: Help Overlay Shows on Every Launch
**Category:** 🟠 P1 High Priority
**Location:** `src/ui/tui.rs:485`
**Files Affected:** `src/ui/tui.rs`

**Current Code:**
```rust
let mut show_help = true;  // Always starts with help visible
```

**Problem:**
Every time you launch CASS, the help overlay pops up. After the first use, this becomes annoying. Power users know the shortcuts; they just want to search.

**Solution:**
1. Add `has_seen_help: bool` to `TuiStatePersisted`
2. On first launch (no state file): show help, set `has_seen_help = true`
3. On subsequent launches: don't show help automatically
4. F1 always toggles help regardless

```rust
#[derive(Serialize, Deserialize, Default)]
struct TuiStatePersisted {
    match_mode: Option<String>,
    context_window: Option<String>,
    has_seen_help: Option<bool>,  // NEW
}

// In run_tui():
let mut show_help = !persisted.has_seen_help.unwrap_or(false);
```

**Why This Matters:**
- Respects returning users' time
- First-launch onboarding is still preserved
- Standard UX pattern (most apps do this)

**Acceptance Criteria:**
- [ ] First launch: help shown
- [ ] Second launch: help not shown
- [ ] F1 always works to toggle
- [ ] Deleting tui_state.json resets to first-launch behavior

---

### ISSUE-007: Query History Not Persisted
**Category:** 🟠 P1 High Priority
**Location:** `src/ui/tui.rs:489`
**Files Affected:** `src/ui/tui.rs`

**Current Code:**
```rust
let mut query_history: VecDeque<String> = VecDeque::new();
// ...never saved to disk
```

**Problem:**
Query history (accessed via Ctrl+R) is lost when CASS exits. Users can't recall yesterday's searches.

**Solution:**
1. Add `query_history: Option<Vec<String>>` to `TuiStatePersisted`
2. Load on startup, save on exit
3. Cap at 50 entries (already defined as `history_cap`)

```rust
#[derive(Serialize, Deserialize, Default)]
struct TuiStatePersisted {
    match_mode: Option<String>,
    context_window: Option<String>,
    has_seen_help: Option<bool>,
    query_history: Option<Vec<String>>,  // NEW
}

// On load:
let mut query_history: VecDeque<String> = persisted
    .query_history
    .map(VecDeque::from)
    .unwrap_or_default();

// On save:
let persisted_out = TuiStatePersisted {
    // ...existing fields...
    query_history: Some(query_history.iter().cloned().collect()),
};
```

**Why This Matters:**
- Shell history persists; search history should too
- Power users rely on recalling past searches
- Trivial to implement with existing infrastructure

**Acceptance Criteria:**
- [ ] Query history survives restart
- [ ] Ctrl+R cycles through persisted queries
- [ ] History capped at 50 entries
- [ ] Duplicate queries not added consecutively

---

### ISSUE-008: Fixed Pane Item Count
**Category:** 🟠 P1 High Priority
**Location:** `src/ui/tui.rs:469`
**Files Affected:** `src/ui/tui.rs`

**Current Code:**
```rust
let mut per_pane_limit: usize = 12;
```

**Problem:**
On a 50-line terminal, 12 items per pane is fine. On a 100-line terminal, it wastes space. On a 20-line terminal, it overflows.

**Solution:**
Calculate based on available space:
```rust
fn calculate_per_pane_limit(terminal_height: u16) -> usize {
    // Layout: 1 margin + 3 search + 1 pills + 2 borders + 1 footer + 1 margin = 9 lines overhead
    // Results area is 70% of remaining
    // Each item is ~2 lines (title + snippet)
    let results_height = ((terminal_height.saturating_sub(9)) as f32 * 0.7) as usize;
    let items = results_height / 2;
    items.clamp(4, 50)  // Respect existing min/max
}

// In render loop:
let term_height = f.area().height;
if per_pane_limit_auto {
    per_pane_limit = calculate_per_pane_limit(term_height);
}
```

**Why This Matters:**
- Maximizes information density
- Works on all terminal sizes
- Users with large monitors see more results

**Acceptance Criteria:**
- [ ] Pane fills available space on large terminals
- [ ] Pane doesn't overflow on small terminals
- [ ] Manual +/- adjustment still works
- [ ] Minimum 4, maximum 50 preserved

---

### ISSUE-009: Detail Pane Too Small
**Category:** 🟡 P2 Medium Priority
**Location:** `src/ui/tui.rs:597-598`
**Files Affected:** `src/ui/tui.rs`

**Current Code:**
```rust
.constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
```

**Problem:**
The detail pane (showing full conversation) gets only 30% of the main area. When viewing long messages or code snippets, this is cramped.

**Solution Options:**
1. **Simple**: Change to 60/40 or 50/50 split
2. **Better**: Make it adjustable with a shortcut (e.g., `=` to increase detail, `-` to decrease)
3. **Best**: When detail is focused, expand to 60%; when results focused, shrink to 30%

Recommended implementation (Option 3):
```rust
let detail_percent = match focus_region {
    FocusRegion::Results => 30,
    FocusRegion::Detail => 50,
};
let results_percent = 100 - detail_percent;

.constraints([
    Constraint::Percentage(results_percent),
    Constraint::Percentage(detail_percent)
])
```

**Why This Matters:**
- Detail view is where users read content
- Code snippets need horizontal and vertical space
- Context-aware sizing is a premium UX pattern

**Acceptance Criteria:**
- [ ] Detail pane expands when focused
- [ ] Transition is visually smooth (or instant, no jarring)
- [ ] Results pane remains usable when detail expanded

---

### ISSUE-010: Status Bar Information Overload
**Category:** 🟡 P2 Medium Priority
**Location:** `src/ui/tui.rs:988-1010`
**Files Affected:** `src/ui/tui.rs`

**Current Code:**
```rust
let mut footer_line = format!(
    "{} | mode:{} | rank:{} | ctx:{}({}) | {}",
    status,
    match match_mode { ... },
    match ranking_mode { ... },
    context_window.label(),
    context_window.size(),
    footer_legend(show_help)
);
```

**Problem:**
The footer tries to show everything:
- Status message
- Match mode
- Ranking mode
- Context window size
- Full keyboard legend (200+ characters!)

Result: Unreadable wall of text, especially on narrow terminals.

**Solution:**
Tiered information display:
1. **Primary**: Status message (what just happened)
2. **Secondary**: Active modes (only if non-default)
3. **Tertiary**: "F1 for help" (not full legend)

```rust
let footer_line = if status.is_empty() {
    "Ready | F1 help".to_string()
} else {
    let mode_info = if match_mode != MatchMode::Prefix {
        format!(" | mode:{}", match_mode.label())
    } else {
        String::new()
    };
    format!("{}{} | F1 help", status, mode_info)
};
```

**Why This Matters:**
- Clean footer looks professional
- Users can find shortcuts in help overlay
- Status messages are actually readable

**Acceptance Criteria:**
- [ ] Footer is single readable line
- [ ] Status message takes priority
- [ ] F1 hint always visible
- [ ] Full legend only in help overlay

---

### ISSUE-011: No Loading Indicator
**Category:** 🟡 P2 Medium Priority
**Location:** `src/ui/tui.rs` (search execution)
**Files Affected:** `src/ui/tui.rs`

**Current State:**
When searching, the UI freezes momentarily with no feedback. Users don't know if:
- The search is running
- The app hung
- There are no results yet

**Solution:**
Add a "Searching..." status while query is pending:
```rust
if should_search {
    status = "Searching...".to_string();
    needs_draw = true;
    // Force a render before search
    terminal.draw(|f| { /* ... */ })?;

    match client.search(&q, filters.clone(), page_size, page * page_size) {
        Ok(hits) => {
            status = format!("Found {} results", hits.len());
            // ...
        }
        // ...
    }
}
```

For longer operations, consider a spinner:
```rust
const SPINNER: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
let spinner_frame = (tick_count % SPINNER.len()) as usize;
status = format!("{} Searching...", SPINNER[spinner_frame]);
```

**Why This Matters:**
- Feedback builds user confidence
- Distinguishes "searching" from "no results"
- Premium apps always show progress

**Acceptance Criteria:**
- [ ] "Searching..." shown during query
- [ ] Spinner animates for long searches
- [ ] Result count shown on completion

---

### ISSUE-012: Fragile Editor Line Detection
**Category:** 🟡 P2 Medium Priority
**Location:** `src/ui/tui.rs:1379-1388`
**Files Affected:** `src/ui/tui.rs`, `src/search/query.rs`

**Current Code:**
```rust
let line_hint = hit.snippet
    .find("line ")
    .and_then(|i| hit.snippet[i + 5..].split_whitespace().next())
    .and_then(|s| s.parse::<usize>().ok());
```

**Problem:**
The code tries to extract line numbers by searching for "line " in the snippet text. This is:
- Fragile: Breaks if snippet doesn't contain "line "
- Incorrect: May match "line " in actual content
- Lossy: Original line info exists but isn't preserved

**Solution:**
1. Add `line_number: Option<usize>` to `SearchHit`
2. Populate from message index or snippet metadata
3. Use directly when opening editor

```rust
// In SearchHit:
pub struct SearchHit {
    // ...existing fields...
    pub line_number: Option<usize>,
}

// When opening:
let line = hit.line_number;
let mut cmd = StdCommand::new(&editor_cmd);
if let Some(n) = line {
    cmd.arg(format!("{}{}", editor_line_flag, n));
}
cmd.arg(&hit.source_path);
```

**Why This Matters:**
- Editor integration is a key feature
- Broken jumps waste user time
- Clean data model prevents hacks

**Acceptance Criteria:**
- [ ] Line number stored in SearchHit
- [ ] Editor opens at correct line
- [ ] No more string parsing for line numbers

---

### ISSUE-013: Cramped Multi-Pane Layout
**Category:** 🟡 P2 Medium Priority
**Location:** `src/ui/tui.rs:655-663`
**Files Affected:** `src/ui/tui.rs`

**Current Code:**
```rust
let pane_width = (100 / std::cmp::max(panes.len(), 1)) as u16;
```

**Problem:**
With 6 agents, each pane is ~16% wide. Titles truncate, snippets are unreadable. The equal-width distribution doesn't scale.

**Solution:**
1. Cap visible panes at 4
2. Add horizontal scrolling for remaining panes
3. Or: Show panes in order of result count, hide empty panes

```rust
const MAX_VISIBLE_PANES: usize = 4;

let visible_panes: Vec<&AgentPane> = panes.iter()
    .filter(|p| !p.hits.is_empty())
    .take(MAX_VISIBLE_PANES)
    .collect();

let pane_width = (100 / visible_panes.len().max(1)) as u16;

// Show indicator if more panes exist
if panes.len() > MAX_VISIBLE_PANES {
    // Render "+2 more" indicator
}
```

**Why This Matters:**
- Readability trumps showing everything
- Most searches hit 2-3 agents anyway
- Users can filter to specific agent if needed

**Acceptance Criteria:**
- [ ] Maximum 4 panes visible
- [ ] Indicator shows hidden pane count
- [ ] Left/Right navigates to hidden panes
- [ ] Each visible pane has usable width

---

### ISSUE-014: No Visual Distinction for Relevance
**Category:** 🟡 P2 Medium Priority
**Location:** `src/ui/tui.rs:670-703`
**Files Affected:** `src/ui/tui.rs`, `src/ui/components/theme.rs`

**Current State:**
All results look identical except for position. A score of 8.5 looks the same as 2.1.

**Solution:**
Visual cues based on score:
```rust
fn score_style(score: f32, theme: &PaneTheme) -> Style {
    if score >= 5.0 {
        // High relevance: bright, bold
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
    } else if score >= 2.0 {
        // Medium relevance: normal
        Style::default().fg(theme.fg)
    } else {
        // Low relevance: dimmed
        Style::default().fg(theme.fg).add_modifier(Modifier::DIM)
    }
}

// Score badge with visual bar
fn score_badge(score: f32) -> String {
    let filled = (score.min(10.0) / 2.0) as usize;  // 0-5 blocks
    let empty = 5 - filled;
    format!("{}{} {:.1}",
        "█".repeat(filled),
        "░".repeat(empty),
        score
    )
}
// Output: "████░ 8.2"
```

**Why This Matters:**
- Visual hierarchy guides user attention
- High-value results should pop
- Matches how search engines show results

**Acceptance Criteria:**
- [ ] Top results visually distinct
- [ ] Score badge shows filled/empty blocks
- [ ] Low-relevance results dimmed

---

### ISSUE-015: No Relative Time Display
**Category:** 🟡 P2 Medium Priority
**Location:** `src/ui/tui.rs:410-414`
**Files Affected:** `src/ui/tui.rs`

**Current Code:**
```rust
dt.format("%Y-%m-%d %H:%M:%S UTC")
// Output: "2024-11-25 14:30:00 UTC"
```

**Problem:**
Full timestamps are hard to parse mentally. "2024-11-25 14:30:00 UTC" takes cognitive effort vs "2 hours ago".

**Solution:**
```rust
fn format_relative_time(ms: i64) -> String {
    let now = chrono::Utc::now().timestamp_millis();
    let diff_ms = now - ms;
    let diff_secs = diff_ms / 1000;

    if diff_secs < 60 {
        return "just now".to_string();
    }
    if diff_secs < 3600 {
        let mins = diff_secs / 60;
        return format!("{}m ago", mins);
    }
    if diff_secs < 86400 {
        let hours = diff_secs / 3600;
        return format!("{}h ago", hours);
    }
    if diff_secs < 604800 {
        let days = diff_secs / 86400;
        return format!("{}d ago", days);
    }

    // Older than a week: show date
    DateTime::<Utc>::from_timestamp_millis(ms)
        .map(|dt| dt.format("%b %d").to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
```

**Why This Matters:**
- "2 hours ago" is instantly understood
- Matches GitHub, Slack, every modern app
- Helps prioritize recent conversations

**Acceptance Criteria:**
- [ ] Times < 1 hour show minutes
- [ ] Times < 24 hours show hours
- [ ] Times < 7 days show days
- [ ] Older times show "Nov 25" format

---

### ISSUE-016: Smooth Focus Transition
**Category:** 🟢 P3 Polish
**Location:** `src/ui/tui.rs:1149-1150`
**Files Affected:** `src/ui/tui.rs`

**Current Code:**
```rust
focus_flash_until = Some(Instant::now() + Duration::from_millis(220));
```

**Problem:**
The pane flash is abrupt: full color → instant off. Premium apps use smooth fades.

**Solution:**
Implement opacity/color interpolation:
```rust
fn flash_color(base: Color, accent: Color, progress: f32) -> Color {
    // progress: 0.0 (start, accent) → 1.0 (end, base)
    match (base, accent) {
        (Color::Rgb(br, bg, bb), Color::Rgb(ar, ag, ab)) => {
            Color::Rgb(
                lerp(ar, br, progress),
                lerp(ag, bg, progress),
                lerp(ab, bb, progress),
            )
        }
        _ => if progress < 0.5 { accent } else { base }
    }
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t) as u8
}
```

**Why This Matters:**
- Smooth animations feel premium
- Abrupt changes feel jarring
- Small detail that elevates overall feel

**Acceptance Criteria:**
- [ ] Flash fades out over 220ms
- [ ] No visible flickering
- [ ] Works with RGB colors

---

### ISSUE-017: Contextual Empty States
**Category:** 🟢 P3 Polish
**Location:** `src/ui/tui.rs:604-653`
**Files Affected:** `src/ui/tui.rs`

**Current Code:**
```rust
lines.push(Line::from("No results found."));
```

**Problem:**
"No results found" with generic tips doesn't help users understand WHY or what to do.

**Solution:**
Context-aware empty states:
```rust
fn empty_state_message(query: &str, filters: &SearchFilters, match_mode: MatchMode) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(Span::styled(
            format!("No results for \"{}\"", query),
            Style::default().add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
    ];

    // Specific suggestions based on state
    let mut suggestions = Vec::new();

    if !filters.agents.is_empty() {
        suggestions.push(format!(
            "• Clear agent filter: {} (Shift+F3)",
            filters.agents.iter().next().unwrap()
        ));
    }

    if !filters.workspaces.is_empty() {
        suggestions.push("• Clear workspace filter (Shift+F4)".to_string());
    }

    if filters.created_from.is_some() || filters.created_to.is_some() {
        suggestions.push("• Remove time filter (Ctrl+Del)".to_string());
    }

    if matches!(match_mode, MatchMode::Standard) {
        suggestions.push("• Try prefix mode for partial matches (F9)".to_string());
    }

    if query.len() > 20 {
        suggestions.push("• Try shorter, more specific terms".to_string());
    }

    if suggestions.is_empty() {
        suggestions.push("• Check spelling".to_string());
        suggestions.push("• Try different keywords".to_string());
        suggestions.push("• Run 'cass index --full' to ensure all data is indexed".to_string());
    }

    lines.push(Line::from(Span::styled("Suggestions:", Style::default().fg(Color::Yellow))));
    for s in suggestions {
        lines.push(Line::from(s));
    }

    lines
}
```

**Why This Matters:**
- Helps users self-serve
- Reduces frustration
- Shows the app "understands" the situation

**Acceptance Criteria:**
- [ ] Empty state mentions the actual query
- [ ] Suggestions are contextual to active filters
- [ ] At least one actionable suggestion always shown

---

### ISSUE-018: Filter Autocomplete
**Category:** 🟢 P3 Polish
**Location:** `src/ui/tui.rs:1603-1637`
**Files Affected:** `src/ui/tui.rs`, potentially new UI component

**Current State:**
When pressing F3 for agent filter, user must know exact agent slug (e.g., "claude_code" not "Claude Code").

**Solution:**
Show autocomplete dropdown:
```rust
// When in Agent input mode:
let known_agents = ["claude_code", "codex", "gemini", "opencode", "cline", "amp"];

// Filter to matching
let matches: Vec<&str> = known_agents
    .iter()
    .filter(|a| a.contains(&input_buffer.to_lowercase()))
    .copied()
    .collect();

// Render as dropdown below search bar
if !matches.is_empty() && input_mode == InputMode::Agent {
    let dropdown_area = Rect::new(chunks[0].x, chunks[0].y + 3, 30, matches.len() as u16 + 2);
    let items: Vec<ListItem> = matches.iter().map(|a| ListItem::new(*a)).collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().bg(palette.accent));
    frame.render_stateful_widget(list, dropdown_area, &mut autocomplete_state);
}
```

**Why This Matters:**
- Users don't need to memorize slugs
- Faster filter application
- Discoverability of available agents

**Acceptance Criteria:**
- [ ] Dropdown appears when typing filter
- [ ] Arrow keys select from dropdown
- [ ] Enter confirms selection
- [ ] Tab cycles through matches

---

### ISSUE-019: Fuzzy Search Suggestions
**Category:** 🟢 P3 Polish
**Location:** `src/ui/tui.rs` (new feature)
**Files Affected:** `src/ui/tui.rs`, `src/search/query.rs`

**Current State:**
Typos like "matirx" return zero results with no help.

**Solution:**
Use Levenshtein distance or similar:
```rust
// When zero results, check if query is close to recent successful queries
fn suggest_correction(query: &str, history: &[String]) -> Option<String> {
    history.iter()
        .filter(|h| levenshtein(query, h) <= 2)  // Max 2 edits
        .min_by_key(|h| levenshtein(query, h))
        .cloned()
}

// Or check against indexed terms
fn suggest_from_index(query: &str, client: &SearchClient) -> Option<String> {
    // Query tantivy for terms starting with same letter
    // Find closest match
}
```

Display:
```
No results for "matirx"

Did you mean: "matrix"? (Enter to search)
```

**Why This Matters:**
- Typos are common
- "Did you mean" is expected in search
- Turns frustration into quick recovery

**Acceptance Criteria:**
- [ ] Suggestion shown for likely typos
- [ ] Enter searches the suggestion
- [ ] Only suggests if edit distance ≤ 2

---

### ISSUE-020: Score Visualization
**Category:** 🟢 P3 Polish
**Location:** `src/ui/tui.rs:679-683`
**Files Affected:** `src/ui/tui.rs`

**Current Code:**
```rust
Span::styled(
    format!("{:.1}", hit.score),
    Style::default().fg(theme.accent),
)
```

**Problem:**
"3.2" is abstract. Users can't intuit if that's good or bad.

**Solution:**
Visual score bar:
```rust
fn render_score(score: f32, max_score: f32) -> Vec<Span<'static>> {
    let normalized = (score / max_score).min(1.0);
    let filled = (normalized * 5.0) as usize;
    let empty = 5 - filled;

    vec![
        Span::styled("█".repeat(filled), Style::default().fg(Color::Green)),
        Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
        Span::raw(format!(" {:.1}", score)),
    ]
}
// Output: ████░ 8.2
```

**Why This Matters:**
- Visual patterns process faster than numbers
- Relative comparison at a glance
- Adds visual interest to results

**Acceptance Criteria:**
- [ ] Score shows as filled/empty bar
- [ ] Color indicates quality (green/yellow/red)
- [ ] Numeric score still visible

---

### ISSUE-021: Workspace Path Truncation
**Category:** 🟢 P3 Polish
**Location:** `src/ui/tui.rs:690-694`
**Files Affected:** `src/ui/tui.rs`

**Current Code:**
```rust
let location = if hit.workspace.is_empty() {
    hit.source_path.clone()
} else {
    format!("{} ({})", hit.source_path, hit.workspace)
};
```

**Problem:**
Long paths like `/home/user/projects/my-really-long-project-name/subdir` overflow.

**Solution:**
Smart truncation:
```rust
fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        return path.to_string();
    }

    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() <= 3 {
        return format!("...{}", &path[path.len().saturating_sub(max_len-3)..]);
    }

    // Keep first and last parts, ellipsis in middle
    let first = parts.first().unwrap_or(&"");
    let last = parts.last().unwrap_or(&"");
    let second_last = parts.get(parts.len().saturating_sub(2)).unwrap_or(&"");

    format!("{}/.../{}/{}", first, second_last, last)
}
// Input:  "/home/user/projects/my-really-long-project-name/subdir"
// Output: "/home/.../my-really-long-project-name/subdir"
```

**Why This Matters:**
- Paths are often long
- Beginning and end are most useful
- Prevents layout breaking

**Acceptance Criteria:**
- [ ] Long paths truncated with "..."
- [ ] First and last components preserved
- [ ] Home directory shown as "~"

---

### ISSUE-022: Result Count in Pane Title
**Category:** 🟢 P3 Polish
**Location:** `src/ui/tui.rs:717-730`
**Files Affected:** `src/ui/tui.rs`

**Current Code:**
```rust
format!("{} ({})", agent_display_name(&pane.agent), pane.hits.len())
// Output: "Claude Code (23)"
```

**Problem:**
"(23)" doesn't tell you if there are more results beyond what's shown.

**Solution:**
Show total count:
```rust
// In AgentPane, add total_count
struct AgentPane {
    agent: String,
    hits: Vec<SearchHit>,
    selected: usize,
    total_count: usize,  // Total matching, not just displayed
}

// In title:
if pane.total_count > pane.hits.len() {
    format!("{} ({}/{})", agent_display_name(&pane.agent), pane.hits.len(), pane.total_count)
    // Output: "Claude Code (12/47)"
} else {
    format!("{} ({})", agent_display_name(&pane.agent), pane.hits.len())
    // Output: "Claude Code (23)"
}
```

**Why This Matters:**
- Users know if scrolling will reveal more
- Informs decision to filter further
- Standard pattern (Gmail, Slack)

**Acceptance Criteria:**
- [ ] Title shows "shown/total" when truncated
- [ ] Shows just count when all results displayed
- [ ] Counts update on filter change

---

### ISSUE-023: Mouse Support
**Category:** 🟢 P3 Polish
**Location:** `src/ui/tui.rs` (new feature)
**Files Affected:** `src/ui/tui.rs`

**Current State:**
No mouse support. Click does nothing.

**Solution:**
Enable crossterm mouse capture:
```rust
// In setup:
execute!(stdout, crossterm::event::EnableMouseCapture)?;

// In event loop:
Event::Mouse(mouse_event) => {
    match mouse_event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let (col, row) = (mouse_event.column, mouse_event.row);

            // Hit test panes
            for (idx, pane_rect) in pane_rects.iter().enumerate() {
                if pane_rect.contains(col, row) {
                    active_pane = idx;
                    // Calculate which item was clicked
                    let item_idx = (row - pane_rect.y - 1) / 2;  // 2 lines per item
                    if let Some(pane) = panes.get_mut(idx) {
                        pane.selected = item_idx.min(pane.hits.len() - 1);
                    }
                    break;
                }
            }
        }
        MouseEventKind::ScrollUp => { /* scroll up */ }
        MouseEventKind::ScrollDown => { /* scroll down */ }
        _ => {}
    }
}

// In teardown:
execute!(stdout, crossterm::event::DisableMouseCapture)?;
```

**Why This Matters:**
- Many users expect mouse interaction
- Scroll wheel is natural for detail pane
- Clicks are faster for occasional use

**Acceptance Criteria:**
- [ ] Click selects result in pane
- [ ] Click switches active pane
- [ ] Scroll wheel works in detail pane
- [ ] Mouse disabled on exit

---

### ISSUE-024: Vim-Style Navigation
**Category:** 🟢 P3 Polish
**Location:** `src/ui/tui.rs:1087-1145`
**Files Affected:** `src/ui/tui.rs`

**Current State:**
`g` and `G` work for first/last, but `j`/`k` don't work for up/down.

**Solution:**
Add vim keybindings:
```rust
KeyCode::Char('j') => {
    // Same as Down
    if let Some(pane) = panes.get_mut(active_pane) {
        if pane.selected + 1 < pane.hits.len() {
            pane.selected += 1;
            cached_detail = None;
        }
    }
}
KeyCode::Char('k') => {
    // Same as Up
    if let Some(pane) = panes.get_mut(active_pane) {
        if pane.selected > 0 {
            pane.selected -= 1;
            cached_detail = None;
        }
    }
}
KeyCode::Char('h') => {
    // Same as Left (previous pane)
    active_pane = active_pane.saturating_sub(1);
}
KeyCode::Char('l') => {
    // Same as Right (next pane)
    if active_pane + 1 < panes.len() {
        active_pane += 1;
    }
}
```

**Why This Matters:**
- Developers use vim
- Home row navigation is faster
- Expected in terminal apps

**Acceptance Criteria:**
- [ ] j/k move up/down in results
- [ ] h/l move left/right between panes
- [ ] g/G already work
- [ ] Doesn't conflict with search input

---

### ISSUE-025: Manual Refresh Shortcut
**Category:** 🟢 P3 Polish
**Location:** `src/ui/tui.rs` (new feature)
**Files Affected:** `src/ui/tui.rs`, `src/lib.rs`

**Current State:**
No way to trigger re-index from TUI. Must exit and run `cass index`.

**Solution:**
Add F5 (or Ctrl+R when not in history mode) to trigger refresh:
```rust
KeyCode::F(5) if !key.modifiers.contains(KeyModifiers::SHIFT)
    && filters.created_from.is_none() => {
    // Trigger background reindex
    status = "Reindexing...".to_string();
    needs_draw = true;

    // Send signal to background indexer thread
    if let Some(tx) = &reindex_tx {
        let _ = tx.send(ReindexCommand::Full);
    }
}
```

Note: This overlaps with F5 for time filter. Need to check current state:
- F5 alone when no time filter → refresh
- F5 with existing filter or Shift+F5 → time preset cycling

**Why This Matters:**
- Users add new conversations during session
- Currently must exit/restart to see them
- Background indexer helps but manual refresh is expected

**Acceptance Criteria:**
- [ ] Shortcut triggers re-index
- [ ] "Reindexing..." status shown
- [ ] Results update after completion
- [ ] Doesn't block UI

---

## Implementation Beads (Task Breakdown)

Below are atomic implementation tasks organized by issue, with dependencies and effort.

### Legend
- **Effort**: S (< 30 min), M (30-60 min), L (1-2 hr), XL (2+ hr)
- **Deps**: List of bead IDs that must complete first
- **Files**: Primary files affected

---

### BEAD-001: Remove unsafe transmute in chips_for_filters
**Issue:** ISSUE-001
**Effort:** M
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Refactor `chips_for_filters` to return `Vec<Span<'static>>` using owned strings instead of transmute.

**Implementation Notes:**
- Change `format!()` calls to produce owned `String`
- Use `Span::styled(owned_string, style)` which takes ownership
- Remove the `unsafe` block entirely
- Test that filter chips still render correctly

**Verification:**
- `cargo build` succeeds
- `cargo test` passes
- Run TUI, apply filters, verify chips display

---

### BEAD-002: Remove unsafe transmute in highlight_terms_owned_with_style
**Issue:** ISSUE-001
**Effort:** M
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Refactor `highlight_terms_owned_with_style` to avoid transmute by ensuring all strings are owned.

**Implementation Notes:**
- Function already takes `String` (owned), but builds `Vec<Span>` that gets transmuted
- Change to build spans with `.to_string()` on slice results
- Return type is `Line<'static>` which should work with owned spans

**Verification:**
- Search results highlight correctly
- No compiler warnings about lifetimes

---

### BEAD-003: Audit all transmute usages
**Issue:** ISSUE-001
**Effort:** S
**Deps:** BEAD-001, BEAD-002
**Files:** `src/ui/tui.rs`

**Task:**
Grep for remaining `transmute` calls and ensure none remain.

**Verification:**
- `rg "transmute" src/` returns no results in ui code
- Codebase compiles

---

### BEAD-004: Fix binary name in index error message
**Issue:** ISSUE-002
**Effort:** S
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Change line 459 from `coding-agent-search index --full` to `cass index --full`.

**Verification:**
- Message uses `cass`
- Test by deleting index and launching TUI

---

### BEAD-005: Audit all user-facing strings for binary name
**Issue:** ISSUE-002
**Effort:** S
**Deps:** BEAD-004
**Files:** All

**Task:**
`rg "coding-agent-search"` and update any user-facing strings.

**Verification:**
- No user messages reference wrong binary name

---

### BEAD-006: Fix F11 → Ctrl+Del in widgets.rs
**Issue:** ISSUE-003
**Effort:** S
**Deps:** None
**Files:** `src/ui/components/widgets.rs`

**Task:**
Change line 41 from "F11 clear" to "Ctrl+Del clear".

**Verification:**
- Search bar tip shows correct shortcut

---

### BEAD-007: Create shortcut constants
**Issue:** ISSUE-003
**Effort:** M
**Deps:** BEAD-006
**Files:** New `src/ui/shortcuts.rs`, update tui.rs, widgets.rs

**Task:**
Create a single source of truth for keyboard shortcuts:
```rust
pub const SHORTCUT_CLEAR_FILTERS: &str = "Ctrl+Del";
pub const SHORTCUT_AGENT_FILTER: &str = "F3";
// etc.
```

Use these constants in help_lines, footer_legend, and widgets.

**Verification:**
- Shortcuts consistent everywhere
- Changing constant updates all locations

---

### BEAD-008: Create time parser module
**Issue:** ISSUE-004
**Effort:** L
**Deps:** None
**Files:** New `src/ui/time_parser.rs`

**Task:**
Implement `parse_time_input(input: &str) -> Option<i64>` that handles:
- Relative: `-7d`, `-24h`, `-1w`, `yesterday`, `today`
- ISO dates: `2024-11-25`
- Numeric: seconds or milliseconds

**Verification:**
- Unit tests for each format
- Invalid input returns None

---

### BEAD-009: Integrate time parser into TUI
**Issue:** ISSUE-004
**Deps:** BEAD-008
**Effort:** M
**Files:** `src/ui/tui.rs`

**Task:**
Replace `input_buffer.trim().parse::<i64>()` with `parse_time_input(&input_buffer)`.
Update status messages to reflect accepted formats.

**Verification:**
- `-7d` sets filter to 7 days ago
- Invalid input shows helpful message

---

### BEAD-010: Update time filter status messages
**Issue:** ISSUE-004
**Deps:** BEAD-009
**Effort:** S
**Files:** `src/ui/tui.rs`

**Task:**
Change prompts from "ms since epoch" to "e.g., -7d, yesterday, 2024-11-20".

**Verification:**
- User knows what to type

---

### BEAD-011: Create format_time_chip function
**Issue:** ISSUE-005
**Effort:** M
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Implement function to format time filter as readable string:
- `Some(ts), None` → "[time: Nov 20 → now]"
- `None, Some(ts)` → "[time: start → Nov 25]"

**Verification:**
- Chips no longer show raw milliseconds

---

### BEAD-012: Apply format_time_chip throughout
**Issue:** ISSUE-005
**Deps:** BEAD-011
**Effort:** S
**Files:** `src/ui/tui.rs`

**Task:**
Replace all `format!("[time:{:?}->{:?}]"...)` with `format_time_chip()`.

**Verification:**
- All time displays human-readable

---

### BEAD-013: Add has_seen_help to TuiStatePersisted
**Issue:** ISSUE-006
**Effort:** S
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Add `has_seen_help: Option<bool>` field to persisted state struct.

**Verification:**
- tui_state.json includes new field

---

### BEAD-014: Conditional help display on launch
**Issue:** ISSUE-006
**Deps:** BEAD-013
**Effort:** S
**Files:** `src/ui/tui.rs`

**Task:**
Set `show_help = !persisted.has_seen_help.unwrap_or(false)`.
On first F1 dismiss, set `has_seen_help = true` in saved state.

**Verification:**
- First launch: help shown
- Second launch: help hidden
- F1 always toggles

---

### BEAD-015: Add query_history to TuiStatePersisted
**Issue:** ISSUE-007
**Effort:** S
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Add `query_history: Option<Vec<String>>` to persisted state.

**Verification:**
- Field present in JSON

---

### BEAD-016: Load query_history on startup
**Issue:** ISSUE-007
**Deps:** BEAD-015
**Effort:** S
**Files:** `src/ui/tui.rs`

**Task:**
Load history from persisted state into `VecDeque`.

**Verification:**
- History available on launch

---

### BEAD-017: Save query_history on exit
**Issue:** ISSUE-007
**Deps:** BEAD-015
**Effort:** S
**Files:** `src/ui/tui.rs`

**Task:**
Save current history to persisted state on TUI exit.

**Verification:**
- History survives restart
- Ctrl+R cycles through old queries

---

### BEAD-018: Calculate dynamic per_pane_limit
**Issue:** ISSUE-008
**Effort:** M
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Create `calculate_per_pane_limit(terminal_height: u16) -> usize`.
Consider overhead (search bar, pills, footer, borders).

**Verification:**
- Large terminal shows more items
- Small terminal doesn't overflow

---

### BEAD-019: Apply dynamic limit in render
**Issue:** ISSUE-008
**Deps:** BEAD-018
**Effort:** S
**Files:** `src/ui/tui.rs`

**Task:**
Call `calculate_per_pane_limit` in render loop when auto mode.
Add flag to track if user has manually adjusted.

**Verification:**
- Resizing terminal adjusts item count
- Manual +/- overrides auto

---

### BEAD-020: Responsive detail pane sizing
**Issue:** ISSUE-009
**Effort:** M
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Change detail percentage based on focus:
- Results focused: 30%
- Detail focused: 50%

**Verification:**
- Tab to detail expands it
- Tab to results shrinks it

---

### BEAD-021: Simplify footer layout
**Issue:** ISSUE-010
**Effort:** M
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Reduce footer to: `{status} | F1 help`
Show mode only if non-default.

**Verification:**
- Footer fits on one line
- Essential info visible

---

### BEAD-022: Add "Searching..." status
**Issue:** ISSUE-011
**Effort:** M
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Set status to "Searching..." before search call.
Force redraw so user sees it.

**Verification:**
- User sees feedback during search

---

### BEAD-023: Add spinner animation
**Issue:** ISSUE-011
**Deps:** BEAD-022
**Effort:** M
**Files:** `src/ui/tui.rs`

**Task:**
Implement rotating spinner character in status during search.
Update on each tick if search pending.

**Verification:**
- Spinner animates
- Stops when search completes

---

### BEAD-024: Add line_number to SearchHit
**Issue:** ISSUE-012
**Effort:** M
**Deps:** None
**Files:** `src/search/query.rs`

**Task:**
Add `line_number: Option<usize>` field to SearchHit struct.

**Verification:**
- Struct compiles
- Serialization works

---

### BEAD-025: Populate line_number in search
**Issue:** ISSUE-012
**Deps:** BEAD-024
**Effort:** M
**Files:** `src/search/query.rs`

**Task:**
Determine line number from message index or stored data.
May require schema change or derive from content.

**Verification:**
- Line numbers populated for results

---

### BEAD-026: Use line_number in editor open
**Issue:** ISSUE-012
**Deps:** BEAD-025
**Effort:** S
**Files:** `src/ui/tui.rs`

**Task:**
Replace string parsing with `hit.line_number`.

**Verification:**
- Editor opens at correct line
- No more "line " string search

---

### BEAD-027: Cap visible panes at 4
**Issue:** ISSUE-013
**Effort:** M
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Filter to max 4 visible panes.
Prioritize by result count.

**Verification:**
- Max 4 panes shown
- Highest-count panes visible

---

### BEAD-028: Add hidden pane indicator
**Issue:** ISSUE-013
**Deps:** BEAD-027
**Effort:** S
**Files:** `src/ui/tui.rs`

**Task:**
If more panes exist, show "+N more" indicator.

**Verification:**
- User knows more agents have results

---

### BEAD-029: Navigate to hidden panes
**Issue:** ISSUE-013
**Deps:** BEAD-027
**Effort:** M
**Files:** `src/ui/tui.rs`

**Task:**
Left/Right navigation beyond visible panes scrolls the pane list.

**Verification:**
- Can reach all agent panes
- Visible set shifts as needed

---

### BEAD-030: Create score_style function
**Issue:** ISSUE-014
**Effort:** S
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Return different styles based on score thresholds.

**Verification:**
- High scores bold/bright
- Low scores dimmed

---

### BEAD-031: Apply score styling to results
**Issue:** ISSUE-014
**Deps:** BEAD-030
**Effort:** S
**Files:** `src/ui/tui.rs`

**Task:**
Use score_style when rendering result titles.

**Verification:**
- Visual hierarchy visible

---

### BEAD-032: Create score badge visualization
**Issue:** ISSUE-020
**Effort:** M
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Implement visual bar for scores: `████░ 8.2`

**Verification:**
- Bar reflects score magnitude

---

### BEAD-033: Create format_relative_time function
**Issue:** ISSUE-015
**Effort:** M
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Return "2h ago", "3d ago", or date for older.

**Verification:**
- Unit tests pass
- Recent times relative, old times absolute

---

### BEAD-034: Apply relative time in detail view
**Issue:** ISSUE-015
**Deps:** BEAD-033
**Effort:** S
**Files:** `src/ui/tui.rs`

**Task:**
Use format_relative_time for message timestamps.

**Verification:**
- Detail shows "2h ago" not full timestamp

---

### BEAD-035: Implement flash fade animation
**Issue:** ISSUE-016
**Effort:** L
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Track flash start time, calculate progress, interpolate colors.

**Verification:**
- Flash smoothly fades
- No abrupt transitions

---

### BEAD-036: Create contextual empty state function
**Issue:** ISSUE-017
**Effort:** M
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Build suggestions based on active filters and query.

**Verification:**
- Suggestions relevant to state
- At least one actionable suggestion

---

### BEAD-037: Apply contextual empty state
**Issue:** ISSUE-017
**Deps:** BEAD-036
**Effort:** S
**Files:** `src/ui/tui.rs`

**Task:**
Replace static "No results" with contextual function.

**Verification:**
- Empty state is helpful

---

### BEAD-038: Build agent autocomplete data
**Issue:** ISSUE-018
**Effort:** M
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Query database for known agent slugs.
Filter based on input.

**Verification:**
- List of agents available

---

### BEAD-039: Render autocomplete dropdown
**Issue:** ISSUE-018
**Deps:** BEAD-038
**Effort:** L
**Files:** `src/ui/tui.rs`

**Task:**
Render dropdown below search bar when in filter mode.
Handle arrow key selection.

**Verification:**
- Dropdown appears
- Selection works

---

### BEAD-040: Apply autocomplete selection
**Issue:** ISSUE-018
**Deps:** BEAD-039
**Effort:** S
**Files:** `src/ui/tui.rs`

**Task:**
Enter/Tab applies selected autocomplete.

**Verification:**
- Full workflow works

---

### BEAD-041: Add fuzzy matching library
**Issue:** ISSUE-019
**Effort:** S
**Deps:** None
**Files:** `Cargo.toml`

**Task:**
Add `strsim` or similar for Levenshtein distance.

**Verification:**
- Dependency available

---

### BEAD-042: Implement suggest_correction
**Issue:** ISSUE-019
**Deps:** BEAD-041
**Effort:** M
**Files:** `src/ui/tui.rs`

**Task:**
Check query against history, suggest if close match.

**Verification:**
- "matirx" suggests "matrix"

---

### BEAD-043: Display "Did you mean?" UI
**Issue:** ISSUE-019
**Deps:** BEAD-042
**Effort:** M
**Files:** `src/ui/tui.rs`

**Task:**
Show suggestion in empty state.
Enter searches suggestion.

**Verification:**
- Suggestion visible
- Enter applies it

---

### BEAD-044: Create truncate_path function
**Issue:** ISSUE-021
**Effort:** M
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Smart truncation preserving first and last components.
Replace home with ~.

**Verification:**
- Long paths truncated sensibly

---

### BEAD-045: Apply path truncation
**Issue:** ISSUE-021
**Deps:** BEAD-044
**Effort:** S
**Files:** `src/ui/tui.rs`

**Task:**
Use truncate_path for workspace and source_path display.

**Verification:**
- No overflow
- Paths still useful

---

### BEAD-046: Track total result count per agent
**Issue:** ISSUE-022
**Effort:** M
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Add total_count to AgentPane.
Populate from search results.

**Verification:**
- Total count tracked separately from displayed

---

### BEAD-047: Show total in pane title
**Issue:** ISSUE-022
**Deps:** BEAD-046
**Effort:** S
**Files:** `src/ui/tui.rs`

**Task:**
Format title as "Agent (12/47)" when truncated.

**Verification:**
- User knows more results exist

---

### BEAD-048: Enable mouse capture
**Issue:** ISSUE-023
**Effort:** S
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Add EnableMouseCapture on startup, DisableMouseCapture on exit.

**Verification:**
- Mouse events received

---

### BEAD-049: Handle mouse click on panes
**Issue:** ISSUE-023
**Deps:** BEAD-048
**Effort:** L
**Files:** `src/ui/tui.rs`

**Task:**
Track pane rectangles, hit test on click.
Select clicked item.

**Verification:**
- Click selects item

---

### BEAD-050: Handle scroll wheel
**Issue:** ISSUE-023
**Deps:** BEAD-048
**Effort:** M
**Files:** `src/ui/tui.rs`

**Task:**
ScrollUp/ScrollDown events scroll detail pane or result list.

**Verification:**
- Scroll wheel works

---

### BEAD-051: Add j/k navigation
**Issue:** ISSUE-024
**Effort:** S
**Deps:** None
**Files:** `src/ui/tui.rs`

**Task:**
Map j to Down, k to Up in Results focus.

**Verification:**
- j/k move selection

---

### BEAD-052: Add h/l navigation
**Issue:** ISSUE-024
**Deps:** None
**Effort:** S
**Files:** `src/ui/tui.rs`

**Task:**
Map h to Left (prev pane), l to Right (next pane).

**Verification:**
- h/l switch panes

---

### BEAD-053: Document vim keys in help
**Issue:** ISSUE-024
**Deps:** BEAD-051, BEAD-052
**Effort:** S
**Files:** `src/ui/tui.rs`

**Task:**
Add vim keys to help_lines.

**Verification:**
- F1 help shows h/j/k/l

---

### BEAD-054: Design refresh command flow
**Issue:** ISSUE-025
**Effort:** M
**Deps:** None
**Files:** Design doc

**Task:**
Decide shortcut (avoid conflict), define behavior.
Option: Ctrl+Shift+R or F5 when no time filter.

**Verification:**
- Design documented

---

### BEAD-055: Implement refresh trigger
**Issue:** ISSUE-025
**Deps:** BEAD-054
**Effort:** L
**Files:** `src/ui/tui.rs`, `src/lib.rs`

**Task:**
Send signal to background indexer.
Show status during reindex.

**Verification:**
- Reindex triggers
- Results update

---

### BEAD-056: Integration testing - run all changes
**Issue:** All
**Effort:** L
**Deps:** All above
**Files:** None

**Task:**
Full manual test of TUI with all changes.
Verify no regressions.

**Verification:**
- All features work
- No crashes

---

## Additional Beads for Completeness

### BEAD-057: Update README with new shortcuts
**Effort:** S
**Deps:** BEAD-007

### BEAD-058: Add CHANGELOG entry
**Effort:** S
**Deps:** All

### BEAD-059: Bump version number
**Effort:** S
**Deps:** BEAD-058

### BEAD-060: Create release PR
**Effort:** M
**Deps:** All

---

## Dependency Graph

```
                    ┌─────────────────────────────────────────┐
                    │              ISSUE-001                   │
                    │        Remove Unsafe Transmute          │
                    │   BEAD-001 ─► BEAD-002 ─► BEAD-003      │
                    └─────────────────────────────────────────┘
                                      │
                    ┌─────────────────────────────────────────┐
                    │              ISSUE-002                   │
                    │         Fix Binary Name                 │
                    │        BEAD-004 ─► BEAD-005             │
                    └─────────────────────────────────────────┘
                                      │
                    ┌─────────────────────────────────────────┐
                    │              ISSUE-003                   │
                    │        Shortcut Consistency             │
                    │        BEAD-006 ─► BEAD-007             │
                    └─────────────────────────────────────────┘
                                      │
    ┌───────────────────────┬─────────┴────────┬───────────────────────┐
    │                       │                  │                       │
    ▼                       ▼                  ▼                       ▼
┌───────────┐        ┌───────────┐      ┌───────────┐           ┌───────────┐
│ ISSUE-004 │        │ ISSUE-005 │      │ ISSUE-006 │           │ ISSUE-007 │
│ Time Parse│        │ Time Chip │      │ Help Once │           │ History   │
│ 008→009→010│       │ 011 → 012 │      │ 013 → 014 │           │015→016→017│
└───────────┘        └───────────┘      └───────────┘           └───────────┘

    ┌───────────────────────┬──────────────────┬───────────────────────┐
    │                       │                  │                       │
    ▼                       ▼                  ▼                       ▼
┌───────────┐        ┌───────────┐      ┌───────────┐           ┌───────────┐
│ ISSUE-008 │        │ ISSUE-009 │      │ ISSUE-010 │           │ ISSUE-011 │
│ Dyn Items │        │ Detail Sz │      │ Footer    │           │ Loading   │
│ 018 → 019 │        │    020    │      │    021    │           │ 022 → 023 │
└───────────┘        └───────────┘      └───────────┘           └───────────┘

    ┌───────────────────────┬──────────────────┬───────────────────────┐
    │                       │                  │                       │
    ▼                       ▼                  ▼                       ▼
┌───────────┐        ┌───────────┐      ┌───────────┐           ┌───────────┐
│ ISSUE-012 │        │ ISSUE-013 │      │ ISSUE-014 │           │ ISSUE-015 │
│ Line Num  │        │ Pane Limit│      │ Score Viz │           │ Rel Time  │
│024→025→026│        │027→028→029│      │ 030→031→032│          │ 033 → 034 │
└───────────┘        └───────────┘      └───────────┘           └───────────┘

    ┌───────────────────────┬──────────────────┬───────────────────────┐
    │                       │                  │                       │
    ▼                       ▼                  ▼                       ▼
┌───────────┐        ┌───────────┐      ┌───────────┐           ┌───────────┐
│ ISSUE-016 │        │ ISSUE-017 │      │ ISSUE-018 │           │ ISSUE-019 │
│ Flash Fade│        │ Empty St  │      │ Autocomp  │           │ Fuzzy Sug │
│    035    │        │ 036 → 037 │      │038→039→040│           │041→042→043│
└───────────┘        └───────────┘      └───────────┘           └───────────┘

    ┌───────────────────────┬──────────────────┬───────────────────────┐
    │                       │                  │                       │
    ▼                       ▼                  ▼                       ▼
┌───────────┐        ┌───────────┐      ┌───────────┐           ┌───────────┐
│ ISSUE-020 │        │ ISSUE-021 │      │ ISSUE-022 │           │ ISSUE-023 │
│ Score Bar │        │ Path Trunc│      │ Pane Count│           │ Mouse     │
│    032    │        │ 044 → 045 │      │ 046 → 047 │           │048→049→050│
└───────────┘        └───────────┘      └───────────┘           └───────────┘

    ┌───────────────────────┬──────────────────┐
    │                       │                  │
    ▼                       ▼                  │
┌───────────┐        ┌───────────┐             │
│ ISSUE-024 │        │ ISSUE-025 │             │
│ Vim Keys  │        │ Refresh   │             │
│051→052→053│        │ 054 → 055 │             │
└───────────┘        └───────────┘             │
                                               │
                    ┌──────────────────────────┘
                    │
                    ▼
           ┌───────────────┐
           │   BEAD-056    │
           │ Integration   │
           │   Testing     │
           └───────────────┘
                    │
        ┌──────────┬┴──────────┐
        ▼          ▼           ▼
   ┌────────┐ ┌────────┐ ┌────────┐
   │BEAD-057│ │BEAD-058│ │BEAD-059│
   │ README │ │CHANGELOG││ Version│
   └────────┘ └────────┘ └────────┘
                    │
                    ▼
              ┌────────┐
              │BEAD-060│
              │Release │
              │   PR   │
              └────────┘
```

---

## Effort Estimates

### Summary by Priority

| Priority | Issue Count | Total Beads | Estimated Hours |
|----------|-------------|-------------|-----------------|
| P0 Critical | 3 | 7 | 2-3 |
| P1 High | 5 | 20 | 6-8 |
| P2 Medium | 6 | 17 | 8-12 |
| P3 Polish | 11 | 24 | 12-18 |
| **Total** | **25** | **68** | **28-41** |

### Suggested Sprint Breakdown

**Sprint 1 (Critical + Quick Wins): ~4 hours**
- BEAD-001 to BEAD-007 (unsafe removal, binary name, shortcuts)
- BEAD-011, BEAD-012 (readable time chips)
- BEAD-013, BEAD-014 (help once)
- BEAD-015 to BEAD-017 (query history)

**Sprint 2 (High Priority UX): ~6 hours**
- BEAD-008 to BEAD-010 (human time input)
- BEAD-018, BEAD-019 (dynamic pane size)
- BEAD-020 (responsive detail)
- BEAD-021 (clean footer)
- BEAD-022, BEAD-023 (loading indicator)

**Sprint 3 (Medium Priority): ~8 hours**
- BEAD-024 to BEAD-026 (editor line numbers)
- BEAD-027 to BEAD-029 (pane limit)
- BEAD-030 to BEAD-032 (score visualization)
- BEAD-033, BEAD-034 (relative time)

**Sprint 4 (Polish): ~12 hours**
- BEAD-035 (flash animation)
- BEAD-036, BEAD-037 (contextual empty state)
- BEAD-038 to BEAD-040 (autocomplete)
- BEAD-041 to BEAD-043 (fuzzy suggestions)
- BEAD-044, BEAD-045 (path truncation)
- BEAD-046, BEAD-047 (pane counts)

**Sprint 5 (Navigation + Release): ~8 hours**
- BEAD-048 to BEAD-050 (mouse support)
- BEAD-051 to BEAD-053 (vim keys)
- BEAD-054, BEAD-055 (manual refresh)
- BEAD-056 to BEAD-060 (testing, docs, release)

---

## Success Metrics

### Functional
- [ ] Zero unsafe transmute calls
- [ ] All shortcuts work as documented
- [ ] Time filters accept human input
- [ ] Query history persists
- [ ] Help only shows on first launch

### Performance
- [ ] Search returns in < 100ms for typical queries
- [ ] TUI renders at 30fps (33ms per frame)
- [ ] No memory growth over extended use

### User Experience
- [ ] New user can search within 10 seconds of launch
- [ ] No user-visible raw debug output (timestamps, Some(), etc.)
- [ ] Every action has visual feedback
- [ ] Empty states provide actionable guidance

### Quality
- [ ] All existing tests pass
- [ ] No compiler warnings
- [ ] Code compiles on Windows, macOS, Linux
- [ ] Clippy reports no new warnings

---

## Appendix: File Reference

| File | Purpose | Key Functions |
|------|---------|---------------|
| `src/ui/tui.rs` | Main TUI implementation | `run_tui`, event loop, rendering |
| `src/ui/components/theme.rs` | Color definitions | `ThemePalette`, `PaneTheme`, `agent_pane` |
| `src/ui/components/widgets.rs` | Reusable widgets | `search_bar` |
| `src/ui/data.rs` | Data types & loading | `load_conversation`, `role_style` |
| `src/search/query.rs` | Search client | `SearchClient`, `SearchHit`, `SearchFilters` |
| `src/lib.rs` | CLI & entry point | `run`, `spawn_background_indexer` |
| `src/connectors/*.rs` | Agent connectors | Per-agent parsing |
| `src/indexer/mod.rs` | Indexing logic | `run_index` |

---

*Document generated as part of the CASS improvement initiative. This document should be kept updated as implementation progresses.*
