var addon = require('../native');

console.dir(addon.latestOpenJdk());
console.dir(addon.scanFileSystem('/usr/lib/jvm'));
