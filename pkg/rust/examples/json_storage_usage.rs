#[cfg(feature = "json-storage")]
mod json_storage_usage {
    use {
        gluesql::{
            json_storage::JsonStorage,
            prelude::{Glue, Payload}
        },
    };

    pub async fn run() {
        let path = "./data/";
        let json_storage = JsonStorage::new(path).unwrap();
        let mut glue = Glue::new(json_storage);

        // 0. Drop table if exists
        glue.execute("DROP TABLE IF EXISTS User").await.unwrap();

        // 1. Create table (User)
        let create_sql = "
          CREATE TABLE User (
            id INT NOT NULL,
            name TEXT NOT NULL
          );
        ";
        glue.execute(create_sql).await.expect("Failed to create User table.");

        // 2. Insert
        let insert_sql = "
          INSERT INTO User VALUES
            (1, 'Yewon'),
            (2, 'GlueSQL'),
            (3, 'rust');
        ";
        glue.execute(insert_sql).await.expect("Failed to insert data into User table.");

        // 3. Select inserted rows
        let select_sql = "SELECT * FROM User;";
        let result = glue.execute(select_sql).await.expect("Failed to fetch rows from User table.");

        println!("{}", "=== Selected Rows ===");
        match &result[0] {
            Payload::Select { labels: _, rows } => {
                for row in rows.iter() {
                    println!("{:?}", row);
                }
            },
            _ => panic!("Unexpected result: {:?}", result),
        };
    }
}

fn main() {
    #[cfg(feature = "json-storage")]
    futures::executor::block_on(json_storage_usage::run());
}
