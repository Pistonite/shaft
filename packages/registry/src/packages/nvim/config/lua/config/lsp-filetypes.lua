local M = {}

require("lspconfig")
require("mason-lspconfig")

vim.api.nvim_create_autocmd("LspAttach", {
    callback = function(event)
        require("piston.keymaps").setup_lsp(event.buf)
        require("config.inlay-hint").enable(event.buf)
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

-- file type -> server[] registration
-- server should be a key in the SERVERS map
-- and a name of supported servers in nvim-lspconfig,
-- *unless* it's an externally handled server (like jdtls) 
-- in which case it should be an object
local FILE_TYPES = {
    lua = "lua_ls",
    python = "pyright",
    c = "clangd",
    cpp = "clangd",
    typescript = { "eslint", "tsgo" },
    typescriptreact = { "eslint", "tsgo" },
    javascript = "tsgo",
    rust = "rust_analyzer",
    java = {
        start = function()
            require("piston_jdtls").start_current_buf()
        end,
        restart = function()
            require("piston_jdtls").restart()
        end
    }
}

-- server registration
-- config = true will require( config.lsp.<server_name> )
local SERVERS = {
    lua_ls = { config = true },
    pyright = {},
    eslint = {},
    ts_ls = {},
    tsgo = { config = true },
    rust_analyzer = { config = true },
    clangd = {},
}

local STARTED_FILE_TYPES = {}

-- Autocommand to auto-load LSP configs based on filetype
vim.api.nvim_create_autocmd("FileType", {
    callback = function()
        local ft = vim.bo.filetype
        local servers = FILE_TYPES[ft]
        if not servers then return end

        local first_start = not STARTED_FILE_TYPES[ft]
        STARTED_FILE_TYPES[ft] = true

        if type(servers) == "string" then
            servers = { servers }
        elseif servers.start then
            servers.start()
            if first_start then
                M.echo("started language server for "..ft)
            end
            return
        end
        local output = ""
        for _, s in ipairs(servers) do
            if type(s) == "string" then
                local config = SERVERS[s]
                if config then
                    if config.config then
                        require("config.lsp."..s)
                        output = output..", [configured "..s.."]"
                    else
                        output = output..", "..s
                    end
                    vim.lsp.enable(s)
                else
                    output = output..", [server not found: "..s.."]"
                end
            end
        end
        if output ~= "" and first_start then
            M.echo("started lsp servers: "..output:sub(3))
        end
    end
})

function M.restart_lsp(bufnr)
    local ft = vim.bo.filetype
    local servers = FILE_TYPES[ft]
    if not servers then return end

    vim.diagnostic.reset(nil, bufnr)
    STARTED_FILE_TYPES[ft] = nil
    if type(servers) ~= "string" and servers.start then
        if servers.restart then
            servers.restart()
            return
        end
    end
    vim.cmd("lsp stop")
    -- we don't call :edit here since the LSP needs to be stopped first,
    -- otherwise the stale diagnostics will come back and be duplicated
end

function M.echo(msg) vim.api.nvim_echo({{"lsp-filetypes: "..msg}}, false, {}) end

return M
