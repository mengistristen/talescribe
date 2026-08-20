pub mod idle {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    #[serde(tag = "action", rename_all = "snake_case")]
    pub enum Request {
        Exit,
        Entry,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(tag = "result", rename_all = "snake_case")]
    pub enum Response {
        Success,
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
        Roll,
        Accept,
        Complete { name: String },
    }

    #[derive(Serialize, Deserialize)]
    #[serde(tag = "result", rename_all = "snake_case")]
    pub enum Response {
        Content { content: String },
        Success,
        InvalidCommand,
    }
}
