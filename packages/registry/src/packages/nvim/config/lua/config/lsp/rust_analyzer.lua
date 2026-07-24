vim.lsp.config["rust_analyzer"] = {
    settings = {
        ["rust-analyzer"] = {
            cachePriming = {
                enable = true -- speeds up slightly (like from 6s to 4s)
            },
            files = {
                exclude = {
                    "node_modules" -- doesn't seem to work :<
                }
            },
        }
    }
}
