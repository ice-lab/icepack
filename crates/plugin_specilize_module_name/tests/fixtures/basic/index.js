console.log("Hello World!");

const { cal } = require("./function")
const { add } = require("./utils");

cal();

console.log("1 + 1 is %s", add(1, 1));
console.log("The 'string' is String type is %s", isString('string'));
