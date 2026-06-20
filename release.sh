#!/bin/bash
# Автоматическое повышение версии и создание тега
TYPE=${1:-patch}
node -e "const fs=require('fs'); const semver=require('semver'); const p=JSON.parse(fs.readFileSync('package.json')); p.version=semver.inc(p.version, '$TYPE'); fs.writeFileSync('package.json', JSON.stringify(p, null, 2));"
VERSION=$(node -e "console.log(require('./package.json').version)")
git add package.json
git commit -m "chore: release v$VERSION"
git tag v$VERSION
echo "Версия поднята до $VERSION и помечена тегом!"
