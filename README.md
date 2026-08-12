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

Install with Homebrew:

```sh
brew install palozano/tap/todolog
```

Install with Cargo:

```sh
cargo install todolog
```

Or run from a checkout:

```sh
cargo run --
cargo run -- --no-scan
cargo run -- scan .
cargo run -- list --open
cargo run -- list --open --emacs
cargo run -- list --open --interactive
cargo run -- done 20260811-141530
cargo run -- open 20260811-141530
```

After installing the binary somewhere on your `PATH`, drop `cargo run --`:

```sh
todolog
todolog --no-scan
todolog scan .
todolog list --open
todolog list --open --emacs
todolog list --open --interactive
todolog done 20260811-141530
```

## Interactive task list

Running `todolog` scans the current directory and then opens a full-screen
terminal UI with open tasks. Use `todolog --no-scan` to skip the scan and open
the last written task file. `todolog list --interactive` opens the same UI
without scanning first, and `--inline` renders below the current prompt instead.

Controls:

- `Up`/`Down` or `k`/`j`: move through tasks
- `Enter`: open the selected task in `$EDITOR` at its line
- `d`: mark the selected task done
- `o`: reopen the selected task
- `q` or `Esc`: close the UI

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

## Editor integrations

`todolog list --quickfix` prints tasks as `file:line:column: message`, which can
be loaded into Vim or Neovim's quickfix list.

The repository ships separate Vim and Neovim packages under `editors/`.

### Vimscript

The Vim package lives in `editors/vim/todolog.vim` and is written in Vimscript.

With Vim's native packages:

```sh
mkdir -p ~/.vim/pack/todolog/start
ln -s /path/to/todolog/editors/vim/todolog.vim ~/.vim/pack/todolog/start/todolog.vim
```

Or with vim-plug:

```lua
Plug '/path/to/todolog/editors/vim/todolog.vim'
```

Commands:

- `:TodologScan [root]`: scan a directory, defaulting to the current working directory
- `:TodologTasks`: load open tasks into the quickfix list
- `:TodologTasks!`: load open tasks and open quickfix at the bottom
- `:TodologDone {id}`: mark a task done and refresh quickfix
- `:TodologOpen {id}`: reopen a task and refresh quickfix

Optional mappings:

```vim
nmap <leader>Ts <Plug>(todolog-scan)
nmap <leader>Tq <Plug>(todolog-tasks)
```

### Neovim Lua

The Neovim package lives in `editors/nvim/todolog.nvim` and is written in Lua.

With Neovim's native packages:

```sh
mkdir -p ~/.local/share/nvim/site/pack/todolog/start
ln -s /path/to/todolog/editors/nvim/todolog.nvim ~/.local/share/nvim/site/pack/todolog/start/todolog.nvim
```

With lazy.nvim:

```lua
{
  dir = "/path/to/todolog/editors/nvim/todolog.nvim",
  opts = {
    keymaps = true,
  },
}
```

With packer.nvim:

```lua
use({
  "/path/to/todolog/editors/nvim/todolog.nvim",
  config = function()
    require("todolog").setup({ keymaps = true })
  end,
})
```

Commands:

- `:TodologScan [root]`: scan a directory, defaulting to the current working directory
- `:TodologTasks`: load open tasks into the quickfix list
- `:TodologDone {id}`: mark a task done and refresh quickfix
- `:TodologOpen {id}`: reopen a task and refresh quickfix

When `keymaps = true`, the package adds:

```lua
vim.keymap.set("n", "<leader>Ts", "<cmd>TodologScan<cr>", {
  desc = "todolog scan project",
})

vim.keymap.set("n", "<leader>Tq", "<cmd>TodologTasks<cr>", {
  desc = "todolog open quickfix tasks",
})

vim.keymap.set("n", "<leader>Td", function()
  vim.ui.input({ prompt = "todolog done ID: " }, function(id)
    if id and id ~= "" then
      vim.cmd.TodologDone(id)
    end
  end)
end, { desc = "todolog mark task done" })

vim.keymap.set("n", "<leader>Tr", function()
  vim.ui.input({ prompt = "todolog open ID: " }, function(id)
    if id and id ~= "" then
      vim.cmd.TodologOpen(id)
    end
  end)
end, { desc = "todolog reopen task" })
```

## Emacs

The repository includes `todolog.el`, an Emacs package that provides a dedicated
`tabulated-list-mode` task buffer. It shells out to the `todolog` binary, so make
sure the CLI is installed and visible on Emacs' `exec-path`.

For local testing, point `load-path` at this checkout:

```elisp
(use-package todolog
  :load-path "/path/to/todolog"
  :commands (todolog-list-open todolog-scan todolog-done todolog-open)
  :bind (("C-c T l" . todolog-list-open)
         ("C-c T s" . todolog-scan)
         ("C-c T d" . todolog-done)
         ("C-c T o" . todolog-open)))
```

Or without `use-package`:

```elisp
(add-to-list 'load-path "/path/to/todolog")
(require 'todolog)

(keymap-global-set "C-c T l" #'todolog-list-open)
(keymap-global-set "C-c T s" #'todolog-scan)
(keymap-global-set "C-c T d" #'todolog-done)
(keymap-global-set "C-c T o" #'todolog-open)
```

Inside the `*todolog tasks*` buffer:

- `RET`: open the task in the current window
- `o`: open the task in another window
- `v`: preview the task in a bottom window
- `g`: refresh
- `s`: scan the current project
- `d`: mark the task done
- `r`: reopen the task
- `q`: quit

To package it through MELPA, add a recipe like this to MELPA's `recipes/`
directory once the repository URL is public:

```elisp
(todolog :fetcher github :repo "palozano/todolog")
```
