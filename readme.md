A Minimalistic Undo/Redo library created for personal projects. Use at your own risk!

Whenever your application data changes to a new value, you can push the old value into the UndoStack using the `UndoStack::push` method. Calling `UndoStack::undo` will walk through the stack returning past_stack values, but possible values above the current position are still available via the `UndoStack::redo` method, which will cause the stack to walk back into the present.

Calling undo and then pushing a new value clears the redo stack, since history was rewritten and now the future_stack state will be different.

When calling undo and redo, the restored values are automatically restored to an associated Project type (the struct containing the data) via the `Undoable` trait, which simply contains a "restore" method where you must provide a way for the restored value to be re-applied to the Project type.

# Creating undo values for continuous changes.

A common case in applications with GUIs is a value that needs to be udpdated continuously on the screen while the user interacts with it (i.e. dragging a slider), but only its initial state and final will be used for undo purposes.

You can easily achieve this using the [`UndoStack::start_buffer`] and [`UndoStack::commit_buffer`] methods at the beginning and end of the user interaction, instead of "push". The final state is only used to compare with the initial state, and nothing is actually stored if they're the same.

# Motivation.

I created this crate for personal use, and to satisfy a few requirements I wished for:
- As minimal as possible while still being useful.
- Does not force unwanted programming patterns, i.e. Command pattern.
- Undo values can be stored in multiple stacks, if needed. A use case for this can be seen in modern 3D Animation software, where the Scene undo is usually separated from the Viewport undo.
- Provides a simple trait (Undoable) with a single "restore" method that allows automatically re-applying the restored value to the application data.
