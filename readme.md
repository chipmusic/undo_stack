**Work in Progress. Not safe for use, still needs a lot of testing.**

A Minimalist Undo/Redo library created for personal projects. Use at your own risk!

Whenever your application data changes to a new value, you can push the old value into the UndoStack using the [UndoStack::push] method. Calling [UndoStack::undo] will walk through the stack returning past_stack values, but possible values above the current position are still available via the [UndoStack::redo] method, which will cause the stack to walk back into the present.

Calling undo and then pushing a new value clears the redo stack, since history was rewritten and now the future_stack state will be different.

When calling undo and redo, the restored values are automatically restored to an associated Project type (the struct containing the data) via the [Undoable] trait, which simply contains a "restore" method where you must provide a way for the restored value to be re-applied to the Project type.

## Motivation.

I created this crate for personal use, and to satisfy a few requirements I wished for:
- As minimal as possible while still being useful.
- No dependencies outside the standard "alloc" crate to use Vecs.
- Undo values can be stored in multiple stacks, if needed. A use case for this can be seen in modern 3D Animation software, where the Scene undo is usually separated from the Viewport undo.
- Provides a simple trait (Undoable) with a single "restore" method that allows automatically re-applying the restored value to the application data.

## Creating undo values for continuous changes.

A common case in applications with GUIs is a value that needs to be updated continuously on the screen while the user interacts with it (i.e. dragging a slider), but only its initial state and final will be used for undo purposes.

You can easily achieve this using the [UndoStack::start_buffer] and [UndoStack::finish_buffer] methods at the beginning and end of the user interaction, instead of "push". The final state is only used to compare with the initial state, and nothing is actually stored if they're the same.

## Creating undo groups

Another common situation is grouping multiple undo values, individually tracked, into a single undo or redo operation. To do that, "open" a group with [UndoStack::start_group], perform all the undo pushes, and then "close" it with [UndoStack::finish_group]. This last step is extremely important, failure to use it may cause unpredictable results and loss of the whole undo stack contents (but hopefully no crashes!).

## Examples

Please download the repo and run *"cargo run -p example_single_values"* at the project root for a simple demonstration. You can also try *"cargo run -p example_group"* for a simple use of undo groups, i.e. undoing multiple values at once.

## Features

By default the "std" feature is disabled. To see warning messages you'll need to enable this feature (the "verbose" field is true by default). Messages prefixed by "Warning:" are helpful since they're evidence you're doing something wrong on your end - like closing a group before opening one - and ideally you should never see any of them.
