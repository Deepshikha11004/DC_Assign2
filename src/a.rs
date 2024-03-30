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
use std::thread::sleep;
use std::time::Duration;

#[allow(dead_code)]
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
            let mut reply_flag = false;
            let mut request_queue = Vec::new();
            loop {
                for stream in listener.incoming() {
                    if let Ok(mut stream) = stream {
                        // get sender's lamport clock and message
                        // 1st 7 : message
                        // let mut buffer = [0; 7];
                        // stream.read(&mut buffer).unwrap();
                        // let sender_lamport_clock = u32::from_be_bytes(buffer[7..].try_into().unwrap());
                        // store self lamport clock
                        let old_clock = self.lamport_clock;
                        match &buffer {
                            b"REQUEST" => {
                                if !reply_flag {
                                    let mut stream_clone = stream.try_clone().unwrap();
                                    stream_clone.write(&b"REPLY"[..]).unwrap();
                                    reply_flag = true;
                                } else {
                                    // send "FAILED" message since S_j has given permission to S_k
                                    let mut stream_clone = stream.try_clone().unwrap();
                                    if old_clock < sender_lamport_clock {
                                        stream_clone.write(&b"FAILED"[..]).unwrap();
                                        request_queue.push(stream.peer_addr().unwrap());
                                    } else {
                                        // send inquire message to site S_k
                                        stream_clone.write(&b"INQUIRE"[..]).unwrap();
                                    }
                                }
                            }
                            b"RELEASE" => {
                                if !request_queue.is_empty() {
                                    let mut stream_clone = stream.try_clone().unwrap();
                                    stream_clone.write(&b"REPLY"[..]).unwrap();
                                    request_queue.remove(0);
                                    reply_flag = true;
                                } else {
                                    reply_flag = false;
                                }
                            }
                            b"INQUIRE" => {
                                // S_k sends a "YIELD" message to site S_j if it has reciebed a "FAILED" message from a site in its request set
                                let mut stream_clone = stream.try_clone().unwrap();
                                stream_clone.write(&b"YIELD"[..]).unwrap();
                            }
                            b"YIELD" => {}
                            _ => {
                                // Handle other cases
                            }
                        }
                    }
                }
            }
        } else {
            // Binding failed, handle the error
            // ...
        }
    }

    fn sendrelease_cs(&self) {
        for node_addr in &self.request_set {
            if let Ok(mut stream) = TcpStream::connect(*node_addr) {
                stream.write(b"RELEASE");
            }
        }
    }
}

fn init_sqrt(params: &Params) -> f64 {
    let sqrt_n = f64::sqrt(params.n as f64);
    return sqrt_n;
}

fn get_grid(params: &Params) -> Vec<Vec<SocketAddr>> {
    // let sqrt_n = init_sqrt(params);
    // vec![vec![0; sqrt_n as usize]; sqrt_n as usize]
    // return grid;
    todo!()
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

fn process_func(mut node: Node, params: &Params) {
    let grid = get_grid(params);
    node.request_set_node(grid);

    for _ in 0..params.k {
        let out_cstime = alpha_dist(params);
        let in_cstime = beta_dist(params);
        sleep(Duration::from_secs_f64(out_cstime));

        /* request CS */
        node.sendrequest_cs();

        /* in CS */
        sleep(Duration::from_secs_f64(in_cstime));

        /* release CS */
        node.sendrelease_cs();
    }
}
