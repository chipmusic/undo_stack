use super::*;

// A project of three numbers, and an undo value that knows which one it was
#[derive(Clone, PartialEq, Debug)]
struct Set {
    at: usize,
    value: i32,
}

impl Undoable for Set {
    type ProjectType = [i32; 3];
    fn restore(self, project: &mut [i32; 3]) -> Self {
        let old = project[self.at];
        project[self.at] = self.value;
        Set { at: self.at, value: old }
    }
}

/// Pushes the old value, then changes it, the way an app would
fn edit(stack: &mut UndoStack<Set>, project: &mut [i32; 3], at: usize, value: i32) {
    stack.push(Set { at, value: project[at] });
    project[at] = value;
}

#[test]
fn undo_and_redo_walk_back_and_forth() {
    let (mut stack, mut p) = (UndoStack::new(), [0; 3]);
    edit(&mut stack, &mut p, 0, 1);
    edit(&mut stack, &mut p, 1, 2);
    stack.undo(&mut p);
    assert_eq!(p, [1, 0, 0]);
    stack.undo(&mut p);
    assert_eq!(p, [0, 0, 0]);
    stack.redo(&mut p);
    stack.redo(&mut p);
    assert_eq!(p, [1, 2, 0]);
}

#[test]
fn a_push_after_undo_clears_redo() {
    let (mut stack, mut p) = (UndoStack::new(), [0; 3]);
    edit(&mut stack, &mut p, 0, 1);
    stack.undo(&mut p);
    edit(&mut stack, &mut p, 1, 5);
    assert_eq!(stack.future_stack_len(), 0);
    assert!(stack.redo(&mut p).is_none());
    assert_eq!(p, [0, 5, 0]);
}

#[test]
fn the_same_value_twice_is_kept_once() {
    let mut stack = UndoStack::new();
    stack.push(Set { at: 0, value: 1 });
    stack.push(Set { at: 0, value: 1 });
    assert_eq!(stack.past_stack_len(), 1);
}

#[test]
fn a_group_undoes_and_redoes_as_one() {
    let (mut stack, mut p) = (UndoStack::new(), [0; 3]);
    stack.start_group();
    edit(&mut stack, &mut p, 0, 1);
    edit(&mut stack, &mut p, 0, 2); // the same number twice
    edit(&mut stack, &mut p, 1, 3);
    stack.finish_group();
    stack.undo(&mut p);
    assert_eq!(p, [0, 0, 0]);
    stack.redo(&mut p);
    assert_eq!(p, [2, 3, 0]);
}

#[test]
fn an_empty_group_leaves_no_undo_step() {
    let (mut stack, mut p) = (UndoStack::new(), [0; 3]);
    edit(&mut stack, &mut p, 0, 1);
    stack.start_group();
    stack.finish_group();
    stack.undo(&mut p); // the edit, not a step that does nothing
    assert_eq!(p, [0, 0, 0]);
}

#[test]
fn undo_inside_an_open_group_closes_it_first() {
    let (mut stack, mut p) = (UndoStack::new(), [0; 3]);
    edit(&mut stack, &mut p, 0, 1);
    edit(&mut stack, &mut p, 1, 1);
    stack.start_group();
    edit(&mut stack, &mut p, 2, 1);
    stack.undo(&mut p); // the group, which only has [2]
    assert_eq!(p, [1, 1, 0]);
    stack.undo(&mut p); // then just [1], not merged with [0]
    assert_eq!(p, [1, 0, 0]);
}

#[test]
fn undo_returns_where_it_happened_for_a_single_value_and_a_group() {
    let (mut stack, mut p) = (UndoStack::new(), [0; 3]);
    edit(&mut stack, &mut p, 0, 1);
    stack.start_group();
    edit(&mut stack, &mut p, 1, 1);
    edit(&mut stack, &mut p, 2, 1);
    stack.finish_group();
    // The group's last value restored was the first one pushed, at [1]
    assert_eq!(stack.undo(&mut p).map(|set| set.at), Some(1));
    assert_eq!(stack.undo(&mut p).map(|set| set.at), Some(0));
    assert_eq!(stack.redo(&mut p).map(|set| set.at), Some(0));
}

#[test]
fn undo_with_nothing_left_returns_none() {
    let (mut stack, mut p) = (UndoStack::new(), [0; 3]);
    assert!(stack.undo(&mut p).is_none());
    edit(&mut stack, &mut p, 0, 1);
    stack.undo(&mut p);
    // The redo stack has a value now, but it's not an undo
    assert!(stack.undo(&mut p).is_none());
    assert_eq!(p, [0, 0, 0]);
}

#[test]
fn a_buffer_keeps_only_a_changed_value() {
    let mut stack = UndoStack::new();
    stack.start_buffer(Set { at: 0, value: 1 });
    stack.finish_buffer(Set { at: 0, value: 1 }); // dragged back to where it was
    assert_eq!(stack.past_stack_len(), 0);
    stack.start_buffer(Set { at: 0, value: 1 });
    stack.finish_buffer(Set { at: 0, value: 4 });
    assert_eq!(stack.past_stack_len(), 1);
}
