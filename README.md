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

`todolog list --quickfix` prints tasks as `file:line:column: message`, which can
be loaded into Vim or Neovim's quickfix list.

Vimscript:

```vim
command! TodologScan silent !todolog scan .
command! TodologTasks cexpr system('todolog list --open --quickfix') | copen
```

Neovim Lua:

```lua
vim.api.nvim_create_user_command("TodologScan", function()
  vim.fn.jobstart({ "todolog", "scan", vim.fn.getcwd() }, { detach = true })
end, {})

vim.api.nvim_create_user_command("TodologTasks", function()
  vim.fn.setqflist({}, "r", {
    title = "todolog",
    lines = vim.fn.systemlist({ "todolog", "list", "--open", "--quickfix" }),
    efm = "%f:%l:%c:%m",
  })
  vim.cmd.copen()
end, {})

vim.keymap.set("n", "<leader>ts", "<cmd>TodologScan<cr>", {
  desc = "todolog scan project",
})

vim.keymap.set("n", "<leader>tq", "<cmd>TodologTasks<cr>", {
  desc = "todolog open quickfix tasks",
})
```

## Emacs

Emacs does not need a dedicated package to run `todolog`. The useful built-in
options are `compile`, `shell-command`, and `start-process`.

Use `compile` when you want a visible buffer with clickable `file:line` output:

```elisp
(defvar todolog-command "todolog")

(defun todolog-project-root ()
  (if-let ((project (project-current)))
      (project-root project)
    default-directory))

(defun todolog-list-open ()
  (interactive)
  (let ((default-directory (todolog-project-root)))
    (compile (format "%s list --open" todolog-command))))
```

Use `shell-command` for simple synchronous commands:

```elisp
(defun todolog-scan ()
  (interactive)
  (let ((default-directory (todolog-project-root)))
    (shell-command (format "%s scan ." todolog-command))))

(defun todolog-done (id)
  (interactive "sTask ID: ")
  (let ((default-directory (todolog-project-root)))
    (shell-command (format "%s done %s" todolog-command id))))

(defun todolog-open (id)
  (interactive "sTask ID: ")
  (let ((default-directory (todolog-project-root)))
    (shell-command (format "%s open %s" todolog-command id))))
```

Use `start-process` for async background execution:

```elisp
(defun todolog-scan-async ()
  (interactive)
  (let ((default-directory (todolog-project-root)))
    (start-process "todolog-scan" "*todolog*" todolog-command "scan" ".")))
```

Example key bindings:

```elisp
(global-set-key (kbd "C-c t s") #'todolog-scan)
(global-set-key (kbd "C-c t l") #'todolog-list-open)
(global-set-key (kbd "C-c t d") #'todolog-done)
(global-set-key (kbd "C-c t o") #'todolog-open)
```
