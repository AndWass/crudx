use crudx::Table;

#[derive(Table)]
struct TestTable {
    pub field: i32,
}

fn it_works() {
    let x = NewTestTable {
        field: 0,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let x = NewTestTable {
            field: 0,
        };
    }
}
