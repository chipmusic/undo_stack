use undo_stack::{UndoStack, Undoable};

// Let's create a small test. Type definitions and trait implementation are below.
fn main() {
    // Create data structures with initial values
    let mut undo_stack = UndoStack::<UndoValue>::new();
    let mut proj = Project { a: 5, b: 1.0 };
    println!("{:?}", proj);

    // Modify a few times, always pushing undo values before we enter new ones.
    // Create an undo group before making all changes.
    undo_stack.start_group();
    undo_stack.push(UndoValue::AllValues {
        a: proj.a,
        b: proj.b,
    });
    proj.a = 50;
    proj.b = 10.0;
    println!("{:?}", proj);

    undo_stack.push(UndoValue::ValueA(proj.a));
    proj.a = 2000000;
    println!("{:?}", proj);

    undo_stack.push(UndoValue::ValueB(proj.b));
    proj.b = 1000000.0;
    println!("{:?}", proj);

    // Final values
    undo_stack.push(UndoValue::AllValues {
        a: proj.a,
        b: proj.b,
    });
    undo_stack.finish_group();
    proj.a = 555;
    proj.b = 222.0;
    println!("{:?}", proj);

    // Test undo!
    println!("\nPerforming single undo, value will match the initial one");
    undo_stack.undo(&mut proj);
    println!("{:?}", proj);

    println!("\nPerforming single redo, value will go all the way to the final one");
    undo_stack.redo(&mut proj);
    println!("{:?}\n", proj);
}

// Our project type that holds the main data.
#[derive(Debug, Clone, PartialEq)]
struct Project {
    a: i32,
    b: f32,
}

// The UndoValue enum needs to account for all values that can be changed in the project.
// Some variants need to include the "insertion point" in the payload, like "page, column, row", etc.
// But for this example we'll keep things extra simple.
#[derive(Clone, PartialEq, Debug)]
enum UndoValue {
    ValueA(i32),
    ValueB(f32),
    AllValues { a: i32, b: f32 },
}

// This trait allows the UndoStack to properly locate and "apply" any Undo value,
// regardless of the operation being an Undo or Redo.
impl Undoable for UndoValue {
    type ProjectType = Project;

    // This function applies the restored value to the project, and returns the replaced value
    fn restore(self, proj: &mut Self::ProjectType) -> Self {
        match self {
            UndoValue::ValueA(value) => {
                let replaced = proj.a; // First we cache the current value
                proj.a = value; // Then we set the new one directly
                UndoValue::ValueA(replaced) // A new UndoValue with the replaced value is returned
            }
            UndoValue::ValueB(value) => {
                let replaced = proj.b;
                proj.b = value;
                UndoValue::ValueB(replaced)
            }
            UndoValue::AllValues { a, b } => {
                let replaced = proj.clone();
                proj.a = a;
                proj.b = b;
                UndoValue::AllValues {
                    a: replaced.a,
                    b: replaced.b,
                }
            }
        }
    }
}
