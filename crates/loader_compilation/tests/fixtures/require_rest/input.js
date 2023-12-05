const { isNode, ...rest } = require("@uni/env");

if (rest.isWeb || isNode) {
  console.log("test");
}