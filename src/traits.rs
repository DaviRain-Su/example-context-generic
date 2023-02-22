pub trait NamedPerson {
    fn name(&self) -> &str;
}


pub trait PersonContext {
    type PersonId;
    type Person: NamedPerson;
}

pub trait HasError {
    type Error;
}

pub trait PersonQuerier: PersonContext + HasError {
    fn query_person(&self, person_id: &Self::PersonId) -> Result<Self::Person, Self::Error>;
}


pub trait Greeter<Context>
where
    Context: PersonContext + HasError,
{
    fn greet(
        &self,
        context: &Context,
        person_id: &Context::PersonId,
    ) -> Result<(), Context::Error>;
}