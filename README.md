# AZ Groups

A smart contract that allows the decentralised management of groups. Built for the Aleph Zero blockchain, it's initial purpose is to use with a decentralised smart contract hub. The idea is to increase trust for users, by being able to associate an address with a group e.g. an upload by an address that is part of the Aleph Zero Foundation group will be more trustable.

### Roles

0 => Banned\
1 => Applicant\
2 => Member\
3 => Admin\
4 => SuperAdmin

### Rules

**Creating a group**:
* Names must unique (case-insensitive).
* Names will have whitespace removed from start and end.
```
fn groups_create(&mut self, name: String) -> Result<Group, AZGroupsError>
```
**Updating a group**:
* Super admin can change the name and enabled status of the group.
```
fn groups_update(&mut self, id: u32, name: String, enabled: bool) -> Result<Group, AZGroupsError>
```
**Joining**:
* Any non-member can apply to join. They can't be invited.
```
fn group_users_create(&mut self, group_id: u32) -> Result<GroupUser, AZGroupsError>
```
**Kicking**: 
* Admin and super admin can kick members of the same role or less.
```
fn group_users_destroy(&mut self, group_id: u32, user: AccountId) -> Result<(), AZGroupsError>
```
**Leaving**:
* All members except for banned and super admin can leave a group.
* Super admin can't leave as it may leave a group without a super admin.
```
fn group_users_destroy(&mut self, group_id: u32, user: AccountId) -> Result<(), AZGroupsError>
```
**Updating roles**:
* Admin and super admin can update the role of members with the same role or less.
```
pub fn group_users_update(&mut self, group_id: u32, user: AccountId, role: u8) -> Result<GroupUser, AZGroupsError>
```

## Integration

Please use the group's id where possible as the name can be changed.

### Contract level

To verify a user's membership, you would have to check that the group is enabled and that the user has a role >= 2. This can be done through:
```
pub fn validate_membership(&self, group_id: u32, user: AccountId) -> Result<u8, AZGroupsError>
```

Here is an example of a cross contract call:
```
fn validate_membership(
    &self,
    group_id: u32,
    user: AccountId,
) -> Result<u8, AZSmartContractHubError> {
    match cfg!(test) {
        true => unimplemented!(
            "`invoke_contract()` not being supported (tests end up panicking)"
        ),
        false => {
            use ink::env::call::{build_call, ExecutionInput, Selector};

            const VALIDATE_MEMBERSHIP_SELECTOR: [u8; 4] =
                ink::selector_bytes!("validate_membership");
            Ok(build_call::<Environment>()
                .call(self.az_groups_address)
                .exec_input(
                    ExecutionInput::new(Selector::new(VALIDATE_MEMBERSHIP_SELECTOR))
                        .push_arg(group_id)
                        .push_arg(user),
                )
                .returns::<core::result::Result<u8, AZGroupsError>>()
                .invoke()?)
        }
    }
}
```

## Getting Started
### Prerequisites

* [Cargo](https://doc.rust-lang.org/cargo/)
* [Rust](https://www.rust-lang.org/)
* [ink!](https://use.ink/)
* [Cargo Contract v2.0.1](https://github.com/paritytech/cargo-contract)
```zsh
cargo install --force --locked cargo-contract --version 2.0.1
```

### Checking code

```zsh
cargo checkmate
cargo sort
```

## Deployment

1. Build contract:
```sh
cargo contract build --release
```
2. If setting up locally, start a local development chain. Download the binary [here](https://github.com/paritytech/substrate-contracts-node/releases), install, then run:
```sh
substrate-contracts-node
```
3. Upload, initialise and interact with contract at [Contracts UI](https://contracts-ui.substrate.io/).

## References 

* https://substrate.stackexchange.com/questions/3765/contract-storage-needs-nested-orderbooks-best-practice-way-to-structure-dapp/3993
