#![feature(vec_remove_item)]

mod audio;
mod gui;

use std::sync::mpsc::sync_channel;

fn main() {
    let (tx_in, rx_in) = sync_channel(16);
    let (tx_out, rx_out) = sync_channel(16);
    let host = audio::Host::new();
    if let Some(default_output_device_name) = host.default_output_device_name() {
        host.use_output_stream_from_device_name(default_output_device_name);
    }
    host.set_receiver(Some(rx_out));
    if let Some(default_input_device_name) = host.default_input_device_name() {
        host.use_input_stream_from_device_name(default_input_device_name);
    }
    host.set_sender(Some(tx_in));
    host.run();
    gui::main_loop(host, rx_in, tx_out);
}
