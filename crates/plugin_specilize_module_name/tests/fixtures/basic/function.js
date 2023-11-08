const { add } = require("./utils");
const { isNumber } = require("lodash");


export function cal() {
  const result = add(2 + 2);
  console.log("2 + 2 is %s, it is a %s number", result, isNumber(result));
}
