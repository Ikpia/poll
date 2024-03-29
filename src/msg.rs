use crate::state::{Ballot, Poll};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub admin: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreatePoll {
        poll_id: String,
        question: String,
        options: Vec<String>,
    },
    Vote {
        poll_id: String,
        vote: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    AllPoll {},
    Poll { poll_id: String },
    Vote { poll_id: String, address: String },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AllPollResponse {
    pub polls: Vec<Poll>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]

pub struct PollResponse {
    pub poll: Option<Poll>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]

pub struct VoteResponse {
    pub vote: Option<Ballot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CustomResponse {
    val: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {}
