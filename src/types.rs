use std::fmt;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    Void,
    Boolean,
    Integer,
    Long,
    Decimal,
    Double,
    String,
    Object,
    Class(String),
    ExternalClass(String),
    List(Box<Type>),
    Set(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Null,
    Error,
}

impl Type {
    pub fn display_name(&self) -> String {
        match self {
            Self::Void => "void".into(),
            Self::Boolean => "Boolean".into(),
            Self::Integer => "Integer".into(),
            Self::Long => "Long".into(),
            Self::Decimal => "Decimal".into(),
            Self::Double => "Double".into(),
            Self::String => "String".into(),
            Self::Object => "Object".into(),
            Self::Class(name) | Self::ExternalClass(name) => name.clone(),
            Self::List(element) => format!("List<{}>", element.display_name()),
            Self::Set(element) => format!("Set<{}>", element.display_name()),
            Self::Map(key, value) => {
                format!("Map<{}, {}>", key.display_name(), value.display_name())
            }
            Self::Null => "null".into(),
            Self::Error => "<error>".into(),
        }
    }

    pub const fn is_numeric(&self) -> bool {
        matches!(
            self,
            Self::Integer | Self::Long | Self::Decimal | Self::Double
        )
    }

    pub const fn is_reference_like(&self) -> bool {
        matches!(
            self,
            Self::String
                | Self::Object
                | Self::Class(_)
                | Self::ExternalClass(_)
                | Self::List(_)
                | Self::Set(_)
                | Self::Map(_, _)
        )
    }

    pub fn accepts(&self, value: &Self) -> bool {
        self == value || matches!(value, Self::Null) && self.is_reference_like()
    }

    pub fn canonical_class_name(&self) -> Option<&str> {
        match self {
            Self::Class(name) | Self::ExternalClass(name) => Some(name),
            _ => None,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::Type;

    #[test]
    fn formats_nested_collection_types_and_applies_assignability_rules() {
        let nested = Type::Map(
            Box::new(Type::String),
            Box::new(Type::List(Box::new(Type::Integer))),
        );
        assert_eq!(nested.display_name(), "Map<String, List<Integer>>");
        assert!(nested.accepts(&Type::Null));
        assert!(!Type::Integer.accepts(&Type::Null));
        assert!(Type::Double.is_numeric());
        assert!(!Type::String.is_numeric());
    }
}
