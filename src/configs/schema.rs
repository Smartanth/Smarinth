use crate::configs::DatabaseScheme;
use crate::entities::{Table, UserTable};

pub struct SchemaManager {
    tables: Vec<Box<dyn Table>>,
}

impl SchemaManager {
    pub fn new(mut tables: Vec<Box<dyn Table>>) -> Self {
        Self::sort_tables(&mut tables);
        Self { tables }
    }

    fn sort_tables(tables: &mut Vec<Box<dyn Table>>) {
        let mut to_sort = std::mem::take(tables);
        let mut deps_list: Vec<_> = to_sort.iter().map(|t| t.dependencies()).collect();
        let mut sorted = Vec::with_capacity(to_sort.len());

        while !to_sort.is_empty() {
            let independent_indices: Vec<usize> = deps_list.iter().enumerate()
                .filter(|(_, deps)| deps.is_empty())
                .map(|(i, _)| i)
                .collect();

            assert!(!independent_indices.is_empty(), "Circular dependency detected or unresolved dependencies exist.");

            for &index in independent_indices.iter().rev() {
                let table = to_sort.swap_remove(index);
                let _ = deps_list.swap_remove(index);
                sorted.push(table);
            }

            for deps in deps_list.iter_mut() {
                deps.retain(|dep_name| {
                    !sorted.iter().any(|resolved_table| resolved_table.name() == *dep_name)
                });
            }
        }

        *tables = sorted;
    }

    pub fn create_schema(&self, scheme: &DatabaseScheme) -> Vec<String> {
        self.tables.iter().map(|table| table.create(scheme)).collect()
    }

    pub fn dispose_schema(&self) -> Vec<String> {
        self.tables.iter().rev().map(|table| table.dispose()).collect()
    }
}

impl Default for SchemaManager {
    fn default() -> Self {
        SchemaManager::new(
            vec![
                Box::new(UserTable),
            ]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct MockGroupTable;
    impl Table for MockGroupTable {
        fn name(&self) -> &'static str {
            "groups"
        }

        fn create(&self, _: &DatabaseScheme) -> String {
            "CREATE TABLE groups;".to_string()
        }

        fn dispose(&self) -> String {
            "DROP TABLE groups;".to_string()
        }

        fn dependencies(&self) -> Vec<&'static str> {
            vec![]
        }
    }

    #[derive(Clone)]
    struct MockUserTable;
    impl Table for MockUserTable {
        fn name(&self) -> &'static str {
            "users"
        }

        fn create(&self, _: &DatabaseScheme) -> String {
            "CREATE TABLE users;".to_string()
        }

        fn dispose(&self) -> String {
            "DROP TABLE users;".to_string()
        }

        fn dependencies(&self) -> Vec<&'static str> {
            vec!["groups"]
        }
    }

    #[derive(Clone)]
    struct MockToolTable;
    impl Table for MockToolTable {
        fn name(&self) -> &'static str {
            "tools"
        }

        fn create(&self, _: &DatabaseScheme) -> String {
            "CREATE TABLE tools;".to_string()
        }

        fn dispose(&self) -> String {
            "DROP TABLE tools;".to_string()
        }

        fn dependencies(&self) -> Vec<&'static str> {
            vec!["groups"]
        }
    }

    #[derive(Clone)]
    struct MockUserRegionTable;
    impl Table for MockUserRegionTable {
        fn name(&self) -> &'static str {
            "users_tools_link"
        }

        fn create(&self, _: &DatabaseScheme) -> String {
            "CREATE TABLE users_tools_link;".to_string()
        }

        fn dispose(&self) -> String {
            "DROP TABLE users_tools_link;".to_string()
        }

        fn dependencies(&self) -> Vec<&'static str> {
            vec!["users", "tools"]
        }
    }

    #[test]
    fn test_correct_creation_order() {
        let tables: Vec<Box<dyn Table>> = vec![
            Box::new(MockUserRegionTable),
            Box::new(MockToolTable),
            Box::new(MockUserTable),
            Box::new(MockGroupTable),
        ];

        let manager = SchemaManager::new(tables);
        let statements = manager.create_schema(&DatabaseScheme::POSTGRES);

        assert_eq!(statements[0], "CREATE TABLE groups;");
        assert_eq!(statements[1], "CREATE TABLE users;");
        assert_eq!(statements[2], "CREATE TABLE tools;");
        assert_eq!(statements[3], "CREATE TABLE users_tools_link;");
    }
}
