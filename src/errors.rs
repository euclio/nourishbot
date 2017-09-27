//! Error handling.

use std::io;

error_chain! {
    foreign_links {
        Io(io::Error) #[doc = "I/O errors."];
    }

    errors {
        /// Network errors.
        Network(reason: String) {
            description("network"),
            display("Problem with the network: {}", reason)
        }

        /// The menu was empty.
        EmptyMenu {
            description("empty menu"),
            display("The menu was empty"),
        }
    }
}
