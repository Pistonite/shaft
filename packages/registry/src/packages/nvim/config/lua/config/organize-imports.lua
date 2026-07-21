local HANDLERS = {}

HANDLERS.java = function()
    require("jdtls").organize_imports()
end

return function()
    local ft = vim.bo.filetype
    local handler_fn = HANDLERS[ft]
    if not handler_fn then
        require("piston.editorapi").warn("organize import not supported for file type: "..ft)
    end
    handler_fn()
end
