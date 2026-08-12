use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::config::load_config;
use crate::constants::{DEFAULT_CONFIG_FILE, DEFAULT_TASK_FILE};
use crate::domain::{TaskId, TaskStatus};
use crate::output::{format_task, format_tasks, TaskListFormat};
use crate::scanner::scan_dir;
use crate::store::{merge_tasks, read_tasks, write_tasks};
use crate::tasks::{set_task_status, TaskQuery};
use crate::tui::{run_task_list, RenderMode, TaskListCommand};

pub(crate) fn run(args: Vec<String>) -> Result<(), String> {
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(err) if err.kind() == clap::error::ErrorKind::DisplayHelp => {
            err.print()
                .map_err(|print_err| format!("failed to print help: {print_err}"))?;
            return Ok(());
        }
        Err(err) if err.kind() == clap::error::ErrorKind::DisplayVersion => {
            err.print()
                .map_err(|print_err| format!("failed to print version: {print_err}"))?;
            return Ok(());
        }
        Err(err) => return Err(err.to_string()),
    };

    match cli.command {
        Some(Command::Scan(options)) => scan(options),
        Some(Command::List(options)) => list(options),
        Some(Command::Done(options)) => set_status(options, TaskStatus::Done),
        Some(Command::Open(options)) => set_status(options, TaskStatus::Open),
        None => run_default(cli.no_scan),
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "todolog",
    about = "Track TODO comments in a plain Markdown task file."
)]
struct Cli {
    /// Skip scanning before opening the default interactive task list.
    #[arg(long)]
    no_scan: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Scan source files and write the task file.
    Scan(ScanOptions),
    /// Display tasks as text or in an interactive terminal UI.
    List(ListOptions),
    /// Mark a task as done.
    Done(IdOptions),
    /// Reopen a task.
    Open(IdOptions),
}

#[derive(Debug, Args)]
struct ScanOptions {
    /// Directory to scan.
    #[arg(default_value = ".")]
    root: PathBuf,
    /// Task file to write.
    #[arg(short, long, default_value = DEFAULT_TASK_FILE)]
    output: PathBuf,
    /// Config file to read.
    #[arg(short, long)]
    config: Option<PathBuf>,
}

impl ScanOptions {
    fn config_path(&self) -> PathBuf {
        self.config
            .clone()
            .unwrap_or_else(|| self.root.join(DEFAULT_CONFIG_FILE))
    }
}

#[derive(Debug, Args)]
struct ListOptions {
    /// Task file to read.
    input: Option<PathBuf>,
    /// Task file to read.
    #[arg(short, long)]
    file: Option<PathBuf>,
    /// Show only open tasks.
    #[arg(long)]
    open: bool,
    /// Display tasks in an interactive full-screen terminal UI.
    #[arg(short, long)]
    interactive: bool,
    /// Display the interactive terminal UI inline.
    #[arg(short = 'l', long)]
    inline: bool,
    /// Display the interactive terminal UI full-screen.
    #[arg(long)]
    full_screen: bool,
    /// Print Vim/Neovim quickfix-compatible output.
    #[arg(long)]
    quickfix: bool,
    /// Output format for non-interactive listing.
    #[arg(long, value_enum)]
    format: Option<ListFormatArg>,
}

impl ListOptions {
    fn input_path(&self) -> PathBuf {
        self.file
            .clone()
            .or_else(|| self.input.clone())
            .unwrap_or_else(|| PathBuf::from(DEFAULT_TASK_FILE))
    }

    fn list_format(&self) -> TaskListFormat {
        if self.quickfix {
            TaskListFormat::Quickfix
        } else {
            self.format
                .map(ListFormatArg::into_task_list_format)
                .unwrap_or(TaskListFormat::Default)
        }
    }

    fn render_mode(&self) -> Option<RenderMode> {
        if self.inline {
            Some(RenderMode::Inline)
        } else if self.interactive || self.full_screen {
            Some(RenderMode::FullScreen)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ListFormatArg {
    Default,
    Quickfix,
}

impl ListFormatArg {
    fn into_task_list_format(self) -> TaskListFormat {
        match self {
            Self::Default => TaskListFormat::Default,
            Self::Quickfix => TaskListFormat::Quickfix,
        }
    }
}

#[derive(Debug, Args)]
struct IdOptions {
    /// Task ID to update.
    id: String,
    /// Task file to update.
    #[arg(short, long, default_value = DEFAULT_TASK_FILE)]
    file: PathBuf,
}

fn scan(options: ScanOptions) -> Result<(), String> {
    let config = load_config(&options.config_path())
        .map_err(|err| format!("failed to read {}: {err}", options.config_path().display()))?;
    let existing = read_tasks(&options.output)
        .map_err(|err| format!("failed to read {}: {err}", options.output.display()))?;
    let scanned = scan_dir(&options.root, &options.output, &config)
        .map_err(|err| format!("failed to scan {}: {err}", options.root.display()))?;
    let merged = merge_tasks(scanned, existing);

    write_tasks(&options.output, &merged)
        .map_err(|err| format!("failed to write {}: {err}", options.output.display()))?;
    println!(
        "wrote {} task{} to {}",
        merged.len(),
        if merged.len() == 1 { "" } else { "s" },
        options.output.display()
    );

    Ok(())
}

fn run_default(no_scan: bool) -> Result<(), String> {
    if !no_scan {
        scan(ScanOptions {
            root: PathBuf::from("."),
            output: PathBuf::from(DEFAULT_TASK_FILE),
            config: None,
        })?;
    }

    list(ListOptions {
        input: None,
        file: None,
        open: true,
        interactive: true,
        inline: false,
        full_screen: false,
        quickfix: false,
        format: None,
    })
}

fn list(options: ListOptions) -> Result<(), String> {
    let input = options.input_path();
    let format = options.list_format();
    let tasks = TaskQuery::new()
        .open_only(options.open)
        .load(&input)
        .map_err(|err| format!("failed to read {}: {err}", input.display()))?;

    if let Some(mode) = options.render_mode() {
        match run_task_list(&tasks, mode).map_err(|err| format!("failed to run TUI: {err}"))? {
            TaskListCommand::Quit => {}
            TaskListCommand::Print(id) => {
                if let Some(task) = tasks.iter().find(|task| task.id == id) {
                    println!("{}", format_task(task, format));
                }
            }
            TaskListCommand::SetStatus(id, status) => {
                set_task_status(&input, &id, status)?;
                println!("updated {id}");
            }
        }
        return Ok(());
    }

    for line in format_tasks(&tasks, format) {
        println!("{line}");
    }

    Ok(())
}

fn set_status(options: IdOptions, status: TaskStatus) -> Result<(), String> {
    let id = TaskId::new(options.id);
    set_task_status(&options.file, &id, status)?;
    println!("updated {id}");
    Ok(())
}
