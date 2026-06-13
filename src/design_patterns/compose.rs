// Sometimes a large struct will cause issues with the borrow checker - although
// fields can be borrowed independently, sometimes the whole struct ends up being used at once,
// preventing other uses. A solution might be to decompose the struct into several smaller structs.
// Then compose these together into the original struct. Then each struct can be borrowed
// separately and have more flexible behaviour.
//
// This will often lead to a better design in other ways: applying this design pattern
// often reveals smaller units of functionality.

// Bad example:
fn bad_example() {
    struct Database {
        connection_string: String,
        timeout: u32,
        pool_size: u32,
    }

    fn print_database(database: &Database) {
        println!("Connection string: {}", database.connection_string);
        println!("Timeout: {}", database.timeout);
        println!("Pool size: {}", database.pool_size);
    }

    fn main() {
        let mut db = Database {
            connection_string: "initial string".to_string(),
            timeout: 30,
            pool_size: 100,
        };

        let connection_string = &mut db.connection_string;
        print_database(&db); // not compile as the design is not good.
        *connection_string = "new string".to_string();
    }
}

fn good_example() {
    // Database is now composed of three structs - ConnectionString, Timeout and PoolSize.
    // Let's decompose it into smaller structs
    #[derive(Debug, Clone)]
    struct ConnectionString(String);

    #[derive(Debug, Clone, Copy)]
    struct Timeout(u32);

    #[derive(Debug, Clone, Copy)]
    struct PoolSize(u32);

    // We then compose these smaller structs back into `Database`
    struct Database {
        connection_string: ConnectionString,
        timeout: Timeout,
        pool_size: PoolSize,
    }

    // print_database can then take ConnectionString, Timeout and Poolsize struct instead
    fn print_database(connection_str: ConnectionString, timeout: Timeout, pool_size: PoolSize) {
        println!("Connection string: {connection_str:?}");
        println!("Timeout: {timeout:?}");
        println!("Pool size: {pool_size:?}");
    }

    fn main() {
        // Initialize the Database with the three structs
        let mut db = Database {
            connection_string: ConnectionString("localhost".to_string()),
            timeout: Timeout(30),
            pool_size: PoolSize(100),
        };

        let connection_string = &mut db.connection_string;
        print_database(connection_string.clone(), db.timeout, db.pool_size);
        *connection_string = ConnectionString("new string".to_string());
    }
}