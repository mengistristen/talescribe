pub mod idle {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    #[serde(tag = "action", rename_all = "snake_case")]
    pub enum Request {
        Exit,
        NewEntry,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(tag = "result", rename_all = "snake_case")]
    pub enum Response {
        Success,
        InvalidCommand,
    }
}

pub mod prompt {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Prompt {
        pub prompt: String,
        pub roll: Option<String>,
    }

    #[derive(Serialize, Deserialize)]
    pub enum Request {
        Get,
        Add,
        Remove { index: usize },
        Edit { index: usize, content: String },
        Roll { index: usize },
        StartEntry,
        Abort,
    }

    #[derive(Serialize, Deserialize)]
    pub enum Response {
        Content { content: Vec<Prompt> },
        Success,
        Missing,
        InvalidCommand,
    }
}

pub mod entry {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    #[serde(tag = "action", rename_all = "snake_case")]
    pub enum Request {
        Get,
        Edit { content: String },
        Complete { name: String },
        Back,
        Abort,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(tag = "result", rename_all = "snake_case")]
    pub enum Response {
        Content {
            content: String,
            prompts: Vec<String>,
        },
        Success,
        InvalidCommand,
    }
}
