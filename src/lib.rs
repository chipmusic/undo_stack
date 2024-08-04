#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/readme.md"))]

#![warn(clippy::std_instead_of_core, clippy::std_instead_of_alloc)]
#![no_std]

#[cfg(feature = "std")] extern crate std;
#[cfg(feature = "std")] pub use std::{print, println};

extern crate alloc;
use alloc::{vec, vec::Vec};

// Helps to identify the beginning and end of a group versus a single value.
#[derive(PartialEq)]
enum Kind<T> {
    Group,
    Single(T),
}

/// When calling undo() or redo(), the restore() function is always called and applies
/// this value TO the project and returns a value with the previous state FROM the project.
/// The "Undoable" type, usually an enum with many variants, needs to know how to restore itself,
/// and may need to contain all information required to place it correctly into the project.
pub trait Undoable
where
    Self: Clone + PartialEq,
{
    type ProjectType;
    fn restore(self, target: &mut Self::ProjectType) -> Self;
}

/// The main struct where all the undo values are kept.
pub struct UndoStack<T>
where
    T: Undoable,
{
    future_stack: Vec<Kind<T>>,
    past_stack: Vec<Kind<T>>,
    undo_buffer: Option<T>,
    // Only a single open group is allowed at a time. That's why it's a single boolean, not a variant of "Kind".
    open_group: bool,
    /// Controls whether warning messages are printed or not. True by default.
    /// Only works if feature "std" is enabled.
    pub verbose: bool,
}

