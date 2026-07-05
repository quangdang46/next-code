//! Re-export stream types from the downstream OpenAI provider crate.
//!
//! jcode-base's `openai.rs` uses `mod stream;` referencing types that now
//! live in `jcode_provider_openai::stream`. This thin shim re-exports them
//! so the `use self::stream::*` paths continue to compile from the base
//! crate without code motion.

pub use jcode_provider_openai::stream::{
    OpenAIResponsesStream, parse_openai_response_event,
};

#[cfg(test)]
pub use jcode_provider_openai::stream::{
    handle_openai_output_item, parse_text_wrapped_tool_call,
};
