/* global console */

import { chromium } from "playwright-core";
import { createServer } from "vite";
import { runPrimaryFlow } from "./primary-flow.mjs";

const server = await createServer({
  logLevel: "error",
  server: { host: "127.0.0.1", port: 1420, strictPort: true },
});

let browser;

try {
  await server.listen();
  browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  await runPrimaryFlow(page);
  console.log("✓ See See primary desktop flow");
} finally {
  await browser?.close();
  await server.close();
}
