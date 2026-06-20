const satisfies = require('semver/functions/satisfies');
const coerce = require('semver/functions/coerce');

// Диапазон версии, которой должен соответствовать узел (например, выше 2.5.0)
const REQUIRED_RANGE = '^2.5.0'; 

function checkVersion(rawVersion) {
  const cleanVersion = coerce(rawVersion);
  if (!cleanVersion) return false;

  return satisfies(cleanVersion, REQUIRED_RANGE);
}

module.exports = { checkVersion };
