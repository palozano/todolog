local M = {}

local config = {
  command = "todolog",
  quickfix_position = "copen",
  keymaps = false,
}

local function notify(message, level)
  vim.notify(message, level or vim.log.levels.INFO, { title = "todolog" })
end

local function map_quickfix_actions()
  if vim.bo.filetype ~= "qf" then
    return
  end

  vim.keymap.set("n", "d", M.done_current, {
    buffer = true,
    desc = "todolog mark selected task done",
  })
  vim.keymap.set("n", "r", M.open_current, {
    buffer = true,
    desc = "todolog reopen selected task",
  })
end

local function refresh_quickfix()
  local output = vim.fn.systemlist({ config.command, "list", "--open", "--quickfix" })
  if vim.v.shell_error ~= 0 then
    notify(table.concat(output, "\n"), vim.log.levels.ERROR)
    return
  end

  vim.fn.setqflist({}, "r", {
    title = "todolog",
    lines = output,
    efm = "%f:%l:%c:%m",
  })
  vim.cmd(config.quickfix_position)
  map_quickfix_actions()
end

local function current_quickfix_task_id()
  local qflist = vim.fn.getqflist({ items = 0 })
  local index = vim.bo.filetype == "qf" and vim.fn.line(".") or vim.fn.getqflist({ idx = 0 }).idx
  local item = qflist.items[index]
  if not item then
    notify("No todolog quickfix item selected", vim.log.levels.WARN)
    return nil
  end

  local id = item.text:match("^%[([^%]]+)%]")
  if not id then
    notify("Selected quickfix item does not contain a todolog ID", vim.log.levels.WARN)
    return nil
  end

  return id
end

local function run_and_refresh(args)
  vim.system(vim.list_extend({ config.command }, args), { text = true }, function(result)
    vim.schedule(function()
      if result.code ~= 0 then
        notify(result.stderr ~= "" and result.stderr or result.stdout, vim.log.levels.ERROR)
        return
      end
      refresh_quickfix()
    end)
  end)
end

function M.scan(root)
  vim.fn.jobstart({ config.command, "scan", root or vim.fn.getcwd() }, { detach = true })
end

function M.tasks()
  refresh_quickfix()
end

function M.done(id)
  run_and_refresh({ "done", id })
end

function M.done_current()
  local id = current_quickfix_task_id()
  if id then
    M.done(id)
  end
end

function M.open(id)
  run_and_refresh({ "open", id })
end

function M.open_current()
  local id = current_quickfix_task_id()
  if id then
    M.open(id)
  end
end

function M.setup(opts)
  config = vim.tbl_deep_extend("force", config, opts or {})

  vim.api.nvim_create_user_command("TodologScan", function(opts_)
    M.scan(opts_.args ~= "" and opts_.args or nil)
  end, { nargs = "?", complete = "dir" })

  vim.api.nvim_create_user_command("TodologTasks", function()
    M.tasks()
  end, {})

  vim.api.nvim_create_user_command("TodologDone", function(opts_)
    M.done(opts_.args)
  end, { nargs = 1 })

  vim.api.nvim_create_user_command("TodologDoneCurrent", function()
    M.done_current()
  end, {})

  vim.api.nvim_create_user_command("TodologOpen", function(opts_)
    M.open(opts_.args)
  end, { nargs = 1 })

  vim.api.nvim_create_user_command("TodologOpenCurrent", function()
    M.open_current()
  end, {})

  if config.keymaps then
    vim.keymap.set("n", "<leader>Ts", M.scan, { desc = "todolog scan project" })
    vim.keymap.set("n", "<leader>Tq", M.tasks, { desc = "todolog open quickfix tasks" })
    vim.keymap.set("n", "<leader>Td", function()
      vim.ui.input({ prompt = "todolog done ID: " }, function(id)
        if id and id ~= "" then
          M.done(id)
        end
      end)
    end, { desc = "todolog mark task done" })
    vim.keymap.set("n", "<leader>Tr", function()
      vim.ui.input({ prompt = "todolog open ID: " }, function(id)
        if id and id ~= "" then
          M.open(id)
        end
      end)
    end, { desc = "todolog reopen task" })
  end
end

return M
