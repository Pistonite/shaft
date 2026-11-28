local M = {}

require("lspconfig")
require("mason-lspconfig")

local mason_registry = require("mason-registry")

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
    python = "basedpyright",
    c = "clangd",
    cpp = "clangd",
    typescript = { "eslint", "oxlint", "tsc" },
    typescriptreact = { "eslint", "oxlint", "tsc" },
    javascript = "tsc",
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


-- Search the file's ancestor directories for any of `names`, stopping at
-- (and including) the project root. Returns true if one is found.
local function has_config_upward(bufnr, file_path, markers, names)
    if file_path == "" then return false end
    local start = vim.fs.dirname(file_path)
    local root = vim.fs.root(bufnr, markers) or start
    local found = vim.fs.find(names, {
        path = start,
        upward = true,
        -- `stop` itself is not searched, so stop at root's parent to include root
        stop = vim.fs.dirname(root),
        limit = 1,
    })
    return #found > 0
end


-- server registration
-- config = true will require( config.lsp.<server_name> )
-- mason_package is the name of the mason package if different from the server name
local SERVERS = {
    lua_ls = {
        config = true,
        mason_package = "lua-language-server"
    },
    basedpyright = {},
    eslint = {
        mason_package = "eslint-lsp",
        should_enable = function(bufnr, file_path)
            local ESLINT_CONFIGS = {
                "eslint.config.js", "eslint.config.mjs", "eslint.config.cjs",
                "eslint.config.ts", "eslint.config.mts", "eslint.config.cts",
                ".eslintrc", ".eslintrc.js", ".eslintrc.cjs",
                ".eslintrc.yaml", ".eslintrc.yml", ".eslintrc.json",
            }
            return has_config_upward(bufnr, file_path, {".git", "package.json"}, ESLINT_CONFIGS)
        end
    },
    oxlint = {
        should_enable = function(bufnr, file_path)
            local OXLINT_CONFIGS = { ".oxlintrc.json", "oxlint.config.mts", "oxlint.config.ts" }
            return has_config_upward(bufnr, file_path, {".git", "package.json"}, OXLINT_CONFIGS)
        end
    },
    tsc = {},
    rust_analyzer = {
        config = true,
        mason_package = "rust-analyzer"
    },
    clangd = {},
}


local STARTED_FILE_TYPES = {}
local ENABLED_SERVERS = {}

-- Autocommand to auto-load LSP configs based on filetype
vim.api.nvim_create_autocmd("FileType", {
    callback = function(args)
        local time_start = vim.uv.hrtime()

        local bufnr = args.buf
        local file_path = vim.api.nvim_buf_get_name(bufnr)
        if file_path == "" then return end

        local ft = args.match
        local servers = FILE_TYPES[ft]
        if not servers then return end

        local first_start = not STARTED_FILE_TYPES[ft]
        STARTED_FILE_TYPES[ft] = true

        if type(servers) == "string" then
            servers = { servers }
        elseif servers.start then
            servers.start()
            if first_start then
                local elapsed_ms = (vim.uv.hrtime() - time_start) / 1e6
                M.echo(string.format("started language server for %s (%.2fms)", ft, elapsed_ms))
            end
            return
        end

        local output = ""
        for _, s in ipairs(servers) do
            if type(s) == "string" then
                local config = SERVERS[s]
                if config then
                    local mason_package = config.mason_package or s;
                    if mason_registry.is_installed(mason_package) then
                        local should_enable = true
                        if config.should_enable then
                            should_enable = config.should_enable(bufnr, file_path)
                        end
                        if should_enable then
                            if config.config then
                                require("config.lsp."..s)
                                output = output..", [configured "..s.."]"
                            else
                                output = output..", "..s
                            end
                            if ENABLED_SERVERS[s] ~= true then
                                -- local config_time_start = vim.uv.hrtime()
                                vim.lsp.enable(s)
                                -- local elapsed_ms = (vim.uv.hrtime() - config_time_start) / 1e6
                                -- output = output..string.format("--enale took %.2fms--", elapsed_ms)
                                ENABLED_SERVERS[s] = true
                            end
                        else
                            output = output..", [skipped: "..mason_package.."]"
                        end
                    else
                        output = output..", [package not installed: "..mason_package.."]"
                    end
                else
                    output = output..", [server not found: "..s.."]"
                end
            end
        end
        if output ~= "" and first_start then
            local elapsed_ms = (vim.uv.hrtime() - time_start) / 1e6
            M.echo(string.format("started lsp servers: %s (%.2fms)", output:sub(3), elapsed_ms))
        end
    end
})

function M.restart_lsp(bufnr)
    local ft = vim.bo.filetype
    local servers = FILE_TYPES[ft]
    if not servers then return end

    ENABLED_SERVERS={}

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
