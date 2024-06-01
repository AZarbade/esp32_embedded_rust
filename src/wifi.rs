//! Wifi helper module
//!
//! credits to: <https://github.com/esp-rs/std-training/blob/main/common/lib/wifi/src/lib.rs>
use anyhow::Result;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::peripheral,
    nvs::EspDefaultNvsPartition,
    sys::EspError,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use log::info;

/// Connects to a Wi-Fi network using the provided SSID and password.
///
/// This function configures and connects to a Wi-Fi network specified by the `ssid` and `pass` parameters.
/// It performs the following steps:
///
/// # Arguments
///
/// * `ssid` - A string slice representing the SSID (name) of the Wi-Fi network to connect to.
/// * `pass` - A string slice representing the password for the Wi-Fi network.
/// * `modem` - An implementation of the `Peripheral` trait for the modem used by the Wi-Fi interface.
/// * `sysloop` - An instance of `EspSystemEventLoop` for handling system events.
/// * `nvs` - An optional instance of `EspDefaultNvsPartition` for non-volatile storage.
///
/// # Returns
///
/// * `Ok(Box<EspWifi<'static>>)` - A boxed instance of `EspWifi` if the Wi-Fi connection is successful.
/// * `Err(EspError)` - An `EspError` if there was an error during the Wi-Fi configuration or connection process.
///
/// # Panics
///
/// This function may panic if:
///
/// - The SSID does not fit into a `String<32>` buffer.
/// - The password does not fit into a `String<64>` buffer.
///
/// # Examples
///
/// ```
/// let _wifi = wifi("Wifi SSID", "Wifi Password", peripherals.modem, sysloop, Some(nvs));
/// ```
pub fn wifi(
    ssid: &str,
    pass: &str,
    modem: impl peripheral::Peripheral<P = esp_idf_svc::hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
    nvs: Option<EspDefaultNvsPartition>,
) -> Result<Box<EspWifi<'static>>, EspError> {
    let mut auth_method = AuthMethod::WPA2Personal;

    ssid.is_empty()
        .then(|| eprintln!("ERROR: missing WiFi name"));
    pass.is_empty().then(|| {
        auth_method = AuthMethod::None;
        info!("ERROR: missing password")
    });

    let mut esp_wifi = EspWifi::new(modem, sysloop.clone(), nvs)?;
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;
    wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;

    info!("Starting WiFi...");
    wifi.start()?;

    info!("Scanning...");

    let ap_info = wifi.scan()?;
    let ap_my = ap_info.into_iter().find(|a| a.ssid == ssid);
    let channel = if let Some(ap_my) = ap_my {
        info!(
            "Found configured access point {} on channel {}",
            ssid, ap_my.channel
        );
        Some(ap_my.channel)
    } else {
        info!(
            "Configured access point {ssid} not found during scanning, will go with unknown channel"
        );
        None
    };

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid
            .try_into()
            .expect("SSID does not fit into String<32> buffer"),
        password: pass
            .try_into()
            .expect("Password does not fit into String<64> buffer"),
        channel,
        auth_method,
        ..Default::default()
    }))?;

    info!("Connecting to WiFi...");
    wifi.connect()?;

    info!("Waiting for DHCP lease...");
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("WiFi DHCP info: {ip_info:?}");

    Ok(Box::new(esp_wifi))
}
