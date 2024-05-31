mod wifi;
use anyhow::{Context, Result};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop, hal::peripherals::Peripherals, nvs::EspDefaultNvsPartition,
};
use log::info;
use std::{thread::sleep, time::Duration};
use wifi::wifi;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Getting started...");
    let app_config = CONFIG;
    let peripherals = Peripherals::take().context("failed to 'take' peripheral control")?;
    let sysloop = EspSystemEventLoop::take().context("faild to 'take' event loop")?;
    let nvs = EspDefaultNvsPartition::take().context("failed to 'take' NVS partition")?;

    info!("Setting up wifi...");
    let _wifi = wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sysloop,
        Some(nvs),
    );

    loop {
        sleep(Duration::from_millis(1000));
    }
}

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
    #[default("")]
    mqtt_user: &'static str,
    #[default("")]
    mqtt_psk: &'static str,
    #[default("broker.local")]
    mqtt_host: &'static str,
}
