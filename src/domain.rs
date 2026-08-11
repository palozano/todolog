use std::fmt;
use std::num::NonZeroUsize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Task {
    pub(crate) id: TaskId,
    pub(crate) status: TaskStatus,
    pub(crate) file: TaskFile,
    pub(crate) line: LineNumber,
    pub(crate) marker: TodoMarker,
    pub(crate) text: TaskText,
    pub(crate) fingerprint: Fingerprint,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TaskId(String);

impl TaskId {
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub(crate) fn with_suffix(&self, suffix: usize) -> Self {
        Self(format!("{}-{suffix}", self.0))
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TaskFile(String);

impl TaskFile {
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TaskFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LineNumber(NonZeroUsize);

impl LineNumber {
    pub(crate) fn new(value: usize) -> Option<Self> {
        NonZeroUsize::new(value).map(Self)
    }

    pub(crate) fn get(self) -> usize {
        self.0.get()
    }
}

impl fmt::Display for LineNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(formatter)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TodoMarker {
    Todo,
    Fixme,
    Xxx,
    Hack,
}

impl TodoMarker {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Todo => "TODO",
            Self::Fixme => "FIXME",
            Self::Xxx => "XXX",
            Self::Hack => "HACK",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "TODO" => Some(Self::Todo),
            "FIXME" => Some(Self::Fixme),
            "XXX" => Some(Self::Xxx),
            "HACK" => Some(Self::Hack),
            _ => None,
        }
    }
}

impl fmt::Display for TodoMarker {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskText(String);

impl TaskText {
    pub(crate) fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            None
        } else {
            Some(Self(value))
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TaskText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Fingerprint(String);

impl Fingerprint {
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Fingerprint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TaskStatus {
    Open,
    Done,
}

impl TaskStatus {
    pub(crate) fn checkbox(self) -> &'static str {
        match self {
            Self::Open => " ",
            Self::Done => "x",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_text_rejects_blank_values() {
        assert_eq!(TaskText::new(""), None);
        assert_eq!(TaskText::new(" \t\n "), None);
    }

    #[test]
    fn line_number_rejects_zero() {
        assert_eq!(LineNumber::new(0), None);
        assert_eq!(LineNumber::new(1).unwrap().get(), 1);
    }
}
