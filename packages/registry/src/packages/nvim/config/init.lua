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
            "lazy/lazy.nvim", tag = "v11.17.5", lazy = false, priority = 9999
        },
        -- ## UI AND EDITOR FUNCTION
        {   -- optimization for big files
            'pteroctopus/faster.nvim',                 commit = "6b7cb8b1a9628d4b87009c4849d510d1a3a14319", lazy = false
        }, {
            'nvim-tree/nvim-tree.lua',                 commit = "07f541fcaa4a5ae019598240362449ab7e9812b3", lazy = false, priority = 2000, -- needs to be before cat for colors in tree to load properly
            config = function() require("config.nvim-tree") end
        }, {
            "catppuccin/nvim", name = "catppuccin",    commit = "605b4603797de970e9f3a4238c199c850da03186", lazy = false, priority = 1000,
            config = function() require("config.theme") end
        }, {
            'nvim-tree/nvim-web-devicons',             commit = "dfbfaa967a6f7ec50789bead7ef87e336c1fa63c",
        }, {
            'nvim-lualine/lualine.nvim',               commit = "221ce6b2d999187044529f49da6554a92f740a96",
        }, {
            'terrortylor/nvim-comment',                commit = "e9ac16ab056695cad6461173693069ec070d2b23",
            cmd = "CommentToggle",
            config = function()
                require("nvim_comment").setup({ create_mappings = false })
            end
            -- U: UNMAINTAINED
        }, {
            "lukas-reineke/indent-blankline.nvim",     commit = "d28a3f70721c79e3c5f6693057ae929f3d9c0a03",
            cmd = { "IBLToggle", "IBLEnable", "IBLDisable" },
            config = function() require("ibl").setup() end
        },
        {
            'mbbill/undotree',                         commit = "6fa6b57cda8459e1e4b2ca34df702f55242f4e4d",
            cmd = "UndotreeToggle"
        }, {
            'voldikss/vim-floaterm', name="floaterm",  commit = "bb4ba7952e906408e1f83b215f55ffe57efcade6",
            cmd = "FloatermToggle"
        }, {
            'esmuellert/codediff.nvim',                tag = "v2.49.2",
            cmd = "CodeDiff",
            config = function()
                require("codediff").setup({
                    keymaps = require("piston.keymaps").get_codediff_mappings()
                })
            end
        }, { -- codediff dependency
            'MunifTanjim/nui.nvim',                    commit = "de740991c12411b663994b2860f1a4fd0937c130",
        }, {
            'nvim-treesitter/nvim-treesitter',         commit = '4916d6592ede8c07973490d9322f187e07dfefac', lazy = false,
            build = ":TSUpdate",
            config = function() require("config.nvim-treesitter")         end
            -- U: UNMAINTAINED due to dick heads pissing off dev
        }, {
            'nvim-treesitter/nvim-treesitter-context', commit = 'b311b30818951d01f7b4bf650521b868b3fece16', lazy = false,
            config = function()
                require('treesitter-context').setup({
                    enable = true,
                    separator = '>',
                })
            end
        }, {
            'nvim-telescope/telescope.nvim',           commit = "7d324792b7943e4aa16ad007212e6acc6f9fe335",
            cmd = "Telescope", event = "LspAttach",
            config = function() require("config.telescope")               end
        }, {
            'nvim-telescope/telescope-ui-select.nvim', commit = "6e51d7da30bd139a6950adf2a47fda6df9fa06d2",
        }, { -- telescope dependency
            'nvim-lua/plenary.nvim',                   commit = "74b06c6c75e4eeb3108ec01852001636d85a932b",
        },

        -- ## LANGUAGE SERVICE
        { -- filetype detection .. this is what triggers the lazy loading of nvim-lspconfig
            dir = configpath.."/lua/config/lsp", name = "lsp-filetypes", event = "BufNew",
            config = function() require("config.lsp-filetypes") end
        }, {
            'mason-org/mason-lspconfig.nvim',          commit = "0a695750d747db1e7e70bcf0267ef8951c95fc83",
            config = function()
                require("mason-lspconfig").setup({
                    automatic_enable = false
                })
            end,
        }, {
            'mason-org/mason.nvim',                    commit = "16ba83bfc8a25f52bb545134f5bee082b195c460",
            cmd = "Mason", build = ":MasonUpdate",
            config = function()
                require("mason").setup({
                    ui = { border = 'rounded', }
                })
            end
        }, {
            'neovim/nvim-lspconfig',                   commit = "ed19590a3a9792901553c388d1aadafce012f80d",
        }, {
            -- 0.11 'felpafel/inlay-hint.nvim',                commit = "ee8aa9806d1e160a2bc08b78ae60568fb6d9dbce",
            'felpafel/inlay-hint.nvim',                commit = "369aa3d5f10b41580242cd6e825bd00cfa565464",
            event = "LspAttach",
            config = function() require("config.inlay-hint") end
        }, {
            'hrsh7th/nvim-cmp',                        commit = "a1d504892f2bc56c2e79b65c6faded2fd21f3eca",
            event = "InsertEnter",
            config = function() require("config.lsp-cmp") end
        },
        { 'hrsh7th/cmp-nvim-lsp',                    commit = "cbc7b02bb99fae35cb42f514762b89b5126651ef" },
        { 'hrsh7th/cmp-path',                        commit = "c642487086dbd9a93160e1679a1327be111cbc25" },
        { 'hrsh7th/cmp-buffer',                      commit = "b74fab3656eea9de20a9b8116afa3cfc4ec09657" },
        { 'hrsh7th/cmp-nvim-lsp-signature-help',     commit = "fd3e882e56956675c620898bf1ffcf4fcbe7ec84" },
        { 'hrsh7th/cmp-nvim-lua',                    commit = "e3a22cb071eb9d6508a156306b102c45cd2d573d" },
        -- language: java (jdtls)
        -- use { 'mfussenegger/nvim-jdtls',                 commit = "ece818f909c6414cbad4e1fb240d87e003e10fda",
        --     ft = { 'java' },
        --     config = function () require('lsp-wrapper.jdtls') end
        -- }

        -- ## AI
        {
            dir = configpath .. '/claudecode.nvim', name = "claudecode",
            cmd = { "ClaudeCode", "ClaudeCodeTreeAdd", "ClaudeCodeAdd", "ClaudeCodeSend" },
            config = function() require("config.claudecode") end
        }
    },
    change_detection = { enabled = false }
}
