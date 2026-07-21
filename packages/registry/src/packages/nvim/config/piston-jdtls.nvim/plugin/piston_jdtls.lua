if vim.g.loaded_piston_jdtls then
  return
end
vim.g.loaded_piston_jdtls = 1
local main_module_ok, _ = pcall(require, "piston_jdtls")
if not main_module_ok then
  vim.notify("piston-jdtls: Failed to load main module.", vim.log.levels.ERROR)
end
