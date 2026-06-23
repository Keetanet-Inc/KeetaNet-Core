#!/bin/bash

# Останавливать скрипт при любой ошибке
set -e

# Проверяем, передан ли тип изменения (major, minor, patch)
if [ -z "$1" ]; then
  echo "❌ Ошибка: Укажите тип релиза (major, minor, patch)"
  echo "Пример: ./release.sh minor"
  exit 1
fi

RELEASE_TYPE=$1

# 1. Считываем текущую версию из package.json
if [ ! -f "package.json" ]; then
  echo "❌ Ошибка: Файл package.json не найден!"
  exit 1
fi

CURRENT_VERSION=$(node -p "require('./package.json').version")
echo "👉 Текущая версия ядра: $CURRENT_VERSION"

# 2. Рассчитываем новую версию с помощью утилиты semver
NEW_VERSION=$(npx semver "$CURRENT_VERSION" -i "$RELEASE_TYPE")
echo "🚀 Новая целевая версия: $NEW_VERSION"

# 3. Обновляем версию в package.json
node -e "
  const fs = require('fs');
  const pkg = require('./package.json');
  pkg.version = '$NEW_VERSION';
  fs.writeFileSync('./package.json', JSON.stringify(pkg, null, 2) + '\n');
"
echo "✅ package.json успешно обновлен до версии $NEW_VERSION"

# 4. Фиксация изменений в Git
git add package.json
git commit -m "build(release): bump version to $NEW_VERSION"
git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION"
echo "✅ Изменения зафиксированы в Git, создан тег v$NEW_VERSION"

# 5. Автоматический деплой ядра в Cloudflare
echo "🌐 Запуск деплоя в Cloudflare..."
npx wrangler deploy
echo "✅ Код успешно развернут в Cloudflare Network"

# 6. Автоматический запуск боевого биллинга
echo "💰 Отправка инвойса клиенту за обновление..."
if [ -f "./trigger-billing.sh" ]; then
  # Передаем секретные ключи окружения внутрь скрипта биллинга
  STRIPE_KEY="$STRIPE_KEY" STRIPE_CUS="$STRIPE_CUS" ./trigger-billing.sh
  echo "✅ Процесс биллинга успешно завершен."
else
  echo "⚠️ Предупреждение: Скрипт trigger-billing.sh не найден."
fi

echo "---"
echo "🎉 Релиз $NEW_VERSION полностью завершен. Конвейер отработал автономно."
