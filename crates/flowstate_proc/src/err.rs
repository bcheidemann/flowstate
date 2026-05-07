use quote::ToTokens;

macro_rules! flowstate_derive_err {
    ($name:ident, $source:ident, |$($arg:ident : $arg_ty:ty),*| $message:expr) => {
        pub struct $source<T>(T);

        pub struct $name<T> {
            source: $source<T>,
            $($arg: $arg_ty,)*
        }

        impl<T> $name<T> {
            pub fn at(source: T) -> $source<T> {
                $source(source)
            }
        }

        impl<T> $source<T> {
            pub fn with(self, $($arg: $arg_ty,)*) -> $name<T> {
                $name { source: self, $($arg,)* }
            }
        }

        impl $name<proc_macro2::Span> {
            pub fn into_syn_error(self) -> syn::Error {
                $(let $arg = self.$arg;)*
                syn::Error::new(self.source.0, $message)
            }
        }

        impl<T: ToTokens> From<$name<T>> for syn::Error {
            fn from(value: $name<T>) -> Self {
                $(let $arg = value.$arg;)*
                syn::Error::new_spanned(value.source.0, $message)
            }
        }
    };
    ($name:ident, $message:expr) => {
        pub struct $name<T>(T);

        impl<T> $name<T> {
            pub fn at(source: T) -> Self {
                Self(source)
            }
        }

        impl $name<proc_macro2::Span> {
            pub fn into_syn_error(self) -> syn::Error {
                syn::Error::new(self.0, $message)
            }
        }

        impl<T: ToTokens> From<$name<T>> for syn::Error {
            fn from(value: $name<T>) -> Self {
                syn::Error::new_spanned(value.0, $message)
            }
        }
    };
}

flowstate_derive_err!(
    UnsupportedEnumOrUnion,
    "Workflow can only be derived for structs"
);
flowstate_derive_err!(
    UnsupportedTupleStruct,
    "Workflow cannot currently be derived for tuple structs"
);
flowstate_derive_err!(
    UnsupportedUnitStruct,
    "Workflow cannot currently be derived for unit structs"
);
flowstate_derive_err!(
    UnexpectedUnnamedField,
    "Encountered unexpected unnamed field. This is likely a bug in flowstate. Please report it."
);
flowstate_derive_err!(
    DuplicateStateAttribute,
    "Workflow must have exactly one field marked with the #[state] attribute"
);
flowstate_derive_err!(
    MissingStateAttribute,
    "Workflow must have exactly one field marked with the #[state] attribute"
);
flowstate_derive_err!(
    InvalidStateFieldType,
    InvalidStateFieldTypeSource,
    |state_field_ident: String, state_param: String| format!(
        "The `{state_field_ident}` field must be of type `{state_param}`"
    )
);
flowstate_derive_err!(
    UnsupportedGenericLifetimeParameter,
    "Generic lifetime parameters are not currently supported for workflows"
);
flowstate_derive_err!(
    UnsupportedGenericConstParameter,
    "Generic const parameters are not currently supported for workflows"
);
flowstate_derive_err!(
    MissingGenericTypeParameterForState,
    "Workflow requires exactly one generic parameter for state"
);
flowstate_derive_err!(
    UnsupportedAdditionalGenericTypeParameters,
    "Additional generic type parameters are not currently supported for workflows"
);
flowstate_derive_err!(
    UnsupportedBoundsOnGenericTypeParameterForState,
    "Type parameter bounds on the state parameter are not currently supported"
);
flowstate_derive_err!(
    UnsupportedGenericWhereClause,
    "Generic where clauses are not currently supported for workflows"
);
flowstate_derive_err!(
    MissingFlowstateAttributeOnWorkflow,
    "Workflow requires a #[flowstate(result = ...)] attribute"
);
flowstate_derive_err!(DuplicateResultArguments, "Duplicate `result` argument");
flowstate_derive_err!(
    UnknownArgument,
    UnknownArgumentSource,
    |name: String| format!("Unknown argument `{name}`")
);
flowstate_derive_err!(MissingResultArgument, "Missing `result` argument");
flowstate_derive_err!(
    UnexpectedArgumentsForStateAttribute,
    "The #[state] attribute takes no arguments"
);
