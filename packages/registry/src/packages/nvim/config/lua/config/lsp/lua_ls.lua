local lsp_capabilities = require('cmp_nvim_lsp').default_capabilities()
vim.lsp.config['lua_ls'] ={
    capabilities = lsp_capabilities,
    settings = {
        Lua = {
            runtime = { version = "LuaJIT" },
            diagnostic = { globals = { 'vim' } }, -- make diagnostic work when configuring nvim
            workspace = {
                library = { vim.env.VIMRUNTIME },
            }
        }
    }
}
