mod app_launcher;
mod system_tray;
use app_launcher::*;
use flaunch_core::settings::*;
use flaunch_core::{load_flaunch_core, logging::*};
use system_tray::*;
// static mut APPLICATION_BUILDER: FLaunchApplication = FLaunchApplication::new("Flaunch");

// // fn get_bundle_url() -> String {
// //     let uritypes = "<key>CFBundleURLTypes</key>
// // 	<array>
// // 		<dict>
// // 			<key>CFBundleURLName</key>
// // 			<string>Visual Studio URI Handler for vscode:// URIs</string>
// // 			<key>CFBundleURLSchemes</key>
// // 			<array>
// // 				<string>vscode</string>
// // 			</array>
// // 		</dict>
// // 	</array>".to_string()
// // }

fn main() {
    let (script_engine, settings) = load_flaunch_core();

    let mut launcher = AppLauncher::new();
    launcher.build();
    let mut system_tray = launcher.build_system_tray();

    let cb: NSCallback = Box::new(move |_sender, _tx| {
        let path = format!(
            "vscode://file/{}",
            master_settings().to_string_lossy().to_string()
        );
        //let res = libc::open(path.as_ptr() as *mut c_char, 1);
        debug!("{}", path);
    });
    let _ = system_tray.add_item(None, "Open Config", cb, false);
    system_tray.add_separator();
    system_tray.add_label("goed verhaal");
    system_tray.add_quit("Quit");

    launcher.run();
}
