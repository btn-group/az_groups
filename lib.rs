#![cfg_attr(not(feature = "std"), no_std)]

mod errors;

#[ink::contract]
mod az_groups {
    use crate::errors::AZGroupsError;
    use ink::{
        prelude::string::{String, ToString},
        storage::Mapping,
    };

    // === EVENTS ===
    #[ink(event)]
    pub struct Create {
        id: u32,
        name: String,
    }

    #[ink(event)]
    pub struct Update {
        id: u32,
        name: String,
        enabled: bool,
    }

    #[ink(event)]
    pub struct GroupUserCreate {
        group_id: u32,
        user: AccountId,
        role: u8,
    }

    #[ink(event)]
    pub struct GroupUserDestroy {
        group_id: u32,
        user: AccountId,
    }

    #[ink(event)]
    pub struct GroupUserUpdate {
        group_id: u32,
        user: AccountId,
        role: u8,
    }

    // === STRUCTS ===
    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Group {
        id: u32,
        name: String,
        enabled: bool,
    }

    // 0: Banned
    // 1: Applicant
    // 2: Member
    // 3: Admin
    // 4: SuperAdmin
    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct GroupUser {
        role: u8,
    }

    #[ink(storage)]
    pub struct AZGroups {
        groups: Mapping<u32, Group>,
        group_id_by_name: Mapping<String, u32>,
        groups_total: u32,
        group_users: Mapping<(u32, AccountId), GroupUser>,
    }
    impl Default for AZGroups {
        fn default() -> Self {
            Self::new()
        }
    }
    impl AZGroups {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                groups: Mapping::default(),
                group_id_by_name: Mapping::default(),
                groups_total: 0,
                group_users: Mapping::default(),
            }
        }

        #[ink(message)]
        pub fn group_users_create(&mut self, group_id: u32) -> Result<GroupUser, AZGroupsError> {
            // check if group exists
            self.groups_show(group_id)?;
            // check if group user already exists
            let user: AccountId = Self::env().caller();
            if self.group_users.get((group_id, user)).is_some() {
                return Err(AZGroupsError::UnprocessableEntity(
                    "Group user has already been taken".to_string(),
                ));
            }

            // Create and set group user
            let group_user: GroupUser = GroupUser { role: 1 };
            self.group_users.insert((group_id, user), &group_user);

            // emit event
            self.env().emit_event(GroupUserCreate {
                group_id,
                user,
                role: group_user.role,
            });

            Ok(group_user)
        }

        // User can leave the group, as long as they aren't a super admin
        // Use can be kicked by an admin or super-admin, as long as they are of the same role level
        // You should be able to destroy your own as long as you aren't a super admin
        // This is because if a super admin kicks themselves, there's a chance that the group would be left without one
        // The only way a super admin can leave the group is to be kicked by another super admin        #[ink(message)]
        pub fn group_users_destroy(
            &mut self,
            group_id: u32,
            user: AccountId,
        ) -> Result<(), AZGroupsError> {
            let caller: AccountId = Self::env().caller();
            let caller_group_user: GroupUser = self.group_users_show(group_id, caller)?;
            let user_group_user: GroupUser = self.group_users_show(group_id, user)?;
            if caller == user {
                if caller_group_user.role == 4 {
                    return Err(AZGroupsError::Unauthorised);
                }
            } else if caller_group_user.role < 3 || caller_group_user.role < user_group_user.role {
                return Err(AZGroupsError::Unauthorised);
            }
            self.group_users.remove((group_id, user));

            // emit event
            self.env().emit_event(GroupUserDestroy { group_id, user });

            Ok(())
        }

        #[ink(message)]
        pub fn group_users_show(
            &self,
            group_id: u32,
            user: AccountId,
        ) -> Result<GroupUser, AZGroupsError> {
            if let Some(group_user) = self.group_users.get((group_id, user)) {
                Ok(group_user)
            } else {
                Err(AZGroupsError::NotFound("GroupUser".to_string()))
            }
        }

        #[ink(message)]
        pub fn group_users_update(
            &mut self,
            group_id: u32,
            user: AccountId,
            role: u8,
        ) -> Result<GroupUser, AZGroupsError> {
            if role > 4 {
                return Err(AZGroupsError::UnprocessableEntity(
                    "Role must be less than or equal to 4".to_string(),
                ));
            }
            let caller: AccountId = Self::env().caller();
            if caller == user {
                return Err(AZGroupsError::Unauthorised);
            }
            let caller_group_user: GroupUser = self.group_users_show(group_id, caller)?;
            // Only an admin can make changes
            if caller_group_user.role < 3 {
                return Err(AZGroupsError::Unauthorised);
            }
            let mut user_group_user: GroupUser = self.group_users_show(group_id, user)?;
            if caller_group_user.role < user_group_user.role {
                return Err(AZGroupsError::Unauthorised);
            }
            if role > caller_group_user.role {
                return Err(AZGroupsError::Unauthorised);
            }

            user_group_user.role = role;
            self.group_users.insert((group_id, user), &user_group_user);

            // emit event
            self.env().emit_event(GroupUserUpdate {
                group_id,
                user,
                role,
            });

            Ok(user_group_user)
        }

        #[ink(message)]
        pub fn groups_create(&mut self, name: String) -> Result<Group, AZGroupsError> {
            let formatted_name: String = name.trim().to_string();
            if formatted_name.is_empty() {
                return Err(AZGroupsError::UnprocessableEntity(
                    "Name can't be blank".to_string(),
                ));
            };
            if self.groups_total == u32::MAX {
                return Err(AZGroupsError::UnprocessableEntity(
                    "Group limit reached".to_string(),
                ));
            }
            // key will be name lowercased
            // check if group with key already exists
            let key: String = formatted_name.to_lowercase();
            if self.group_id_by_name.get(key.clone()).is_some() {
                return Err(AZGroupsError::UnprocessableEntity(
                    "Group has already been taken".to_string(),
                ));
            }

            let user: AccountId = Self::env().caller();
            // Create group
            let group: Group = Group {
                id: self.groups_total,
                name: formatted_name.clone(),
                enabled: true,
            };
            self.groups.insert(group.id, &group);

            // Map group name to id
            self.group_id_by_name.insert(key, &group.id);

            // Create and set group user
            let group_user: GroupUser = GroupUser { role: 4 };
            self.group_users.insert((group.id, user), &group_user);

            // Increase groups_total
            self.groups_total += 1;

            // emit event
            self.env().emit_event(Create {
                id: group.id,
                name: formatted_name,
            });
            self.env().emit_event(GroupUserCreate {
                group_id: group.id,
                user,
                role: group_user.role,
            });

            Ok(group)
        }

        #[ink(message)]
        pub fn groups_show(&self, id: u32) -> Result<Group, AZGroupsError> {
            if let Some(group) = self.groups.get(id) {
                Ok(group)
            } else {
                Err(AZGroupsError::NotFound("Group".to_string()))
            }
        }

        #[ink(message)]
        pub fn groups_update(
            &mut self,
            id: u32,
            new_name: Option<String>,
            enabled: Option<bool>,
        ) -> Result<Group, AZGroupsError> {
            let mut group: Group = self.groups_show(id)?;
            let caller: AccountId = Self::env().caller();
            let caller_group_user: GroupUser = self.group_users_show(id, caller)?;
            if caller_group_user.role < 4 {
                return Err(AZGroupsError::Unauthorised);
            }

            if let Some(mut new_name_unwrapped) = new_name {
                new_name_unwrapped = AZGroups::format_group_name(new_name_unwrapped);
                if new_name_unwrapped.is_empty() {
                    return Err(AZGroupsError::UnprocessableEntity(
                        "Name can't be blank".to_string(),
                    ));
                };

                let new_key: String = new_name_unwrapped.to_lowercase();
                let old_key: String = group.name.to_lowercase();
                if new_key != old_key && self.group_id_by_name.get(new_key.clone()).is_some() {
                    return Err(AZGroupsError::UnprocessableEntity(
                        "Group has already been taken".to_string(),
                    ));
                }

                // remove old mapping
                self.group_id_by_name.remove(old_key);
                group.name = new_name_unwrapped;
                self.group_id_by_name.insert(new_key, &id);
            }
            if let Some(enabled_unwrapped) = enabled {
                group.enabled = enabled_unwrapped
            }
            self.groups.insert(id, &group);

            // emit event
            self.env().emit_event(Update {
                id,
                name: group.name.clone(),
                enabled: group.enabled,
            });

            Ok(group)
        }

        fn format_group_name(name: String) -> String {
            name.trim().to_string()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{test::DefaultAccounts, DefaultEnvironment};

        // === CONSTANTS ===
        const MOCK_GROUP_NAME: &str = "The Next Wave";

        // === HELPERS ===
        fn init() -> (DefaultAccounts<DefaultEnvironment>, AZGroups) {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            let az_groups = AZGroups::new();
            (accounts, az_groups)
        }

        // === TEST HANDLES ===
        #[ink::test]
        fn test_group_users_create() {
            let (accounts, mut az_groups) = init();
            let group_name: String = MOCK_GROUP_NAME.to_string();
            // when group with id does not exist
            // * it raises an error
            let mut result = az_groups.group_users_create(0);
            assert_eq!(result, Err(AZGroupsError::NotFound("Group".to_string())));
            // when group with id exists
            az_groups.groups_create(group_name).unwrap();
            // = when GroupUser exists
            result = az_groups.group_users_create(0);
            // = * it raises an error
            assert_eq!(
                result,
                Err(AZGroupsError::UnprocessableEntity(
                    "Group user has already been taken".to_string()
                ))
            );
            // = when GroupUser doesn't exist
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            // = * it creates the group user with the role applicant
            result = az_groups.group_users_create(0);
            assert_eq!(result.unwrap().role, 1);
        }

        #[ink::test]
        fn test_group_users_destroy() {
            let (accounts, mut az_groups) = init();
            let group_name: String = MOCK_GROUP_NAME.to_string();
            //  when group with key exists
            az_groups.groups_create(group_name.clone()).unwrap();
            // = when caller does not have a group user for team
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
            let mut result = az_groups.group_users_destroy(0, accounts.bob);
            // = * it raises an error
            assert_eq!(
                result,
                Err(AZGroupsError::NotFound("GroupUser".to_string()))
            );
            // = when caller has a group user for team
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            // == when user does not have a group user for team
            result = az_groups.group_users_destroy(0, accounts.charlie);
            // == * it raises an error
            assert_eq!(
                result,
                Err(AZGroupsError::NotFound("GroupUser".to_string()))
            );
            // == when user has a group user for team
            // === when caller equals user
            // ==== when role is super admin
            // ==== * it raises an error
            result = az_groups.group_users_destroy(0, accounts.bob);
            assert_eq!(result, Err(AZGroupsError::Unauthorised));
            // ==== when role is not super admin
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);

            az_groups.group_users_create(0).unwrap();
            // ==== * it destroys UserGroup
            az_groups.group_users_destroy(0, accounts.charlie).unwrap();
            assert!(az_groups.group_users.get((0, accounts.charlie)).is_none());
            // === when caller does not equal user
            // ==== when caller role is less than 3 (less than admin)
            az_groups.group_users_create(0).unwrap();
            // ==== * it raises an error
            result = az_groups.group_users_destroy(0, accounts.bob);
            assert_eq!(result, Err(AZGroupsError::Unauthorised));
            // ==== when caller role is greater than or equal to 3
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            az_groups
                .group_users_update(0, accounts.charlie, 3)
                .unwrap();
            // ===== when caller's role is less than user's role
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
            // ===== * it raises an error
            result = az_groups.group_users_destroy(0, accounts.bob);
            assert_eq!(result, Err(AZGroupsError::Unauthorised));
            // ===== when caller's role is greater than or equal to user's role
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            az_groups
                .group_users_update(0, accounts.charlie, 4)
                .unwrap();
            // ===== * it destroys UserGroup
            az_groups.group_users_destroy(0, accounts.charlie).unwrap();
            assert!(az_groups.group_users.get((0, accounts.charlie)).is_none());
        }

        #[ink::test]
        fn test_group_users_update() {
            let (accounts, mut az_groups) = init();
            let group_name: String = MOCK_GROUP_NAME.to_string();
            // when role is greater than 4
            // * it raises an error
            let mut result = az_groups.group_users_update(0, accounts.alice, 5);
            assert_eq!(
                result,
                Err(AZGroupsError::UnprocessableEntity(
                    "Role must be less than or equal to 4".to_string()
                ))
            );
            // when role is less than or equal to 4
            // = when group with key exists
            az_groups.groups_create(group_name).unwrap();
            // == when caller equals user
            // == * it raises an error
            result = az_groups.group_users_update(0, accounts.bob, 4);
            assert_eq!(result, Err(AZGroupsError::Unauthorised));
            // == when caller is different to user
            // === when caller does not have a group user for team
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
            result = az_groups.group_users_update(0, accounts.bob, 4);
            // === * it raises an error
            assert_eq!(
                result,
                Err(AZGroupsError::NotFound("GroupUser".to_string()))
            );
            // === when caller has a group user for team
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            // ==== when caller's role is less than 3
            let mut caller_group_user: GroupUser =
                az_groups.group_users.get((0, accounts.bob)).unwrap();
            caller_group_user.role = 2;
            az_groups
                .group_users
                .insert((0, accounts.bob), &caller_group_user);
            // ==== * it raises an error
            result = az_groups.group_users_update(0, accounts.bob, 2);
            assert_eq!(result, Err(AZGroupsError::Unauthorised));
            // ==== when caller's role is 3 or more
            caller_group_user.role = 3;
            az_groups
                .group_users
                .insert((0, accounts.bob), &caller_group_user);
            // ===== when user does not have a group user for team
            result = az_groups.group_users_update(0, accounts.charlie, 4);
            // ===== * it raises an error
            assert_eq!(
                result,
                Err(AZGroupsError::NotFound("GroupUser".to_string()))
            );
            // ===== when user has a role with team
            // ====== when caller's role is less than user's role
            let mut user_group_user: GroupUser = GroupUser { role: 4 };
            az_groups
                .group_users
                .insert((0, accounts.charlie), &user_group_user);
            // ====== * it raises an error
            result = az_groups.group_users_update(0, accounts.charlie, 4);
            assert_eq!(result, Err(AZGroupsError::Unauthorised));
            // ====== when caller's role is greater than or equal to user's role
            user_group_user = GroupUser { role: 3 };
            az_groups
                .group_users
                .insert((0, accounts.charlie), &user_group_user);
            // ======= when new role is less than or equal to caller's role
            // ======= * it updates the user's role
            result = az_groups.group_users_update(0, accounts.charlie, 3);
            assert_eq!(result.unwrap().role, 3);
            // ======= when new role is greater than caller's role
            // ======= * it raises an error
            result = az_groups.group_users_update(0, accounts.charlie, 4);
            assert_eq!(result, Err(AZGroupsError::Unauthorised));
        }

        #[ink::test]
        fn test_groups_create() {
            let (accounts, mut az_groups) = init();
            let group_name: String = MOCK_GROUP_NAME.to_string();
            let key: String = group_name.to_lowercase();
            // when group with key does not exist
            // * it creates the group with the supplied name
            // * it sets the group to enabled
            // * it returns the created group
            let mut result = az_groups.groups_create(group_name.clone());
            let group = result.unwrap();
            assert_eq!(group.name, group_name);
            assert_eq!(group.enabled, true);
            // * it creates and sets a new GroupUser with the caller as super admin
            let group_user: GroupUser = az_groups.group_users.get((0, accounts.bob)).unwrap();
            assert_eq!(group_user.role, 4);
            // * it sets the group_id_by_name
            assert_eq!(az_groups.group_id_by_name.get(key.clone()).unwrap(), 0);
            // * it increases the groups total by one
            assert_eq!(az_groups.groups_total, 1);
            // when group with key already exists
            // * it raises an error
            result = az_groups.groups_create(key);
            assert_eq!(
                result,
                Err(AZGroupsError::UnprocessableEntity(
                    "Group has already been taken".to_string()
                ))
            );
            // when groups_total is u32 max
            az_groups.groups_total = u32::MAX;
            // * it raises an error
            result = az_groups.groups_create("XXXX".to_string());
            assert_eq!(
                result,
                Err(AZGroupsError::UnprocessableEntity(
                    "Group limit reached".to_string()
                ))
            );
            // when group_name is blank
            // * it raises an error
            result = az_groups.groups_create(" ".to_string());
            assert_eq!(
                result,
                Err(AZGroupsError::UnprocessableEntity(
                    "Name can't be blank".to_string()
                ))
            );
        }

        #[ink::test]
        fn test_groups_update() {
            let (accounts, mut az_groups) = init();
            let group_name: String = MOCK_GROUP_NAME.to_string();
            let key: String = group_name.to_lowercase();
            // when group with key does not exist
            // * it raises an error
            let mut result = az_groups.groups_update(0, None, None);
            assert_eq!(result, Err(AZGroupsError::NotFound("Group".to_string())));
            // when group with key exists
            az_groups.groups_create(group_name.clone()).unwrap();
            // = when caller is not part of group
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
            // = * it raises an error
            result = az_groups.groups_update(0, None, None);
            assert_eq!(
                result,
                Err(AZGroupsError::NotFound("GroupUser".to_string()))
            );
            // = when caller is part of group
            az_groups.group_users_create(0).unwrap();
            // == when caller is not a super admin
            // == * it raises an error
            result = az_groups.groups_update(0, None, None);
            assert_eq!(result, Err(AZGroupsError::Unauthorised));
            // == when caller is a super admin
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            // === when new_name is present
            // ==== when new_name is empty blank
            // ==== * it raises an error
            result = az_groups.groups_update(0, Some(" ".to_string()), Some(false));
            assert_eq!(
                result,
                Err(AZGroupsError::UnprocessableEntity(
                    "Name can't be blank".to_string()
                ))
            );
            // ==== when new_name is available
            // ==== * it updates the group
            let mut new_name: String = "King Kong".to_string();
            result = az_groups.groups_update(0, Some(new_name.clone()), Some(false));
            assert_eq!(
                result.unwrap(),
                Group {
                    id: 0,
                    name: new_name.clone(),
                    enabled: false
                }
            );
            // ==== * it removes the old group_id_by_name map
            assert!(az_groups.group_id_by_name.get(key).is_none());
            // ==== * it create the new group_id_by_name map
            assert_eq!(
                az_groups
                    .group_id_by_name
                    .get(new_name.to_lowercase())
                    .unwrap(),
                0
            );
            // ==== when new_name is taken
            // ===== when new_name's key is the same as the original key
            new_name = new_name.to_uppercase() + " ";
            result = az_groups.groups_update(0, Some(new_name.clone()), Some(true));
            // ===== * it updates
            assert_eq!(
                result.unwrap(),
                Group {
                    id: 0,
                    name: AZGroups::format_group_name(new_name),
                    enabled: true
                }
            );
            // ===== when new_name's key is different from the original key
            az_groups.group_id_by_name.insert("a".to_string(), &1);
            result = az_groups.groups_update(0, Some("A".to_string()), Some(true));
            // ===== * it raises an error
            assert_eq!(
                result,
                Err(AZGroupsError::UnprocessableEntity(
                    "Group has already been taken".to_string()
                ))
            );
        }
    }
}
