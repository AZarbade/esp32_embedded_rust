mod wifi;
use anyhow::{Context, Result};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{gpio::PinDriver, peripherals::Peripherals},
    http::{
        server::{Configuration, EspHttpServer},
        Method,
    },
    nvs::EspDefaultNvsPartition,
    sys::EspError,
};
use log::info;
use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};
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

    // TODO: check if can use anyhow instead of expect
    let led_pin = Arc::new(Mutex::new(
        PinDriver::output(peripherals.pins.gpio2).expect("ERROR: faild to get led pin"),
    ));

    let mut server = EspHttpServer::new(&Configuration::default())
        .context("ERROR: failed to create web server")?;
    server.fn_handler("/", Method::Get, |request| {
        // TODO: get rid of these unwraps
        let mut response = request.into_ok_response().unwrap();
        response.write("hello from esp32".as_bytes()).unwrap();
        led_pin.lock().unwrap().toggle().unwrap();
        Ok::<(), EspError>(())
    })?;

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
}
