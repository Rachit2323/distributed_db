use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::raft::{RaftNode, Role};

const ELECTION_TIMEOUT_MS: u64 = 300;

pub fn start_election_timer(node: Arc<Mutex<RaftNode>>) {
    let thread = thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(ELECTION_TIMEOUT_MS));
            let elapsed = node.lock().unwrap().last_heartbeat.elapsed().as_millis() as u64;
            if elapsed >= ELECTION_TIMEOUT_MS {
                let mut node = node.lock().unwrap();
                if node.role == Role::Follower || node.role == Role::Candidate {
                    start_election(&mut *node);
                }
            }
        }
    });
}

fn start_election(node: &mut RaftNode) {
    node.current_term += 1;
    node.role = Role::Candidate;
    node.voted_for = Some(node.id);
    println!("Node {} started election for term {}", node.id, node.current_term);

    let mut votes = 1; // vote for self
    let total = node.peers.len() + 1;
    let majority = total / 2 + 1;

    for peer in &node.peers {
        let mut stream = match TcpStream::connect(peer) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let _ = writeln!(stream, "VOTE|{}|{}", node.current_term, node.id);

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        if reader.read_line(&mut response).is_ok() && response.trim() == "YES" {
            votes += 1;
        }
    }

    if votes >= majority {
        node.role = Role::Leader;
        println!("Node {} became Leader for term {}", node.id, node.current_term);
    }
}
