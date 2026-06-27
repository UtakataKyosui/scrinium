#!/usr/bin/env python3
"""
PreToolUse hook: rewrite legacy CLI commands to modern alternatives.
  ls   → eza   (modern ls)
  find → fd    (modern find)
  grep → rg    (modern grep / ripgrep)
  cat  → bat   (modern cat)
"""

import json
import re
import shlex
import sys

# Separators that start a new command in a shell string
_SEP_RE = re.compile(r'(&&|\|\||\|(?!\|)|;|\n)')

# find options that make conversion to fd unsafe
_FIND_BAIL_OPTIONS = frozenset({
    '-exec', '-execdir', '-ok', '-okdir',
    '-mtime', '-atime', '-ctime', '-mmin', '-amin', '-cmin',
    '-newer', '-anewer', '-cnewer',
    '-not', '!', '-o', '-or',
    '-prune', '-delete', '-print0',
    '-printf', '-fprintf', '-ls', '-fls',
    '-empty', '-perm', '-user', '-group', '-nouser', '-nogroup',
    '-inum', '-links', '-size',
    '-L', '-P', '-H', '-follow',
    '-mount', '-xdev',
    '-path', '-ipath', '-wholename', '-iwholename',
    '-regex', '-iregex',
    '-depth',
})

# grep options that ripgrep does not accept
_GREP_UNSUPPORTED = frozenset({
    '-P', '--perl-regexp',
    '-G', '--basic-regexp',
    '-z', '--null-data',
    '--include', '--exclude', '--exclude-dir',
    '-f', '--file',
    '-b', '--byte-offset',
    '-T', '--initial-tab',
    '-u', '--unix-byte-offsets',
})

# grep options removed because rg is recursive by default
_GREP_REMOVE = frozenset({'-r', '-R', '--recursive'})


def rewrite_command(cmd: str) -> str | None:
    """Return rewritten command, or None if nothing changed."""
    result = _process_chain(cmd)
    return result if result != cmd else None


def _process_chain(cmd: str) -> str:
    parts = _SEP_RE.split(cmd)
    return ''.join(
        p if _SEP_RE.fullmatch(p) else _rewrite_segment(p)
        for p in parts
    )


def _rewrite_segment(segment: str) -> str:
    stripped = segment.lstrip()
    indent = segment[: len(segment) - len(stripped)]

    if not stripped:
        return segment

    try:
        tokens = shlex.split(stripped, posix=True)
    except ValueError:
        return segment

    if not tokens:
        return segment

    name = tokens[0]

    if name == 'ls':
        return indent + 'eza' + stripped[len('ls'):]

    if name == 'grep':
        return indent + _rewrite_grep(stripped, tokens)

    if name == 'cat':
        return indent + _rewrite_cat(stripped, tokens)

    if name == 'find':
        return indent + _rewrite_find(stripped, tokens)

    return segment


# ── grep → rg ──────────────────────────────────────────────────────────────

def _rewrite_grep(stripped: str, tokens: list) -> str:
    args = tokens[1:]

    for a in args:
        if a in _GREP_UNSUPPORTED:
            return stripped  # unsafe conversion

    if not any(a in _GREP_REMOVE for a in args):
        return 'rg' + stripped[len('grep'):]

    # Remove -r / -R / --recursive (rg is recursive by default)
    filtered = [a for a in args if a not in _GREP_REMOVE]
    return 'rg ' + ' '.join(shlex.quote(a) for a in filtered)


# ── cat → bat ──────────────────────────────────────────────────────────────

def _rewrite_cat(stripped: str, tokens: list) -> str:
    # Heredoc: cat << EOF — bat does not support this
    if '<<' in stripped:
        return stripped

    # Output redirect: cat > file or cat >> file
    if re.search(r'(?<![<>])>{1,2}\s*\S', stripped):
        return stripped

    # No actual file arguments (stdin-only usage)
    file_args = [t for t in tokens[1:] if t != '-' and not t.startswith('-')]
    if not file_args:
        return stripped

    return 'bat' + stripped[len('cat'):]


# ── find → fd ──────────────────────────────────────────────────────────────

def _rewrite_find(stripped: str, tokens: list) -> str:
    args = tokens[1:]

    for a in args:
        if a in _FIND_BAIL_OPTIONS:
            return stripped  # too complex to convert safely

    path = None
    name_pattern = None
    type_arg = None
    maxdepth = None
    case_insensitive = False

    i = 0
    while i < len(args):
        a = args[i]
        if a in ('-name',) and i + 1 < len(args):
            name_pattern = args[i + 1]; i += 2
        elif a == '-iname' and i + 1 < len(args):
            name_pattern = args[i + 1]; case_insensitive = True; i += 2
        elif a == '-type' and i + 1 < len(args):
            type_arg = args[i + 1]; i += 2
        elif a == '-maxdepth' and i + 1 < len(args):
            maxdepth = args[i + 1]; i += 2
        elif not a.startswith('-') and path is None:
            path = a; i += 1
        else:
            return stripped  # unknown option

    parts = ['fd']
    if case_insensitive:
        parts.append('-i')
    parts.append(shlex.quote(name_pattern) if name_pattern else '.')
    if path and path != '.':
        parts.append(shlex.quote(path))
    if type_arg:
        parts += ['--type', type_arg]
    if maxdepth:
        parts += ['--max-depth', maxdepth]

    return ' '.join(parts)


# ── entry point ────────────────────────────────────────────────────────────

def main() -> None:
    try:
        data = json.load(sys.stdin)
    except json.JSONDecodeError as e:
        print(f"modern-cli hook: JSON parse error: {e}", file=sys.stderr)
        sys.exit(1)

    if data.get('tool_name') != 'Bash':
        sys.exit(0)

    original = data.get('tool_input', {}).get('command', '')
    if not original:
        sys.exit(0)

    rewritten = rewrite_command(original)
    if rewritten is None:
        sys.exit(0)

    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "permissionDecisionReason": f"modern-cli: rewritten to modern alternative",
            "updatedInput": {"command": rewritten},
        }
    }))


if __name__ == '__main__':
    main()
