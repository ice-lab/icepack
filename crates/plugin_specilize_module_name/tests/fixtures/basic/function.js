const { add } = require("./utils");
const { add_3 } = require("./util_1");

export function cal() {
  const result = add(2 + 2);
  console.log("2 + 2 is %s, it is a %s number", result, isNumber(result));
}

export function cal2() {
  return add_3(0);
}
