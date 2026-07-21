local M = {};

local _jdtls = require("jdtls")

M._JDTLS_HOME = vim.fn.stdpath("data") .. "/piston/jdtls"
M._PISTON_JDTLS_HOME = vim.fn.resolve(vim.fn.stdpath('config') .. "/piston-jdtls.nvim")
M._VERSION = "1.60.0"
M._BUILD = "202606262232"
M._DOWNLOAD_LINK = "https://www.eclipse.org/downloads/download.php?file=/jdtls/milestones/"
    ..M._VERSION.."/jdt-language-server-"..M._VERSION.."-"..M._BUILD..".tar.gz"
-- major LTS releases, in the order jdtls should see them
M._LTS = {"1.8", "11", "17", "21", "25"}

function M.setup_commands()
    vim.api.nvim_create_user_command("JdtlsCheck", M.check, {
        desc = "Check JDTLS installation",
    })
    vim.api.nvim_create_user_command("JdtlsInstall", M.check, {
        desc = "Install or update JDTLS",
    })
    vim.api.nvim_create_user_command("JdtlsClean", M.clean, {
        desc = "Clean JDTLS workspaces",
    })
end


-- COMMANDS

function M.check()
    local _, message = M._check_internal(false)
    M.warn(message)
end

function M.clean()
    vim.fs.rm(M._JDTLS_HOME .. "/workspaces", {recursive = true, force = true})
    M.warn("cleaned all workspaces")
end

function M.install()
    if vim.fn.executable("wget") == 0 then
        M.warn("cannot find wget, please install it to download jdtls")
        return
    end
    local piston_home = vim.fn.fnamemodify(M._JDTLS_HOME, ":h")
    local tarball = piston_home.."/jdtls.tar.gz"
    vim.fn.mkdir(piston_home, "p")

    M.echo("downloading jdtls "..M._VERSION.."...")

    -- wget writes its progress to stderr, one dot line per chunk
    local function on_stderr(_, data)
        if not data then return end
        local percent
        for p in data:gmatch("(%d+)%%") do percent = p end
        if percent then
            vim.schedule(function() M.echo("downloading jdtls "..M._VERSION.."... "..percent.."%") end)
        end
    end

    vim.system({
        "wget", "-q", "--show-progress", "--progress=dot:mega",
        "-O", tarball, M._DOWNLOAD_LINK
    }, { stderr = on_stderr }, function(res)
        vim.schedule(function()
            if res.code ~= 0 then
                vim.fn.delete(tarball)
                M.warn("download failed")
                return
            end
            M.echo("extracting...")
            if vim.fn.isdirectory(M._JDTLS_HOME) == 1 then
                vim.fn.delete(M._JDTLS_HOME, "rf")
            end
            vim.fn.mkdir(M._JDTLS_HOME, "p")

            vim.fn.system({"tar", "-xzf", tarball, "-C", M._JDTLS_HOME})
            local failed = vim.v.shell_error ~= 0
            vim.fn.delete(tarball)
            if failed then
                M.warn("extraction failed")
                return
            end

            M.warn("installed jdtls "..M._VERSION.." to "..M._JDTLS_HOME)
        end)
    end)
end

M._is_first_start = true
M._server_jdk_cache = nil
M._server_cmd_cache = nil
M._server_root_dir = nil
M._server_workspace = nil
function M.start_current_buf()
    local is_first_start = M._is_first_start
    M._is_first_start = false
    local ok, message = M._check_internal(true)
    if not ok then
        if is_first_start then
            M.warn(message)
        end
        return
    end

    if M._server_jdk_cache == nil then
        M._server_jdk_cache = M._find_jdks()
    end
    if M._server_root_dir == nil or M._server_workspace == nil then
        M._server_root_dir = vim.fn.resolve(require('jdtls.setup').find_root({'.git', 'build.gradle', 'gradlew', 'mvnw'}))
        local dir_hash = vim.fn.sha256(M._server_root_dir):sub(1, 32)
        M._server_workspace = M._JDTLS_HOME.."/workspaces/"..vim.fn.fnamemodify(M._server_root_dir, ":t").."-"..dir_hash
    end

    if M._server_cmd_cache == nil then
        local python_path = vim.fn.exepath('python')
        local launcher_py = M._PISTON_JDTLS_HOME .. "/launcher.py"
        M._server_cmd_cache = {
            python_path, launcher_py, "-jdtls", M._JDTLS_HOME, "-data", M._server_workspace
        }
    end
    local config = {
        cmd = vim.tbl_extend("force", {}, M._server_cmd_cache),
        root_dir = M._server_root_dir,
        init_options = {
            bundles = {
                M._PISTON_JDTLS_HOME .. "/dg.jdt.ls.decompiler.common-0.0.3.jar",
                M._PISTON_JDTLS_HOME .. "/dg.jdt.ls.decompiler.fernflower-0.0.3.jar",
            }
        },
        settings = {
            java = {
                import = {
                    gradle = { enabled = true },
                    maven = { enabled = false },
                    exclusions = {}
                },
                format = {
                    settings = {
                        url = 'https://raw.githubusercontent.com/Pistonight/mono-dev/refs/heads/main/java/eclipse-formatter.xml',
                        profile = "MonoDevJavaStyle",
                    }
                },
                signatureHelp = { enabled = true },
                contentProvider = { preferred = 'fernflower' },
                sources = {
                    organizeImports = {
                        starThreshold = 9999;
                        staticStarThreshold = 9999;
                    },
                },
                codeGeneration = {
                    useBlock = true,
                },
                configuration = {
                    runtimes = vim.deepcopy(M._server_jdk_cache)
                }
            }
        }
    }
    if is_first_start then
        M.echo("starting jdtls... (if this takes too long, run JdtlsClean)")
    end
    _jdtls.start_or_attach(config)
