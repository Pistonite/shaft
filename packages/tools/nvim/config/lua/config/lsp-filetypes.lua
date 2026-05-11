local SERVERS = {
    lua_ls = { config = true },
    pyright = {},
    eslint = {},
    ts_ls = {},
    tsgo = { config = true },
    rust_analyzer = {},
}
local FILE_TYPES = {
    lua = "lua_ls",
    python = "pyright",
    typescript = { "eslint", "tsgo" },
    typescriptreact = { "eslint", "tsgo" },
    javascript = "tsgo",
    rust = "rust_analyzer",
}
local warn = function(msg) vim.notify("lsp_filetypes: "..msg, vim.log.levels.INFO) end
-- Autocommand to auto-load LSP configs based on filetype
vim.api.nvim_create_autocmd("FileType", {
    callback = function()
        local ft = vim.bo.filetype
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
                require("config.lsp").enable(s)
                table.insert(enabled, s)
            end
        end
        warn("enabled "..vim.inspect(enabled).." for file type '"..ft.."'")
    end
})
