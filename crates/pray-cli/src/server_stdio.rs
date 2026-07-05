use crate::server::handle_rpc;
use pray_core::ssh_rpc::{read_frame, write_frame, RpcRequest, RpcResponse};
use pray_core::{PrayError, PrayResult};
use std::io::{self, BufReader, Write};
use std::path::PathBuf;

pub fn run_stdio_server(root: PathBuf) -> PrayResult<()> {
    std::env::set_var("PRAY_SERVE_STDIO", "1");
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    loop {
        let frame = match read_frame(&mut reader) {
            Ok(frame) => frame,
            Err(PrayError::Resolution(message)) if message == "rpc stream closed" => return Ok(()),
            Err(error) => return Err(error),
        };
        let request: RpcRequest =
            serde_json::from_slice(&frame).map_err(|error| PrayError::Parse {
                kind: "ssh rpc request",
                message: error.to_string(),
            })?;
        let response = match handle_rpc(&root, &request) {
            Ok(response) => response,
            Err(error) => RpcResponse::error(&request.id, 500, error.to_string()),
        };
        let payload = serde_json::to_vec(&response)
            .map_err(|error| PrayError::Manifest(error.to_string()))?;
        write_frame(&mut stdout, &payload)?;
        stdout.flush()?;
    }
}
