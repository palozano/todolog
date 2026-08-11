use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;

use crate::config::Config;
use crate::constants::DEFAULT_TASK_FILE;
use crate::domain::{LineNumber, Task, TaskFile, TaskStatus, TaskText, TodoMarker};
use crate::id::{fingerprint, task_id};

const TODO_MARKERS: &[TodoMarker] = &[
    TodoMarker::Todo,
    TodoMarker::Fixme,
    TodoMarker::Xxx,
    TodoMarker::Hack,
];

pub(crate) fn scan_dir(root: &Path, task_file: &Path, config: &Config) -> io::Result<Vec<Task>> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let task_file = task_file
        .canonicalize()
        .unwrap_or_else(|_| task_file.to_path_buf());
    let mut tasks = Vec::new();
    visit_dir(&root, &root, &task_file, config, &mut tasks)?;
    tasks.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    Ok(tasks)
}

fn visit_dir(
    root: &Path,
    dir: &Path,
    task_file: &Path,
    config: &Config,
    tasks: &mut Vec<Task>,
) -> io::Result<()> {
    if should_skip_dir(dir) {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            visit_dir(root, &path, task_file, config, tasks)?;
        } else if metadata.is_file() && path != task_file && should_scan_file(&path) {
            scan_file(root, &path, config, tasks)?;
        }
    }

    Ok(())
}

fn scan_file(root: &Path, path: &Path, config: &Config, tasks: &mut Vec<Task>) -> io::Result<()> {
    let contents = fs::read_to_string(path)?;
    let file = TaskFile::new(
        path.strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/"),
    );

    for (line_index, line) in contents.lines().enumerate() {
        if let Some((marker, text)) = parse_todo(line) {
            let fingerprint = fingerprint(file.as_str(), text.as_str());
            tasks.push(Task {
                id: task_id(&fingerprint, config.id_strategy),
                status: TaskStatus::Open,
                file: file.clone(),
                line: LineNumber::new(line_index + 1).expect("enumerated line numbers start at 1"),
                marker,
                text,
                fingerprint,
            });
        }
    }

    Ok(())
}

pub(crate) fn parse_todo(line: &str) -> Option<(TodoMarker, TaskText)> {
    let line = comment_text(line)?;
    let mut best: Option<(usize, TodoMarker)> = None;
    for marker in TODO_MARKERS {
        if let Some(index) = line.find(marker.as_str()) {
            if best.is_none_or(|(best_index, _)| index < best_index) {
                best = Some((index, *marker));
            }
        }
    }

    let (index, marker) = best?;
    let after_marker = &line[index + marker.as_str().len()..];
    let text = after_marker
        .trim_start_matches(|ch: char| ch == ':' || ch == '-' || ch.is_whitespace())
        .trim();

    TaskText::new(text.to_string()).map(|text| (marker, text))
}

fn comment_text(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('*') {
        return Some(trimmed.trim_start_matches('*'));
    }
    if trimmed.starts_with(';') {
        return Some(trimmed.trim_start_matches(';'));
    }

    comment_start_outside_quotes(line).map(|start| &line[start..])
}

fn comment_start_outside_quotes(line: &str) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;

    for (index, ch) in line.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '/' if line[index..].starts_with("//") || line[index..].starts_with("/*") => {
                return Some(index + 2);
            }
            '<' if line[index..].starts_with("<!--") => return Some(index + 4),
            '-' if line[index..].starts_with("--") => return Some(index + 2),
            '#' => return Some(index + 1),
            _ => {}
        }
    }

    None
}

fn should_scan_file(path: &Path) -> bool {
    let name = path.file_name().and_then(OsStr::to_str).unwrap_or("");
    if name == DEFAULT_TASK_FILE {
        return false;
    }

    matches!(
        path.extension().and_then(OsStr::to_str),
        Some(
            "c" | "cc"
                | "cpp"
                | "cs"
                | "css"
                | "el"
                | "ex"
                | "exs"
                | "go"
                | "h"
                | "hpp"
                | "html"
                | "java"
                | "js"
                | "jsx"
                | "kt"
                | "lua"
                | "php"
                | "py"
                | "rb"
                | "rs"
                | "scala"
                | "sh"
                | "swift"
                | "ts"
                | "tsx"
                | "vim"
                | "vue"
        )
    )
}

fn should_skip_dir(path: &Path) -> bool {
    let name = path.file_name().and_then(OsStr::to_str).unwrap_or("");
    matches!(
        name,
        ".git" | ".hg" | ".svn" | "node_modules" | "target" | "dist" | "build" | ".next"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, IdStrategy};
    use crate::id::task_id;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("todolog-{name}-{}-{nanos}", std::process::id()))
    }

    #[test]
    fn parses_todo_text() {
        assert_eq!(
            parse_todo("    // TODO: wire editor command"),
            Some((
                TodoMarker::Todo,
                TaskText::new("wire editor command").unwrap()
            ))
        );
    }

    #[test]
    fn parses_supported_todo_markers() {
        assert_eq!(
            parse_todo("# FIXME - handle failures"),
            Some((TodoMarker::Fixme, TaskText::new("handle failures").unwrap()))
        );
        assert_eq!(
            parse_todo("-- XXX revisit query shape"),
            Some((
                TodoMarker::Xxx,
                TaskText::new("revisit query shape").unwrap()
            ))
        );
        assert_eq!(
            parse_todo("; HACK keep editor bridge synchronous"),
            Some((
                TodoMarker::Hack,
                TaskText::new("keep editor bridge synchronous").unwrap()
            ))
        );
    }

    #[test]
    fn ignores_todo_in_string_literals() {
        assert_eq!(parse_todo("let marker = \"TODO\";"), None);
        assert_eq!(
            parse_todo("assert_eq!(parse_todo(\"// TODO: not real\"), None);"),
            None
        );
    }

    #[test]
    fn ignores_empty_todos() {
        assert_eq!(parse_todo("# TODO"), None);
    }

    #[test]
    fn scan_uses_configured_id_strategy() {
        let dir = temp_dir("scan-config");
        let src_dir = dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("lib.rs"), "// TODO: configurable IDs\n").unwrap();
        fs::write(dir.join(".todolog"), "id = uid\n").unwrap();

        let config = Config {
            id_strategy: IdStrategy::Uid,
        };
        let tasks = scan_dir(&dir, &dir.join("TASKS.md"), &config).unwrap();

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, task_id(&tasks[0].fingerprint, IdStrategy::Uid));

        let _ = fs::remove_dir_all(&dir);
    }
}
