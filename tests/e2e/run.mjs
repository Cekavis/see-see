import { Launcher } from "@wdio/cli";
import process from "node:process";
import { createServer } from "vite";

const server = await createServer({
  logLevel: "error",
  server: { host: "127.0.0.1", port: 1420, strictPort: true },
});

try {
  await server.listen();
  process.exitCode = (await new Launcher("wdio.conf.ts").run()) ?? 0;
} finally {
  await server.close();
}
