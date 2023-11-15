import { copyAndCleanUp, getGithubInfo } from "./github.mjs";

const { temp, dest } = getGithubInfo();
copyAndCleanUp(temp, dest);
