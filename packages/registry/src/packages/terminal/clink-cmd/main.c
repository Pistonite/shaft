#define UNICODE
#define _UNICODE
#include <stddef.h>
#include <stdio.h>
#include <windows.h>

// 32766, just minus 5 for fun
#define CLINK_CMD_MAX_COMMAND_LEN 32761
#define MY_LSTRLEN(x) (sizeof(x) / sizeof(WCHAR) - 1)
#define MY_WRITE(out, size, literal)                                           \
    MY_WRITE_LEN(out, size, literal, MY_LSTRLEN(literal))
#define MY_WRITE_LEN(out, size, literal, literal_length)                       \
    do {                                                                       \
        if (size + literal_length >= CLINK_CMD_MAX_COMMAND_LEN) {              \
            return 1;                                                          \
        }                                                                      \
        errno_t error =                                                        \
            wcsncpy_s(out + size, CLINK_CMD_MAX_COMMAND_LEN - 1 - size,        \
                      literal, literal_length);                                \
        if (error != 0) {                                                      \
            return 1;                                                          \
        }                                                                      \
        size += literal_length;                                                \
    } while (0)

BOOL WINAPI CtrlHandler(DWORD fdwCtrlType) {
    UNREFERENCED_PARAMETER(fdwCtrlType);
    // do nothing; let the child process handle it
    return TRUE;
}

// decls
int wmain(void);
int EnsureNullTerminatedAndExecCmdReplace(LPWSTR realcmd, size_t size);
int ExecCmdReplaceWrapped(LPWSTR lpcommandline);
int ExecCmdReplace(LPWSTR lpcommandline);


// impls
int wmain(void) {
    LPWSTR lpcommandline = GetCommandLineW();
    // Skip past the executable name to get the arguments
    if (*lpcommandline == L'"') {
        ++lpcommandline;
        while (*lpcommandline != L'\0' && *lpcommandline != L'"')
            ++lpcommandline;
        if (*lpcommandline == L'"')
            ++lpcommandline;
    } else {
        while (*lpcommandline != L'\0' && *lpcommandline != L' ' &&
               *lpcommandline != L'\t')
            ++lpcommandline;
    }
    while (*lpcommandline == L' ' || *lpcommandline == L'\t')
        ++lpcommandline;
    size_t size = 0;
    BOOL is_c_or_k = FALSE;
    BOOL preserve_quotes = TRUE;
    BOOL is_slash = FALSE;
    WCHAR* remaining = lpcommandline;
    for (; *remaining != L'\0'; ++remaining, ++size) {
        if (size >= CLINK_CMD_MAX_COMMAND_LEN) {
            return 1;
        }

        WCHAR c = *remaining;
        if (c == L'/') {
            is_slash = TRUE;
            continue;
        }
        if (!is_slash) {
            continue;
        }
        is_slash = FALSE;
        switch (c) {
        case L'c':
        case L'C':
        case L'k':
        case L'K':
            ++remaining;
            is_c_or_k = TRUE;
            break;
        case L's':
        case L'S':
            // - preserve if: no /S switch
            preserve_quotes = FALSE;
            continue;
        case L'?': // help
            return ExecCmdReplaceWrapped(lpcommandline);
        default:
            continue;
        }
        break;
    }
    // skip leading whitespaces in the command
    for (; *remaining == L' ' || *remaining == L'\t'; ++remaining, ++size) {
        if (size >= CLINK_CMD_MAX_COMMAND_LEN) {
            return 1;
        }
    }



    BOOL has_s = !preserve_quotes;

    if (*remaining != L'"') {
        // if command does not start with quote, then quote will be
        // preserved anyway
        preserve_quotes = TRUE;
    } else if (preserve_quotes) {
        WCHAR* remaining_probe = remaining;
        BOOL found_whitespace = FALSE;
        int quote_counts = 0;
        size_t size_probe = size;
        for (; *remaining_probe != L'\0'; ++remaining_probe, ++size_probe) {
            if (size_probe >= CLINK_CMD_MAX_COMMAND_LEN) {
                return 1;
            }

            WCHAR c = *remaining_probe;
            if (c == L'"') {
                ++quote_counts;
                if (quote_counts > 2) {
                    break;
                }
            }
            if (quote_counts == 1) {
                if (c == L'&' || c == L'<' || c == L'>' || c == L'(' ||
                    c == L')' || c == L'@' || c == L'^' || c == L'|') {
                    // - preserve if : no special characters between quotes
                    break;
                }
                if (c == L' ' || c == '\t') {
                    found_whitespace = TRUE;
                }
            }
        }
        // - preserve if: exactly 2 quotes
        if (quote_counts != 2) {
            preserve_quotes = FALSE;
        }
        // - preserve if: there are whitespace between quotes
        if (!found_whitespace) {
            preserve_quotes = FALSE;
        }

        // we are not checking the last condition:
        // - the string between the 2 quote characters is the name of an executable file
        //
        // it should be fine in 99% of the cases
    }

    // build the command
    WCHAR realcmd[CLINK_CMD_MAX_COMMAND_LEN];
    realcmd[CLINK_CMD_MAX_COMMAND_LEN - 1] = L'0';
    size = 0;
    MY_WRITE(realcmd, size, CMD_EXECUTABLE);
    MY_WRITE(realcmd, size, L" ");
    if (!has_s) {
        MY_WRITE(realcmd, size, L"/S");
    }
    if (!is_c_or_k) {
        MY_WRITE(realcmd, size, L"/K");
    }
    // this would give me everything until after the /c or /k (or not if remaining is empty)
    MY_WRITE_LEN(realcmd, size, lpcommandline, remaining - lpcommandline);

    MY_WRITE(realcmd, size, L" \"");
    MY_WRITE(realcmd, size, CLINK_INJECT);

    if (!is_c_or_k) {
        // no command in the input, just run the inject payload
        MY_WRITE(realcmd, size, L"\"");
        return EnsureNullTerminatedAndExecCmdReplace(realcmd, size);
    }

    // splice in the command
    MY_WRITE(realcmd, size, L" && ");
    if (preserve_quotes) {
        // remaining is something like "xxx" foo bar
        // we will wrap it: ""clink.exe" inject && "init.cmd" && "xxx" foo bar"
        MY_WRITE_LEN(realcmd, size, remaining,
                     wcsnlen(remaining, CLINK_CMD_MAX_COMMAND_LEN - 1 - size));
    } else {
        // we know first char is quote, strip it
        ++remaining;
        // remaining is something like echo "b" adfa"ar
        // we need to splice out the last quote
        // so it would be: ""clink.exe" inject && "init.cmd" && echo "b" adfaar"
        WCHAR* last_quote_remaining = NULL;
        size_t bound = 0;
        for (WCHAR* remaining_probe = remaining; *remaining_probe != '\0';
             ++remaining_probe, ++bound) {
            if (bound >= CLINK_CMD_MAX_COMMAND_LEN) {
                return 1;
            }
            if (*remaining_probe == L'"') {
                last_quote_remaining = remaining_probe;
            }
        }
        if (last_quote_remaining == NULL) {
            // no quotes, just write the remaining command
            MY_WRITE_LEN(
                realcmd, size, remaining,
                wcsnlen(remaining, CLINK_CMD_MAX_COMMAND_LEN - 1 - size));
        } else {
            // this would give me everything until the last quote
            MY_WRITE_LEN(realcmd, size, remaining,
                         last_quote_remaining - remaining);
            // then finish the rest
            ++last_quote_remaining;
            MY_WRITE_LEN(realcmd, size, last_quote_remaining,
                         wcsnlen(last_quote_remaining,
                                 CLINK_CMD_MAX_COMMAND_LEN - 1 - size));
        }
    }
    // close the quote
    MY_WRITE(realcmd, size, L"\"");

    return EnsureNullTerminatedAndExecCmdReplace(realcmd, size);
}

