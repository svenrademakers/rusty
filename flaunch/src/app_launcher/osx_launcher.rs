use app_launcher::AppLauncherT;
use fruitbasket::*;
use std::sync::mpsc::{self, Receiver, Sender};

pub struct OsxLauncher {
    fruit_app: FruitApp,
    trampoline: Trampoline,
}

impl OsxLauncher {
    pub fn new() -> Self {
        let names = format!("{}{}", app_meta::APP_INFO.author, app_meta::APP_INFO.name);

        let mut trampoline =
            fruitbasket::Trampoline::new(app_meta::APP_NAME, app_meta::APP_NAME, &names);
        trampoline.version(app_meta::VERSION);
        trampoline.icon(app_meta::ICON);

        let nsapp = trampoline
            .build(fruitbasket::InstallDir::UserApplications)
            .unwrap();
        nsapp.set_activation_policy(fruitbasket::ActivationPolicy::Regular);

        OsxLauncher {
            fruit_app: nsapp,
            trampoline: trampoline,
        }
    }
}

impl AppLauncherT for OsxLauncher {
    fn set_resources() {}

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
    fn configure_url_scheme(scheme: &str, description: &str) {}
}
