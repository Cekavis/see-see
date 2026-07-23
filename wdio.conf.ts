export const config = {
  runner: "local",
  specs: ["./tests/e2e/**/*.spec.ts"],
  maxInstances: 1,
  capabilities: [{ browserName: "chrome" }],
  logLevel: "error",
  framework: "mocha",
  reporters: ["spec"],
  waitforTimeout: 5_000,
};
