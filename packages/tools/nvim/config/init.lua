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
            'pteroctopus/faster.nvim',                 commit = "2e7a50f659711854b9a7fdc76d943b59b30d7852", lazy = false
        }, {
            'nvim-tree/nvim-tree.lua',                 commit = "a0db8bf7d6488b1dcd9cb5b0dfd6684a1e14f769", lazy = false, priority = 2000, -- needs to be before cat for colors in tree to load properly
            config = function() require("config.nvim-tree") end
        }, {
            "catppuccin/nvim", name = "catppuccin",    commit = "cb5665990a797b102715188e73c44c3931b3b42e", lazy = false, priority = 1000,
            config = function() require("config.theme") end
        }, {
            'nvim-tree/nvim-web-devicons',             commit = "6788013bb9cb784e606ada44206b0e755e4323d7",
        }, {
            'nvim-lualine/lualine.nvim',               commit = "47f91c416daef12db467145e16bed5bbfe00add8",
        }, {
            'terrortylor/nvim-comment',                commit = "e9ac16ab056695cad6461173693069ec070d2b23",
            cmd = "CommentToggle",
            config = function()
                require("nvim_comment").setup({ create_mappings = false })
            end
            -- U: UNMAINTAINED
        }, {
            "lukas-reineke/indent-blankline.nvim",     commit = "005b56001b2cb30bfa61b7986bc50657816ba4ba",
            cmd = { "IBLToggle", "IBLEnable", "IBLDisable" },
            config = function() require("ibl").setup() end
        },
        {
            'mbbill/undotree',                         commit = "0f1c9816975b5d7f87d5003a19c53c6fd2ff6f7f",
            cmd = "UndotreeToggle"
        }, {
            'voldikss/vim-floaterm', name="floaterm",  commit = "a11b930f55324e9b05e2ef16511fe713f1b456a7",
            cmd = "FloatermToggle"
        }, {
            'esmuellert/codediff.nvim',                tag = "v2.9.3",
            cmd = "CodeDiff",
            config = function()
                require("codediff").setup({
                    keymaps = require("piston.keymaps").get_codediff_mappings()
                })
            end
        }, { -- codediff dependency
            'MunifTanjim/nui.nvim',                    commit = "de740991c12411b663994b2860f1a4fd0937c130",
        }, {
            'nvim-treesitter/nvim-treesitter',         commit = '4967fa48b0fe7a7f92cee546c76bb4bb61bb14d5', lazy = false,
            build = ":TSUpdate",
            config = function() require("config.nvim-treesitter")         end
        }, {
            'nvim-treesitter/nvim-treesitter-context', commit = '64dd4cf3f6fd0ab17622c5ce15c91fc539c3f24a', lazy = false,
            config = function()
                require('treesitter-context').setup({
                    enable = true,
                    separator = '>',
                })
            end
        }, {
            'nvim-telescope/telescope.nvim',           commit = "4d0f5e0e7f69071e315515c385fab2a4eff07b3d",
            cmd = "Telescope", event = "LspAttach",
            config = function() require("config.telescope")               end
        }, {
            'nvim-telescope/telescope-ui-select.nvim', commit = "6e51d7da30bd139a6950adf2a47fda6df9fa06d2",
        }, { -- telescope dependency
            'nvim-lua/plenary.nvim',                   commit = "b9fd5226c2f76c951fc8ed5923d85e4de065e509",
        },

        -- ## LANGUAGE SERVICE
        { -- filetype detection .. this is what triggers the lazy loading of nvim-lspconfig
            dir = configpath.."/lua/config/lsp", name = "lsp-filetypes", event = "BufNew",
            config = function() require("config.lsp-filetypes") end
        }, {
            'mason-org/mason-lspconfig.nvim',          commit = "9f9c67795d0795a6e8612f5a899ca64a074a1076",
            config = function()
                require("mason-lspconfig").setup({
                    automatic_enable = false
                })
            end,
        }, {
            'mason-org/mason.nvim',                    commit = "57e5a8addb8c71fb063ee4acda466c7cf6ad2800",
            cmd = "Mason", build = ":MasonUpdate",
            config = function()
                require("mason").setup({
                    ui = { border = 'rounded', }
                })
            end
        }, {
            'neovim/nvim-lspconfig',                   commit = "d696e36d5792daf828f8c8e8d4b9aa90c1a10c2a",
        }, {
            'felpafel/inlay-hint.nvim',                commit = "ee8aa9806d1e160a2bc08b78ae60568fb6d9dbce",
            event = "LspAttach",
            config = function() require("config.inlay-hint") end
        }, {
            'hrsh7th/nvim-cmp',                        commit = "d97d85e01339f01b842e6ec1502f639b080cb0fc",
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
