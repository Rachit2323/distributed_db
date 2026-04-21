use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::raft::{RaftNode, Role};

const ELECTION_TIMEOUT_MS: u64 = 300;   


pub fn start_election_timer(node: Arc<Mutex<RaftNode>>) {
    let thread = thread::spawn(move || {
        loop{
            thread::sleep(Duration::from_millis(ELECTION_TIMEOUT_MS));
               let mut  node =  node.lock().unwrap();
             if node.role == Role::Follower || node.role == Role::Candidate {
                start_election(&mut *node);

             }
        }

    });
    
}

fn start_election(node: &mut RaftNode) {
    node.current_term += 1;
    node.role=Role::Candidate;
    node.voted_for=Some(node.id);
    println!("Node {} started electin for term {}",node.id,node.current_term);
    for peer in &node.peers{

          println!("Sending RequestVote to {}", peer);
        
    }
    

}