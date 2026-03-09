mod platform {
    #[cfg(feature = "target-obsidian")]
    pub mod obsidian;

    #[cfg(feature = "target-os")]
    pub mod os {
        pub mod server;
        pub mod ui;
    }
}

#[cfg(feature = "target-os")]
mod website;

#[cfg(feature = "target-os")]
pub use platform::os::server::server;
#[cfg(feature = "target-os")]
pub use platform::os::ui;
