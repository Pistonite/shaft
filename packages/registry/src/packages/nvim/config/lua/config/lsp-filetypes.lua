local M = {}

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
local warn = function(msg)
    vim.api.nvim_echo({{"lsp-filetypes: "..msg}}, false, {})
end

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
        FILE_TYPES[ft] = nil -- remove the config for the file type that we already enabled
        if type(servers) == "string" then
            servers = { servers }
        end
        local enabled = {}
        for _, s in ipairs(servers) do
            local config = SERVERS[s]
            if config then
                SERVERS[s] = nil    -- remove the config for the server that we enabled
                if config.config then
                    require("config.lsp."..s)
                end
                require("config.lsp-setup-once")
                vim.lsp.enable(s)
                table.insert(enabled, s)
            end
        end
        warn("enabled "..vim.inspect(enabled).." for file type '"..ft.."'")
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
