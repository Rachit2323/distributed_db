
#[derive(Debug,PartialEq)]
pub struct LogEntry {
    pub term : u64 ,
    pub command : String 
}

#[derive(Debug,PartialEq)]
pub struct RaftLog {
    pub logentry : Vec<LogEntry> 
}


