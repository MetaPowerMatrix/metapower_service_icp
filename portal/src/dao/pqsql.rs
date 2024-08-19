use postgres::{Client, NoTls, Row};

const PG_SERVER: &str = "localhost";
const PG_USER: &str = "postgres";
const PG_DBNAME: &str = "metapowerassitant";

pub fn pg_connect() -> Result<postgres::Client, anyhow::Error>{
    let connect_string = format!("host={} user={} dbname={}", PG_SERVER, PG_USER, PG_DBNAME);
    let client = Client::connect(&connect_string, NoTls)?;
    
    Ok(client)
}

pub fn pg_create_table(client: &mut postgres::Client, create_sql: &str) -> Result<(), anyhow::Error>{
    client.batch_execute(create_sql)?;

    Ok(())
}
pub fn pg_insert_rec(client: &mut postgres::Client, insert_sql: &str) -> Result<(), anyhow::Error>{
    client.batch_execute(insert_sql)?;

    Ok(())
}
pub fn pg_update_by_id(client: &mut postgres::Client, table: String, id: String, field: String, value: String) -> Result<(), anyhow::Error>{
    let delete_sql = "UPDATE ".to_string() + &table + " SET " + &field + " = $1 WHERE id = $2";
    let statement = client.prepare(&delete_sql)?;
        
    let mut transaction = client.transaction()?;
    transaction.execute(&statement, &[&value, &id])?;
    transaction.commit()?;

    Ok(())
}
pub fn pg_delete_by_id(client: &mut postgres::Client, table: String, id: String) -> Result<(), anyhow::Error>{
    let delete_sql = "DELETE FROM ".to_string() + &table + " WHERE id = $1";
    let statement = client.prepare(&delete_sql)?;
        
    let mut transaction = client.transaction()?;
    transaction.execute(&statement, &[&id])?;
    transaction.commit()?;

    Ok(())
}
pub fn pg_query_all(client: &mut postgres::Client, table: String) -> Result<Vec<Row>, anyhow::Error>{
    let mut rows: Vec<Row> = vec![];
    let query_sql = "SELECT * FROM ".to_string() + &table;
    let statement = client.prepare(&query_sql)?;

    for row in client.query(&statement, &[])? {
        rows.push(row);
    }    

    Ok(rows)
}
pub fn pg_query_by_id(client: &mut postgres::Client, table: String, id: String) -> Result<Row, anyhow::Error>{
    let query_sql = "SELECT * FROM ".to_string() + &table + " WHERE id = $1";
    let statement = client.prepare(&query_sql)?;
        
    let row = client.query_one(&statement, &[&id])?;

    Ok(row)
}
