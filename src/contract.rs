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
    Ok(Response::new())
}

fn execute_vote(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    poll_id: String,
    vote: String,
) -> Result<Response, ContractError> {
    let poll = POLL.may_load(deps.storage, poll_id.clone())?;

    match poll {
        Some(mut poll) => {
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
        }
        None => (),
    };
    Ok(Response::new())
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
mod tests {}
