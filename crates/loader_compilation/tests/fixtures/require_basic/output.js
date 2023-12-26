var isWeb = import.meta.renderer === "client" && import.meta.target === "web";
if (isWeb) {
    console.log("test");
}
