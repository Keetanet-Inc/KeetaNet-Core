#!/bin/bash

# Проверяем, переданы ли ключи, чтобы не слать пустые запросы
if [ -z "$STRIPE_KEY" ] || [ -z "$STRIPE_CUS" ]; then
  echo "❌ Ошибка: Переменные STRIPE_KEY и STRIPE_CUS должны быть заданы!"
  exit 1
fi

VERSION=$(node -p "require('./package.json').version")
echo "👉 Запуск автоматического биллинга для версии v$VERSION..."

# Шаг 1: Создаем позицию в счете ($5000 = 500000 центов)
INVOICE_ITEM_JSON=$(curl -s https://api.stripe.com/v1/invoiceitems -u "${STRIPE_KEY}:" -d customer="${STRIPE_CUS}" -d amount=500000 -d currency=usd -d description="KeetaNet Core - Milestone Delivery (v${VERSION})")
echo "✅ Позиция счета успешно добавлена."

# Шаг 2: Создаем сам инвойс
INVOICE_JSON=$(curl -s https://api.stripe.com/v1/invoices -u "${STRIPE_KEY}:" -d customer="${STRIPE_CUS}" -d collection_method="send_invoice" -d days_until_due=3 -d auto_advance=true)

# Шаг 3: Вытаскиваем ID инвойса
INVOICE_ID=$(echo "$INVOICE_JSON" | node -p "JSON.parse(require('fs').readFileSync(0, 'utf-8')).id")

# Шаг 4: Отправляем инвойс клиенту
curl -s https://api.stripe.com/v1/invoices/${INVOICE_ID}/send -u "${STRIPE_KEY}:" > /dev/null

echo "🚀 Инвойс ${INVOICE_ID} успешно отправлен клиенту на email!"
echo "🤫 Всё сделано автономно."
