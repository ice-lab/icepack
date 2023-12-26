const env = {
    isWeb: import.meta.renderer === "client" && import.meta.target === "web",
    isClient: import.meta.renderer === "client",
    isNode: import.meta.renderer === "server",
    isWeex: import.meta.renderer === "client" && import.meta.target === "weex",
    isKraken: import.meta.renderer === "client" && import.meta.target === "kraken",
    isMiniApp: false,
    isByteDanceMicroApp: false,
    isBaiduSmartProgram: false,
    isKuaiShouMiniProgram: false,
    isWeChatMiniProgram: false,
    isQuickApp: false,
    isPHA: import.meta.renderer === "client" && import.meta.target === "web" && typeof pha === "object",
    isWindVane: import.meta.renderer === "client" && /.+AliApp\((\w+)\/((?:\d+\.)+\d+)\).* .*(WindVane)(?:\/((?:\d+\.)+\d+))?.*/.test(typeof navigator ? navigator.userAgent || navigator.swuserAgent : "") && typeof WindVane !== "undefined" && typeof WindVane.call !== "undefined",
    isFRM: false
};
if (env.isWeb) {
    console.log("test");
}
