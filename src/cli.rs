use std::path::PathBuf;

use crate::config::load_config;
use crate::constants::{DEFAULT_CONFIG_FILE, DEFAULT_TASK_FILE};
use crate::domain::{TaskId, TaskStatus};
use crate::scanner::scan_dir;
use crate::store::{merge_tasks, read_tasks, update_status, write_tasks};

pub(crate) fn run(args: Vec<String>) -> Result<(), String> {
    let command = args.get(1).map(String::as_str).unwrap_or("help");

    match command {
        "scan" => {
            let options = ScanOptions::parse(&args[2..])?;
            let config = load_config(&options.config_path()).map_err(|err| {
                format!("failed to read {}: {err}", options.config_path().display())
            })?;
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
        "list" => {
            let options = ListOptions::parse(&args[2..])?;
            let tasks = read_tasks(&options.input)
                .map_err(|err| format!("failed to read {}: {err}", options.input.display()))?;
            for task in tasks {
                if options.open_only && task.status == TaskStatus::Done {
                    continue;
                }
                println!(
                    "{} [{}] {}:{} {}",
                    task.id,
                    task.status.checkbox(),
                    task.file,
                    task.line,
                    task.text
                );
            }
            Ok(())
        }
        "done" => {
            let options = IdOptions::parse("done", &args[2..])?;
            update_status(&options.file, &options.id, TaskStatus::Done)?;
            println!("updated {}", options.id);
            Ok(())
        }
        "open" => {
            let options = IdOptions::parse("open", &args[2..])?;
            update_status(&options.file, &options.id, TaskStatus::Open)?;
            println!("updated {}", options.id);
            Ok(())
        }
        "help" | "-h" | "--help" => {
            print_help();
            Ok(())
        }
        unknown => Err(format!("unknown command `{unknown}`; try `todolog help`")),
    }
}

#[derive(Debug)]
struct ScanOptions {
    root: PathBuf,
    output: PathBuf,
    config: Option<PathBuf>,
}

impl ScanOptions {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut root = PathBuf::from(".");
        let mut output = PathBuf::from(DEFAULT_TASK_FILE);
        let mut config = None;
        let mut index = 0;

        while index < args.len() {
            match args[index].as_str() {
                "-o" | "--output" => {
                    index += 1;
                    output = args
                        .get(index)
                        .map(PathBuf::from)
                        .ok_or("expected a path after --output")?;
                }
                "-c" | "--config" => {
                    index += 1;
                    config = Some(
                        args.get(index)
                            .map(PathBuf::from)
                            .ok_or("expected a path after --config")?,
                    );
                }
                value if value.starts_with('-') => {
                    return Err(format!("unknown option `{value}` for scan"));
                }
                value => root = PathBuf::from(value),
            }
            index += 1;
        }

        Ok(Self {
            root,
            output,
            config,
        })
    }

    fn config_path(&self) -> PathBuf {
        self.config
            .clone()
            .unwrap_or_else(|| self.root.join(DEFAULT_CONFIG_FILE))
    }
}

#[derive(Debug)]
struct ListOptions {
    input: PathBuf,
    open_only: bool,
}

impl ListOptions {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut input = PathBuf::from(DEFAULT_TASK_FILE);
        let mut open_only = false;
        let mut index = 0;

        while index < args.len() {
            match args[index].as_str() {
                "-f" | "--file" => {
                    index += 1;
                    input = args
                        .get(index)
                        .map(PathBuf::from)
                        .ok_or("expected a path after --file")?;
                }
                "--open" => open_only = true,
                value if value.starts_with('-') => {
                    return Err(format!("unknown option `{value}` for list"));
                }
                value => input = PathBuf::from(value),
            }
            index += 1;
        }

        Ok(Self { input, open_only })
    }
}

#[derive(Debug)]
struct IdOptions {
    file: PathBuf,
    id: TaskId,
}

impl IdOptions {
    fn parse(command: &str, args: &[String]) -> Result<Self, String> {
        let mut file = PathBuf::from(DEFAULT_TASK_FILE);
        let mut id = None;
        let mut index = 0;

        while index < args.len() {
            match args[index].as_str() {
                "-f" | "--file" => {
                    index += 1;
                    file = args
                        .get(index)
                        .map(PathBuf::from)
                        .ok_or("expected a path after --file")?;
                }
                value if value.starts_with('-') => {
                    return Err(format!("unknown option `{value}` for {command}"));
                }
                value => id = Some(value.to_string()),
            }
            index += 1;
        }

        Ok(Self {
            file,
            id: id
                .map(TaskId::new)
                .ok_or_else(|| format!("usage: todolog {command} <TASK-ID>"))?,
        })
    }
}

fn print_help() {
    println!(
        "\
todolog tracks TODO comments in a plain Markdown file.

Usage:
  todolog scan [ROOT] [-o TASKS.md] [-c .todolog]
  todolog list [TASKS.md] [--open]
  todolog done <TASK-ID> [-f TASKS.md]
  todolog open <TASK-ID> [-f TASKS.md]

Examples:
  todolog scan .
  todolog list --open
  todolog done 20260811-141530
"
    );
}
