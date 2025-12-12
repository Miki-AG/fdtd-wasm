use fdtd_wasm::comms::packet::{PacketDecoder, PacketState};

#[test]
fn test_packet_decoder_state_transitions() {
    let mut decoder = PacketDecoder::new();
    assert_eq!(decoder.get_state(), PacketState::SearchPreamble);
    
    // Feed Preamble 0xAA (10101010)
    // LSB first into push_bit logic in packet.rs? 
    // self.buffer = (self.buffer << 1) | (bit & 1); => MSB enters first into LSB position?
    // No, shift left means we push into LSB.
    // So sequence 1, 0, 1, 0... results in 1, 10, 101...
    // So to get 0xAA (10101010), we push 1, 0, 1, 0...
    
    let preamble = vec![1, 0, 1, 0, 1, 0, 1, 0];
    for bit in preamble {
        decoder.push_bit(bit);
    }
    
    // Should be in SearchSync
    assert_eq!(decoder.get_state(), PacketState::SearchSync);
    
    // Feed Sync 0x7E (01111110)
    let sync = vec![0, 1, 1, 1, 1, 1, 1, 0];
    for bit in sync {
        decoder.push_bit(bit);
    }
    
    // Should be Reading Length
    assert_eq!(decoder.get_state(), PacketState::ReadLength);
}