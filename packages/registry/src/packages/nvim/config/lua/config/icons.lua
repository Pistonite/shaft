local devicons = require("nvim-web-devicons")
local icons = devicons.get_icons_by_extension()

devicons.setup({
    strict = true,
    override_by_extension = {
        tsx = icons.jsx,
        sh = vim.tbl_extend("force", icons.sh, {
            color = icons.ps1.color
        }),
        patch = vim.tbl_extend("force", icons.patch, {
            color = "#f5c2e7"
        }),
    },
    override_by_filename = {
    }
})
