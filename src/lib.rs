//! *Work in Progress. Currently does not work outside project it was created for.*
//! A Minimalistic Undo/Redo library created for personal projects. Use at your own risk!
//!
//! Whenever your application data changes to a new value, you can push the old value into the UndoStack using the [`UndoStack::push`] method. Calling [`UndoStack::undo`] will walk through the stack returning past_stack values, but possible values above the current position are still available via the [`UndoStack::redo`] method, which will cause the stack to walk back into the present.
//!
//! Calling undo and then pushing a new value clears the redo stack, since history was rewritten and now the future_stack state will be different.
//!
//! When calling undo and redo, the restored values are automatically restored to an associated Project type (the struct containing the data) via the `Undoable` trait, which simply contains a "restore" method where you must provide a way for the restored value to be re-applied to the Project type.
//!
//! # Creating undo values for continuous changes.
//!
//! A common case in applications with GUIs is a value that needs to be udpdated continuously on the screen while the user interacts with it (i.e. dragging a slider), but only its initial state and final will be used for undo purposes.
//!
//! You can easily achieve this using the [`UndoStack::start_buffer`] and [`UndoStack::commit_buffer`] methods at the beginning and end of the user interaction, instead of "push". The final state is only used to compare with the initial state, and nothing is actually stored if they're the same.
//!
//! # Motivation.
//!
//! I created this crate for personal use, and to satisfy a few requirements I wished for:
//! - As minimal as possible while still being useful.
//! - Does not force unwanted programming patterns, i.e. Command pattern.
//! - Undo values can be stored in multiple stacks, if needed. A use case for this can be seen in modern 3D Animation software, where the Scene undo is usually separated from the Viewport undo.
//! - Provides a simple trait (Undoable) with a single "restore" method that allows automatically re-applying the restored value to the application data.
//!
//! # Example
//!
//! Please download the repo and run "cargo run -p example" at the project root for a simple demonstration.
//! The example is located under the "example" subfolder.


#![warn(clippy::std_instead_of_core, clippy::std_instead_of_alloc)]
#![no_std]
#[cfg(feature = "std")] extern crate std;
#[cfg(feature = "std")] pub use std::{print, println};

extern crate alloc;
use alloc::{vec, vec::Vec};


/// When calling undo() or redo(), the restore() function is always called and applies
/// this value TO the project and returns a value with the previous state FROM the project.
/// The "Undoable" type, usually an enum with many variants, needs to know how to restore itself,
/// and may need to contain all information required to place it correctly into the project.
pub trait Undoable
where Self:Clone + PartialEq {
    type ProjectType;
    fn restore(self, target:&mut Self::ProjectType) -> Self;
}


/// The main struct where all the undo values are kept.
pub struct UndoStack<T> where T:Undoable {
    future_stack:Vec<T>,
    past_stack:Vec<T>,
    undo_buffer: Option<T>,
    /// Controls whether warning messages are printed or not. True by default.
    pub verbose: bool,
}


impl<T> UndoStack<T> where T:Undoable {

    /// Creates a new, empty Undo stack.
    pub fn new(verbose:bool) -> Self {
        Self {
            future_stack: vec![],
            past_stack: vec![],
            undo_buffer: None,
            verbose,
        }
    }


    /// Push a discrete "Undoable" value to the undo stack. Automatically clears future_stack redo values.
    pub fn push(&mut self, undo_value:T){
        // if let Some(top_value) = self.past_stack.last() {
        //     // Skips if value is equal
        //     if *top_value == undo_value {
        //         return
        //     }
        // }
        self.past_stack.push(undo_value);
        self.future_stack.clear();
    }


    /// Checks if a new undo value is different from the one currently at the top of the stack.
    /// Can be used to prevent pushing redundant values. In some cases, redundant values can be useful,
    /// so this check is not performed by default.
    pub fn value_is_different(&self, undo_value:T) -> bool {
        if let Some(top_value) = self.past_stack.last() {
            return *top_value != undo_value
        }
        false
    }


