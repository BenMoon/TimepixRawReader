use std::alloc::handle_alloc_error;
use std::net::UdpSocket;
use std::ops::Add;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{spawn, JoinHandle};
use std::{thread, time};

use rayon::prelude::*;

fn udp_sampler() {
    let socket = UdpSocket::bind("127.0.0.1:50000").expect("couldn't bind to address");

    let mut bytes_recved: usize = 0;
    let mut buffer_list = [[0_u8; 8]; 4];
    let mut buffer_list_idx = 0;
    let mut buf = &mut buffer_list[buffer_list_idx];
    loop {
        bytes_recved += socket
            .recv(&mut buf[bytes_recved..])
            .expect("Didn't receive data");

        if bytes_recved > 5 {
            buffer_list_idx += 1;
            buffer_list_idx %= 4;

            println!("index: {}", buffer_list_idx);
            println!("{:?}", &buf[..bytes_recved]);
            println!(
                "{:?} bytes received, data: {:?}",
                bytes_recved, &mut buffer_list
            );

            buf = &mut buffer_list[buffer_list_idx];
            bytes_recved = 0;
        }
    }
}

fn packet_processor() {
    let data_recved: Vec<u8> = (0..8 * 256)
        .enumerate()
        .map(|(i, _)| if i % 8 == 0 { (i / 8) as u8 } else { 0 })
        .collect();
    let mut data_converted = [0_u64; 8];

    for i in 0..8 {
        //data_recved.len()/8 {
        data_converted[i] = u64::from_le_bytes(
            data_recved[i * 8..i * 8 + 8]
                .try_into()
                .expect("slice with incorrect length"),
        );
    }
    println!("{:?}", data_converted);

    let converted_iter = (0..8)
        .into_par_iter()
        .map(|i| {
            u64::from_le_bytes(
                data_recved[i * 8..i * 8 + 8]
                    .try_into()
                    .expect("slice with incorrect length"),
            )
        })
        .collect::<Vec<u64>>();
    println!("{:?}", converted_iter);
}

fn start_udp_sampler(cancle_flag: Arc<AtomicBool>) -> (Receiver<Vec<u64>>, JoinHandle<std::io::Result<()>>) {
    let (sender, receiver) = channel();

    let handle = spawn(move || {
        let mut i = 0;
        loop {
            println!("send {}", i);
            sender.send(vec![i; 10]);
            i += 1;

            if cancle_flag.load(Ordering::SeqCst) { break; }
            thread::sleep(time::Duration::from_millis(500));
        }
        Ok(())
    });

    (receiver, handle)
}

fn start_packet_processor(cancle_flag: Arc<AtomicBool>, tpx_raw: Receiver<Vec<u64>>) -> JoinHandle<()> {
    spawn(move || {
    for (i, data) in tpx_raw.into_iter().enumerate() {
            println!("{} received {:?}", i, data);

            // pretend this work takes a bit longer
            thread::sleep(time::Duration::from_secs(1));
            
            // have to duy an unmature death
            if cancle_flag.load(Ordering::SeqCst) { 
                //println!("{:?} elements remain in queue", &tpx_raw.into_iter().count());
                break;
            }
        }
    })

}

//fn main() -> std::io::Result<()> {z
fn main() {
 

    let cancel_flag = Arc::new(AtomicBool::new(false));
    let cancle_udp = cancel_flag.clone();
    let cancle_process= cancel_flag.clone();


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
