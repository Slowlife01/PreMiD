import "./app.postcss";

import App from "./App.svelte";
import "@fontsource-variable/outfit";

const app = new App({
  target: document.getElementById("app"),
});

export default app;
