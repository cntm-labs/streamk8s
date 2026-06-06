use streamk8s_sdk::{get_resource_count, notify, register_plugin, Plugin};

#[derive(Default)]
struct MyPlugin;

impl Plugin for MyPlugin {
    fn init(&self) {
        // Called on load
        let count = get_resource_count();
        if count > 0 {
            notify(1); // Notify success code
        }
    }

    fn on_action(&self, name: &str, _payload: &str) {
        if name == "refresh" {
            notify(2); // Notify refresh triggered
        }
    }
}

register_plugin!(MyPlugin);
