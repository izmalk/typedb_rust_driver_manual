use typedb_driver::{
    answer::ConceptMap, concept::{Attribute}, Connection, DatabaseManager, Error, Options, Promise, Session, SessionType, TransactionType
};

fn main() -> Result<(), Error> {
    const DB_NAME: &str = "sample_db3";
    const SERVER_ADDR: &str = "127.0.0.1:1729";

    println!("TypeDB Manual sample code");

    println!(
        "Attempting to connect to a TypeDB Core server: {}",
        SERVER_ADDR
    );
    let driver = Connection::new_core(SERVER_ADDR)?;
    let databases = DatabaseManager::new(driver);

    for db in databases.all()? {
        println!("{}", db.name());
    }

    if databases.contains(DB_NAME)? {
        let _ = databases.get(DB_NAME)?.delete();
    }
    let _ = databases.create(DB_NAME);

    if databases.contains(DB_NAME)? {
        println!("Database setup complete");
    }

    {
        let db = databases.get(DB_NAME)?;
        let session = Session::new(db, SessionType::Schema)?;
        let tx = session.transaction(TransactionType::Write)?;
        let define_query = "
                                define
                                email sub attribute, value string;
                                name sub attribute, value string;
                                friendship sub relation, relates friend;
                                user sub entity,
                                    owns email @key,
                                    owns name,
                                    plays friendship:friend;
                                admin sub user;
                                ";
        tx.query().define(define_query).resolve()?;
        tx.commit().resolve()?;
    }

    {
        let db = databases.get(DB_NAME)?;
        let session = Session::new(db, SessionType::Schema)?;
        let tx = session.transaction(TransactionType::Write)?;
        let undefine_query = "undefine admin sub user;";
        tx.query().undefine(undefine_query).resolve()?;
        tx.commit().resolve()?;
    }

    {
        let db = databases.get(DB_NAME)?;
        let session = Session::new(db, SessionType::Data)?;
        let tx = session.transaction(TransactionType::Write)?;
        let insert_query = "
                                insert
                                $user1 isa user, has name 'Alice', has email 'alice@vaticle.com';
                                $user2 isa user, has name 'Bob', has email 'bob@vaticle.com';
                                $friendship (friend:$user1, friend: $user2) isa friendship;
                                ";
        let _ = tx.query().insert(insert_query)?;
        tx.commit().resolve()?;        
    }

    {
        let db = databases.get(DB_NAME)?;
        let session = Session::new(db, SessionType::Data)?;
        let tx = session.transaction(TransactionType::Write)?;
        let match_insert_query = "
                                match
                                $u isa user, has name 'Bob';
                                insert
                                $new-u isa user, has name 'Charlie', has email 'charlie@vaticle.com';
                                $f($u,$new-u) isa friendship;
                                ";
        let response_count = tx.query().insert(match_insert_query)?.count();
        if response_count == 1 {
            tx.commit().resolve()?;
        } else {
            tx.force_close(); 
        }
               
    }

    {
        let db = databases.get(DB_NAME)?;
        let session = Session::new(db, SessionType::Data)?;
        let tx = session.transaction(TransactionType::Write)?;
        let delete_query = "
                                match
                                $u isa user, has name 'Charlie';
                                $f ($u) isa friendship;
                                delete
                                $f isa friendship;
                                ";
        tx.query().delete(delete_query).resolve();
        tx.commit().resolve()?;       
    }

    {
        let db = databases.get(DB_NAME)?;
        let session = Session::new(db, SessionType::Data)?;
        let tx = session.transaction(TransactionType::Write)?;
        let update_query = "
                                match
                                $u isa user, has name 'Charlie', has email $e;
                                delete
                                $u has $e;
                                insert
                                $u has email 'charles@vaticle.com';
                                ";
        let response_count = tx.query().update(update_query)?.count();
        if response_count == 1 {
            tx.commit().resolve()?;
        } else {
            tx.force_close(); 
        }     
    }

    Ok({})
}
