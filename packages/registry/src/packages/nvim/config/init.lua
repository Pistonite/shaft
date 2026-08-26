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
vim.opt.rtp:prepend(lazypath) -- assume lazypath exists, saves 1 fs_stat on start up, `python setup.py apply`
-- Load Own configurations
require('piston.options')
require('piston.keymaps').setup()
require('piston.extra')
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
            -- https://github.com/lazy/lazy.nvim
            "lazy/lazy.nvim", tag = "v11.17.5", lazy = false, priority = 9999
        },
        -- ## UI AND EDITOR FUNCTION
        {   -- optimization for big files
            -- https://github.com/pteroctopus/faster.nvim
            'pteroctopus/faster.nvim',                 commit = "6b7cb8b1a9628d4b87009c4849d510d1a3a14319", lazy = false
        }, {
            -- https://github.com/nvim-tree/nvim-tree.lua
            'nvim-tree/nvim-tree.lua',                 commit = "b2aadda94b107480c48e548d6db51c6840b7b33c", lazy = false, priority = 2000, -- needs to be before cat for colors in tree to load properly
            config = function() require("config.nvim-tree") end
        }, {
            -- https://github.com/catppuccin/nvim
            "catppuccin/nvim", name = "catppuccin",    commit = "edefef779ab08ce1a4a404713e3012b0d202bd35", lazy = false, priority = 1000,
            config = function() require("config.theme") end
        }, {
            -- https://github.com/nvim-tree/nvim-web-devicons
            'nvim-tree/nvim-web-devicons',             commit = "2ae6958df7ced50baac5035cec0c15799eedfbf7",
        }, {
            -- https://github.com/nvim-lualine/lualine.nvim
            'nvim-lualine/lualine.nvim',               commit = "221ce6b2d999187044529f49da6554a92f740a96",
        }, {
            -- https://github.com/terrortylor/nvim-comment
            'terrortylor/nvim-comment',                commit = "e9ac16ab056695cad6461173693069ec070d2b23",
            cmd = "CommentToggle",
            config = function()
                require("nvim_comment").setup({ create_mappings = false })
            end
            -- U: UNMAINTAINED
        }, {
            -- https://github.com/lukas-reineke/indent-blankline.nvim
            "lukas-reineke/indent-blankline.nvim",     commit = "d28a3f70721c79e3c5f6693057ae929f3d9c0a03",
            cmd = { "IBLToggle", "IBLEnable", "IBLDisable" },
            config = function() require("ibl").setup() end
        },
        {
            -- https://github.com/mbbill/undotree
            'mbbill/undotree',                         commit = "6fa6b57cda8459e1e4b2ca34df702f55242f4e4d",
            cmd = "UndotreeToggle"
        }, {
            -- https://github.com/voldikss/vim-floaterm
            'voldikss/vim-floaterm', name="floaterm",  commit = "bb4ba7952e906408e1f83b215f55ffe57efcade6",
            cmd = "FloatermToggle"
        }, {
            -- https://github.com/esmuellert/codediff.nvim
            'esmuellert/codediff.nvim',                tag = "v2.67.0",
            cmd = "CodeDiff",
            config = function()
                require("codediff").setup({
                    keymaps = require("piston.keymaps").get_codediff_mappings()
                })
            end
        }, { -- codediff dependency
            -- https://github.com/MunifTanjim/nui.nvim
            'MunifTanjim/nui.nvim',                    commit = "10fc361835c856ba4233ef5ea135b919bf3dce97",
        }, {
            -- https://github.com/nvim-treesitter/nvim-treesitter
            'nvim-treesitter/nvim-treesitter',         commit = "8b98b4470eb326f1c7b50dae79f8c963568e5720", lazy = false,
            build = ":TSUpdate",
            config = function() require("config.nvim-treesitter")         end
        }, {
            -- https://github.com/nvim-treesitter/nvim-treesitter-context
            'nvim-treesitter/nvim-treesitter-context', commit = "f3061339b8eaf9fda873600bc425b8d2d8502533", lazy = false,
            config = function()
                require('treesitter-context').setup({
                    enable = true,
                    separator = '>',
                })
            end
        }, {
            -- https://github.com/nvim-telescope/telescope.nvim
            'nvim-telescope/telescope.nvim',           commit = "40aedd8a68c78a656a10a8d62d80c54af59420fb",
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
            'mason-org/mason-lspconfig.nvim',          commit = "24d4ab0838b250753b307a8747ade06dc99aed9d",
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
            'neovim/nvim-lspconfig',                   commit = "af9adce488c75ca0a81017945c2b7fa7b461bc23",
        }, {
            -- https://github.com/felpafel/inlay-hint.nvim
            'felpafel/inlay-hint.nvim',                commit = "369aa3d5f10b41580242cd6e825bd00cfa565464",
        }, {
            -- https://github.com/hrsh7th/nvim-cmp
            'hrsh7th/nvim-cmp',                        commit = "2ffe79f1f021def8dd1fcd81deb16f1bb0d989f3",
            event = "InsertEnter",
            config = function() require("config.lsp-cmp") end
        },
        -- https://github.com/hrsh7th/cmp-nvim-lsp
        { 'hrsh7th/cmp-nvim-lsp',                    commit = "cbc7b02bb99fae35cb42f514762b89b5126651ef" },
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
