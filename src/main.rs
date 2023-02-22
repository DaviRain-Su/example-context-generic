use traits::{PersonQuerier, HasError, PersonContext, Greeter};

use crate::traits::NamedPerson;
pub mod traits;

#[derive(Debug)]
pub struct PersonId(pub String);

pub struct Person {
    pub name: String,
}

impl NamedPerson for Person {
    fn name(&self) -> &str {
        &self.name
    }
}

struct AppContext;

impl HasError for AppContext {
    type Error = anyhow::Error;
}

impl PersonContext for AppContext {
    type PersonId = String;
    type Person = Person;
}

impl PersonQuerier for AppContext {
    fn query_person(&self, _person_id: &Self::PersonId) -> Result<Person, Self::Error> {
        Ok(Person {
            name: format!("{:?}", _person_id),
        })
    }
}

struct SimpleGreeter;

impl<Context> Greeter<Context> for SimpleGreeter
    where Context: PersonQuerier,
 {
    fn greet(
        &self,
        context: &Context,
        person_id: &Context::PersonId,
    ) -> Result<(), Context::Error> {
        let person = context.query_person(person_id)?;
        println!("Hello, {}", person.name());
        Ok(())
    }
}
fn main() -> anyhow::Result<()>{
    let appcontext = AppContext;
    let person_id = "davirain".to_string();

    let simple = SimpleGreeter;

    // call greet
    simple.greet(&appcontext, &person_id)?;

    println!("Hello, world!");
    Ok(())
}
