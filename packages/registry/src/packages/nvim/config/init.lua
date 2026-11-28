--[[
MIT License

Copyright (c) 2025-2026 Michael

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
]]

-- disable nvim builtin file explorer
vim.g.loaded_netrw = 1
vim.g.loaded_netrwPlugin = 1
-- Bootstrap lazy.nvim
local lazypath = vim.fn.stdpath("data") .. "/lazy/lazy.nvim"
vim.opt.rtp:prepend(lazypath) -- assume lazypath exists, saves 1 fs_stat on start up, `python3 setup.py apply`
-- Load Own configurations
require('piston.options')
require('piston.keymaps').setup()
local configpath = vim.fn.stdpath("config")
require("lazy").setup {
    defaults = {
        lazy = true,
    },
    -- U: Upstream appears to be unmaintained. No need to check for updates
    -- L: Lock to this version because of an issue, put the issue link like L#123
    spec = {
        {
            -- note the version must be kept in sync with info.json
            -- https://github.com/folke/lazy.nvim
            "folke/lazy.nvim", tag = "v11.17.5", lazy = false, priority = 9999
        },
        -- ## UI AND EDITOR FUNCTION
        {   -- optimization for big files
            -- https://github.com/pteroctopus/faster.nvim
            'pteroctopus/faster.nvim',                 commit = "6b7cb8b1a9628d4b87009c4849d510d1a3a14319", lazy = false
        }, {
            -- https://github.com/nvim-tree/nvim-tree.lua
            'nvim-tree/nvim-tree.lua',                 commit = "d0b8d652d3a776a29b09f9c14f87717e455c52f6", lazy = false, priority = 2000, -- needs to be before cat for colors in tree to load properly
            config = function() require("config.nvim-tree") end
        }, {
            -- https://github.com/catppuccin/nvim
            "catppuccin/nvim", name = "catppuccin",    commit = "edefef779ab08ce1a4a404713e3012b0d202bd35", lazy = false, priority = 1000,
            config = function() require("config.theme") end
        }, {
            -- https://github.com/nvim-tree/nvim-web-devicons
            'nvim-tree/nvim-web-devicons',             commit = "0dc0cfb0c6858041174a53c7aa4adaf65e994b3c",
            config = function() require("config.icons") end
        }, {
            -- https://github.com/nvim-lualine/lualine.nvim
            'nvim-lualine/lualine.nvim',               commit = "221ce6b2d999187044529f49da6554a92f740a96",
        }, { -- patch for commentstring in tsx (see editorapi.toggle_comment)
            -- https://github.com/JoosepAlviste/nvim-ts-context-commentstring
            'JoosepAlviste/nvim-ts-context-commentstring', commit = "6141a40173c6efa98242dc951ed4b6f892c97027",
            ft = { "typescriptreact", "javascriptreact" },
            config = function() -- this is called once so the override is ok
                -- https://github.com/JoosepAlviste/nvim-ts-context-commentstring/issues/109
                local get_option = vim.filetype.get_option
                ---@diagnostic disable-next-line: duplicate-set-field
                vim.filetype.get_option = function(filetype, option)
                  return option == "commentstring"
                    and require("ts_context_commentstring.internal").calculate_commentstring()
                    or get_option(filetype, option)
                end
            end
        }, {
            -- https://github.com/lukas-reineke/indent-blankline.nvim
            "lukas-reineke/indent-blankline.nvim",     commit = "fbc6d88222b5f75613cb95a22e89aa2f4e318a81",
            cmd = { "IBLToggle", "IBLEnable", "IBLDisable" },
            config = function() require("ibl").setup() end
        },
        {
            -- https://github.com/mbbill/undotree
            'mbbill/undotree',                         commit = "6fa6b57cda8459e1e4b2ca34df702f55242f4e4d",
            cmd = "UndotreeToggle"
        }, {
            -- https://github.com/voldikss/vim-floaterm
            'voldikss/vim-floaterm', name="floaterm",  commit = "c74c5cd791befcd1000cbee35f07d45b912908e3",
            cmd = "FloatermToggle"
        }, {
            -- https://github.com/esmuellert/codediff.nvim
            'esmuellert/codediff.nvim',                tag = "v4.0.8",
            cmd = "CodeDiff",
            config = function()
                require("codediff").setup({
                    keymaps = require("piston.keymaps").get_codediff_mappings()
                })
            end
        }, {
            -- https://github.com/nvim-treesitter/nvim-treesitter
            'nvim-treesitter/nvim-treesitter',         commit = "52773f0d13a0c11d0c3ef33c14ab6ecc1c17148b", lazy = false,
            build = ":TSUpdate",
            config = function() require("config.nvim-treesitter")         end
        }, {
            -- https://github.com/nvim-treesitter/nvim-treesitter-context
            'nvim-treesitter/nvim-treesitter-context', commit = "8decd73460d26ef0a0befb745d022dd1fc2c38b8", lazy = false,
            config = function()
                require('treesitter-context').setup({
                    enable = true,
                    separator = '>',
                })
            end
        }, {
            -- https://github.com/nvim-telescope/telescope.nvim
            'nvim-telescope/telescope.nvim',           commit = "1e1bab730218eaf97fce743be3f56e51075928ac",
            cmd = "Telescope", event = "LspAttach",
            config = function() require("config.telescope")               end
        }, {
            -- https://github.com/nvim-telescope/telescope-ui-select.nvim
            'nvim-telescope/telescope-ui-select.nvim', commit = "6e51d7da30bd139a6950adf2a47fda6df9fa06d2",
        }, { -- telescope dependency
            -- https://github.com/nvim-lua/plenary.nvim
            'nvim-lua/plenary.nvim',                   commit = "74b06c6c75e4eeb3108ec01852001636d85a932b",
        },

        -- ## LANGUAGE SERVICE
        { -- filetype detection .. this is what triggers the lazy loading of nvim-lspconfig and other lsp plugins
            dir = configpath.."/lua/config/lsp", name = "lsp-filetypes",
            lazy = false, -- the file type registration is itself very small and registers auto commands, so we need to
                          -- make sure it's always there (so it's not a suspect when there is something wrong with LSP
            config = function() require("config.lsp-filetypes") end
        }, {
            -- https://github.com/mason-org/mason-lspconfig.nvim
            'mason-org/mason-lspconfig.nvim',          commit = "761d376b5fbe6b21f659432ffadc1205c660176c",
            config = function()
                require("mason-lspconfig").setup({ automatic_enable = false })
            end,
        }, {
            -- https://github.com/mason-org/mason.nvim
            'mason-org/mason.nvim',                    commit = "2a6940af80375532e5e9e7c1f2fc6319a1b7a69d",
            cmd = "Mason",
            build = ":MasonUpdate",
            config = function()
                require("mason").setup({ ui = { border = 'rounded' } })
            end
        }, {
            -- https://github.com/neovim/nvim-lspconfig
            -- https://github.com/neovim/nvim-lspconfig/pull/4591
            'Pistonight/nvim-lspconfig',                   commit = "207232333c401e24c66ef809fa1c3495c4af1a02",
        }, {
            -- https://github.com/felpafel/inlay-hint.nvim
            'felpafel/inlay-hint.nvim',                commit = "369aa3d5f10b41580242cd6e825bd00cfa565464",
        }, {
            -- https://github.com/hrsh7th/nvim-cmp
            'hrsh7th/nvim-cmp',                        commit = "fe29aba4117227a6df637bc41e4b4fb4e98864a0",
            event = "InsertEnter",
            config = function() require("config.lsp-cmp") end
        },
        -- https://github.com/hrsh7th/cmp-nvim-lsp
        { 'hrsh7th/cmp-nvim-lsp',                    commit = "cd4ee08a988deb47ac3726f418fb27fe1a95729d" },
        -- https://github.com/hrsh7th/cmp-path
        { 'hrsh7th/cmp-path',                        commit = "c642487086dbd9a93160e1679a1327be111cbc25" },
        -- https://github.com/hrsh7th/cmp-buffer
        { 'hrsh7th/cmp-buffer',                      commit = "b74fab3656eea9de20a9b8116afa3cfc4ec09657" },
        -- https://github.com/hrsh7th/cmp-nvim-lsp-signature-help
        { 'hrsh7th/cmp-nvim-lsp-signature-help',     commit = "fd3e882e56956675c620898bf1ffcf4fcbe7ec84" },
        -- https://github.com/hrsh7th/cmp-nvim-lua
        { 'hrsh7th/cmp-nvim-lua',                    commit = "e3a22cb071eb9d6508a156306b102c45cd2d573d" },

        -- ## Language servers that require special setups
        -- lsp: java (jdtls)
        {
            dir = configpath .. '/piston-jdtls.nvim', name = "piston-jdtls",
            cmd = { "JdtlsCheck", "JdtlsInstall", "JdtlsClean" },
            config = function() require("piston_jdtls").setup_commands() end
        },
        { -- needed by piston-jdtls
            -- https://github.com/mfussenegger/nvim-jdtls
            'mfussenegger/nvim-jdtls',                 commit = "6e9d953f0b82bccdb834cfde0e893f3119c22592"
        },

        -- ## AI
        {
            dir = configpath .. '/claudecode.nvim', name = "claudecode",
            cmd = { "ClaudeCode", "ClaudeCodeTreeAdd", "ClaudeCodeAdd", "ClaudeCodeSend" },
            config = function() require("config.claudecode") end
        }
    },
    change_detection = { enabled = false }
}
-- Register extra file types
-- au BufRead,BufNewFile *.md.txtpp        set filetype=markdown
-- au BufRead,BufNewFile *.html.txtpp      set filetype=html
-- au BufRead,BufNewFile *.rs.txtpp        set filetype=rust
-- au BufRead,BufNewFile *.ts.txtpp        set filetype=typescript
-- au BufRead,BufNewFile *.tsx.txtpp       set filetype=typescriptreact
-- au BufRead,BufNewFile *.js.txtpp        set filetype=javascript
-- au BufRead,BufNewFile *.jsx.txtpp       set filetype=javascriptreact
-- au BufRead,BufNewFile *.py.txtpp        set filetype=python
-- au BufRead,BufNewFile *.css.txtpp       set filetype=css
-- au BufRead,BufNewFile *.json.txtpp      set filetype=json
-- au BufRead,BufNewFile *.yaml.txtpp      set filetype=yaml
-- au BufRead,BufNewFile *.yml.txtpp       set filetype=yaml
-- au BufRead,BufNewFile *.toml.txtpp      set filetype=toml
-- au BufRead,BufNewFile *.bash.txtpp      set filetype=bash
vim.cmd [[
  au BufRead,BufNewFile *.rasi            set filetype=css
  au BufRead,BufNewFile *.mdx             set filetype=markdown
]]
