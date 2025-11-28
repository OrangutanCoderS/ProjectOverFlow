use std::{os::unix::net::{UnixListener, UnixStream}, io::{Read, Write}, thread, path::Path};
use serde::{Serialize, Deserialize};
use crate::manager::{SpoofManager, ManagerError};

#[derive(Debug, Serialize, Deserialize)]
pub struct SpoofQuery { pub pid: i32 }

#[derive(Debug, Serialize, Deserialize)]
pub struct SpoofResponse {
    pub ok: bool,
    pub fake_pid: Option<i32>,
    pub fake_ppid: Option<i32>,
    pub fake_name: Option<String>,
    pub fake_env: Option<std::collections::HashMap<String,String>>,
    pub error: Option<String>,
}

pub fn start_server(path: &str, mgr: SpoofManager) -> std::io::Result<()> {
    if Path::new(path).exists() { std::fs::remove_file(path)?; }
    let listener = UnixListener::bind(path)?;
    thread::spawn(move || {
        for conn in listener.incoming() {
            if let Ok(mut stream) = conn {
                let mut buf = Vec::new();
                if stream.read_to_end(&mut buf).is_ok() {
                    if let Ok(q) = serde_json::from_slice::<SpoofQuery>(&buf) {
                        let resp = mgr.query(q.pid).map_or_else(
                            || SpoofResponse{ ok:false, fake_pid:None, fake_ppid:None, fake_name:None, fake_env:None, error:Some("not found".into()) },
                            |s| SpoofResponse{ ok:true, fake_pid:Some(s.fake_pid), fake_ppid:Some(s.fake_ppid), fake_name:s.fake_name.clone(), fake_env:s.fake_env.clone(), error:None }
                        );
                        let _ = stream.write_all(&serde_json::to_vec(&resp).unwrap());
                    }
                }
            }
        }
    });
    Ok(())
}

pub fn query_server(path: &str, pid: i32) -> Result<SpoofResponse, ManagerError> {
    let mut stream = UnixStream::connect(path)
        .map_err(|e| ManagerError::Ipc(e.to_string()))?;
    let q = SpoofQuery { pid };
    stream.write_all(&serde_json::to_vec(&q).unwrap())
        .map_err(|e| ManagerError::Ipc(e.to_string()))?;
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf)
        .map_err(|e| ManagerError::Ipc(e.to_string()))?;
    serde_json::from_slice::<SpoofResponse>(&buf)
        .map_err(|e| ManagerError::Ipc(e.to_string()))
}