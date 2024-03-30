use crate::Params;
use rand::distributions::Distribution;
use rand_distr::Exp;
use std::f64;
use std::io::Write;
// use std::net::IpAddr;
use std::io::Read;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
// use std::thread::sleep;
// use std::time::Duration;

pub static mut REQUEST_QUEUE: Vec<SocketAddr> = Vec::new();
pub static mut REPLY_FLAG: Option<SocketAddr> = None;
pub static mut FAIL_FLAG: Option<SocketAddr> = None;
pub static mut YIELD_FLAG: Option<SocketAddr> = None;

struct Node {
    i: u32,
    j: u32,
    lamport_clock: u32,
    addr: SocketAddr,
    request_set: Vec<SocketAddr>,
}

impl Node {
    fn new(i: u32, j: u32, lamport_clock: u32, addr: SocketAddr) -> Self {
        Self {
            i,
            j,
            lamport_clock,
            addr,
            request_set: vec![],
        }
    }

    fn request_set_node(&mut self, grid: Vec<Vec<SocketAddr>>) {
        for i_val in 0..self.i {
            self.request_set.push(grid[i_val as usize][self.j as usize]);
        }

        for j_val in 0..self.j {
            self.request_set.push(grid[self.i as usize][j_val as usize]);
        }
        dbg!(&self.request_set);
    }

    fn sendrequest_cs(&mut self) {
        loop {
            for node_addr in &self.request_set {
                if let Ok(mut stream) = TcpStream::connect(*node_addr) {
                    self.lamport_clock += 1;
                    stream.write(b"REQUEST");
                    // send Sender lamport clock also
                    stream.write(&self.lamport_clock.to_be_bytes());
                    // send terminating character
                    stream.write(&b"\n"[..]);
                }
            }
        }
    }

    fn recieve_cs(&self) {
        if let Ok(listener) = TcpListener::bind(self.addr) {
            loop {
                for stream in listener.incoming() {
                    if let Ok(mut stream) = stream {
                        let mut buffer = String::new();
                        stream.read_to_string(&mut buffer).unwrap();

                        if &buffer[0..7] == "REQUEST" {
                            if unsafe { REPLY_FLAG.is_none() } {
                                /* send REPLY message */
                                let mut stream_clone = stream.try_clone().unwrap();
                                stream_clone.write(&b"REPLY"[..]).unwrap();
                                // stream_clone.write(&self.lamport_clock.to_be_bytes());
                            } else {
                                buffer = buffer[7..].to_string();

                                let sender_lamport_clock = buffer.parse::<u32>().unwrap();
                                /*get socket_addr of REPLY_FLAG */
                                let reply_flag = unsafe { REPLY_FLAG.unwrap() };

                                /*get lamport_clock of reply_flag */
                                let reply_flag_lamport_clock =
                                    reply_flag.to_string().parse::<u32>().unwrap();

                                /*if sender_lamport_clock is greater than  reply_flag_lamport_clock*/
                                if sender_lamport_clock > reply_flag_lamport_clock {
                                    /* send FAILED message */
                                    let mut stream_clone = stream.try_clone().unwrap();
                                    stream_clone.write(&b"FAILED"[..]).unwrap();
                                } else {
                                    /* send INQUIRE message */
                                    let mut stream_clone = stream.try_clone().unwrap();
                                    stream_clone.write(&b"INQUIRE"[..]).unwrap();
                                }
                            }
                        } else if &buffer[0..7] == "INQUIRE" {
                            /* if FAIL message */
                            if unsafe {
                                FAIL_FLAG.is_some()
                                    && (YIELD_FLAG.is_some() && REPLY_FLAG.is_none())
                            } {
                                /* send YIELD message */
                                let mut stream_clone = stream.try_clone().unwrap();
                                stream_clone.write(&b"YIELD"[..]).unwrap();
                            }
                        } else if &buffer[0..5] == "YIELD" {
                            /* send REPLY message */
                            let mut stream_clone = stream.try_clone().unwrap();
                            stream_clone.write(&b"REPLY"[..]).unwrap();
                            /* add sender to REQUEST_QUEUE */
                            unsafe { REQUEST_QUEUE.push(self.addr) };
                        } else if &buffer[0..6] == "FAILED" {
                            /* get sender address */
                            let sender = unsafe { FAIL_FLAG.unwrap() };
                            /* FAIL_FLAG  from sender */
                            unsafe { FAIL_FLAG = Some(sender) };
                        } else if &buffer[0..5] == "REPLY" {
                            /* get sender address */
                            let sender = unsafe { REPLY_FLAG.unwrap() };
                            /* REPLY_FLAG  from sender */
                            unsafe { REPLY_FLAG = Some(sender) };
                        } else if &buffer[0..7] == "RELEASE" {
                            if !unsafe { REQUEST_QUEUE.is_empty() } {
                                /* send REPLY message */
                                let mut stream_clone = stream.try_clone().unwrap();
                                stream_clone.write(&b"REPLY"[..]).unwrap();
                                /* remove sender from REQUEST_QUEUE */
                                unsafe { REQUEST_QUEUE.remove(0) };
                                /* set REPLY_FLAG to true */
                                unsafe { REPLY_FLAG = Some(self.addr) };
                            } else {
                                /* set REPLY_FLAG to false */
                                unsafe { REPLY_FLAG = None };
                            }
                        }
                    }
                }
            }
        }
    }

    fn sendrelease_cs(&mut self) {
        for node_addr in &self.request_set {
            if let Ok(mut stream) = TcpStream::connect(*node_addr) {
                self.lamport_clock += 1;

                stream.write(b"RELEASE").unwrap();
                // send Sender lamport clock also
                stream.write(&self.lamport_clock.to_be_bytes()).unwrap();
                // send terminating character
                stream.write(&b"\n"[..]).unwrap();
            }
        }
    }
}

fn init_sqrt(params: &Params) -> f64 {
    let sqrt_n = f64::sqrt(params.n as f64);
    return sqrt_n;
}

fn alpha_dist(params: &Params) -> f64 {
    let mut rng = rand::thread_rng();
    let exp_dist = Exp::new(1.0 / (params.alpha as f64)).unwrap(); // Set alpha to the desired average time
    let sleep_time = exp_dist.sample(&mut rng);
    return sleep_time;
}

fn beta_dist(params: &Params) -> f64 {
    let mut rng = rand::thread_rng();
    let exp_dist = Exp::new(1.0 / (params.beta as f64)).unwrap(); // Set beta to the desired average time
    let sleep_time = exp_dist.sample(&mut rng);
    return sleep_time;
}


