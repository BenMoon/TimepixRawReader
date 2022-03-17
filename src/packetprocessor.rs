use rayon::prelude::*;

pub struct PacketProcessor {

}

impl PacketProcessor {
    pub fn process(pixdata: Vec<u64>) {
        // extract trigger events from data stream
        let triggers = pixdata
            .iter()
            .filter(|x| {
                ((((**x & 0xF000000000000000) >> 60) & 0xF) == (0x4 | 0x6))
                    & ((((**x & 0x0F00000000000000) >> 56) & 0xF) == 0xF)
            })
            .cloned()
            .collect::<Vec<u64>>();

        // extract pixel events from data stream
        let pixels = pixdata
            .into_iter()
            .filter(|x| (((x & 0xF000000000000000) >> 60) & 0xF) == (0xA | 0xB))
            .collect::<Vec<u64>>();
    }
}
