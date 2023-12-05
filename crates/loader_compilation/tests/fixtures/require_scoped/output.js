var isNode = import.meta.renderer === "server";
function check() {
    var isWeb = import.meta.renderer === "client" && import.meta.target === "web";
    if (isWeb) {
        var isServer = import.meta.renderer === "server";
        var isClient = import.meta.renderer === "client";
        console.log("test");
    }
}
