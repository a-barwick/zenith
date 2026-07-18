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
    Id(Option<String>),
    SObjectDomain(String),
    Class(String),
    ExternalClass(String),
    List(Box<Type>),
    Set(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Nullable(Box<Type>),
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
            Self::Id(Some(domain)) => format!("Id<{domain}>"),
            Self::Id(None) => "Id".into(),
            Self::SObjectDomain(name) => name.clone(),
            Self::Class(name) | Self::ExternalClass(name) => name.clone(),
            Self::List(element) => format!("List<{}>", element.display_name()),
            Self::Set(element) => format!("Set<{}>", element.display_name()),
            Self::Map(key, value) => {
                format!("Map<{}, {}>", key.display_name(), value.display_name())
            }
            Self::Nullable(inner) => format!("{}?", inner.display_name()),
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
                | Self::Id(_)
                | Self::Class(_)
                | Self::ExternalClass(_)
                | Self::List(_)
                | Self::Set(_)
                | Self::Map(_, _)
                | Self::Nullable(_)
        )
    }

    pub fn accepts(&self, value: &Self) -> bool {
        self == value
            || matches!(self, Self::Object) && !matches!(value, Self::Void | Self::Error)
            || matches!(
                (self, value),
                (Self::Id(None), Self::Id(Some(_))) | (Self::Nullable(_), Self::Null)
            )
            || match self {
                Self::Nullable(inner) => inner.accepts(value),
                _ => false,
            }
    }

    pub fn canonical_class_name(&self) -> Option<&str> {
        match self {
            Self::Class(name) | Self::ExternalClass(name) => Some(name),
            _ => None,
        }
    }

    pub fn non_nullable(&self) -> Option<&Self> {
        match self {
            Self::Nullable(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn into_nullable(self) -> Self {
        match self {
            Self::Nullable(_) | Self::Error => self,
            other => Self::Nullable(Box::new(other)),
        }
    }

    pub fn apex_name(&self) -> String {
        match self {
            Self::Nullable(inner) => inner.apex_name(),
            Self::Id(_) => "Id".into(),
            Self::SObjectDomain(name) => name.clone(),
            Self::List(element) => format!("List<{}>", element.apex_name()),
            Self::Set(element) => format!("Set<{}>", element.apex_name()),
            Self::Map(key, value) => format!("Map<{}, {}>", key.apex_name(), value.apex_name()),
            other => other.display_name(),
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
        assert!(!nested.accepts(&Type::Null));
        assert!(Type::Nullable(Box::new(nested.clone())).accepts(&Type::Null));
        assert!(!Type::Integer.accepts(&Type::Null));
        assert!(Type::Double.is_numeric());
        assert!(!Type::String.is_numeric());
    }
}
