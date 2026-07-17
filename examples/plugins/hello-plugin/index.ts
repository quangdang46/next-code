/** dual-read: legacy plugins may use `jcode` instead of `nextcode`. */
// Hello Plugin — real working example for next-code's plugin system.
//
// This file is:
//   1. Discovered by next-code-plugin-runtime::PluginLoader::scan_directory
//      (looks for *.ts files in ~/.next-code/plugins/ (or legacy ~/.jcode/plugins/) or configured plugin_dirs)
//   2. Transpiled by next-code-plugin-runtime::Transpiler (SWC, TypeScript -> JS)
//   3. Evaluated by next-code-plugin-runtime::SandboxContext (QuickJS)
//   4. The `nextcode` object is injected by next-code-plugin-runtime::api::PluginApiBindings
//      (dual-read: legacy plugins may use `jcode` / `__jcode_api`)
//
// Available APIs on `nextcode` (plugin global):
//   nextcode.on(event, handler)              — register event handler
//   nextcode.registerTool(toolDef)           — register a tool the LLM can call
//   nextcode.logger.{info,warn,error,debug}  — log to next-code's tracing system
//   nextcode.kv.{get,set}                    — per-plugin key/value store
//   nextcode.sleep(ms)                       — sleep (capped at 5s)
//   nextcode.uuid()                          — generate a UUID
//   nextcode.cwd                             — current working directory (string)

// Subscribe to SessionStart event.
nextcode.on("SessionStart", function(event) {
    nextcode.logger.info("hello-plugin: session starting, id=" + event.sessionId);
});

// Subscribe to PreToolUse event.
nextcode.on("PreToolUse", function(event) {
    nextcode.logger.info("hello-plugin: tool about to be called: " + event.toolName);
});

// Register a tool the LLM could (eventually) call.
nextcode.registerTool({
    name: "hello",
    description: "Say hello and return a greeting",
});

// Use the per-plugin key/value store.
next-code.kv.set("hello-plugin:loaded-at", "test");

// Generate a UUID.
var instanceUuid = nextcode.uuid();
nextcode.logger.info("hello-plugin: instance uuid = " + instanceUuid);

// Final log line — proves the plugin's top-level code ran to completion.
nextcode.logger.info("hello-plugin: registered 2 handlers + 1 tool stub, kv set done");
