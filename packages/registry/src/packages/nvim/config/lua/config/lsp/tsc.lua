vim.lsp.config['tsc'] ={
    settings = {
        typescript = {
            inlayHints = {
                parameterNames = {
                    enabled = 'literals',
                    suppressWhenArgumentMatchesName = true,
                },
                parameterTypes = { enabled = true },
                variableTypes = { enabled = true },
                propertyDeclarationTypes = { enabled = true },
                functionLikeReturnTypes = { enabled = true },
                enumMemberValues = { enabled = true },
            },
        },
    },
}
