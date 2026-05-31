/// The base trait that every StreamK8s plugin must implement.
pub trait Plugin {
    /// Called when the plugin is first loaded into the workspace.
    fn init(&self);

    /// Called when a UI action (button click, input) is triggered.
    fn on_action(&self, name: &str, payload: &str);
}

// Low-level WASM imports provided by the StreamK8s host.
extern "C" {
    /// Returns the number of resources found in the active cluster context.
    pub fn get_k8s_resources_count() -> i32;

    /// Triggers a system notification via the host application.
    pub fn show_notification(code: i32);
}

/// A safe wrapper to get the current resource count.
pub fn get_resource_count() -> i32 {
    unsafe { get_k8s_resources_count() }
}

/// A safe wrapper to trigger a host notification.
pub fn notify(code: i32) {
    unsafe { show_notification(code) }
}

/// Macro to register a plugin implementation.
/// This handles the `no_mangle` exports that the WASM host expects.
#[macro_export]
macro_rules! register_plugin {
    ($t:ty) => {
        thread_local! {
            static PLUGIN_INSTANCE: $t = <$t>::default();
        }

        #[no_mangle]
        pub extern "C" fn init() {
            PLUGIN_INSTANCE.with(|p| p.init());
        }

        #[no_mangle]
        pub extern "C" fn on_action(
            name_ptr: *const u8,
            name_len: usize,
            payload_ptr: *const u8,
            payload_len: usize,
        ) {
            // Memory safety: In a real SDK we would decode the string pointers here.
            // For Milestone 16, we demonstrate the macro structure.
            PLUGIN_INSTANCE.with(|p| p.on_action("action", "payload"));
        }
    };
}
