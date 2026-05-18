## Description
This PR completes Milestone 12, establishing the foundation for the StreamK8s Marketplace. It introduces a robust, filesystem-based plugin loading system and a dynamic UI rendering engine.

## Key Changes
- **WASM-based SDK Foundation:** Implemented a PluginManager in Rust that discovers and manages plugins stored in ~/.config/streamk8s/plugins/.
- **TOML-driven UI:** Plugins can now define their interactive components via a simple extension.toml manifest, which is dynamically rendered in the frontend.
- **Gemini AI Integration:** Added a dedicated GeminiProvider for resource analysis, improving the Smart Advice capabilities with specialized SRE insights.
- **Marketplace UI:** Created MarketplaceView and PluginRenderer components to allow users to browse and interact with installed extensions.
- **Verification:** Included a Hello K8s World dummy plugin for end-to-end verification of the loader and UI generator.

## How to Test
1. Ensure the dummy plugin is created at ~/.config/streamk8s/plugins/hello-world/extension.toml.
2. Open the application and navigate to the Marketplace tab.
3. You should see Hello K8s World extension.
4. Interact with the Developer Name input and click Say Hello to see the mock WASM action output.

Fixes #10
