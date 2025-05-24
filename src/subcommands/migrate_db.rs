use jsj_backend::database;

pub fn migrate_db() {
    println!("migrating database");
    database::migrate();
}
