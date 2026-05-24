local cmp = require('cmp')
-- ensure sources are loaded
require("cmp_nvim_lsp")
require("cmp_path")
require("cmp_buffer")
require("cmp_nvim_lsp_signature_help")
-- cmp_nvim_lua is not loaded because ft=lua will already load it
cmp.setup({
    -- completion menu kep mapping
    mapping = require("piston.keymaps").get_cmp_mappings(),

    -- Installed sources:
    sources = {
        { name = 'buffer',                 keyword_length = 2 }, -- source current buffer
        { name = 'path' },                     -- file paths
        { name = 'nvim_lsp',               keyword_length = 2 }, -- from language server
        { name = 'nvim_lsp_signature_help' },  -- display function signatures with current parameter emphasized
        { name = 'nvim_lua',               keyword_length = 2 }, -- complete neovim's Lua runtime API such vim.lsp.*
    },
    window = {
        completion = cmp.config.window.bordered(),
        documentation = cmp.config.window.bordered(),
    },
    formatting = {
        fields = { 'abbr', 'kind' },
    },
})

