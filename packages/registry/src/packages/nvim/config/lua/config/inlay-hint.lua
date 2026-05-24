require('inlay-hint').setup({
    virt_text_pos = 'eol',
    -- this is taken from https://github.com/felpafel/inlay-hint.nvim
    -- and modified to my liking
    display_callback = function(line_hints, options, bufnr)
        local param_hints = {}
        local type_hints = {}
        table.sort(line_hints, function(a, b)
            return a.position.character < b.position.character
        end)
        for _, hint in pairs(line_hints) do
            local label = hint.label
            local kind = hint.kind
            local text = ''
            if type(label) == 'string' then
                text = label
            else
                for _, part in ipairs(label) do
                    text = text .. part.value
                end
            end
            if kind == 1 then
                param_hints[#param_hints + 1] = text:gsub('^:%s*', '')
            else
                type_hints[#type_hints + 1] = text:gsub(':$', '')
            end
        end
        local text = ''
        if #type_hints > 0 then
            text = ' (' .. table.concat(type_hints, ',') .. ')'
        end
        if #text > 0 then
            text = text .. ' '
        end
        if #param_hints > 0 then
            text = text ..  table.concat(param_hints, ',')
        end
        return text
    end,
})
