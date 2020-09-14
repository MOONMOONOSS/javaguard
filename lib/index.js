var addon = require('../native');

console.dir(addon.latestOpenJdk());
console.dir(addon.scanFileSystem('/usr/lib/jvm'));
console.dir(addon.javaValidate('/usr/lib/jvm'));
