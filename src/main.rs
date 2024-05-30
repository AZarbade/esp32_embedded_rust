mod wifi;
use anyhow::{Context, Result};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        adc::{self, attenuation::DB_11},
        gpio::*,
        peripherals::Peripherals,
    },
    mqtt::client::{EspMqttClient, MqttClientConfiguration, QoS},
    nvs::EspDefaultNvsPartition,
};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{thread::sleep, time::Duration};
use wifi::wifi;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Getting started...");
    let peripherals = Peripherals::take().context("failed to 'take' peripheral control")?;
    let sysloop = EspSystemEventLoop::take().context("faild to 'take' event loop")?;
    let nvs = EspDefaultNvsPartition::take().context("failed to 'take' NVS partition")?;
    let mut adc1 = adc::AdcDriver::new(
        peripherals.adc1,
        &adc::config::Config::new().calibration(true),
    )
    .context("failed to new ADC Driver")?;

    let app_config = CONFIG;

    info!("Setting up wifi...");
    let _wifi = wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sysloop,
        Some(nvs),
    );

    let mut pin = adc::AdcChannelDriver::<'_, DB_11, Gpio36>::new(peripherals.pins.gpio36)
        .context("failed to set ADC Pin")?;

    let broker_url = if app_config.mqtt_user != "" {
        format!(
            "mqtt://{}:{}@{}",
            app_config.mqtt_user, app_config.mqtt_psk, app_config.mqtt_host,
        )
    } else {
        format!("mqtt://{}", app_config.mqtt_host)
    };

    let mqtt_config = MqttClientConfiguration::default();
    let mut mqtt_client = EspMqttClient::new_cb(&broker_url, &mqtt_config, move |_msg| {})?;

    let payload: &[u8] = &[];
    mqtt_client.publish("home/default", QoS::AtLeastOnce, true, payload)?;

    loop {
        sleep(Duration::from_millis(100));
        let reading = adc1.read(&mut pin)?;

        let sensor_reading = SensorReading {
            heart_rate: reading,
        };

        let json_string = serde_json::to_string(&sensor_reading)?;

        mqtt_client.publish(
            "sensors/heart",
            QoS::AtLeastOnce,
            false,
            json_string.as_bytes(),
        )?;
    }
}

#[derive(Serialize, Deserialize)]
struct SensorReading {
    heart_rate: u16,
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