impl<T> UndoStack<T>
where
    T: Undoable,
{
    /// Creates a new, empty Undo stack.
    pub fn new(verbose: bool) -> Self {
        Self {
            future_stack: vec![],
            past_stack: vec![],
            undo_buffer: None,
            open_group: false,
            verbose,
        }
    }

    /// Push a discrete "Undoable" value to the undo stack. Automatically clears future_stack redo values.
    /// Will do nothing if value matches the value on top of undo stack.
    pub fn push(&mut self, undo_value: T) {
        if let Some(Kind::Single(top_value)) = self.past_stack.last() {
            if *top_value == undo_value {
                return;
            }
        }
        self.past_stack.push(Kind::Single(undo_value));
        self.future_stack.clear();
    }

    /// Starts a "group" with multiple undo values that can be undone simultaneously.
    pub fn start_group(&mut self) {
        if !self.open_group {
            self.past_stack.push(Kind::Group);
            self.open_group = true;
        } else {
            self.maybe_print("UndoStack: Warning, can't open new group before closing current one.");
        }
    }

    /// Finishes the previously started undo group.
    pub fn finish_group(&mut self) {
        if self.open_group {
            self.past_stack.push(Kind::Group);
            self.open_group = false;
        } else {
            self.maybe_print("UndoStack: Warning, no open groups to close.");
        }
    }

    /// EXPERIMENTAL: Changes the last existing undo value into an open group state,
    /// **IF** that value represents a group end. This allows retroactively pushing additional
    /// single undo values to the group.
    pub fn reopen_group(&mut self) {
        if let Some(kind) = self.past_stack.pop(){
            match kind {
                Kind::Group => {
                    if self.open_group {
                        self.maybe_print("UndoStack: Warning, last value is already an open group. Skipping.");
                    } else {
                        self.open_group = true;
                    }
                },
                Kind::Single(value) => {
                    self.maybe_print("UndoStack: Warning, last value is not a group. Skipping.");
                    self.past_stack.push(Kind::Single(value));
                },
            }
        }
    }

    /// Performs undo, which will call the "restore" method on the restored value.
    /// Returns an option with the undone value for convenience.
    pub fn undo(&mut self, project: &mut T::ProjectType) -> Option<&T> {
        self.move_undo_value(project, false)
    }

    /// Performs redo, which will call the "restore" method on the restored value.
    /// Returns an option with the redone value for convenience.
    pub fn redo(&mut self, project: &mut T::ProjectType) -> Option<&T> {
        self.move_undo_value(project, true)
    }

    /// The internal undo workhorse: moves the values to/from the appropriate stack.
    /// Returns an option with the top value being moved.
    fn move_undo_value(&mut self, project: &mut T::ProjectType, is_redo: bool) -> Option<&T> {
        // Set appropriate stacks, depending on "undo" or "redo"
        let from_stack: &mut Vec<Kind<T>>;
        let to_stack: &mut Vec<Kind<T>>;
        if is_redo {
            from_stack = &mut self.future_stack;
            to_stack = &mut self.past_stack;
        } else {
            from_stack = &mut self.past_stack;
            to_stack = &mut self.future_stack;
        };

        // Process undo value and location
        match from_stack.pop() {
            Some(kind) => match kind {
                Kind::Group => {
                    to_stack.push(Kind::Group);
                    loop {
                        let next_value = from_stack.pop();
                        match next_value {
                            Some(Kind::Group) => {
                                to_stack.push(Kind::Group);
                                break;
                            }
                            Some(Kind::Single(value)) => {
                                let old_value = value.restore(project);
                                to_stack.push(Kind::Single(old_value));
                            }
                            None => {
                                #[cfg(feature = "std")]{
                                    println!("UndoStack: Warning, Undo failure due to incomplete Undo Group");
                                }
                                break;
                            }
                        }
                    }
                }
                Kind::Single(value) => {
                    let old_value = value.restore(project);
                    to_stack.push(Kind::Single(old_value));
                }
            },
            None => {
                #[cfg(feature = "std")]{
                    if self.verbose {
                        println!("UndoStack: No value to undo/redo.");
                    }
                }
            }
        }
        //Return an option with whatever is at the top of the stack
        to_stack.last().map_or_else(
            || None,
            |kind| match kind {
                Kind::Group => None,
                Kind::Single(value) => Some(value),
            },
        )
    }

    /// Completely empties the undo and redo stacks and the temporary buffer.
    pub fn clear(&mut self) {
        self.undo_buffer = None;
        self.past_stack.clear();
        self.future_stack.clear();
        self.open_group = false;
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
    pub fn start_buffer(&mut self, value: T) {
        if let Some(ref value) = self.undo_buffer {
            self.push(value.clone());
            self.maybe_print("UndoStack: Warning, previous undo value was commited automatically");
        }
        self.maybe_print("UndoStack: Initiating undo buffer...");
        self.undo_buffer = Some(value);
    }

    /// Finalizes continuous interaction, stores value from the buffer if change has occurred.
    pub fn finish_buffer(&mut self, final_value: T) {
        if let Some(ref value) = self.undo_buffer {
            if *value != final_value {
                self.push(value.clone());
            } else {
                self.maybe_print("UndoStack: Skipping commit, values don't differ.");
            }
            self.undo_buffer = None;
        } else {
            self.maybe_print("UndoStack: Warning, buffer is empty and can't be committed.");
        }
    }

    /// Returns a copy of buffer contents
    pub fn buffer(&self) -> Option<T> {
        self.undo_buffer.clone()
    }


    fn maybe_print(&self, _text:&str) {
        #[cfg(feature = "std")]{
            if self.verbose {
                println!("{}", _text);
            }
        }
    }


    // /// EXPERIMENTAL: Allows manipulating the last existing undo value into a group start,
    // /// retroactively grouping undo values together.
    // pub fn set_last_to_group_start(&mut self) {
    //     if let Some(kind) = self.past_stack.pop(){
    //         match kind {
    //             Kind::Group => {
    //                 if !self.open_group {
    //                     self.open_group = true;
    //                 } else {
    //                     #[cfg(feature = "std")]{
    //                         if self.verbose {
    //                             println!("UndoStack: Warning, last value is already a group start. Skipping.");
    //                         }
    //                     }
    //                 }
    //             },
    //             Kind::Single(value) => {
    //                 self.start_group();
    //                 self.past_stack.push(Kind::Single(value));
    //             },
    //         }
    //     }
    // }

    // /// EXPERIMENTAL: Allows manipulating the last existing undo value into a group end.
    // pub fn set_last_to_group_end(&mut self) {
    //     if let Some(kind) = self.past_stack.pop(){
    //         match kind {
    //             Kind::Group => {
    //                 if !self.open_group {
    //                     #[cfg(feature = "std")]{
    //                         if self.verbose {
    //                             println!("UndoStack: Warning, last value is already a group end. Skipping.");
    //                         }
    //                     }
    //                 } else {
    //                     self.open_group = false;
    //                 }
    //             },
    //             Kind::Single(value) => {
    //                 self.past_stack.push(Kind::Single(value));
    //                 self.finish_group();
    //             },
    //         }
    //     }
    // }


    // /// Pops the top value in the undo stack (past_stack) and returns it as an option.
    // pub fn pop_undo(&mut self) -> Option<T> {
    //     self.past_stack.pop().map_or_else(
    //         || None,
    //         |kind| {
    //             match kind {
    //                 Kind::Group => None,
    //                 Kind::Single(value) => Some(value),
    //             }
    //         }
    //     )
    // }

    // /// Pops the top value in the redo stack (future_stack) and returns it  as an option.
    // pub fn pop_redo(&mut self) -> Option<T> {
    //     self.future_stack.pop().map_or_else(
    //         || None,
    //         |kind| {
    //             match kind {
    //                 Kind::Group => None,
    //                 Kind::Single(value) => Some(value),
    //             }
    //         }
    //     )
    // }

    // /// Returns an immutable reference to the past_stack vector (undo stack).
    // pub fn past_stack(&self) -> &Vec<T> { &self.past_stack }

    // /// Returns an immutable reference to the future_stack vector (redo stack).
    // pub fn future_stack(&self) -> &Vec<T> { &self.future_stack }

    // /// Checks if a new undo value is different from the one currently at the top of the stack.
    // /// Can be used to prevent pushing redundant values. In some cases, redundant values can be useful,
    // /// so this check is not performed by default.
    // pub fn value_is_different(&self, undo_value:T) -> bool {
    //     if let Some(kind) = self.past_stack.last() {
    //         if let Kind::Single(top_value) = kind {
    //             return *top_value != undo_value
    //         }
    //     }
    //     false
    // }
}

impl<T> Default for UndoStack<T>
where
    T: Undoable,
{
    fn default() -> Self {
        Self::new(false)
    }
}
