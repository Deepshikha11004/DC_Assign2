/* Number of Processes: N */
/* Message Transmission Time : T */
/* Critical section Execution TIme :E */

/* Roucairol and Carvalho’s Algorithm */

/* Token which has socket Addr */
pub static mut TOKEN: Option<SocketAddr> = None;

/* REQUEST QUEUE */
pub static mut REQUEST_QUEUE: Vec<SocketAddr> = Vec::new();

struct Node {
    /* unique ID */
    addr: SocketAddr,
    /* Logical clock */
    lamport_clock: u32,
}

impl Node {
    fn new(id: u32) -> Self {
        Self {
            id,
            /* initialise lamport clock with some random number */
            lamport_clock: rand::random(),
        }
    }

    fn send_request(&mut self, node: &Node) {
        /* if TOKEN has some */
        if let Some(token) = unsafe { TOKEN } {
            /* if token is not with me */
            if token != self.addr {
                /* send request to the node */
                if let Ok(mut stream) = TcpStream::connect(node.addr) {
                    self.lamport_clock += 1;
                    stream.write(b"REQUEST");
                    stream.write(&self.lamport_clock.to_be_bytes());
                    stream.write(&b"\n"[..]);
                }
            }
        }
    }

    fn recieve_cs(&mut self) {
        if let Ok(listener) = TcpListener::bind(self.addr) {
            loop {
                for stream in listener.incoming() {
                    if let Ok(mut stream) = stream {
                        let mut buffer = String::new();
                        stream.read_to_string(&mut buffer).unwrap();
                        /* get sender's clock */
                        let sender_clock =
                            u32::from_be_bytes(buffer[7..].as_bytes().try_into().unwrap());

                        /* if sender's logical clock is less */
                        if sender_clock < self.lamport_clock {
                            /* send reply */
                            stream.write(b"REPLY");
                            stream.write(&self.lamport_clock.to_be_bytes());
                            stream.write(&b"\n"[..]);
                            /* put sender's addr in TOKEN */
                            unsafe {
                                TOKEN = Some(stream.peer_addr().unwrap());
                            }
                        } else {
                            /* if sender's logical clock is more */
                            unsafe {
                                /* requests the queue */
                                REQUEST_QUEUE.push(stream.peer_addr().unwrap());
                            }
                        }
                    }
                }
            }
        }
    }

    fn release(&mut self) {
        /* release TOKEN */
        unsafe {
            TOKEN = None;
        }
        /* send it to next process */
        if let Some(next_node) = REQUEST_QUEUE.pop() {
            if let Ok(mut stream) = TcpStream::connect(next_node) {
                self.lamport_clock += 1;
                stream.write(b"TOKEN");
                stream.write(&self.lamport_clock.to_be_bytes());
                stream.write(&b"\n"[..]);
                /* put next_node in TOKEN */
                unsafe {
                    TOKEN = Some(next_node);
                }
            }
        }
    }
}
