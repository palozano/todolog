use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::Path;

use crate::domain::{
    Fingerprint, LineNumber, Task, TaskFile, TaskId, TaskStatus, TaskText, TodoMarker,
};
use crate::id::fingerprint;

pub(crate) fn merge_tasks(mut scanned: Vec<Task>, existing: Vec<Task>) -> Vec<Task> {
    let existing_by_fingerprint: BTreeMap<Fingerprint, Task> = existing
        .into_iter()
        .map(|task| (task.fingerprint.clone(), task))
        .collect();
    let mut used_ids = BTreeSet::new();

    for task in &mut scanned {
        if let Some(existing) = existing_by_fingerprint.get(&task.fingerprint) {
            task.id = existing.id.clone();
            task.status = existing.status;
        }

        let original_id = task.id.clone();
        let mut suffix = 2;
        while used_ids.contains(&task.id) {
            task.id = original_id.with_suffix(suffix);
            suffix += 1;
        }
        used_ids.insert(task.id.clone());
    }

    scanned
}

pub(crate) fn read_tasks(path: &Path) -> io::Result<Vec<Task>> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err),
    };

    let mut tasks = Vec::new();
    let mut current_file = None;

    for line in contents.lines() {
        if let Some(file) = line.strip_prefix("## ") {
            current_file = Some(TaskFile::new(file.trim().to_string()));
        } else if let Some(file) = &current_file {
            if let Some(task) = parse_task_line(line, file) {
                tasks.push(task);
            }
        }
    }

    Ok(tasks)
}

pub(crate) fn parse_task_line(line: &str, current_file: &TaskFile) -> Option<Task> {
    let rest = line.strip_prefix("- [")?;
    let (status_text, rest) = rest.split_once("] ")?;
    let status = match status_text {
        " " => TaskStatus::Open,
        "x" | "X" => TaskStatus::Done,
        _ => return None,
    };
    let rest = rest.strip_prefix('`')?;
    let (id, rest) = rest.split_once("` ")?;
    let rest = rest.strip_prefix("L")?;
    let (line_text, rest) = rest.split_once(" - ")?;
    let line_number = LineNumber::new(line_text.parse().ok()?)?;
    let (text, metadata) = rest.split_once(" <!-- todolog:")?;
    let metadata = metadata.strip_suffix(" -->")?;
    let marker = metadata_value(metadata, "marker")
        .and_then(|value| TodoMarker::parse(&value))
        .unwrap_or(TodoMarker::Todo);
    let fingerprint = metadata_value(metadata, "fingerprint")
        .map(Fingerprint::new)
        .unwrap_or_else(|| fingerprint(current_file.as_str(), text.trim()));

    Some(Task {
        id: TaskId::new(id),
        status,
        file: current_file.clone(),
        line: line_number,
        marker,
        text: TaskText::new(text.trim().to_string())?,
        fingerprint,
    })
}

fn metadata_value(metadata: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}=\"");
    let start = metadata.find(&prefix)? + prefix.len();
    let value = &metadata[start..];
    let end = value.find('"')?;
    Some(value[..end].to_string())
}

pub(crate) fn write_tasks(path: &Path, tasks: &[Task]) -> io::Result<()> {
    let mut by_file: BTreeMap<&TaskFile, Vec<&Task>> = BTreeMap::new();
    for task in tasks {
        by_file.entry(&task.file).or_default().push(task);
    }

    let mut output = String::from("# Code Tasks\n\n");
    for (file, tasks) in by_file {
        output.push_str("## ");
        output.push_str(file.as_str());
        output.push_str("\n\n");
        for task in tasks {
            output.push_str(&format!(
                "- [{}] `{}` L{} - {} <!-- todolog: marker=\"{}\" fingerprint=\"{}\" -->\n",
                task.status.checkbox(),
                task.id,
                task.line,
                task.text,
                task.marker,
                task.fingerprint
            ));
        }
        output.push('\n');
    }

    fs::write(path, output)
}

