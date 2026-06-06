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

    /// Fetches the length of the resource details JSON manifest.
    pub fn get_resource_details_len(
        kind_ptr: *const u8,
        kind_len: usize,
        ns_ptr: *const u8,
        ns_len: usize,
        name_ptr: *const u8,
        name_len: usize,
    ) -> i32;
}

/// A safe wrapper to get the current resource count.
pub fn get_resource_count() -> i32 {
    unsafe { get_k8s_resources_count() }
}

/// A safe wrapper to trigger a host notification.
pub fn notify(code: i32) {
    unsafe { show_notification(code) }
}

/// A safe wrapper to fetch deep resource details.
pub fn fetch_resource_details(kind: &str, ns: &str, name: &str) -> String {
    // For Milestone 18, we demonstrate the plumbing.
    // In a final SDK, this would allocate a buffer, call the len function,
    // and then call a 'read_resource_details' function.
    // For now, return a placeholder string or implement the first part of the flow.
    unsafe {
        get_resource_details_len(
            kind.as_ptr(),
            kind.len(),
            ns.as_ptr(),
            ns.len(),
            name.as_ptr(),
            name.len(),
        );
    }
    format!("MANIFEST FOR {}/{}", ns, name)
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
