if vim.g.loaded_todolog_nvim == 1 then
  return
end
vim.g.loaded_todolog_nvim = 1

require("todolog").setup()
