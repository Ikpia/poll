#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    AllPollResponse, ExecuteMsg, InstantiateMsg, PollResponse, QueryMsg, VoteResponse,
};
use crate::state::{Ballot, Config, Poll, BALLOT, CONFIG, POLL};

const CONTRACT_NAME: &str = "crates.io:poll";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let admin = msg.admin.unwrap_or(info.sender.to_string());
    let validated_admin = deps.api.addr_validate(&admin)?;
    let config = Config {
        admin: validated_admin.clone(),
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute(admin, validated_admin.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreatePoll {
            poll_id,
            question,
            options,
        } => execute_create_poll(deps, env, info, poll_id, question, options),
        ExecuteMsg::Vote { poll_id, vote } => execute_vote(deps, env, info, poll_id, vote),
    }
}

fn execute_create_poll(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    poll_id: String,
    question: String,
    options: Vec<String>,
) -> Result<Response, ContractError> {
    if options.len() > 10 {
        return Err(ContractError::TooManyPollOptions {});
    }

    let mut opts: Vec<(String, u64)> = vec![];
    for option in options {
        opts.push((option, 0))
    }

    let poll = Poll {
        admin: info.sender,
        question,
        options: opts,
    };
    POLL.save(deps.storage, poll_id, &poll)?;
    Ok(Response::new().add_attribute("action", "create poll"))
}

fn execute_vote(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    poll_id: String,
    vote: String,
) -> Result<Response, ContractError> {
    let poll = POLL.may_load(deps.storage, poll_id.clone())?;

    if let Some(mut poll) = poll {
        BALLOT.update(
            deps.storage,
            (info.sender, poll_id.clone()),
            |ballot| -> StdResult<Ballot> {
                match ballot {
                    Some(ballot) => {
                        let position_of_old_vote = poll
                            .options
                            .iter()
                            .position(|option| option.0 == ballot.option)
                            .unwrap();
                        poll.options[position_of_old_vote].1 -= 1;
                        Ok(Ballot {
                            option: vote.clone(),
                        })
                    }
                    None => Ok(Ballot {
                        option: vote.clone(),
                    }),
                }
            },
        )?;
        let position = poll
            .options
            .iter()
            .position(|option| option.0 == vote)
            .unwrap();
        poll.options[position].1 += 1;
        POLL.save(deps.storage, poll_id, &poll)?;
        return Ok(Response::new().add_attribute("action", "vote in poll"));
    } else {
        return Err(ContractError::CustomError {
            val: "Poll not found".to_string(),
        });
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AllPoll {} => query_all_poll(deps, env),
        QueryMsg::Poll { poll_id } => query_poll(deps, env, poll_id),
        QueryMsg::Vote { poll_id, address } => query_vote(deps, env, poll_id, address),
    }
}

fn query_all_poll(deps: Deps, _env: Env) -> StdResult<Binary> {
    let polls = POLL
        .range(deps.storage, None, None, Order::Ascending)
        .map(|p| Ok(p?.1))
        .collect::<StdResult<Vec<_>>>()?;
    to_binary(&AllPollResponse { polls })
}

fn query_poll(deps: Deps, _env: Env, poll_id: String) -> StdResult<Binary> {
    let poll = POLL.may_load(deps.storage, poll_id)?;
    to_binary(&PollResponse { poll })
}

fn query_vote(deps: Deps, _env: Env, address: String, poll_id: String) -> StdResult<Binary> {
    let validated_address = deps.api.addr_validate(&address)?;
    let vote = BALLOT.may_load(deps.storage, (validated_address, poll_id))?;
    to_binary(&VoteResponse { vote })
}

#[cfg(test)]
mod tests {
    use crate::contract::{execute, execute_create_poll, instantiate, query};
    use crate::msg::{ExecuteMsg, InstantiateMsg, PollResponse, QueryMsg};
    use crate::state::Poll;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{attr, from_binary, Addr};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr1", &[]);
        let msg = InstantiateMsg {
            admin: Some("addr1".to_string()),
        };
        let resp = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(
            resp.attributes,
            vec![
                attr("action", "instantiate"),
                attr("addr1".to_string(), "addr1".to_string())
            ]
        );
    }

    #[test]
    fn test_execute_create_poll() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr1", &[]);
        let msg = InstantiateMsg {
            admin: Some("addr1".to_string()),
        };
        let resp = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(
            resp.attributes,
            vec![
                attr("action", "instantiate"),
                attr("addr1".to_string(), "addr1".to_string())
            ]
        );
        let poll_id = "1".to_string();
        let question = "Should We Have a Meeting Today".to_string();
        let options = vec![String::from("Yes"), String::from("No")];
        let resp =
            execute_create_poll(deps.as_mut(), env, info, poll_id, question, options).unwrap();
        assert_eq!(resp.attributes, vec![attr("action", "create poll")])
    }

    #[test]
    fn test_execute_vote() {
        let mut deps = mock_dependencies();
        let info = mock_info("addr1", &[]);
        let env = mock_env();
        let msg = InstantiateMsg {
            admin: Some("addr1".to_string()),
        };
        let resp = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(
            resp.attributes,
            vec![
                attr("action", "instantiate"),
                attr("addr1".to_string(), "addr1".to_string())
            ]
        );

        let poll_id = "1".to_string();
        let question = "Should We Have a Meeting Today".to_string();
        let options = vec![String::from("Yes"), String::from("No")];
        let resp = execute_create_poll(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            poll_id,
            question,
            options,
        )
        .unwrap();
        assert_eq!(resp.attributes, vec![attr("action", "create poll")]);

        let msg = ExecuteMsg::Vote {
            poll_id: "1".to_string(),
            vote: "No".to_string(),
        };
        let resp = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(resp.attributes, vec![attr("action", "vote in poll")])
    }

    #[test]
    fn test_query_poll() {
        let mut deps = mock_dependencies();
        let info = mock_info("addr1", &[]);
        let env = mock_env();
        let msg = InstantiateMsg {
            admin: Some("addr1".to_string()),
        };
        let resp = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(
            resp.attributes,
            vec![
                attr("action", "instantiate"),
                attr("addr1".to_string(), "addr1".to_string())
            ]
        );
        let poll_id = "1".to_string();
        let question = "Should We Have a Meeting Today".to_string();
        let options = vec![String::from("Yes"), String::from("No")];
        let resp =
            execute_create_poll(deps.as_mut(), env.clone(), info, poll_id, question, options)
                .unwrap();
        assert_eq!(resp.attributes, vec![attr("action", "create poll")]);

        let msg = QueryMsg::Poll {
            poll_id: "1".to_string(),
        };

        let resp = query(deps.as_ref(), env, msg).unwrap();
        let get_poll: PollResponse = from_binary(&resp).unwrap();
        assert_eq!(
            get_poll,
            PollResponse {
                poll: Some(Poll {
                    admin: Addr::unchecked("addr1"),
                    question: "Should We Have a Meeting Today".to_string(),
                    options: vec![(String::from("Yes"), 0), (String::from("No"), 0)],
                })
            }
        );
    }
}
