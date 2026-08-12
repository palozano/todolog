use crate::domain::Task;

#[derive(Debug, Clone, Copy)]
pub(crate) enum TaskListFormat {
    Default,
    Quickfix,
    Emacs,
}

pub(crate) fn format_task(task: &Task, format: TaskListFormat) -> String {
    match format {
        TaskListFormat::Default => format!(
            "{} [{}] {}:{} {}",
            task.id,
            task.status.checkbox(),
            task.file,
            task.line,
            task.text
        ),
        TaskListFormat::Quickfix => {
            format!("{}:{}:1: [{}] {}", task.file, task.line, task.id, task.text)
        }
        TaskListFormat::Emacs => {
            format!("{}:{}:1: {} [{}]", task.file, task.line, task.text, task.id)
        }
    }
}

pub(crate) fn format_tasks<'a>(
    tasks: impl IntoIterator<Item = &'a Task>,
    format: TaskListFormat,
) -> Vec<String> {
    tasks
        .into_iter()
        .map(|task| format_task(task, format))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        Fingerprint, LineNumber, TaskFile, TaskId, TaskStatus, TaskText, TodoMarker,
    };

    fn task() -> Task {
        Task {
            id: TaskId::new("20260811-141530"),
            status: TaskStatus::Open,
            file: TaskFile::new("src/main.rs"),
            line: LineNumber::new(42).unwrap(),
            marker: TodoMarker::Todo,
            text: TaskText::new("wire editor command").unwrap(),
            fingerprint: Fingerprint::new("0123456789abcdef"),
        }
    }

    #[test]
    fn formats_default_list_output() {
        assert_eq!(
            format_task(&task(), TaskListFormat::Default),
            "20260811-141530 [ ] src/main.rs:42 wire editor command"
        );
    }

    #[test]
    fn formats_quickfix_list_output() {
        assert_eq!(
            format_task(&task(), TaskListFormat::Quickfix),
            "src/main.rs:42:1: [20260811-141530] wire editor command"
        );
    }

    #[test]
    fn formats_emacs_list_output() {
        assert_eq!(
            format_task(&task(), TaskListFormat::Emacs),
            "src/main.rs:42:1: wire editor command [20260811-141530]"
        );
    }
}
