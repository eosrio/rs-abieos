#[cfg(feature = "cpp-backend")]
#[allow(dead_code)]
mod cpp;

#[cfg(feature = "rust-backend")]
mod rust;

#[cfg(feature = "rust-backend")]
pub(crate) use rust::get_type_for_table_name;

#[cfg(all(feature = "cpp-backend", not(feature = "rust-backend")))]
pub mod bindings {
    pub use super::cpp::*;
}

#[cfg(feature = "rust-backend")]
pub mod bindings {
    pub use super::rust::*;
}

#[cfg(feature = "cpp-oracle")]
pub mod cpp_oracle {
    pub use super::cpp::*;
}

#[cfg(not(any(feature = "cpp-backend", feature = "rust-backend")))]
compile_error!("enable either `cpp-backend` or `rust-backend`");
