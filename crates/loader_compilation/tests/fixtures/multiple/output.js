var isWeb = import.meta.renderer === "client" && import.meta.target === "web";
var isPHA = import.meta.renderer === "client" && import.meta.target === "web" && typeof pha === "object";
if (isWeb && isPHA) {
    console.log("test");
}
