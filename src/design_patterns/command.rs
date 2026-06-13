// The basic idea of the Command pattern is to separate out actions into its own objects and pass
// them as parameters.

// Suppose we have a sequence of actions or transactions encapsulated as objects. We want these
// actions or commands to be executed or invoked in some order later at different time. These
// commands may also be triggered as a result of some event. For example, when a user pushes a
// button, or on arrival of a data packet. In addition, these commands might be undoable. This
// may come in useful for operations of an editor. We might want to store logs of executed
// commands so that we could reapply the changes later if the system crashes.

// Example: Define two database operations create table and add field. Each of these operations
// is a command which knows how to undo the command, e.g., drop table and remove field. When a
// user invokes a database migration operation then each command is executed in the defined order,
// and when the user invokes the rollback operation then the whole set of commands is invoked in
// reverse order.

// command using trait objects.
fn command_via_trait() {
    pub trait Migration {
        fn execute(&self) -> &str;
        fn rollback(&self) -> &str;
    }

    pub struct CreateTable;
    impl Migration for CreateTable {
        fn execute(&self) -> &str {
            "create table"
        }
        fn rollback(&self) -> &str {
            "drop table"
        }
    }

    pub struct AddField;
    impl Migration for AddField {
        fn execute(&self) -> &str {
            "add field"
        }
        fn rollback(&self) -> &str {
            "remove field"
        }
    }

    struct Schema {
        commands: Vec<Box<dyn Migration>>,
    }

    impl Schema {
        fn new() -> Self {
            Self { commands: vec![] }
        }

        fn add_migration(&mut self, cmd: Box<dyn Migration>) {
            self.commands.push(cmd);
        }

        fn execute(&self) -> Vec<&str> {
            self.commands.iter().map(|cmd| cmd.execute()).collect()
        }
        fn rollback(&self) -> Vec<&str> {
            self.commands
                .iter()
                .rev() // reverse iterator's direction
                .map(|cmd| cmd.rollback())
                .collect()
        }
    }

    fn main() {
        let mut schema = Schema::new();

        let cmd = Box::new(CreateTable);
        schema.add_migration(cmd);
        let cmd = Box::new(AddField);
        schema.add_migration(cmd);

        assert_eq!(vec!["create table", "add field"], schema.execute());
        assert_eq!(vec!["remove field", "drop table"], schema.rollback());
    }
}

// command via function pointers
fn command_via_function_ptr() {
    type FnPtr = fn() -> String;
    struct Command {
        execute: FnPtr,
        rollback: FnPtr,
    }

    struct Schema {
        commands: Vec<Command>,
    }

    impl Schema {
        fn new() -> Self {
            Self { commands: vec![] }
        }
        fn add_migration(&mut self, execute: FnPtr, rollback: FnPtr) {
            self.commands.push(Command { execute, rollback });
        }
        fn execute(&self) -> Vec<String> {
            self.commands.iter().map(|cmd| (cmd.execute)()).collect()
        }
        fn rollback(&self) -> Vec<String> {
            self.commands
                .iter()
                .rev()
                .map(|cmd| (cmd.rollback)())
                .collect()
        }
    }

    fn add_field() -> String {
        "add field".to_string()
    }

    fn remove_field() -> String {
        "remove field".to_string()
    }

    fn main() {
        let mut schema = Schema::new();
        schema.add_migration(|| "create table".to_string(), || "drop table".to_string());
        schema.add_migration(add_field, remove_field);
        assert_eq!(vec!["create table", "add field"], schema.execute());
        assert_eq!(vec!["remove field", "drop table"], schema.rollback());
    }
}

fn command_via_fn_trait() {
    // dyn Fn() -> &'a str is a trait object that represents some function that, when called,
    // returns a reference with exactly the lifetime 'a. The compiler will infer 'a with a specific
    // lifetime when you create a Schema instance.
    type Migration<'a> = Box<dyn Fn() -> &'a str>;

    struct Schema<'a> {
        executes: Vec<Migration<'a>>,
        rollbacks: Vec<Migration<'a>>,
    }

    impl<'a> Schema<'a> {
        fn new() -> Self {
            Self {
                executes: vec![],
                rollbacks: vec![],
            }
        }
        fn add_migration<E, R>(&mut self, execute: E, rollback: R)
        where
        // the generic E and R are bounded with a Trait: Fn() -> &'a str + 'static, which means
        // that E and R must be functions that return a reference with lifetime 'a and can live
        // for the entire duration of the program (i.e., they don't capture any non-static references).
            E: Fn() -> &'a str + 'static,
            R: Fn() -> &'a str + 'static,
        {
            self.executes.push(Box::new(execute));
            self.rollbacks.push(Box::new(rollback));
        }
        fn execute(&self) -> Vec<&str> {
            self.executes.iter().map(|cmd| cmd()).collect()
        }
        fn rollback(&self) -> Vec<&str> {
            self.rollbacks.iter().rev().map(|cmd| cmd()).collect()
        }
    }

    fn add_field() -> &'static str {
        "add field"
    }

    fn remove_field() -> &'static str {
        "remove field"
    }

    fn main() {
        let mut schema = Schema::new();
        schema.add_migration(|| "create table", || "drop table");
        schema.add_migration(add_field, remove_field);
        assert_eq!(vec!["create table", "add field"], schema.execute());
        assert_eq!(vec!["remove field", "drop table"], schema.rollback());
    }
}