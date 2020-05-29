mod audio;
mod config;
mod gui;

fn main() {
    let buffer = 2;
    let (tx_in, rx_in) = audio::stream::sync_chunk_channel(buffer);
    let (tx_out, rx_out) = audio::stream::chunk_channel();
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
