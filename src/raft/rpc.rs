pub struct AppendEntries {
    term : u64 ,
    leader_id : u64 ,
    entries :Vec<String> 
}

pub struct AppendEntriesResponse {
    term: u64 ,
    sucess: bool
}

pub struct RequestVote {
    term : u64 ,
    candidate_id :u64 
}

pub struct RequestVoteResponse {
    term : u64 ,
    vote_granted : bool 
}