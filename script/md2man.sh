#!/bin/bash

# Simple markdown to man page converter using sed
# Usage: ./md2man.sh input.md output.1

if [ $# -ne 2 ]; then
    echo "Usage: $0 input.md output.1"
    exit 1
fi

input="$1"
output="$2"

# Extract title from first line
title=$(head -1 "$input" | sed 's/^# \([^ ]*\).*/\1/' | tr '[:lower:]' '[:upper:]')

# Create man page header
cat > "$output" << EOF
.TH ${title} 1

EOF

# Convert using sed
sed '
# Convert headers
/^## / {
    s/^## \(.*\)$/.SH \U\1/
}

# Convert bold text
s/\*\*\([^*]*\)\*\*/\\fB\1\\fR/g

# Convert italic text
s/\*\([^*]*\)\*/\\fI\1\\fR/g

# Convert option lists
/^- \*\*--/ {
    s/^- \*\*\([^ ]*\)\*\*/.TP\n.B --\1/
}

# Handle code blocks (simple version)
/^```/,/^```$/ {
    /^```/d
    /^```$/d
    s/^/    /
}
' "$input" >> "$output"

echo "Converted $input to $output"