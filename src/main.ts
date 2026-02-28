import { mount } from "svelte";

import App from "./App.svelte";

import "./app.css";

/* eslint-disable @typescript-eslint/no-non-null-assertion */
const app = mount(App, {
  target: document.getElementById("app")!,
});
/* eslint-enable @typescript-eslint/no-non-null-assertion */

export default app;
