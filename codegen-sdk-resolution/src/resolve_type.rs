pub trait ResolutionStack<'db, T>: Clone + Copy {
    fn bottom(self, db: &'db dyn Db) -> &'db T;
    fn entries(self, db: &'db dyn Db) -> &'db Vec<T>;
}

use crate::Db;
// Get definitions for a given type
pub trait ResolveType<'db>
where
    Self: 'db,
{
    type Type: Clone; // Possible types this trait can be defined as
    type Stack: ResolutionStack<'db, Self::Type>;
    // Resolve the type this node resolves to. IE: For a function call, this is the function return type
    fn resolve_type(self, db: &'db dyn Db) -> &'db Vec<Self::Stack>;
    // Find the definition of this type
    fn definition(self, db: &'db dyn Db) -> Option<Self::Type>
    where
        Self: Sized,
    {
        self.definitions(db).into_iter().next()
    }
    // Find all definitions for this type
    fn definitions(self, db: &'db dyn Db) -> Vec<Self::Type>
    where
        Self: Sized,
    {
        let mut results = Vec::new();
        for stack in self.resolve_definition_stack(db) {
            results.push(stack.clone().bottom(db).clone());
        }
        results
    }
    // Find the definitions of this type and the trace of how it was resolved
    fn resolve_definition_stack(self, db: &'db dyn Db) -> &'db Vec<Self::Stack>
    where
        Self: Sized,
    {
        self.resolve_type(db)
    }
}
