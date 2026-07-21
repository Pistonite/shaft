
local editorapi = require("piston.editorapi")

local M = {}

-- helper for remapping
local function noremap(mode, lhs, rhs, opts)
    local options = { noremap = true }
    if opts then
        options = vim.tbl_extend('force', options, opts)
    end
    vim.keymap.set(mode, lhs, rhs, options)
end

---Setup global keymaps
function M.setup()
    -- toggle relative line number
    noremap('n', '<leader>0', function() vim.o.relativenumber = not vim.o.relativenumber end)
    -- turn off search highlight
    noremap('n', '<leader> ', vim.cmd.nohlsearch)
    -- cursor movement
    -- 15 lines is about where the text moves and I can still see what's going on
    noremap('n', '<C-d>', '15j')     -- bukl move down
    noremap('n', '<C-u>', '15k')     -- bulk move up
    noremap('n', 'n', 'nzz')         -- move to next match and center
    noremap('n', 'N', 'Nzz')         -- move to previous match and center
    -- Move to next '_', uppercase letter, or word boundary
    noremap('n', '<S-l>', function() editorapi.jump_half_word(nil) end)
    noremap('n', 'df<S-l>', function() editorapi.jump_half_word("d", "f") end)
    noremap('n', 'dt<S-l>', function() editorapi.jump_half_word("d" ,"t") end)
    noremap('n', 'cf<S-l>', function() editorapi.jump_half_word("c", "f") end)
    noremap('n', 'ct<S-l>', function() editorapi.jump_half_word("c" ,"t") end)

    -- line movement (note: the : cannot be replaced by <cmd>)
    noremap('v', '<A-j>', [[:m '>+1<cr>gv=gv]]) -- move selection down
    noremap('v', '<A-k>', [[:m '<-2<cr>gv=gv]]) -- move selection up
    -- turn off recording so I don't accidentally hit it
    noremap('n', 'q', '<nop>')
    noremap('n', 'Q', '<nop>')
    -- change window size
    noremap('n', '<C-w>>', '<C-w>20>')
    noremap('n', '<C-w><', '<C-w>20<')
    noremap('n', '<C-w>+', '<C-w>10+')
    noremap('n', '<C-w>-', '<C-w>10-')
    -- copy to system clipboard (see extra.lua)
    noremap('v', '<leader>y', '"ay')
    -- convert between Rust /// doc and JS /** doc */
    noremap('n', '<leader>J', '0f/wBR/**<esc>A */<esc>')
    noremap('v', '<leader>J', '<esc>\'<lt>O<esc>0C/**<esc>\'>o<esc>0C */<esc><cmd>\'<lt>,\'>s/\\/\\/\\// */<cr>gv`<lt>koj=<cmd>nohl<cr>')
    noremap('n', '<leader>R', '0f*wBR///<esc>A<esc>xxx')
    noremap('v', '<leader>R', '<esc>\'<lt>dd\'>ddgv<esc><cmd>\'<lt>,\'>s/\\*/\\/\\/\\//<cr>gv`<lt>koj=<cmd>nohl<cr>')
    -- jumping to diagnostics
    noremap('n', '[d', function() editorapi.jump_diagnostic(-1, false) end)
    noremap('n', ']d', function() editorapi.jump_diagnostic(1, false) end)
    noremap('n', '[D', function() editorapi.jump_diagnostic(-1, true) end)
    noremap('n', ']D', function() editorapi.jump_diagnostic(-1, true) end)
    -- toggle comment
    noremap('n', '<leader>c', vim.cmd.CommentToggle)
    noremap('v', '<leader>c', "V<cmd>'<,'>CommentToggle<cr>gv")

    -- view helpers
    noremap('n', '<leader>vi', function()
        vim.lsp.inlay_hint.enable(not vim.lsp.inlay_hint.is_enabled())
    end)
    local ibl_enabled = false
    noremap('n', '<leader>vg', function() if ibl_enabled then vim.cmd.IBLToggle() return end ibl_enabled = true vim.cmd.IBLEnable() end)

    -- view change
    -- toggle undotree
    noremap('n', '<leader>u', vim.cmd.UndotreeToggle)
    noremap('n', '<leader>w', editorapi.editview_swap_files)
    noremap('n', '<leader>dl', function() editorapi.editview_duplicate(true) end)
    noremap('n', '<leader>dh', function() editorapi.editview_duplicate(false) end)
    noremap('n', '<leader>do', editorapi.editview_close_other)

    -- floaterm
    noremap({'n', 't'}, [[<C-\>]], editorapi.editview_floaterm_toggle)
    noremap({'n', 't'}, [[<leader><C-\>]], editorapi.editview_floaterm_new)
    noremap({'n', 't'}, '<C-n>', editorapi.editview_floaterm_cycle)
    noremap('t', '<C-w>', editorapi.editview_terminal_escape)

    -- telescopers
    noremap('n', '<leader>fr', editorapi.editview_openfinder_last)
    noremap({'n', 't'}, '<leader>ff', editorapi.editview_openfinder_file)
    noremap({'n', 't'}, '<leader>fg', editorapi.editview_openfinder_live_grep)
    noremap({'n', 't'}, '<leader>fb', editorapi.editview_openfinder_buffer)
    noremap('n', '<leader>fs', editorapi.editview_openfinder_symbol)
    noremap('n', 'gr', editorapi.editview_openfinder_reference)
    noremap('n', 'gd', editorapi.editview_openfinder_definition)
    noremap('n', 'gi', editorapi.editview_openfinder_implementation)
    noremap('n', '<leader>fd', editorapi.editview_openfinder_diagnostic)
    -- ai coder
    noremap({'n', 'v'}, '<leader>bb', editorapi.editview_aicoder_open)
    noremap({'n', 't'}, '<leader>bh', editorapi.aicoder_close)
    noremap({'n', 't'}, '<leader>bv', editorapi.aicoder_open_or_accept_diff)
    noremap({'n', 't'}, '<leader>bd', editorapi.aicoder_deny_diff)
    noremap('n', '<leader>bl', function() editorapi.aicoder_send(false) end)
    noremap('v', '<leader>bl', function() editorapi.aicoder_send(true) end)
    noremap('n', '<leader>gs', editorapi.diffview_git_view_or_status)
    noremap('n', '<leader>gd', editorapi.diffview_git_diff_new)

    -- focus on tree (edit mode and diff mode)
    noremap('n', '<leader>t', function() editorapi.open_file_tree(false) end)
    noremap('n', '<leader>T', function() editorapi.open_file_tree(true) end)

    -- fix lsp (the key bind is from :e which tends to fix most issues but not always)
    noremap('n', '<leader>e', function() editorapi.fix_buffer_issues(false) end)
    noremap('n', '<leader>E', function() editorapi.fix_buffer_issues(false) end)

    noremap({'n', 'v'}, 'I', editorapi.multipurpose_toggle_shift_i)
end

---Setup key maps for nvim tree buffer
function M.setup_nvim_tree(bufnr)
    local api = require("nvim-tree.api")
    local RootNode = require("nvim-tree.node.root")
    local core = require("nvim-tree.core")
    local is_win32 = vim.fn.has("win32") ~= 0
    local tree_noremap = function (key, func, desc, opts)
        opts = opts or {}
        local keymap_opts =  {
            desc = 'nvim-tree: ' .. desc,
            buffer = bufnr,
            noremap = true,
            silent = true,
            nowait = true
        }
        -- see https://github.com/nvim-tree/nvim-tree.lua/issues/3292
        -- windows have filesystem watcher disabled
        if is_win32 and opts.refresh then
            local inner = func
            func = function()
                inner()
                api.tree.reload()
            end
        end

        -- nop if the node is root, prevent accidentally changing root
        if not opts.allow_root then
            local inner = func
            func = function()
                local expl = core.get_explorer()
                local node = expl and expl:get_node_at_cursor() or nil
                if node and (node.name == ".." or node:as(RootNode)) then
                    editorapi.warn("operation not allowed for workspace root")
                    return
                end
                inner()
            end
        end
        vim.keymap.set('n', key, func, keymap_opts)
    end
    tree_noremap('K', api.node.show_info_popup, 'Info')
    -- vim.keymap.set('n', 'O', api.node.navigate.parent_close, opts('Close parent'))
    tree_noremap('P', api.node.navigate.parent, 'Go to parent')
    tree_noremap('m', api.fs.rename_sub, 'Move', { refresh = true })
    tree_noremap('o', api.node.open.edit, 'Open')
    tree_noremap('v', editorapi.editview_open_split, 'Open: vertical')
    tree_noremap('s', api.node.open.horizontal, 'Open: split')
    tree_noremap('a', api.fs.create, 'Create', { allow_root = true })
    tree_noremap('c', api.fs.copy.node, 'Copy')
    tree_noremap('p', api.fs.paste, 'Paste', { allow_root = true })
    tree_noremap('d', api.fs.remove, 'Delete', { refresh = true })
    tree_noremap('x', api.fs.cut, 'Cut')
    tree_noremap('r', api.fs.rename_sub, 'Rename', { refresh = true })
    tree_noremap('-', api.marks.toggle, 'Select')
    tree_noremap('bd', api.marks.bulk.delete, 'Delete: selected')
    tree_noremap('bm', api.marks.bulk.move, 'Move: selected')
    tree_noremap('[', api.node.navigate.diagnostics.prev, 'Prev diagnostic')
    tree_noremap(']', api.node.navigate.diagnostics.next, 'Next diagnostic')
    tree_noremap('B', api.tree.toggle_no_buffer_filter, 'Toggle opened', { allow_root = true })
    tree_noremap('H', api.tree.toggle_hidden_filter, 'Toggle dotfiles', { allow_root = true })
    tree_noremap('E', api.node.open.toggle_group_empty, 'Toggle group empty')
    tree_noremap('R', api.tree.reload, 'Refresh', { allow_root = true })
    -- tree_noremap('gy', api.fs.copy.absolute_path, 'Copy absolute path')
    -- tree_noremap('y', api.fs.copy.filename, 'Copy name')
    -- tree_noremap('Y', api.fs.copy.relative_path, 'Copy relative path')
    tree_noremap('g?', api.tree.toggle_help, 'Help', { allow_root = true })
end

function M.setup_lsp(bufnr)
    local opts = { buffer = bufnr }
    -- keys that only work when LSP is attached (so they are buffer-local)
    vim.keymap.set('n', '<leader>r', vim.lsp.buf.rename, opts)
    vim.keymap.set('n', '<leader>f', vim.lsp.buf.format, opts)
    vim.keymap.set('n', 'K', function()
        local max_width = math.floor(vim.o.columns / 3)
        vim.lsp.buf.hover({ border = "rounded", max_width = max_width })
    end, opts)
    vim.keymap.set('n', 'gt', vim.lsp.buf.type_definition, opts)
    -- code action menu
    vim.keymap.set({ 'n', 'v' }, '<leader>a', vim.lsp.buf.code_action, opts)
    -- signature help in input mode
    vim.keymap.set('i', '<C-h>', vim.lsp.buf.signature_help, opts)
    vim.keymap.set('n', '<leader>O', function()
        local organize_imports_fn = require("config.organize-imports")
        organize_imports_fn()
    end, opts)
    -- allow big E to restart LSP
    vim.keymap.set('n', '<leader>E', function() editorapi.fix_buffer_issues(true) end, opts)
end

function M.get_telescope_mappings()
    return {
        i = {
            ["<A-j>"] = "move_selection_next",
            ["<A-k>"] = "move_selection_previous",
        }
    }
end

function M.get_codediff_mappings()
    return {
        view = {
            quit = "<esc>",                    -- Close diff tab
            toggle_explorer = "<leader>pT",  -- Toggle explorer visibility (explorer mode only)
            next_hunk = "]c",   -- Jump to next change
            prev_hunk = "[c",   -- Jump to previous change
            next_file = "]f",   -- Next file in explorer mode
            prev_file = "[f",   -- Previous file in explorer mode
            -- diff_get = "do",    -- Get change from other buffer (like vimdiff)
            -- diff_put = "dp",    -- Put change to other buffer (like vimdiff)
        },
        explorer = {
            select = "o",    -- Open diff for selected file
            --     hover = "K",        -- Show file diff preview
            --     refresh = "R",      -- Refresh git status
            toggle_view_mode = "i",  -- Toggle between 'list' and 'tree' views
            toggle_stage = nil,--"s", -- Stage/unstage selected file
            stage_all = nil,--"S",    -- Stage all files
            unstage_all = nil,--"U",  -- Unstage all files
            restore = nil--"x",      -- Discard changes (restore file)
        },
        conflict = {
            accept_incoming = nil,--j"<leader>ct",  -- Accept incoming (theirs/left) change
            accept_current = nil,--"<leader>co",   -- Accept current (ours/right) change
            accept_both = nil,--"<leader>cb",      -- Accept both changes (incoming first)
            discard = nil,--"<leader>cx",          -- Discard both, keep base
            next_conflict = "]x",            -- Jump to next conflict
            prev_conflict = "[x",            -- Jump to previous conflict
            diffget_incoming = "2do",        -- Get hunk from incoming (left/theirs) buffer
            diffget_current = "3do",         -- Get hunk from current (right/ours) buffer
        },
    }
end

function M.get_cmp_mappings()
    local cmp = require('cmp')
    return {
        -- accept completion
        ['<CR>'] = cmp.mapping.confirm({ select = false }),
        -- trigger completion
        ['<C-n>'] = cmp.mapping.complete(),
        -- abort completion
        ['<C-e>'] = cmp.mapping.abort(),
        -- nagivate
        ['<A-k>'] = cmp.mapping.select_prev_item(),
        ['<A-j>'] = cmp.mapping.select_next_item(),
    }
end

return M
