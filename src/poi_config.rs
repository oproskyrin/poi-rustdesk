//! Встроенная конфигурация POI Connect: сервер организации по умолчанию и бренд.
//!
//! Применяется один раз при запуске любого процесса (UI, служба, трей), если
//! конфиг ещё не задан другим способом:
//!   1. опциями, записанными установщиком POI Connect или `--import-config`;
//!   2. конфигом, зашитым в имя файла (`app-host=...,key=...,.exe` —
//!      см. `custom_server.rs`, механизм custom client RustDesk);
//!   3. подписанным `custom.txt` рядом с exe (см. `common.rs::load_custom_client`).
//! Существующий конфиг никогда не перекрывается — пользователь и установщик
//! всегда имеют приоритет над значениями по умолчанию.
//!
//! Это позволяет голому `rustdesk.exe` (без установщика и переименования)
//! сразу подключаться к серверу организации.

use hbb_common::{config::Config, log};

pub const POI_APP_NAME: &str = "POI Connect";

/// ID/rendezvous-сервер hbbs (порт 21116 TCP+UDP по умолчанию).
pub const POI_DEFAULT_HOST: &str = "172.16.0.63";
/// Ретранслятор hbbr (порт 21117 по умолчанию).
pub const POI_DEFAULT_RELAY: &str = "172.16.0.63";
/// Панель POI Connect (API сервера).
pub const POI_DEFAULT_API: &str = "http://172.16.0.63:8080";
/// Публичный ключ сервера RustDesk (закладывается при развёртывании hbbs/hbbr).
pub const POI_DEFAULT_KEY: &str = "TKu7I8JdohWj";

pub fn apply_embedded_defaults() {
    apply_embedded_app_name();
    apply_embedded_policies();
    if have_server_config() {
        return;
    }
    Config::set_option("custom-rendezvous-server".into(), POI_DEFAULT_HOST.into());
    Config::set_option("relay-server".into(), POI_DEFAULT_RELAY.into());
    Config::set_option("api-server".into(), POI_DEFAULT_API.into());
    Config::set_option("key".into(), POI_DEFAULT_KEY.into());
    log::info!("POI Connect: применены встроенные настройки сервера по умолчанию");
}

/// Политики UI (тот же механизм, что у custom client генератора RustDesk:
/// HARD_SETTINGS/BUILTIN_SETTINGS). `or_insert` — подписанный custom.txt,
/// если появится, сохраняет приоритет.
fn apply_embedded_policies() {
    let mut builtin = hbb_common::config::BUILTIN_SETTINGS.write().unwrap();
    // Вкладка «Сеть» и поля сервера: сервер зашит, менять его в UI не нужно.
    builtin
        .entry("hide-network-settings".to_owned())
        .or_insert_with(|| "Y".to_owned());
    builtin
        .entry("hide-server-settings".to_owned())
        .or_insert_with(|| "Y".to_owned());
    builtin
        .entry("hide-proxy-settings".to_owned())
        .or_insert_with(|| "Y".to_owned());
    drop(builtin);
    // Вход в аккаунт RustDesk: замкнутый контур со своей панелью.
    hbb_common::config::HARD_SETTINGS
        .write()
        .unwrap()
        .entry("disable-account".to_owned())
        .or_insert_with(|| "Y".to_owned());
}

/// Бренд: имя приложения, если кастомизация его не задала.
/// `APP_NAME != "RustDesk"` также включает режим custom client
/// (иконка из flutter_assets/assets/icon.ico, свои заголовки окон и т.д.).
fn apply_embedded_app_name() {
    if crate::get_app_name() == "RustDesk" {
        *hbb_common::config::APP_NAME.write().unwrap() = POI_APP_NAME.to_owned();
    }
}

fn have_server_config() -> bool {
    if !Config::get_option("custom-rendezvous-server").is_empty() {
        return true;
    }
    // Конфиг, зашитый в имя файла.
    let exe = std::env::current_exe()
        .map(|x| x.to_str().unwrap_or_default().to_owned())
        .unwrap_or_default();
    crate::custom_server::get_custom_server_from_string(&exe).is_ok()
}
