use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};

const LASM_CLUSTER_RELAY_BUFFER_BYTES: usize = 16 * 1024;

pub(crate) enum LasmClusterRelayPumpStep {
    Progressed,
    Idle,
    Complete,
}

pub(crate) struct LasmClusterRelayPump {
    client: TcpStream,
    upstream: TcpStream,
    client_to_upstream: Vec<u8>,
    upstream_to_client: Vec<u8>,
    c2u_start: usize,
    c2u_end: usize,
    u2c_start: usize,
    u2c_end: usize,
    client_read_closed: bool,
    upstream_read_closed: bool,
    client_write_closed: bool,
    upstream_write_closed: bool,
}

impl LasmClusterRelayPump {
    pub(crate) fn new(client: TcpStream, upstream: TcpStream) -> Result<Self, String> {
        Self::new_with_buffers(
            client,
            upstream,
            vec![0_u8; LASM_CLUSTER_RELAY_BUFFER_BYTES],
            vec![0_u8; LASM_CLUSTER_RELAY_BUFFER_BYTES],
        )
    }

    pub(crate) fn new_with_buffers(
        client: TcpStream,
        upstream: TcpStream,
        mut client_to_upstream: Vec<u8>,
        mut upstream_to_client: Vec<u8>,
    ) -> Result<Self, String> {
        client
            .set_nonblocking(true)
            .map_err(|err| format!("could not set client proxy stream nonblocking: {err}"))?;
        upstream
            .set_nonblocking(true)
            .map_err(|err| format!("could not set upstream proxy stream nonblocking: {err}"))?;
        if client_to_upstream.len() != LASM_CLUSTER_RELAY_BUFFER_BYTES {
            client_to_upstream.resize(LASM_CLUSTER_RELAY_BUFFER_BYTES, 0_u8);
        }
        if upstream_to_client.len() != LASM_CLUSTER_RELAY_BUFFER_BYTES {
            upstream_to_client.resize(LASM_CLUSTER_RELAY_BUFFER_BYTES, 0_u8);
        }
        Ok(Self {
            client,
            upstream,
            client_to_upstream,
            upstream_to_client,
            c2u_start: 0,
            c2u_end: 0,
            u2c_start: 0,
            u2c_end: 0,
            client_read_closed: false,
            upstream_read_closed: false,
            client_write_closed: false,
            upstream_write_closed: false,
        })
    }

    pub(crate) fn pump_once(&mut self) -> Result<LasmClusterRelayPumpStep, String> {
        let mut progressed = false;

        while !self.client_read_closed && self.c2u_end < self.client_to_upstream.len() {
            match self
                .client
                .read(&mut self.client_to_upstream[self.c2u_end..])
            {
                Ok(0) => {
                    self.client_read_closed = true;
                    progressed = true;
                    break;
                }
                Ok(bytes_read) => {
                    self.c2u_end += bytes_read;
                    progressed = true;
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(err) => {
                    return Err(format!("could not read proxy client stream: {err}"));
                }
            }
        }

        while self.c2u_start < self.c2u_end {
            match self
                .upstream
                .write(&self.client_to_upstream[self.c2u_start..self.c2u_end])
            {
                Ok(0) => {
                    return Err(
                        "could not relay client payload to upstream: write returned 0 bytes"
                            .to_string(),
                    );
                }
                Ok(bytes_written) => {
                    self.c2u_start += bytes_written;
                    progressed = true;
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(err) => {
                    return Err(format!("could not relay client payload to upstream: {err}"));
                }
            }
        }
        if self.c2u_start == self.c2u_end {
            self.c2u_start = 0;
            self.c2u_end = 0;
            if self.client_read_closed && !self.upstream_write_closed {
                let _ = self.upstream.shutdown(Shutdown::Write);
                self.upstream_write_closed = true;
                progressed = true;
            }
        }

        while !self.upstream_read_closed && self.u2c_end < self.upstream_to_client.len() {
            match self
                .upstream
                .read(&mut self.upstream_to_client[self.u2c_end..])
            {
                Ok(0) => {
                    self.upstream_read_closed = true;
                    progressed = true;
                    break;
                }
                Ok(bytes_read) => {
                    self.u2c_end += bytes_read;
                    progressed = true;
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(err) => {
                    return Err(format!("could not read proxy upstream stream: {err}"));
                }
            }
        }

        while self.u2c_start < self.u2c_end {
            match self
                .client
                .write(&self.upstream_to_client[self.u2c_start..self.u2c_end])
            {
                Ok(0) => {
                    return Err(
                        "could not relay upstream response to client: write returned 0 bytes"
                            .to_string(),
                    );
                }
                Ok(bytes_written) => {
                    self.u2c_start += bytes_written;
                    progressed = true;
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(err) => {
                    return Err(format!(
                        "could not relay upstream response to client: {err}"
                    ));
                }
            }
        }
        if self.u2c_start == self.u2c_end {
            self.u2c_start = 0;
            self.u2c_end = 0;
            if self.upstream_read_closed && !self.client_write_closed {
                let _ = self.client.shutdown(Shutdown::Write);
                self.client_write_closed = true;
                progressed = true;
            }
        }

        if self.client_write_closed
            && self.upstream_write_closed
            && self.c2u_end == 0
            && self.u2c_end == 0
        {
            return Ok(LasmClusterRelayPumpStep::Complete);
        }

        if progressed {
            Ok(LasmClusterRelayPumpStep::Progressed)
        } else {
            Ok(LasmClusterRelayPumpStep::Idle)
        }
    }

    pub(crate) fn into_buffers(self) -> (Vec<u8>, Vec<u8>) {
        (self.client_to_upstream, self.upstream_to_client)
    }
}