    /// Performs undo, which will call the "restore" method on the restored value.
    /// Returns an option with the undone value for convenience.
    pub fn undo(&mut self, project:&mut T::ProjectType) -> Option<&T> {
        self.move_undo_value(project, false)
    }


    /// Performs redo, which will call the "restore" method on the restored value.
    /// Returns an option with the redone value for convenience.
    pub fn redo(&mut self, project:&mut T::ProjectType) -> Option<&T> {
        self.move_undo_value(project, true)
    }


    /// The internal undo workhorse: moves the values to/from the appropriate stack.
    /// Returns an option with the top value being moved.
    fn move_undo_value(&mut self, project:&mut T::ProjectType, is_redo:bool) -> Option<&T> {
        // Set appropriate stacks, depending on "undo" or "redo"
        let from_stack:&mut Vec<T>;
        let to_stack:&mut Vec<T>;
        if is_redo {
            from_stack = &mut self.future_stack;
            to_stack = &mut self.past_stack;
        } else {
            from_stack = &mut self.past_stack;
            to_stack = &mut self.future_stack;
        };

        // Process undo value and location
        match from_stack.pop() {
            Some(value) => {
                let old_value = value.restore(project);
                to_stack.push(old_value);
            }
            None => {
                #[cfg(feature = "std")]{
                    if self.verbose { println!("UndoStack: No value to undo/redo."); }
                }
            }
        }
        //Return an option with whatever is at the top of the stack
        to_stack.last()
    }


    /// Completely empties the undo and redo stacks and the temporary buffer.
    pub fn clear(&mut self) {
        self.undo_buffer = None;
        self.past_stack.clear();
        self.future_stack.clear();
    }


    /// Returns true if both the undo and redo stacks are empty.
    pub fn is_empty(&self) -> bool {
        self.past_stack.is_empty() && self.future_stack.is_empty()
    }


    /// Returns true is the temporary buffer is empty.
    pub fn buffer_is_empty(&self) -> bool {
        self.undo_buffer.is_none()
    }


    /// Stores the initial value of a continuously changing interaction - i.e. dragging a GUI slider.
    pub fn start_buffer(&mut self, value:T){
        if let Some(ref value) = self.undo_buffer{
            self.push(value.clone());
            if self.verbose {
                #[cfg(feature = "std")]{
                    println!("UndoStack: Warning, previous undo value was commited automatically")
                }
            }
        }
        if self.verbose {
            #[cfg(feature = "std")]{
                println!( "UndoStack: Initiating undo buffer...");
            }
        }
        self.undo_buffer = Some(value);
    }


    /// Finalizes continuous interaction, stores value from the buffer if change has occurred.
    pub fn commit_buffer(&mut self, final_value:T){
        if let Some(ref value) = self.undo_buffer{
            if *value != final_value{
                self.push(value.clone());
            } else  if self.verbose {
                #[cfg(feature = "std")]{
                    println!("UndoStack: Skipping commit, values don't differ.");
                }
            }
            self.undo_buffer = None;
        } else if self.verbose {
            // println!("UndoStack: Warning, buffer is empty and can't be committed.")
        }
    }


    /// Returns a copy of buffer contents
    pub fn buffer(&self) -> Option<T> { self.undo_buffer.clone() }


    /// Returns an immutable reference to the past_stack vector (undo stack).
    pub fn past_stack(&self) -> &Vec<T> { &self.past_stack }


    /// Returns an immutable reference to the future_stack vector (redo stack).
    pub fn future_stack(&self) -> &Vec<T> { &self.future_stack }


    /// Pops the top value in the undo stack (past_stack) and returns it as an option.
    pub fn pop_undo(&mut self) -> Option<T> { self.past_stack.pop() }


    /// Pops the top value in the redo stack (future_stack) and returns it  as an option.
    pub fn pop_redo(&mut self) -> Option<T> { self.future_stack.pop() }

}


impl<T> Default for UndoStack<T> where T: Undoable {
    fn default() -> Self { Self::new(false) }
}
