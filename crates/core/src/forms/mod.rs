//! Implementation of the forms system available in Minecraft.
//!
//! Forms allow you to create (basic) custom interfaces which users can respond to.
//! There are three different kinds of forms: [`Custom`], [`Menu`] and [`Modal`].
//!
//! * Modal forms - This is the simplest form and consists of a body of text and two buttons
//! with customisable text. The response simply tells you whether the confirm button was pressed.
//! * Menu forms - Menu forms are similar to modal forms apart from the fact that menus can have an arbitrary amount
//! of buttons which can also have optional images next to them.
//! * Custom forms - These forms allow you to add any elements you want to the body and create your own form type.
//!
//! Submitting forms to the user should be done with the [`Subscriber`]. Every [`BedrockUser`](crate::network::BedrockUser) has one.
//! This subscriber keeps track of the active forms
//! and will automatically validate responses, returning them via a channel.
//!
//! A form response can be of two types: cancelled or success. A form will be cancelled if the user closed it manually
//! or if the user was busy (such as having their chat window opened). The success response will contain data submitted by
//! the user.

mod content;
mod custom;
mod form;
mod menu;
mod modal;

pub mod response;

pub use content::*;
pub use custom::*;
pub use form::*;
pub use menu::*;
pub use modal::*;

#[doc(inline)]
pub use response::{Response, Subscriber};
