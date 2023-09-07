use ink::{
    env::Error as InkEnvError,
    prelude::{format, string::String},
    LangError,
};
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum AZGroupsError {
    ContractCall(LangError),
    InkEnvError(String),
    NotFound(String),
    Unauthorised,
    UnprocessableEntity(String),
}
impl From<InkEnvError> for AZGroupsError {
    fn from(e: InkEnvError) -> Self {
        AZGroupsError::InkEnvError(format!("{e:?}"))
    }
}
impl From<LangError> for AZGroupsError {
    fn from(e: LangError) -> Self {
        AZGroupsError::ContractCall(e)
    }
}
