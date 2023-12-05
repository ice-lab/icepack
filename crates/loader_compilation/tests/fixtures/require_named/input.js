const { isWeb: web } = require("@uni/env");

if (web) {
  console.log("test");
}