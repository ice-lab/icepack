const { isNode } = require("@uni/env");
function check() {
  const { isWeb } = require("@uni/env");
  if (isWeb) {
    const { isServer, isClient } = require("@uni/env");
    console.log("test");
  }
}