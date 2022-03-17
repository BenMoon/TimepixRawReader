mod packetprocessor;

use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};
use std::{thread, time};

use crate::packetprocessor::PacketProcessor;


fn udp_sampler(sender: Sender<Vec<u64>>, cancle_flag: Arc<AtomicBool>) {
    let socket = UdpSocket::bind("127.0.0.1:50000").expect("couldn't bind to address");

    let mut bytes_recved: usize = 0;
    let mut buffer_list = [[0_u8; 4096]; 4];
    let mut buffer_list_idx = 0;
    let mut buf = &mut buffer_list[buffer_list_idx];
    loop {
        bytes_recved += socket
            .recv(&mut buf[bytes_recved..])
            .expect("Didn't receive data");

        if bytes_recved > 128 {
            buffer_list_idx += 1;
            buffer_list_idx %= 4;

            sender
                .send(
                    (0..bytes_recved / 8)
                        .into_iter()
                        .map(|i| {
                            u64::from_le_bytes(
                                buf[i * 8..i * 8 + 8]
                                    .try_into()
                                    .expect("slice with incorrect length"),
                            )
                        })
                        .collect::<Vec<u64>>(),
                )
                .ok();

            buf = &mut buffer_list[buffer_list_idx];
            bytes_recved = 0;
        }
        if cancle_flag.load(Ordering::SeqCst) {
            println!("End UPD Sampler");
            break;
        }
    }
}

fn start_udp_sampler(
    cancle_flag: Arc<AtomicBool>,
) -> (Receiver<Vec<u64>>, JoinHandle<std::io::Result<()>>) {
    let (sender, receiver) = channel();

    let handle = spawn(move || {
        udp_sampler(sender, cancle_flag);

        Ok(())
    });

    (receiver, handle)
}

fn start_packet_processor(
    cancle_flag: Arc<AtomicBool>,
    tpx_raw: Receiver<Vec<u64>>,
) -> JoinHandle<()> {
    spawn(move || {
        for (i, data) in tpx_raw.into_iter().enumerate() {
            PacketProcessor::process(data);
            /*
            println!("{} received {:?}", i, data);

            // pretend this work takes a bit longer
            thread::sleep(time::Duration::from_secs(1));
            */
            // have to duy an unmature death
            if cancle_flag.load(Ordering::SeqCst) {
                //println!("{:?} elements remain in queue", &tpx_raw.into_iter().count());
                println!("End UPD Sampler");
                break;
            }
        }
    })
}

//fn main() -> std::io::Result<()> {z
fn main() {
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let cancle_udp = cancel_flag.clone();
    let cancle_process = cancel_flag.clone();

    let (tpx_bin, h1) = start_udp_sampler(cancle_udp);
    let h2 = start_packet_processor(cancle_process, tpx_bin);

    thread::sleep(time::Duration::from_secs(5));
    cancel_flag.store(true, Ordering::SeqCst);
    // Wait for threads to finish, holding on to any errors that they encounter.
    let r1 = h1.join().unwrap();
    h2.join();

    // Return the first error encountered, if any.
    // (As it happens, h2 and h3 can't fail: those threads
    // are pure in-memory data processing.)
    //r1?;
}

#[test]
fn test_udpsampler() {
    /// test which sends a few numbers and checks to receive them back
    use bytemuck;

    // first start udp_thread
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let cancle_udp = cancel_flag.clone();
    let cancle_process = cancel_flag.clone();

    let (tpx_bin, h1) = start_udp_sampler(cancle_udp);

    // send data
    let socket = UdpSocket::bind("127.0.0.1:0").expect("binding socket failed");
    socket
        .connect("127.0.0.1:50000")
        .expect("connect function failed");

    let test_data = (0..64_u64).into_iter().collect::<Vec<u64>>();
    let test_bytes_data: &[u8] = bytemuck::cast_slice(&test_data);
    socket
        .send(test_bytes_data)
        .expect("sending of test data failed");

    assert_eq!(test_data, tpx_bin.iter().next().unwrap());
    assert_eq!(1, 2);
}
