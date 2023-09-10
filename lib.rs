#![cfg_attr(not(feature = "std"), no_std)]

mod errors;

#[ink::contract]
mod az_groups {
    use crate::errors::AZGroupsError;
    use ink::{
        prelude::string::{String, ToString},
        storage::Mapping,
    };

    // === STRUCTS ===
    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Group {
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
        groups: Mapping<String, Group>,
        group_users: Mapping<(String, AccountId), GroupUser>,
    }
    impl AZGroups {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                groups: Mapping::default(),
                group_users: Mapping::default(),
            }
        }

        #[ink(message)]
        pub fn group_users_create(&mut self, name: String) -> Result<GroupUser, AZGroupsError> {
            // check if group with key exists
            let key: String = name.to_lowercase();
            self.groups_show(key.clone())?;
            // check if group user already exists
            let caller: AccountId = Self::env().caller();
            if self.group_users.get((key.clone(), caller)).is_some() {
                return Err(AZGroupsError::UnprocessableEntity(
                    "Group user has already been taken".to_string(),
                ));
            }

            // Create and set group user
            let group_user: GroupUser = GroupUser { role: 1 };
            self.group_users.insert((key.clone(), caller), &group_user);

            Ok(group_user)
        }

        #[ink(message)]
        pub fn group_users_update(
            &mut self,
            name: String,
            user: AccountId,
            role: u8,
        ) -> Result<GroupUser, AZGroupsError> {
            if role > 4 {
                return Err(AZGroupsError::UnprocessableEntity(
                    "Role must be less than or equal to 4".to_string(),
                ));
            }
            // check if group with key exists
            let key: String = name.to_lowercase();
            let caller: AccountId = Self::env().caller();
            if caller == user {
                return Err(AZGroupsError::Unauthorised);
            }
            let caller_group_user: GroupUser = self.group_users_show(key.clone(), caller)?;
            // Only an admin can make changes
            if caller_group_user.role < 3 {
                return Err(AZGroupsError::Unauthorised);
            }
            let mut user_group_user: GroupUser = self.group_users_show(key.clone(), user)?;
            if caller_group_user.role < user_group_user.role {
                return Err(AZGroupsError::Unauthorised);
            }
            if role > caller_group_user.role {
                return Err(AZGroupsError::Unauthorised);
            }

            user_group_user.role = role;
            self.group_users
                .insert((key.clone(), user), &user_group_user);

            Ok(user_group_user)
        }

        #[ink(message)]
        pub fn group_users_show(
            &self,
            name: String,
            user: AccountId,
        ) -> Result<GroupUser, AZGroupsError> {
            if let Some(group_user) = self.group_users.get((name.to_lowercase(), user)) {
                Ok(group_user)
            } else {
                Err(AZGroupsError::NotFound("GroupUser".to_string()))
            }
        }

        #[ink(message)]
        pub fn groups_create(&mut self, name: String) -> Result<Group, AZGroupsError> {
            // key will be name lowercased
            // check if group with key already exists
            let key: String = name.to_lowercase();
            if self.groups.get(key.clone()).is_some() {
                return Err(AZGroupsError::UnprocessableEntity(
                    "Group has already been taken".to_string(),
                ));
            }

            // Create group
            let group: Group = Group {
                name: name.clone(),
                enabled: true,
            };
            self.groups.insert(key.clone(), &group);

            // Create and set group user
            let group_user: GroupUser = GroupUser { role: 4 };
            self.group_users
                .insert((key, Self::env().caller()), &group_user);

            Ok(group)
        }

        #[ink(message)]
        pub fn groups_show(&self, name: String) -> Result<Group, AZGroupsError> {
            if let Some(group) = self.groups.get(name.to_lowercase()) {
                Ok(group)
            } else {
                Err(AZGroupsError::NotFound("Group".to_string()))
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{test::DefaultAccounts, DefaultEnvironment};

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
            let group_name: String = "The Next Wave".to_string();
            let key: String = group_name.to_lowercase();
            // when group with key does not exist
            // * it raises an error
            let mut result = az_groups.group_users_create(key.clone());
            assert_eq!(result, Err(AZGroupsError::NotFound("Group".to_string())));
            // when group with key exists
            az_groups.groups_create(key.clone()).unwrap();
            // = when GroupUser exists
            result = az_groups.group_users_create(key.clone());
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
            result = az_groups.group_users_create(key);
            assert_eq!(result.unwrap().role, 1);
        }

        #[ink::test]
        fn test_group_users_update() {
            let (accounts, mut az_groups) = init();
            let group_name: String = "The Next Wave".to_string();
            // when role is greater than 4
            // * it raises an error
            let mut result = az_groups.group_users_update(group_name.clone(), accounts.alice, 5);
            assert_eq!(
                result,
                Err(AZGroupsError::UnprocessableEntity(
                    "Role must be less than or equal to 4".to_string()
                ))
            );
            // when role is less than or equal to 4
            // = when group with key exists
            az_groups.groups_create(group_name.clone()).unwrap();
            // == when caller equals user
            // == * it raises an error
            result = az_groups.group_users_update(group_name.clone(), accounts.bob, 4);
            assert_eq!(result, Err(AZGroupsError::Unauthorised));
            // == when caller is different to user
            // === when caller does not have a group user for team
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
            result = az_groups.group_users_update(group_name.clone(), accounts.bob, 4);
            // === * it raises an error
            assert_eq!(
                result,
                Err(AZGroupsError::NotFound("GroupUser".to_string()))
            );
            // === when caller has a group user for team
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            // ==== when caller's role is less than 3
            let mut caller_group_user: GroupUser = az_groups
                .group_users
                .get((group_name.to_lowercase(), accounts.bob))
                .unwrap();
            caller_group_user.role = 2;
            az_groups.group_users.insert(
                (group_name.to_lowercase(), accounts.bob),
                &caller_group_user,
            );
            // ==== * it raises an error
            result = az_groups.group_users_update(group_name.clone(), accounts.bob, 2);
            assert_eq!(result, Err(AZGroupsError::Unauthorised));
            // ==== when caller's role is 3 or more
            caller_group_user.role = 3;
            az_groups.group_users.insert(
                (group_name.to_lowercase(), accounts.bob),
                &caller_group_user,
            );
            // ===== when user does not have a group user for team
            result = az_groups.group_users_update(group_name.clone(), accounts.charlie, 4);
            // ===== * it raises an error
            assert_eq!(
                result,
                Err(AZGroupsError::NotFound("GroupUser".to_string()))
            );
            // ===== when user has a role with team
            // ====== when caller's role is less than user's role
            let mut user_group_user: GroupUser = GroupUser { role: 4 };
            az_groups.group_users.insert(
                (group_name.to_lowercase(), accounts.charlie),
                &user_group_user,
            );
            // ====== * it raises an error
            result = az_groups.group_users_update(group_name.clone(), accounts.charlie, 4);
            assert_eq!(result, Err(AZGroupsError::Unauthorised));
            // ====== when caller's role is greater than or equal to user's role
            user_group_user = GroupUser { role: 3 };
            az_groups.group_users.insert(
                (group_name.to_lowercase(), accounts.charlie),
                &user_group_user,
            );
            // ======= when new role is less than or equal to caller's role
            // ======= * it updates the user's role
            result = az_groups.group_users_update(group_name.clone(), accounts.charlie, 3);
            assert_eq!(result.unwrap().role, 3);
            // ======= when new role is greater than caller's role
            // ======= * it raises an error
            result = az_groups.group_users_update(group_name.clone(), accounts.charlie, 4);
            assert_eq!(result, Err(AZGroupsError::Unauthorised));
        }

        #[ink::test]
        fn test_groups_create() {
            let (accounts, mut az_groups) = init();
            let group_name: String = "The Next Wave".to_string();
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
            let group_user: GroupUser = az_groups
                .group_users
                .get((key.clone(), accounts.bob))
                .unwrap();
            assert_eq!(group_user.role, 4);

            // when group with key already exists
            // * it raises an error
            result = az_groups.groups_create(key);
            assert_eq!(
                result,
                Err(AZGroupsError::UnprocessableEntity(
                    "Group has already been taken".to_string()
                ))
            );
        }
    }
}
