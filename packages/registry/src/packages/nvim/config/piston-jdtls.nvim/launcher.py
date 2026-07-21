"""
usage: python launcher.py -jdtls JDTLS_HOME -data DATA_DIR
"""
import os
import sys
import subprocess
import shutil


def main():
    jdtls_home, data_dir = parse_args()
    jdtls_plugins = os.path.join(jdtls_home, "plugins")
    java_path = shutil.which("java")
    if java_path is None:
        raise Exception("cannot find java")
    launcher = find_launcher(jdtls_plugins)
    if sys.platform == "darwin":
        config_path = os.path.join(jdtls_home, "config_mac")
    elif sys.platform == "win32":
        config_path = os.path.join(jdtls_home, "config_win")
    else:
        config_path = os.path.join(jdtls_home, "config_linux")
    exec_args = [
        '-Declipse.application=org.eclipse.jdt.ls.core.id1',
        '-Dosgi.bundles.defaultStartLevel=4',
        '-Declipse.product=org.eclipse.jdt.ls.core.product',
        '-Dlog.protocol=true',
        '-Dlog.level=ALL',
        '-Xmx4g',
        '--add-modules=ALL-SYSTEM',
        '--add-opens', 'java.base/java.util=ALL-UNNAMED',
        '--add-opens', 'java.base/java.lang=ALL-UNNAMED',
        '-jar', launcher,
        '-configuration', config_path,
        '-data', data_dir
    ]
    print("Starting jdtls with args: " + str(exec_args))
    if os.name == 'posix':
        os.execvp(java_path, exec_args)
    else:
        subprocess.run([java_path] + exec_args)

def parse_args():
    jdtls_home = None
    data_dir = None
    flag = None
    for arg in sys.argv:
        match flag:
            case "-jdtls":
                jdtls_home = arg
            case "-data":
                data_dir = arg
            case _:
                if arg.startswith("-"):
                    flag = arg
                    continue
        flag = None
    if jdtls_home is None:
        raise Exception("missing -jdtls parameter")
    if data_dir is None:
        raise Exception("missing -data parameter")
    return jdtls_home, data_dir


def find_launcher(jdtls_plugins):
    launcher = None
    for jar in os.listdir(jdtls_plugins):
        if jar.startswith("org.eclipse.equinox.launcher_"):
            launcher = os.path.join(jdtls_plugins, jar)
            break
    if launcher is None:
        raise Exception("Cannot find equinox launcher")
    return launcher

if __name__ == "__main__":
    main()
