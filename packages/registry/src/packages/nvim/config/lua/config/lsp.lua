local M = {}

vim.api.nvim_create_autocmd("LspAttach", {
    callback = function(event)
        require("piston.keymaps").setup_lsp(event.buf)
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

-- ensure dependencies are loaded
require("lspconfig")
require("mason-lspconfig")

function M.enable(lspserver)
    vim.lsp.enable(lspserver)
end

return M
