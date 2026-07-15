//! Type-safe definition objects for signals, queries, and updates.
//!
//! These definitions provide compile-time type safety for workflow
//! interactions. A `SignalDef<[Address]>` ensures the caller passes
//! the correct argument types when signaling a workflow.

use std::marker::PhantomData;

/// A type-safe signal definition.
///
/// Create with `signal::<Args>("name")`. Use with `handle.signal(def, args)`.
#[derive(Clone)]
pub struct SignalDef<Args> {
    /// The signal name as registered on the workflow.
    pub name: String,
    _phantom: PhantomData<Args>,
}

/// A type-safe query definition.
///
/// Create with `query::<Ret, Args>("name")`. Use with `handle.query(def, args)`.
#[derive(Clone)]
pub struct QueryDef<Ret, Args = ()> {
    /// The query name as registered on the workflow.
    pub name: String,
    _phantom: PhantomData<(Ret, Args)>,
}

/// A type-safe update definition.
///
/// Create with `update::<Ret, Args>("name")`. Use with `handle.executeUpdate(def, args)`.
#[derive(Clone)]
pub struct UpdateDef<Ret, Args = ()> {
    /// The update name as registered on the workflow.
    pub name: String,
    _phantom: PhantomData<(Ret, Args)>,
}

/// Creates a type-safe signal definition.
///
/// # Example
///
/// ```rust,ignore
/// let update_address = signal::<Address>("updateAddress");
/// await handle.signal(update_address, address);
/// ```
pub fn signal<Args>(name: &str) -> SignalDef<Args> {
    SignalDef {
        name: name.to_string(),
        _phantom: PhantomData,
    }
}

/// Creates a type-safe query definition.
///
/// # Example
///
/// ```rust,ignore
/// let get_status = query::<OrderStatus, ()>("getStatus");
/// let status = handle.query(get_status, ()).await?;
/// ```
pub fn query<Ret, Args>(name: &str) -> QueryDef<Ret, Args> {
    QueryDef {
        name: name.to_string(),
        _phantom: PhantomData,
    }
}

/// Creates a type-safe update definition.
///
/// # Example
///
/// ```rust,ignore
/// let refund = update::<bool, f64>("refund");
/// let success = handle.execute_update(refund, 50.0).await?;
/// ```
pub fn update<Ret, Args>(name: &str) -> UpdateDef<Ret, Args> {
    UpdateDef {
        name: name.to_string(),
        _phantom: PhantomData,
    }
}
