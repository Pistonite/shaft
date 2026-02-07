local function on_attach_nvim_tree(bufnr)
    -- setup lualine only after nvim tree attach
    -- to avoid loading it too early onto the tree
    -- only attack keys i need
    require("config.lualine")
    require("piston.keymaps").setup_nvim_tree(bufnr)
end
local config_gen = require("piston.config_gen")
require("nvim-tree").setup({
    on_attach = on_attach_nvim_tree,
    git = {
        enable = config_gen.git
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
        }
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
    }
})
