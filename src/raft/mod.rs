use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use std::{io::Write, sync::Mutex};

use crate::raft::log::{LogEntry, RaftLog};

pub mod election;
mod log;
mod rpc;

#[derive(Debug, PartialEq)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug)]
pub struct RaftNode {
    pub id: u64,
    pub role: Role,
    pub current_term: u64,
    pub voted_for: Option<u64>,
    pub log: RaftLog,
    pub peers: Vec<String>,
    pub commit_index: usize,
    pub last_heartbeat: Instant,
}

impl RaftNode {
    pub fn new(id: u64, peers: Vec<String>) -> RaftNode {
        RaftNode {
            id,
            role: if id == 1 { Role::Leader } else { Role::Follower },
            current_term: 0,
            voted_for: None,
            log: RaftLog { logentry: vec![] },
            peers,
            commit_index: 0,
            last_heartbeat: Instant::now(),
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
        if term >= self.current_term {
            self.current_term = term;
            self.role = Role::Follower;
            self.last_heartbeat = Instant::now();
        }
        if command != "HEARTBEAT" {
            let entry = LogEntry { term, command: command.clone() };
            self.log.logentry.push(entry);
            println!("Node {} appended entry to log: {}", self.id, command);
        }
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
                if parts.len() >= 2 && parts[0] == "APPEND" {
                    let term: u64 = match parts[1].parse() {
                        Ok(t) => t,
                        Err(_) => continue,
                    };
                    let command = if parts.len() == 3 { parts[2].to_string() } else { "HEARTBEAT".to_string() };
                    node.lock().unwrap().handle_append_entried(term, command);
                } else if parts.len() == 3 && parts[0] == "VOTE" {
                    let term: u64 = match parts[1].parse() {
                        Ok(t) => t,
                        Err(_) => continue,
                    };
                    let mut n = node.lock().unwrap();
                    if term >= n.current_term && n.voted_for.is_none() {
                        n.current_term = term;
                        n.voted_for = parts[2].parse().ok();
                        let mut stream = reader.into_inner();
                        let _ = writeln!(stream, "YES");
                        println!("Node {} voted YES for term {}", n.id, term);
                        continue;
                    }
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
