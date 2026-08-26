local devicons = require("nvim-web-devicons")
local icons = devicons.get_icons_by_extension()

local llvm_icon = {
    icon = "",
    color = "#97C3E6",
    name = "LLVM"
}

local taskfile_icon = {
    icon = "󰆦",
    color = "#69D3C9",
    name = "Taskfile"
}

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
        [".clang-tidy"] = llvm_icon,
        [".clang-format"] = llvm_icon,
        [".clangd"] = llvm_icon,
        ["Taskfile.yml"] = taskfile_icon,
    }
})
