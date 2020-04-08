# Intro

It appeared that I've implemented the Loaner pattern recently while doing assignment on Information theory course, so I decided to present it here as a submission for the Modern Programming Paradigms course. After all, the ability to recognize patterns (especially in your own code) is of no less importance than implementing or using them.

# Story

First time I encountered the Loaner patter, not so long ago, I didn't realize it was the pattern. In fact, it was part of QtQuick library, [`QtQuick.LocalStorage 2.0`](http://doc.qt.io/qt-5/qtquick-localstorage-qmlmodule.html). The idea was simple:

1) get a `sqlite3` database connection object: `var db = LocalStorage.openDatabaseSync(...)`
2) request a transaction: `db.transaction()` or immutable (SELECT only) `db.readTransaction()`
3) execute SQL inside transaction callback and process the results:

```js
db.transaction(
    function(tx) {
        var rs = tx.executeSql('SELECT * FROM Greeting');
        var r = "";
        for (var i = 0; i < rs.rows.length; i++) {
            r += rs.rows.item(i).salutation + ", " + rs.rows.item(i).salutee + "\n";
        }
        text = r; // text is some outer, non-local variable captured inside
                  // the closure by reference
    }
)
```

# Loaner Pattern

So, I was collecting statistics for the assignment and storing it in SQLite3. When I got tired of manually managing a connection inside every function that needs access to the database, I realize I can do better than that: create dedicated function to manage the connection. Due to the Rust language's ownership constraint, connection's lifetime must be scoped, which means it can not just be returned from a function, and returning a pointer requires specifying fixed lifetime. The only sane option left was to follow the pattern of QtQuick which appeared to be the Loaner pattern: take a callback as an argument, get a lock on a global connection protected by `Mutex`, pass a reference to the connection into the callback.

```rust
pub fn connection<F, T, E>(f: F) -> result::Result<T, E>
    where F: FnOnce(&Connection) -> result::Result<T, E>
{
    let guard = GLOBAL_CONNECTION.lock().unwrap();
    f(&*guard)  // &* is a magic dereferencing MutexGuard to an underlying Connection
}

```

Since Rust does not have exceptions, there's no need to worry about them: any erroneous situation is signaled by returning `Result::Err` enum variant. Thanks to the RAII, `MutexGuard`, acquired earlier, is dropped when going out of scope, thus releasing `Connection`.

# Example

Initialization of a database. The whole function consists of a single call to a `connection` with a callback. Question marks are the early returns from erroneous results.

```rust
pub fn create_schema() -> ::Result<()> {
    connection(|conn: &Connection| {
        let mut f = fs::File::open(SCHEMA)?;
        let mut schema = String::new();
        f.read_to_string(&mut schema)?;

        conn.execute_batch(&schema)?;

        Ok(())
    })
}
```

# Conclusion

It's cool for me to learn new things about programming, but its much more fun to realize you already did those things — the only is without _knowing_ their names.

The Rust code is stored in `db.rs` file. Most probably it won't compile without the rest of the project, which unfortunately was never released on GitHub or something, but you get the idea...