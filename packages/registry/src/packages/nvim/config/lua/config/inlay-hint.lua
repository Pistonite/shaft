local M = {}

-- this is taken from https://github.com/felpafel/inlay-hint.nvim
-- and modified to my liking
local display_callback = function(line_hints, options, bufnr)
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
end

require('inlay-hint').setup({
    virt_text_pos = 'eol',
    display_callback = display_callback
})


local uv = vim.uv or vim.loop

local POLL_INTERVAL = 1000
local POLL_TIMEOUT = 30000

-- one poll timer per buffer
local timers = {}

local function stop_poll(bufnr)
    local timer = timers[bufnr]
    if not timer then
        return
    end
    timers[bufnr] = nil
    if not timer:is_closing() then
        timer:stop()
        timer:close()
    end
end

function M.enable(bufnr)
    if bufnr == nil or bufnr == 0 then
        bufnr = vim.api.nvim_get_current_buf()
    end
    vim.lsp.inlay_hint.enable(true, { bufnr = bufnr })

    -- servers sometimes attach/index too late to answer the first request,
    -- so keep re-asking every second until hints show up (or we give up)
    stop_poll(bufnr)
    local elapsed = 0
    local timer = uv.new_timer()
    if not timer then return end
    timers[bufnr] = timer

    timer:start(POLL_INTERVAL, POLL_INTERVAL, vim.schedule_wrap(function()
        elapsed = elapsed + POLL_INTERVAL
        if not vim.api.nvim_buf_is_valid(bufnr) then
            stop_poll(bufnr)
            return
        end
        if #vim.lsp.inlay_hint.get({ bufnr = bufnr }) > 0 then
            stop_poll(bufnr)
            local secs = math.floor(elapsed / 1000)
            if secs > 0 then
                M.echo("took " .. secs.." seconds")
            end
            return
        end
        if elapsed >= POLL_TIMEOUT then
            stop_poll(bufnr)
            -- will let it refresh as normal
            return
        end
        vim.lsp.inlay_hint.enable(false, { bufnr = bufnr })
        vim.defer_fn(function()
            vim.lsp.inlay_hint.enable(true, { bufnr = bufnr })
        end, 100)
    end))
end

function M.echo(msg) vim.api.nvim_echo({{"inlay-hint: "..msg}}, false, {}) end

return M
