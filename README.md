# todolog

`todolog` is a small Rust CLI for turning TODO-style comments in code into a
plain Markdown task file.

It scans source files for `TODO`, `FIXME`, `XXX`, and `HACK`, assigns each item an
ID like `20260811-141530`, and writes tasks grouped by filename:

```markdown
# Code Tasks

## src/main.rs

- [ ] `20260811-141530` L42 - wire editor command <!-- todolog: marker="TODO" fingerprint="..." -->
```

The HTML comment keeps the file easy to read while giving the CLI enough
metadata to preserve IDs and done/open status across future scans.

## Usage

```sh
cargo run -- scan .
cargo run -- list --open
cargo run -- done 20260811-141530
cargo run -- open 20260811-141530
```

After installing the binary somewhere on your `PATH`, drop `cargo run --`:

```sh
todolog scan .
todolog list --open
todolog done 20260811-141530
```

## Config

`todolog scan` loads an optional `.todolog` file from the scan root. The current
config supports choosing the ID strategy for newly discovered tasks:

```text
id = timestamp
```

Supported values:

- `timestamp`: wall-clock IDs like `20260811-141530`
- `uid`: short deterministic hex IDs like `T-0123456789ab`
- `uuid`: deterministic UUID-shaped IDs

You can also pass a config file explicitly:

```sh
todolog scan . --config path/to/todolog.conf
```

## Neovim

```lua
vim.api.nvim_create_user_command("TodologScan", function()
  vim.fn.jobstart({ "todolog", "scan", vim.fn.getcwd() }, { detach = true })
end, {})
```

## Emacs

```elisp
(defun todolog-scan ()
  (interactive)
  (compile "todolog scan ."))
```

