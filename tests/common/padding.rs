pub const fn padding_count(mtu: usize, payload_size: usize) -> usize {
    let mtu_without_tail_byte = mtu - 1;
    let last_frame_data_bytes = payload_size % mtu_without_tail_byte;

    if last_frame_data_bytes == 0 {
        0
    } else {
        mtu_without_tail_byte - last_frame_data_bytes
    }
}
