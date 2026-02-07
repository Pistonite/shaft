-- All of global options

-- windows shell options
if vim.fn.has("win32") ~= 0 then
    local win_shell = "pwsh"
    if vim.fn.executable("pwsh.exe") == 0 then
        win_shell = "powershell"
    end
    vim.o.shell = win_shell
    vim.o.shellcmdflag = "-Nologo -command"
    vim.o.shellquote = '"'
    vim.o.shellxquote = ""
end

-- line numbers
vim.opt.number = true    -- Enable line numbers
vim.opt.rnu = true       -- Relative line numbers by default
-- hidden characters (controlled by keymapping)
vim.opt.listchars = "tab:▸ ,space:·,trail:·,nbsp:␣,extends:»,precedes:«,eol:↲"
-- indent
vim.opt.expandtab = true -- Tab become spaces
vim.opt.shiftwidth = 4   -- Indent 4
vim.opt.tabstop = 4
vim.opt.softtabstop = 4
vim.opt.smartindent = true
vim.opt.wrap = false
-- diff
vim.opt.fillchars:append { diff = "╱" }
vim.opt.termguicolors = true -- colors
-- set up undo dir
vim.opt.swapfile = false
vim.opt.backup = false
vim.opt.undofile = true
vim.opt.undodir = vim.fn.stdpath("data") .. '/piston/undodir'

-- folds
vim.opt.foldenable = false   -- no fold at startup
vim.opt.foldlevel = 99
-- search
vim.opt.hlsearch = true
vim.opt.incsearch = true -- should be the default
-- scrolling
vim.opt.scrolloff = 8
vim.opt.sidescrolloff = 8

-- competion options
-- menuone: popup even when there's only one match
-- noinsert: Do not insert text until a selection is made
-- noselect: Do not select, force to select one from the menu
-- shortmess: avoid showing extra messages when using completion
-- updatetime: set updatetime for CursorHold
vim.opt.completeopt = { 'menuone', 'noselect', 'noinsert' }
vim.opt.shortmess = vim.opt.shortmess + { c = true }
vim.api.nvim_set_option_value('updatetime', 300, {})

-- Floaterm style
vim.g.floaterm_title = 'Terminal [$1/$2]'
-- Undotree style
vim.g.undotree_WindowLayout = 0
vim.g.undotree_SetFocusWhenToggle = 1

vim.diagnostic.config({
    virtual_text = true,
    float = {
        border = 'rounded',
    }
})
