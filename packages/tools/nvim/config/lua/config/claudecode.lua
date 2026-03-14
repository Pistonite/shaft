local terminal_cmd = require("piston.config_gen").aicoder_command
require("claudecode").setup {
    log_level = "error",
    auto_start = true,
    focus_after_send = false,
    terminal_cmd = terminal_cmd or "claude",
    terminal = {
        provider = "native",
        split_width_percentage = 0.8,
        cwd_provider = function(ctx)
            return ctx.cwd
        end,
    },
    diff_opts = { -- we patch the plugin to handle our own diff UI
        open_in_new_tab = false,
        auto_close_on_accept = false,
        keep_terminal_focus = false,
    }
}
