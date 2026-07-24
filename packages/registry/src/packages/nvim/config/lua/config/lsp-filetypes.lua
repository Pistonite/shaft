local M = {}

require("lspconfig")
require("mason-lspconfig")
vim.api.nvim_create_autocmd("LspAttach", {
    callback = function(event)
        require("piston.keymaps").setup_lsp(event.buf)
        vim.lsp.inlay_hint.enable(true, { bufnr = event.buf })
    end
})

-- Remove if no longer needed
-- https://github.com/neovim/neovim/issues/30985 workaround for LSP error from rust-analyzer
for _, method in ipairs({
    'textDocument/diagnostic',
    'textDocument/semanticTokens/full/delta',
    'textDocument/inlayHint',
    'workspace/diagnostic'
}) do
    local default_diagnostic_handler = vim.lsp.handlers[method]
    vim.lsp.handlers[method] = function(err, result, context, config)
        if err ~= nil then
            if err.code == -32802 then
                return
            end
            if err.code == -32603 then
                return
            end
        end

        return default_diagnostic_handler(err, result, context, config)
    end
end

local SERVERS = {
    lua_ls = { config = true },
    pyright = {},
    eslint = {},
    ts_ls = {},
    tsgo = { config = true },
    rust_analyzer = {},
    clangd = {},
    -- jdtls = {}, -- special handling
}
local FILE_TYPES = {
    lua = "lua_ls",
    python = "pyright",
    c = "clangd",
    cpp = "clangd",
    typescript = { "eslint", "tsgo" },
    typescriptreact = { "eslint", "tsgo" },
    javascript = "tsgo",
    rust = "rust_analyzer",
    -- java = "jdtls" -- special handling
}

local SPECIAL_HANDLER = {
    java = {
        start = function()
            require("piston_jdtls").start_current_buf()
        end,
        restart = function()
            require("piston_jdtls").restart()
        end
    }
}

-- Autocommand to auto-load LSP configs based on filetype
vim.api.nvim_create_autocmd("FileType", {
    callback = function()
        local ft = vim.bo.filetype
        local special = SPECIAL_HANDLER[ft]
        if special then
            local start_fn = special.start
            if start_fn then
                require("config.lsp-setup-once")
                start_fn()
                return
            end
        end
        local servers = FILE_TYPES[ft]
        if not servers then
            return
        end
        if type(servers) == "string" then
            servers = { servers }
        end
        for _, s in ipairs(servers) do
            local config = SERVERS[s]
            if config then
                if config.config then
                    require("config.lsp."..s)
                end
                require("config.lsp-setup-once")
                vim.lsp.enable(s)
            end
        end
    end
})

function M.restart_lsp()
    local ft = vim.bo.filetype
    local special = SPECIAL_HANDLER[ft]
    if special then
        local restart_fn = special.restart
        if restart_fn then
            restart_fn()
            return
        end
    end
    vim.cmd("lsp restart")
end

return M
