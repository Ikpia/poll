#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
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
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
