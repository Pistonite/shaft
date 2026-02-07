
local cache_dir = vim.fn.stdpath('config')..'/lua/piston/cache'
local cache_lua = cache_dir..'/tree-sitter-installed.lua'

local cached_fts = {}
if vim.uv.fs_stat(cache_lua)~=nil then
    cached_fts = require("piston.cache.tree-sitter-installed")
end
local install_dir = vim.fn.stdpath('data')..'/piston/tree-sitter-install'
require('nvim-treesitter').setup({
    install_dir = install_dir,
})

local enable_treesitter_for_curr_buf = function()
    vim.treesitter.start()
    vim.bo.indentexpr = "v:lua.require'nvim-treesitter'.indentexpr()"
    vim.wo[0][0].foldmethod = 'expr'
    vim.wo[0][0].foldexpr = 'v:lua.vim.treesitter.foldexpr()'
end

local create_treesitter_autocmd = function(fts)
    if #fts == 0 then return end
    vim.api.nvim_create_autocmd('FileType', {
        pattern = fts,
        callback = enable_treesitter_for_curr_buf,
    })
end

local reload_cache = function()
    local parser_dir = install_dir..'/parser'
    local new_fts = {}

    local handle = vim.uv.fs_scandir(parser_dir)
    if handle then
        while true do
            local name, type = vim.uv.fs_scandir_next(handle)
            if not name then break end
            if type == 'file' then
                local lang = name:match('^(.+)%.dll$') or name:match('^(.+)%.so$')
                if lang then
                    local ok, filetypes = pcall(vim.treesitter.language.get_filetypes, lang)
                    if ok and filetypes then
                        vim.list_extend(new_fts, filetypes)
                    end
                end
            end
        end
    end

    table.sort(new_fts)
    new_fts = vim.fn.uniq(new_fts)

    vim.fn.mkdir(cache_dir, 'p')
    local file = io.open(cache_lua, 'w')
    if file then
        file:write('return {\n')
        for _, ft in ipairs(new_fts) do
            file:write(string.format('    %q,\n', ft))
        end
        file:write('}\n')
        file:close()
    end

    local cached_set = {}
    for _, ft in ipairs(cached_fts) do
        cached_set[ft] = true
    end
    local added_fts = vim.tbl_filter(function(ft)
        return not cached_set[ft]
    end, new_fts)
    create_treesitter_autocmd(added_fts)
    if #added_fts ~= 0 then
        print("now enabled treesitter for "..vim.inspect(added_fts))
    end

    if vim.list_contains(new_fts, vim.bo.filetype) then
        enable_treesitter_for_curr_buf()
    end
end

create_treesitter_autocmd(cached_fts)
vim.api.nvim_create_user_command('TSReload', function(_)
    reload_cache()
end, {
        desc = 'Reload treesitter cached filetypes'
    })
