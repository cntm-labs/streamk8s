use sysinfo::System;

pub struct Profiler;

impl Profiler {
    pub fn scan_heavy_apps(sys: &System) -> Vec<String> {
        let heavy_signatures = [
            "blender",
            "unity",
            "unreal",
            "cyberpunk",
            "photoshop",
            "premiere",
        ];
        let mut detected = Vec::new();

        for (_pid, process) in sys.processes() {
            let name = process.name().to_lowercase();
            if heavy_signatures.iter().any(|&s| name.contains(s)) {
                detected.push(name);
            }
        }
        detected
    }
}
