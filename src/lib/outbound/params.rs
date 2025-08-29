use lombok::*;

#[derive(Getter, GetterMut, Setter, NoArgsConstructor, AllArgsConstructor, ToString, Clone)]
pub struct ConnectionParams {
    pub host: String,
    pub port: u16,
    pub dbname: String,
    pub user: String,
    pub password: String,
}

impl ConnectionParams {
    pub fn connect_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user,
            self.password,
            self.host,
            self.port,
            self.dbname
        )
    }
}