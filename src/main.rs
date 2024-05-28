mod wifi;
use anyhow::{Context, Result};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop, hal::peripherals::Peripherals, nvs::EspDefaultNvsPartition,
};
use log::info;
use wifi::wifi;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Getting started...");
    let peripherals = Peripherals::take().context("ERROR: failed to 'take' peripheral control")?;
    let sysloop = EspSystemEventLoop::take().context("ERROR: faild to 'take' event loop")?;
    let nvs = EspDefaultNvsPartition::take().context("ERROR: failed to 'take' NVS partition")?;

    let app_config = CONFIG;

    info!("Setting up wifi...");
    let _wifi = wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sysloop,
        Some(nvs),
    );

    Ok(())
}

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}
