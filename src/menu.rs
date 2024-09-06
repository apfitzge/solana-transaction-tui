use tui_menu::{MenuItem, MenuState};

#[derive(Debug, Clone)]
pub enum AppMenuItem {
    Endpoint(&'static str),
}

pub fn menu() -> MenuState<AppMenuItem> {
    let items = vec![MenuItem::group(
        "Endpoint",
        vec![
            MenuItem::item(
                "mainnet-beta",
                AppMenuItem::Endpoint("https://api.mainnet-beta.solana.com"),
            ),
            MenuItem::item(
                "testnet",
                AppMenuItem::Endpoint("https://api.testnet.solana.com"),
            ),
            MenuItem::item(
                "devnet",
                AppMenuItem::Endpoint("https://api.devnet.solana.com"),
            ),
        ],
    )];
    MenuState::new(items)
}
