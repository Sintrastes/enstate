#![feature(
    coroutines,
    coroutine_trait,
    trait_alias,
    never_type,
    exhaustive_patterns,
    impl_trait_in_assoc_type
)]
#![no_std]

pub mod coroutines;
pub mod integration;
pub mod machine;
pub mod optics;
