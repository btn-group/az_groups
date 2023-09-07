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

    // 0: Applicant
    // 1: Banned
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
        pub fn groups_create(&mut self, name: String) -> Result<Group, AZGroupsError> {
            // key will be name lowercased
            // check if group with key already exists
            let key: String = name.to_lowercase();
            if self.groups.get(key.clone()).is_some() {
                return Err(AZGroupsError::UnprocessableEntity(
                    "Group not unique.".to_string(),
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
        pub fn group_users_create(&mut self, name: String) -> Result<GroupUser, AZGroupsError> {
            // check if group with key exists
            let key: String = name.to_lowercase();
            if self.groups.get(key.clone()).is_none() {
                return Err(AZGroupsError::NotFound("Group".to_string()));
            }
            // check if group user already exists
            let caller: AccountId = Self::env().caller();
            if self.group_users.get((key.clone(), caller)).is_some() {
                return Err(AZGroupsError::UnprocessableEntity(
                    "GroupUser not unique.".to_string(),
                ));
            }

            // Create and set group user
            let group_user: GroupUser = GroupUser { role: 0 };
            self.group_users.insert((key.clone(), caller), &group_user);

            Ok(group_user)
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
                    "GroupUser not unique.".to_string()
                ))
            );
            // = when GroupUser doesn't exist
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            // = * it creates the group user with the role applicant
            result = az_groups.group_users_create(key);
            assert_eq!(result.unwrap().role, 0);
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
                    "Group not unique.".to_string()
                ))
            );
        }
    }
}
