local function on_attach_nvim_tree(bufnr)
    -- setup lualine only after nvim tree attach
    -- to avoid loading it too early onto the tree
    -- only attack keys i need
    require("config.lualine")
    require("piston.keymaps").setup_nvim_tree(bufnr)
end
local config_gen = require("piston.config_gen")
local config = {
    on_attach = on_attach_nvim_tree,
    git = {
        enable = config_gen.nvim_tree_git
    },
    renderer = {
        icons = {
            glyphs = {
                bookmark = "-",
                git = {
                    unstaged = "M",
                    staged = "✓",
                    unmerged = "",
                    renamed = "R",
                    untracked = "U",
                    deleted = "D",
                    ignored = "i",
                },
            }
        },
        group_empty = true
    },
    diagnostics = {
        enable = true,
        show_on_dirs = true,
        icons = {
            hint = "H",
            info = "I",
            warning = "W",
            error = "E",
        }
    },
    filesystem_watchers = {}
}
if vim.fn.has("win32") ~= 0 then
    -- see https://github.com/nvim-tree/nvim-tree.lua/issues/3292
    -- and documentation (nvim-tree-os-specific)
    -- when filesystem watching is disabled, manually refreshing the tree is more often needed
    -- but this is much better than run-away memory leak (which happens every time when you rm -rf in PowerShell)
    -- we wrap nvim tree APIs with a refresh call in the keymap so just doing operations
    -- inside nvim-tree is mostly unaffected
    config.filesystem_watchers.whitelist_dirs = function()
        return false
    end
end
require("nvim-tree").setup(config)
