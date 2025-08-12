use cpal::traits::{DeviceTrait, HostTrait};

fn main() {
    let hosts = cpal::available_hosts();
    println!("Available hosts: {:?}", hosts);
    for host_id in hosts {
        let host = cpal::host_from_id(host_id).expect("Failed to get host from id");
        let devices = host.devices().expect("Failed to get devices");

        for device in devices {
            println!("Device: {}", device.name().expect("Failed to get device name"));
            if let Ok(config) = device.default_input_config() {
                println!(
                    "        Default input config: {:?}",
                    config
                );
                for supported_config in device.supported_input_configs().expect("Failed to get supported input config") {
                    if supported_config.min_sample_rate() == supported_config.max_sample_rate() {
                        let supported_config = supported_config.with_sample_rate(supported_config.min_sample_rate());
                        if config == supported_config {
                            continue;
                        }
                        println!(
                            "        - Supported input config: {:?}",
                            supported_config
                        );
                    } else {
                        println!(
                            "        - Supported input config: {:?}",
                            supported_config
                        );
                    }
                }
            }
            if let Ok(config) = device.default_output_config() {
                println!(
                    "        Default output config: {:?}",
                    config
                );
                for supported_config in device.supported_output_configs().expect("Failed to get supported output config") {
                    if supported_config.min_sample_rate() == supported_config.max_sample_rate() {
                        let supported_config = supported_config.with_sample_rate(supported_config.min_sample_rate());
                        if config == supported_config {
                            continue;
                        }
                        println!(
                            "        - Supported output config: {:?}",
                            supported_config
                        );
                    } else {
                        println!(
                            "        - Supported output config: {:?}",
                            supported_config
                        );
                    }
                }
            }
        }

    }
}
