use ffi::{MDBX_db_flags_t, MDBX_env_flags_t, MDBX_put_flags_t};

pub type ENV = MDBX_env_flags_t;
pub type DB = MDBX_db_flags_t;

#[doc = "[MDBX_put_flags_t](https://erthink.github.io/libmdbx/group__c__crud.html#ga5b8137591c45143c6d9439799be77136)"]
pub type PUT = MDBX_put_flags_t;
