local telescope = require('telescope')
telescope.setup({
    defaults = {
        mappings = require("piston.keymaps").get_telescope_mappings()
    },
    extensions = {
        ["ui-select"] = {
            require("telescope.themes").get_dropdown { }
        }
    }
})
telescope.load_extension("ui-select")
