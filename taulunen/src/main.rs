use taulunen::{DataType, Index, Query, Table, Value};

#[derive(Debug, Clone)]
struct User<'a> {
    name: &'a str,
    age: u8,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum UserIndex {
    Name,
    Age,
}

impl Index<User<'_>> for UserIndex {
    fn data_type(&self) -> DataType {
        match self {
            UserIndex::Name => DataType::String,
            UserIndex::Age => DataType::Int,
        }
    }

    fn extract(&self, user: &User) -> Option<Value> {
        match self {
            UserIndex::Name => Some(Value::String(user.name.to_string())),
            UserIndex::Age => Some(Value::Int(user.age as i64)),
        }
    }

    fn is_unique(&self) -> bool {
        false
    }
}

fn main() {
    let mut user_table = Table::empty()
        .add_index(UserIndex::Name)
        .add_index(UserIndex::Age);
    let max = user_table.insert(User {
        name: "Max",
        age: 29,
    });
    user_table.insert(User {
        name: "Jalai",
        age: 29,
    });
    user_table.insert(User {
        name: "Pekka",
        age: 44,
    });

    println!("user = {:?}", user_table);
    println!("max = {:?}", user_table.get(max));

    user_table.update(max, |v| v.age = 30);
    println!("max = {:?}", user_table.get(max));

    user_table.remove_if(max, |v| v.age == 29);
    println!("max = {:?}", user_table.get(max));

    let results = user_table.where_eq(UserIndex::Age, Value::int(29));
    println!("results = {:?}", results);

    user_table.remove(max);
    println!("max = {:?}", user_table.get(max));

    let q = Query::or([
        Query::eq(UserIndex::Age, Value::int(29)),
        Query::eq(UserIndex::Name, Value::string("Max")),
    ]);
    println!("q = {:?}", q);
}
