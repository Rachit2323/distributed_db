use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::{io::Write, sync::Mutex};

use crate::raft::log::{LogEntry, RaftLog};

mod election;
mod log;
mod rpc;

#[derive(Debug, PartialEq)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug, PartialEq)]
pub struct RaftNode {
    id: u64,
    role: Role,
    current_term: u64,
    voted_for: Option<u64>,
    log: RaftLog,
    peers: Vec<String>,
    commit_index: usize,
}

impl RaftNode {
    pub fn new(id: u64, peers: Vec<String>) -> RaftNode {
        RaftNode {
            id,
            role: Role::Follower,
            current_term: 0,
            voted_for: None,
            log: RaftLog { logentry: vec![] },
            peers,
            commit_index: 0,
        }
    }

    pub fn propose(&mut self, command: String) -> Result<(), String> {
        if self.role != Role::Leader {
            return Err("Not a leader".to_string());
        }

        let entry = LogEntry {
            term: self.current_term,
            command: command.clone(),
        };
        self.log.logentry.push(entry);
        for peer in &self.peers {
            let mut stream = match TcpStream::connect(peer) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let _ = writeln!(stream, "APPEND|{}|{}", self.current_term, command);
        }
        Ok(())
    }

    pub fn handle_append_entried(&mut self, term: u64, command: String) {
        if term > self.current_term {
            self.current_term = term;
            self.role = Role::Follower;
        }
        let entry = LogEntry {
            term: term,
            command: command,
        };
        self.log.logentry.push(entry);
        println!("Node {} appended entry to log", self.id);
    }

    pub fn start_raft_listener(raft_port: u16, node: Arc<Mutex<RaftNode>>) {
        thread::spawn(move || {
            let listener = TcpListener::bind(format!("127.0.0.1:{}", raft_port))
                .expect("Cannot bind raft port");

            for stream in listener.incoming() {
                let stream = match stream {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                if reader.read_line(&mut line).is_err() {
                    continue;
                }
                let line = line.trim().to_string();

                let parts: Vec<&str> = line.splitn(3, '|').collect();
                if parts.len() == 3 && parts[0] == "APPEND" {
                    let term: u64 = match parts[1].parse() {
                        Ok(t) => t,
                        Err(_) => continue,
                    };
                    let command = parts[2].to_string();
                    node.lock().unwrap().handle_append_entried(term, command);
                }
            }
        });
    }

    pub fn start_heartbeat(node: Arc<Mutex<RaftNode>>) {
        let thread = thread::spawn(move || loop {
             thread::sleep(Duration::from_millis(150));    
            let node = node.lock().unwrap();
            if node.role == Role::Leader {
                for peer in &node.peers {
                    let mut stream = match TcpStream::connect(peer) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };
                    let _ = writeln!(stream, "Append|{}|Heartbeat", node.current_term);
                }
            }
        });
    }


}
