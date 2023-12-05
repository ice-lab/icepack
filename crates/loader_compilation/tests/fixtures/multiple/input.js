import { isWeb, isPHA } from "@uni/env";

if (isWeb && isPHA) {
  console.log("test");
}