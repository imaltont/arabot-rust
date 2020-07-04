pub struct ChatMessage{
    pub user: String,
    pub roles: Elevation,
    pub text: String,
    pub channel: String
}

pub struct Reply{
    pub user: String,
    pub text: String
}

#[derive(Copy, Clone)]
pub enum Elevation{
    Broadcaster,
    Moderator,
    Viewer
}

pub struct ChatCommand{
    pub command: String,
    pub elevation: Elevation,
    pub response: fn(),
    pub help: String,
    pub repeat_interval: i64,
}

impl ChatCommand{
    fn new(c: String, e: Elevation, r: fn(), h: String, ri: i64)->ChatCommand{
        ChatCommand{command: c, elevation: e, response: r, help: h, repeat_interval: ri}    
    }
}

