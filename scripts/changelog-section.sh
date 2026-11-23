#!/bin/sh
# Print the body of one section of a changelog, without its heading.
# Use: scripts/changelog-section.sh VERSION [FILE]
#   VERSION is the version in the heading, for example 1.1 or 5.6.0, or the word Unreleased.
#   FILE is the changelog. The default is CHANGELOG.md, then docs/CHANGELOG.md, at the root of the repository.
# The script accepts these headings of level 2:
#   ## [X.Y] - DATE, ## [X.Y.Z] - DATE, ## X.Y.Z (DATE), ## X.Y.Z - DATE,
#   ## Unreleased and ## [Unreleased].
# The section stops at the next heading of level 2, or at a link reference definition of a version,
# such as "[1.1]: https://..." or "[Unreleased]: https://...". These definitions are the footer of the file.
# The script removes the blank lines at the start and at the end of the body.
# Exit status: 0 with the body on stdout; 1 when the section is absent or empty; 2 for a usage error.
# This file is the reference copy for the repositories of ProcurAuth.
set -eu

usage() {
    echo "use: changelog-section.sh VERSION [FILE]" >&2
    exit 2
}

[ "$#" -ge 1 ] && [ "$#" -le 2 ] || usage
version=$1
[ -n "$version" ] || usage

if [ "$#" -eq 2 ]; then
    file=$2
else
    root="$(dirname "$0")/.."
    if [ -f "$root/CHANGELOG.md" ]; then
        file="$root/CHANGELOG.md"
    else
        file="$root/docs/CHANGELOG.md"
    fi
fi
if [ ! -f "$file" ] || [ ! -r "$file" ]; then
    echo "error: cannot read the file $file" >&2
    exit 2
fi

# The awk program compares strings, not patterns: a dot in VERSION is not a wildcard.
# It keeps the lines of the section, and the END rule prints them from the first to the last line with text.
# The exit status of awk is 1 when the section is absent or holds only blank lines.
if ! awk -v want="$version" '
    # Give the version of a heading of level 2, or "" when the heading has no known form.
    function heading_version(line,    rest, bracket, space, found, tail) {
        rest = substr(line, 4)
        sub(/[ \t\r]+$/, "", rest)
        if (rest == "Unreleased" || rest == "[Unreleased]") return "Unreleased"
        if (substr(rest, 1, 1) == "[") {
            bracket = index(rest, "]")
            if (bracket < 3) return ""
            found = substr(rest, 2, bracket - 2)
            tail = substr(rest, bracket + 1)
        } else {
            space = index(rest, " ")
            if (space < 2) return ""
            found = substr(rest, 1, space - 1)
            tail = substr(rest, space)
        }
        if (tail ~ /^ - [^ ]/ || tail ~ /^ \([^()]+\)$/) return found
        return ""
    }
    /^## / {
        if (inside) exit
        if (heading_version($0) == want) inside = 1
        next
    }
    # The footer of Keep a Changelog: the link reference definitions of the versions.
    inside && /^\[(Unreleased|v?[0-9]+\.[0-9][^]]*)\]: / { exit }
    inside {
        count++
        kept[count] = $0
        if ($0 ~ /[^ \t\r]/) {
            if (!first) first = count
            last = count
        }
    }
    END {
        if (!first) exit 1
        for (i = first; i <= last; i++) print kept[i]
    }' "$file"; then
    echo "error: the section '$version' of $file is absent or empty" >&2
    exit 1
fi
