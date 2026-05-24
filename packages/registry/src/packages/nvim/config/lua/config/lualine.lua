local lualine_theme = require("lualine.themes.catppuccin")
lualine_theme.normal.a.gui = ""
lualine_theme.insert.a.gui = ""
lualine_theme.visual.a.gui = ""
lualine_theme.replace.a.gui = ""
lualine_theme.command.a.gui = ""
lualine_theme.inactive.a.gui = ""
require('lualine').setup({
    options = {
        theme = lualine_theme,
        disabled_filetypes = {
            'NvimTree',
            'undotree',
            'codediff-explorer',
        },
    },
    sections = {
        lualine_b = {
            'branch',
            'diff',
            {
                'diagnostics',
                colored = true,
                symbols = {
                    error = 'E',
                    warn = 'W',
                    hint = 'H',
                    info = 'I',
                }
            }
        }
    }
})
