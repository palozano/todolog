use std::io;
use std::path::Path;

use crate::domain::{Task, TaskId, TaskStatus};
use crate::store::{read_tasks, write_tasks};

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TaskQuery {
    open_only: bool,
}

impl TaskQuery {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn open_only(mut self, open_only: bool) -> Self {
        self.open_only = open_only;
        self
    }

    pub(crate) fn load(self, path: &Path) -> io::Result<Vec<Task>> {
        let mut tasks = read_tasks(path)?;
        if self.open_only {
            tasks.retain(|task| task.status == TaskStatus::Open);
        }
        Ok(tasks)
    }
}

pub(crate) fn set_task_status(path: &Path, id: &TaskId, status: TaskStatus) -> Result<(), String> {
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
    use crate::domain::{Fingerprint, LineNumber, TaskFile, TaskText, TodoMarker};
    use std::env;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("todolog-{name}-{}-{nanos}", std::process::id()))
    }

    fn task(id: &str, status: TaskStatus, text: &str) -> Task {
        Task {
            id: TaskId::new(id),
            status,
            file: TaskFile::new("src/main.rs"),
            line: LineNumber::new(42).unwrap(),
            marker: TodoMarker::Todo,
            text: TaskText::new(text).unwrap(),
            fingerprint: Fingerprint::new(format!("{id}-fingerprint")),
        }
    }

    #[test]
    fn query_can_filter_done_tasks() {
        let dir = temp_dir("query-open");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let task_file = dir.join("TASKS.md");
        write_tasks(
            &task_file,
            &[
                task("T-OPEN", TaskStatus::Open, "open task"),
                task("T-DONE", TaskStatus::Done, "done task"),
            ],
        )
        .unwrap();

        let tasks = TaskQuery::new().open_only(true).load(&task_file).unwrap();

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, TaskId::new("T-OPEN"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn set_task_status_changes_only_matching_task() {
        let dir = temp_dir("status");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let task_file = dir.join("TASKS.md");
        write_tasks(
            &task_file,
            &[
                task("T-FIRST", TaskStatus::Open, "first task"),
                task("T-SECOND", TaskStatus::Open, "second task"),
            ],
        )
        .unwrap();

        set_task_status(&task_file, &TaskId::new("T-SECOND"), TaskStatus::Done).unwrap();

        let updated = read_tasks(&task_file).unwrap();
        assert_eq!(updated[0].status, TaskStatus::Open);
        assert_eq!(updated[1].status, TaskStatus::Done);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn set_task_status_errors_for_unknown_task() {
        let dir = temp_dir("missing-status");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let task_file = dir.join("TASKS.md");
        write_tasks(
            &task_file,
            &[task("T-FIRST", TaskStatus::Open, "first task")],
        )
        .unwrap();

        let err =
            set_task_status(&task_file, &TaskId::new("T-MISSING"), TaskStatus::Done).unwrap_err();

        assert!(err.contains("T-MISSING"));

        let _ = fs::remove_dir_all(&dir);
    }
}
