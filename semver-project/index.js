const { checkVersion } = require('./semverChecker');

// Имитируем входящие версии от разных узлов сети
const incomingVersions = [
  'v2.6.1',          // Валидная, подходит под ^2.5.0
  '3.0.0',           // Мажорная выше — не подойдет (breaking изменения)
  '  =v2.5.5   ',    // Грязная строка — будет очищена коерсом и пройдет
  'invalid-string'   // Сломанная строка — вернет false вместо падения приложения
];

incomingVersions.forEach(ver => {
  const isCompatible = checkVersion(ver);
  console.log(`Версия: "${ver.trim()}" -> Совместима? ${isCompatible ? '✅ ДА' : '❌ НЕТ'}`);
});
