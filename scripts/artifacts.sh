mkdir -p dist
find artifacts -name '*' -type f | xargs -I{} mv {} dist