int EnsureNullTerminatedAndExecCmdReplace(LPWSTR realcmd, size_t size) {
    if (size >= CLINK_CMD_MAX_COMMAND_LEN) {
        return 1;
    }
    realcmd[size] = L'\0';
    return ExecCmdReplace(realcmd);
}

int ExecCmdReplaceWrapped(LPWSTR lpcommandline) {
    WCHAR realcmd[CLINK_CMD_MAX_COMMAND_LEN];
    realcmd[CLINK_CMD_MAX_COMMAND_LEN - 1] = L'0';
    size_t size = 0;
    MY_WRITE(realcmd, size, CMD_EXECUTABLE);
    MY_WRITE(realcmd, size, L" ");
    MY_WRITE_LEN(realcmd, size, lpcommandline,
                 wcsnlen(lpcommandline, CLINK_CMD_MAX_COMMAND_LEN - 1 - size));
    return ExecCmdReplace(realcmd);
}

#if PRINT_INSTEAD_OF_EXEC == 1
int ExecCmdReplace(LPWSTR lpcommandline) {
    wprintf(L"%s\n", lpcommandline);
    return 0;
}
#else
int ExecCmdReplace(LPWSTR lpcommandline) {
    SetConsoleCtrlHandler(CtrlHandler, TRUE /*add*/);
    STARTUPINFOW si;
    ZeroMemory(&si, sizeof(si));
    si.cb = sizeof(si);
    si.dwFlags = STARTF_USESTDHANDLES;
    si.hStdInput = GetStdHandle(STD_INPUT_HANDLE);
    si.hStdOutput = GetStdHandle(STD_OUTPUT_HANDLE);
    si.hStdError = GetStdHandle(STD_ERROR_HANDLE);

    PROCESS_INFORMATION pi;
    ZeroMemory(&pi, sizeof(pi));

    BOOL ok = CreateProcessW(CMD_EXECUTABLE, lpcommandline,
                             NULL, // lpProcessAttributes
                             NULL, // lpThreadAttributes
                             TRUE, // bInheritHandles
                             0,    // dwCreationFlags
                             NULL, // lpEnvironment (inherit)
                             NULL, // lpCurrentDirectory (inherit)
                             &si,  // lpStartupInfo
                             &pi   // lpProcessInformation
                             );
    if (!ok) {
        return GetLastError();
    }
    WaitForSingleObject(pi.hProcess, INFINITE);

    CloseHandle(pi.hThread);
    CloseHandle(pi.hProcess);
    return 0;
}
#endif