pub(crate) fn update_status(path: &Path, id: &TaskId, status: TaskStatus) -> Result<(), String> {
    let mut tasks =
        read_tasks(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let mut found = false;

    for task in &mut tasks {
        if &task.id == id {
            task.status = status;
            found = true;
        }
    }

    if !found {
        return Err(format!("task `{id}` was not found in {}", path.display()));
    }

    write_tasks(path, &tasks)
        .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::IdStrategy;
    use crate::id::{fingerprint, task_id};
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("todolog-{name}-{}-{nanos}", std::process::id()))
    }

    fn task(
        id: &str,
        status: TaskStatus,
        file: &str,
        line: usize,
        marker: TodoMarker,
        text: &str,
        fingerprint: Fingerprint,
    ) -> Task {
        Task {
            id: TaskId::new(id),
            status,
            file: TaskFile::new(file),
            line: LineNumber::new(line).unwrap(),
            marker,
            text: TaskText::new(text).unwrap(),
            fingerprint,
        }
    }

    #[test]
    fn parses_task_line_with_metadata() {
        let file = TaskFile::new("src/main.rs");
        let task = parse_task_line(
            "- [x] `T-ABCDEFGH` L42 - ship it <!-- todolog: marker=\"FIXME\" fingerprint=\"0123456789abcdef\" -->",
            &file,
        )
        .unwrap();

        assert_eq!(task.id, TaskId::new("T-ABCDEFGH"));
        assert_eq!(task.status, TaskStatus::Done);
        assert_eq!(task.file, file);
        assert_eq!(task.line, LineNumber::new(42).unwrap());
        assert_eq!(task.marker, TodoMarker::Fixme);
        assert_eq!(task.text, TaskText::new("ship it").unwrap());
        assert_eq!(task.fingerprint, Fingerprint::new("0123456789abcdef"));
    }

    #[test]
    fn parse_task_line_rejects_zero_line_number() {
        let file = TaskFile::new("src/main.rs");
        assert_eq!(
            parse_task_line(
                "- [ ] `T-ABCDEFGH` L0 - ship it <!-- todolog: marker=\"TODO\" fingerprint=\"0123456789abcdef\" -->",
                &file,
            ),
            None
        );
    }

    #[test]
    fn parse_task_line_defaults_missing_or_unknown_marker() {
        let file = TaskFile::new("src/main.rs");
        let task = parse_task_line(
            "- [ ] `T-ABCDEFGH` L1 - ship it <!-- todolog: marker=\"NOTE\" -->",
            &file,
        )
        .unwrap();

        assert_eq!(task.marker, TodoMarker::Todo);
        assert_eq!(task.fingerprint, fingerprint("src/main.rs", "ship it"));
    }

    #[test]
    fn renders_and_reads_markdown_tasks() {
        let dir = temp_dir("roundtrip");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let task_file = dir.join("TASKS.md");
        let tasks = vec![task(
            "T-ABCDEFGH",
            TaskStatus::Open,
            "src/main.rs",
            12,
            TodoMarker::Todo,
            "build it",
            Fingerprint::new("0123456789abcdef"),
        )];

        write_tasks(&task_file, &tasks).unwrap();
        assert_eq!(read_tasks(&task_file).unwrap(), tasks);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn merge_preserves_done_status() {
        let fingerprint = fingerprint("src/lib.rs", "ship it");
        let scanned = vec![task(
            &task_id(&fingerprint, IdStrategy::Uid).to_string(),
            TaskStatus::Open,
            "src/lib.rs",
            3,
            TodoMarker::Todo,
            "ship it",
            fingerprint.clone(),
        )];
        let existing = vec![task(
            "T-CUSTOM1",
            TaskStatus::Done,
            "src/lib.rs",
            1,
            TodoMarker::Todo,
            "ship it",
            fingerprint,
        )];

        let merged = merge_tasks(scanned, existing);
        assert_eq!(merged[0].id, TaskId::new("T-CUSTOM1"));
        assert_eq!(merged[0].status, TaskStatus::Done);
        assert_eq!(merged[0].line, LineNumber::new(3).unwrap());
    }

    #[test]
    fn merge_suffixes_duplicate_task_ids() {
        let id = TaskId::new("T-DUPLICATE");
        let scanned = vec![
            task(
                &id.to_string(),
                TaskStatus::Open,
                "src/a.rs",
                1,
                TodoMarker::Todo,
                "first",
                Fingerprint::new("aaaaaaaaaaaaaaaa"),
            ),
            task(
                &id.to_string(),
                TaskStatus::Open,
                "src/b.rs",
                2,
                TodoMarker::Todo,
                "second",
                Fingerprint::new("bbbbbbbbbbbbbbbb"),
            ),
            task(
                &id.to_string(),
                TaskStatus::Open,
                "src/c.rs",
                3,
                TodoMarker::Todo,
                "third",
                Fingerprint::new("cccccccccccccccc"),
            ),
        ];

        let merged = merge_tasks(scanned, Vec::new());
        assert_eq!(merged[0].id, TaskId::new("T-DUPLICATE"));
        assert_eq!(merged[1].id, TaskId::new("T-DUPLICATE-2"));
        assert_eq!(merged[2].id, TaskId::new("T-DUPLICATE-3"));
    }

    #[test]
    fn update_status_changes_only_matching_task() {
        let dir = temp_dir("status");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let task_file = dir.join("TASKS.md");
        let tasks = vec![
            task(
                "T-FIRST",
                TaskStatus::Open,
                "src/a.rs",
                1,
                TodoMarker::Todo,
                "first",
                Fingerprint::new("aaaaaaaaaaaaaaaa"),
            ),
            task(
                "T-SECOND",
                TaskStatus::Open,
                "src/b.rs",
                2,
                TodoMarker::Hack,
                "second",
                Fingerprint::new("bbbbbbbbbbbbbbbb"),
            ),
        ];

        write_tasks(&task_file, &tasks).unwrap();
        update_status(&task_file, &TaskId::new("T-SECOND"), TaskStatus::Done).unwrap();
        let updated = read_tasks(&task_file).unwrap();

        assert_eq!(updated[0].status, TaskStatus::Open);
        assert_eq!(updated[1].status, TaskStatus::Done);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn update_status_errors_for_unknown_task() {
        let dir = temp_dir("missing-status");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let task_file = dir.join("TASKS.md");
        write_tasks(
            &task_file,
            &[task(
                "T-FIRST",
                TaskStatus::Open,
                "src/a.rs",
                1,
                TodoMarker::Todo,
                "first",
                Fingerprint::new("aaaaaaaaaaaaaaaa"),
            )],
        )
        .unwrap();

        let error =
            update_status(&task_file, &TaskId::new("T-MISSING"), TaskStatus::Done).unwrap_err();
        assert!(error.contains("T-MISSING"));

        let _ = fs::remove_dir_all(&dir);
    }
}