end

function M.restart()
    if M._server_workspace == nil then
        M.start_current_buf()
        return
    end
    M.warn("removing jdtls workspace (please be patient :)...")
    pcall(function()
        vim.fs.rm(M._server_workspace, { recursive = true, force = true })
    end)
    require("jdtls.setup").restart()
end

-- PRIVATE

---Check JDTLS installation and either return (false, error) or (true, version)
M._check_cache = nil
function M._check_internal(allow_cache)
    if allow_cache then
        if M._check_cache ~= nil then
            return M._check_cache
        end
    end
    local do_check = function()
        if vim.fn.isdirectory(M._JDTLS_HOME) == 0 then
            return false, "jdtls not installed: JDTLS_HOME not found at "..M._JDTLS_HOME
        end
        local jars = vim.fn.glob(M._JDTLS_HOME.."/plugins/org.eclipse.jdt.ls.core_*.jar", true, true)
        if #jars == 0 then
            return false, "jdtls not installed: cannot find org.eclipse.jdt.ls.core_*.jar"
        end
        if #jars > 1 then
            return false, "found multiple jdtls JARs, please reinstall with JdtlsInstall"
        end
        local jar = jars[1]
        local version = vim.fn.fnamemodify(jar, ":t:r"):match("^org%.eclipse%.jdt%.ls%.core_(.+)$")
        local output = version or "unknown"
        if version ~= M._VERSION .. "." .. M._BUILD then
            output = output .. " (" .. M._VERSION .." available)"
        end
        return true, output
    end
    M._check_cache = do_check()
    return M._check_cache
end

-- Find JDKs by listing the directories that hold them, and parsing each
-- sub-directory to infer the JDK version with best-effort attempt.
--
-- This approach works on my setups because I have a setup script
-- that installs JDKs the same way on all my machines
function M._find_jdks()
    local java_path = vim.fn.resolve(vim.fn.exepath("java"))
    if java_path == "" then
        M.warn("cannot find JDK installations")
        return {}
    end

    -- might be:
    -- <path_to_jdks>/<jdk_name>
    -- <path_to_jdks>/<jdk_name>/Contents/Home -- macos WHYYYYY
    local java_home = vim.fn.fnamemodify(java_path, ":h:h")

    if vim.fn.fnamemodify(java_home, ":t") == "Home" and vim.fn.fnamemodify(java_home, ":h:t") == "Contents"
    then
        java_home = vim.fn.fnamemodify(java_home, ":h:h")
    end
    local jdk_home = vim.fn.fnamemodify(java_home, ":h")

    -- "jdk-17.0.2" -> "17", "openjdk@8.0" -> "1.8", non-LTS -> nil
    local _infer_jdk_version = function (name)
        local trimmed = name:gsub("^%D+", "")
        if vim.startswith(trimmed, "1.8") or vim.startswith(trimmed, "8") then
            return "1.8"
        end
        for _, version in ipairs(M._LTS) do
            if version ~= "1.8" and vim.startswith(trimmed, version) then
                return version
            end
        end
        return nil
    end

    local _prefer = function (name, over)
        -- prefer longer i.e 8.0.100 over 8.0.99
        if #name ~= #over then return #name > #over end
        -- prefer bigger
        return name > over
    end

    local best = {}
    for _, dir in ipairs(vim.fn.glob(jdk_home.."/*", true, true)) do
        for _, j in ipairs({dir, dir.."/Contents/Home"}) do
            if vim.fn.isdirectory(j) == 1 and vim.fn.executable(j.."/bin/java") == 1
            then
                local name = vim.fn.fnamemodify(dir, ":t")
                local version = _infer_jdk_version(name)
                if version and (not best[version] or _prefer(name, best[version].name)) then
                    best[version] = { name = name, path = j }
                end
                break
            end
        end
    end

    local runtimes = {}
    for _, version in ipairs(M._LTS) do
        local jdk = best[version]
        if jdk then
            table.insert(runtimes, {
                name = "JavaSE-"..version,
                path = jdk.path,
            })
        end
    end
    return runtimes
end

---The data directory jdtls uses for a project, derived from the project path
---so each project gets its own isolated workspace
function M._get_workspace(project_dir)
    local dir_hash = vim.fn.sha256(project_dir):sub(1, 32)
    return M._JDTLS_HOME.."/workspaces/"..vim.fn.fnamemodify(project_dir, ":t").."-"..dir_hash
end

-- HELPERS
function M.warn(msg) vim.notify("piston-jdtls: "..msg, vim.log.levels.WARN) end
function M.echo(msg) vim.api.nvim_echo({{"piston-jdtls: "..msg}}, false, {}) end

return M
